//! Sincronização entre pessoas (docs/colaboracao.md, seção 6): `POST /api/mudancas` aplica operações por item, `GET /api/eventos` as leva por SSE
//! a quem lê o workspace, com `id:` = versão e retomada por `Last-Event-ID`, e `POST /api/presenca` diz quem está onde. Tudo o que é desta frente
//! na memória mora aqui: o buffer dos últimos lotes (até `MAX_LOTES` ou `JANELA`), os assinantes e a presença. Nada roda em repouso: cada
//! conexão dorme no canal dela e o batimento é o do SSE (25 s).
mod eventos;
pub mod operacoes;
mod presenca;
pub mod rotas;

#[cfg(test)]
mod tests;

pub use eventos::Fluxo;

use crate::contexto::Contexto;
use crate::dados::Autor;
use crate::erro::{Erro, Resultado};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub const MAX_LOTES: usize = 1000;
pub const JANELA: Duration = Duration::from_secs(15 * 60);

// ---- os pontos únicos de autoria e permissão: ao integrar a frente ACESSO, trocar por `ctx.nome` e `ctx.exige(Acao::..., ws, bp)` ----

/// Quem assina as operações e a presença.
pub fn autor(ctx: &Contexto) -> Autor {
    Autor { id: ctx.usuario.clone(), nome: ctx.usuario.clone() }
}

/// Pode gravar no workspace/blueprint? `administrar` = criar ou apagar workspace, mudar o workspace raiz.
pub fn exige_gravar(ctx: &Contexto, administrar: bool, _workspace: &str, _blueprint: Option<&str>) -> Resultado<()> {
    if administrar && !ctx.pode_administrar() {
        return Err(Erro::Proibido(format!("{} não pode administrar", ctx.usuario)));
    }
    ctx.exige_editar()
}

/// Pode ler o workspace (ou o blueprint)? Também filtra quem recebe os eventos.
pub fn pode_ler(ctx: &Contexto, _workspace: &str, _blueprint: Option<&str>) -> bool {
    ctx.pode_ler()
}

/// A presença só existe no modo hospedado (`BLUEPRINT_HOSPEDAR=1`; a frente ACESSO troca pela configuração).
pub fn presenca_do_ambiente() -> bool {
    std::env::var("BLUEPRINT_HOSPEDAR").is_ok_and(|v| v == "1")
}

struct Guardado {
    versao: u64,
    em: Instant,
    workspace_id: String,
    texto: String,
}

pub(crate) struct Assinante {
    id: u64,
    workspace_id: String,
    tx: Sender<String>,
}

pub(crate) struct Interno {
    lotes: VecDeque<Guardado>,
    /// a última versão publicada (lote ou PUT)
    atual: u64,
    /// a versão do último PUT (ou da abertura): retomar de antes dela exige recarregar
    marca: u64,
    assinantes: Vec<Assinante>,
    proximo: u64,
    presenca: presenca::Presencas,
}

pub struct Sincronia {
    pub(crate) interno: Arc<Mutex<Interno>>,
    pub presenca_ligada: bool,
}

/// Enquanto a conexão SSE vive: ao cair, sai dos assinantes e da presença.
pub struct Guarda {
    interno: Arc<Mutex<Interno>>,
    id: u64,
}

impl Drop for Guarda {
    fn drop(&mut self) {
        let ws = {
            let mut i = travar(&self.interno);
            i.assinantes.retain(|a| a.id != self.id);
            i.presenca.sair(self.id)
        };
        if let Some(ws) = ws {
            presenca::avisar(&self.interno, &ws);
        }
    }
}

pub(crate) fn travar(m: &Mutex<Interno>) -> MutexGuard<'_, Interno> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn sse(id: Option<u64>, evento: &str, dados: &Value) -> String {
    let id = id.map(|v| format!("id: {v}\n")).unwrap_or_default();
    format!("{id}event: {evento}\ndata: {dados}\n\n")
}

