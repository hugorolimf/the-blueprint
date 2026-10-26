//! Sincronização entre pessoas (docs/colaboracao.md, seção 6): `POST /api/mudancas` aplica operações por item, `GET /api/eventos` as leva por SSE
//! a quem lê o workspace ou o blueprint, com `id:` = versão e retomada por `Last-Event-ID`, e `POST /api/presenca` diz quem está onde. Tudo o que
//! é desta frente na memória mora aqui: o buffer dos últimos lotes (até `MAX_LOTES` ou `JANELA`), os assinantes (cada um com o seu Contexto,
//! que filtra cada evento) e a presença. Nada roda em repouso: cada conexão dorme no canal dela e o batimento é o do SSE (25 s).
mod eventos;
pub mod operacoes;
mod presenca;
pub mod rotas;

#[cfg(test)]
mod tests;

pub use eventos::Fluxo;

use crate::contexto::{Acao, Contexto};
use crate::dados::Autor;
use crate::erro::Resultado;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

pub const MAX_LOTES: usize = 1000;
pub const JANELA: Duration = Duration::from_secs(15 * 60);

// ---- os pontos únicos de autoria e permissão (o Contexto vem resolvido do despacho: local, membro ou visitante) ----

/// Quem assina as operações e a presença.
pub fn autor(ctx: &Contexto) -> Autor {
    Autor { id: ctx.usuario.clone(), nome: ctx.nome.clone() }
}

/// Pode gravar no workspace/blueprint? `administrar` (criar ou apagar workspace, mudar o workspace raiz) é só do dono de todos (`'*'`).
pub fn exige_gravar(ctx: &Contexto, administrar: bool, workspace: &str, blueprint: Option<&str>) -> Resultado<()> {
    if administrar { ctx.exige(Acao::Administrar, "*", None) } else { ctx.exige(Acao::Gravar, workspace, blueprint) }
}

/// Pode ler o workspace (blueprint None) ou só o blueprint? Também filtra cada evento para cada assinante.
pub fn pode_ler(ctx: &Contexto, workspace: &str, blueprint: Option<&str>) -> bool {
    ctx.pode(Acao::Ler, workspace, blueprint)
}

/// Um evento `mudancas` de um lote: o workspace e o blueprint tocados ("" = o próprio workspace) e se ele tem um `mover` que cruza
/// blueprints (para o visitante, que só lê um blueprint, vira `recarregar`).
pub struct Evento {
    pub workspace_id: String,
    pub blueprint_id: String,
    pub dados: Value,
    pub cruza: bool,
}

struct Guardado {
    versao: u64,
    em: Instant,
    workspace_id: String,
    blueprint_id: String,
    texto: String,
    /// o mesmo evento para o visitante: sem `agente.comentarios`, ou `recarregar` se cruza blueprints
    visitante: String,
}

pub(crate) struct Assinante {
    id: u64,
    workspace_id: String,
    ctx: Contexto,
    tx: Sender<String>,
}

impl Assinante {
    fn le(&self, ws: &str, bp: &str) -> bool {
        ws == self.workspace_id && pode_ler(&self.ctx, ws, (!bp.is_empty()).then_some(bp))
    }
    fn texto<'a>(&self, g: &'a Guardado) -> &'a str {
        if self.ctx.visitante { &g.visitante } else { &g.texto }
    }
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
    /// Manda `texto` a quem passa no filtro; quem não recebe mais (a conexão caiu) sai da lista.
    fn enviar(&mut self, filtro: impl Fn(&Assinante) -> bool, texto: &str) {
        self.assinantes.retain(|a| !filtro(a) || a.tx.send(texto.to_string()).is_ok());
    }
}

impl Sincronia {
    pub fn nova(versao: u64, presenca_ligada: bool) -> Sincronia {
        let interno = Interno { lotes: VecDeque::new(), atual: versao, marca: versao, assinantes: Vec::new(), proximo: 1, presenca: presenca::Presencas::default() };
        Sincronia { interno: Arc::new(Mutex::new(interno)), presenca_ligada }
    }

    /// Um lote gravado: um evento `mudancas` por blueprint tocado, guardado no buffer e enviado a quem lê aquele workspace/blueprint.
    pub fn publicar(&self, versao: u64, eventos: Vec<Evento>) {
        let mut i = travar(&self.interno);
        i.atual = versao;
        let agora = Instant::now();
        for e in eventos {
            let visitante = if e.cruza {
                sse(None, "recarregar", &json!({"versao": versao}))
            } else {
                let mut d = e.dados.clone();
                crate::acesso::filtro::para_visitante(&mut d);
                sse(Some(versao), "mudancas", &d)
            };
            let g = Guardado { versao, em: agora, texto: sse(Some(versao), "mudancas", &e.dados), visitante, workspace_id: e.workspace_id, blueprint_id: e.blueprint_id };
            i.assinantes.retain(|a| !a.le(&g.workspace_id, &g.blueprint_id) || a.tx.send(a.texto(&g).to_string()).is_ok());
            i.lotes.push_back(g);
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
        i.enviar(|_| true, &sse(Some(versao), "substituido", &json!({"versao": versao})));
    }

    /// Fecha as conexões de quem perdeu o acesso (`ativa` = `Acesso::sessao_ativa`): manda `sessao-encerrada` e solta o canal, e o fluxo
    /// termina. As rotas chamam antes de abrir uma conexão e antes de publicar, então nenhum evento chega a quem foi removido.
    pub fn podar(&self, ativa: impl Fn(&Contexto) -> bool) {
        let saiu: Vec<String> = {
            let mut i = travar(&self.interno);
            let (fora, ficam): (Vec<Assinante>, Vec<Assinante>) = std::mem::take(&mut i.assinantes).into_iter().partition(|a| !ativa(&a.ctx));
            i.assinantes = ficam;
            let mut saiu = Vec::new();
            for a in fora {
                let _ = a.tx.send(sse(None, "sessao-encerrada", &json!({})));
                saiu.extend(i.presenca.sair(a.id));
            }
            saiu
        };
        for ws in saiu {
            presenca::avisar(&self.interno, &ws);
        }
    }

    /// Abre a assinatura do workspace a partir de `desde`: reenvia o que está no buffer depois dela (só o que `ctx` lê) ou manda
    /// `recarregar` se houver buraco. Com `cliente` a conexão entra na presença (nunca a do visitante).
    pub fn assinar(&self, workspace_id: &str, desde: Option<u64>, ctx: &Contexto, cliente: Option<&str>) -> (Receiver<String>, Guarda) {
        let (tx, rx) = channel();
        let mut i = travar(&self.interno);
        let id = i.proximo;
        i.proximo += 1;
        let a = Assinante { id, workspace_id: workspace_id.to_string(), ctx: ctx.clone(), tx };
        if let Some(desde) = desde.filter(|d| *d != i.atual) {
            let coberto = desde >= i.marca && desde < i.atual && i.lotes.iter().any(|l| l.versao == desde + 1);
            if coberto {
                for l in i.lotes.iter().filter(|l| l.versao > desde && a.le(&l.workspace_id, &l.blueprint_id)) {
                    let _ = a.tx.send(a.texto(l).to_string());
                }
            } else {
                let _ = a.tx.send(sse(None, "recarregar", &json!({"versao": i.atual})));
            }
        }
        i.assinantes.push(a);
        let entrou = self.presenca_ligada && !ctx.visitante && cliente.is_some();
        if let (true, Some(cliente)) = (entrou, cliente) {
            i.presenca.entrar(id, &autor(ctx), cliente, workspace_id);
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