impl Interno {
    fn enviar(&mut self, workspace: Option<&str>, texto: &str) {
        self.assinantes.retain(|a| workspace.is_some_and(|w| w != a.workspace_id) || a.tx.send(texto.to_string()).is_ok());
    }
}

impl Sincronia {
    pub fn nova(versao: u64, presenca_ligada: bool) -> Sincronia {
        let interno = Interno { lotes: VecDeque::new(), atual: versao, marca: versao, assinantes: Vec::new(), proximo: 1, presenca: presenca::Presencas::default() };
        Sincronia { interno: Arc::new(Mutex::new(interno)), presenca_ligada }
    }

    /// Um lote gravado: um evento `mudancas` por blueprint tocado, guardado no buffer e enviado a quem assina o workspace.
    pub fn publicar(&self, versao: u64, eventos: Vec<(String, Value)>) {
        let mut i = travar(&self.interno);
        i.atual = versao;
        let agora = Instant::now();
        for (workspace_id, dados) in eventos {
            let texto = sse(Some(versao), "mudancas", &dados);
            i.enviar(Some(&workspace_id), &texto);
            i.lotes.push_back(Guardado { versao, em: agora, workspace_id, texto });
        }
        while i.lotes.len() > MAX_LOTES || i.lotes.front().is_some_and(|l| agora.duration_since(l.em) > JANELA) {
            i.lotes.pop_front();
        }
    }

    /// Um `PUT /api/dados` trocou tudo: avisa todos e marca o buffer (retomar de antes dele recebe `recarregar`).
    pub fn substituido(&self, versao: u64) {
        let mut i = travar(&self.interno);
        (i.atual, i.marca) = (versao, versao);
        i.lotes.clear();
        i.enviar(None, &sse(Some(versao), "substituido", &json!({"versao": versao})));
    }

    /// Abre a assinatura do workspace a partir de `desde`: reenvia o que está no buffer depois dela ou manda `recarregar` se houver buraco.
    /// `cliente` (opcional) e o autor põem a conexão na presença.
    pub fn assinar(&self, workspace_id: &str, desde: Option<u64>, presente: Option<(&Autor, &str)>) -> (Receiver<String>, Guarda) {
        let (tx, rx) = channel();
        let mut i = travar(&self.interno);
        if let Some(desde) = desde.filter(|d| *d != i.atual) {
            let coberto = desde >= i.marca && desde < i.atual && i.lotes.iter().any(|l| l.versao == desde + 1);
            if coberto {
                for l in i.lotes.iter().filter(|l| l.versao > desde && l.workspace_id == workspace_id) {
                    let _ = tx.send(l.texto.clone());
                }
            } else {
                let _ = tx.send(sse(None, "recarregar", &json!({"versao": i.atual})));
            }
        }
        let id = i.proximo;
        i.proximo += 1;
        i.assinantes.push(Assinante { id, workspace_id: workspace_id.to_string(), tx });
        let entrou = self.presenca_ligada && presente.is_some();
        if let (true, Some((autor, cliente))) = (entrou, presente) {
            i.presenca.entrar(id, autor, cliente, workspace_id);
        }
        drop(i);
        if entrou {
            presenca::avisar(&self.interno, workspace_id);
        }
        (rx, Guarda { interno: self.interno.clone(), id })
    }

    /// Quantas conexões estão abertas (medição e testes).
    pub fn assinantes(&self) -> usize {
        travar(&self.interno).assinantes.len()
    }

    /// `POST /api/presenca`: atualiza a entrada (autor, cliente) e avisa o workspace (no máximo a cada 500 ms).
    pub fn atualizar_presenca(&self, autor: &Autor, cliente: &str, dados: presenca::Onde) {
        if !self.presenca_ligada {
            return;
        }
        let ws = travar(&self.interno).presenca.atualizar(autor, cliente, dados);
        for w in ws {
            presenca::avisar(&self.interno, &w);
        }
    }
}

pub use presenca::Onde;
