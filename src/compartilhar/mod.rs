//! Compartilhar um workspace pelo próprio app (docs/compartilhar.md; desenho e desvios em docs/colaboracao.md, "Compartilhar pelo app").
//! Ao ligar, o app abre a porta dos convidados (um segundo listener em 127.0.0.1, porta livre) e um quick tunnel da Cloudflare aponta para
//! ela. Tudo o que chega por essa porta é convidado: resolvido como no modo hospedado, com a URL do túnel como URL pública, e preso ao
//! workspace compartilhado. A porta local (3000) continua do usuário local, dono implícito. Desligado, nada roda: sem listener, sem
//! processo e sem thread.
pub mod fluxo;
pub mod rotas;
pub mod tunel;

use crate::contexto::{Acesso, Contexto, Papel};
use crate::http::config::Hospedagem;
use crate::http::pedido::{falha, ErroHttp};
use crate::http::Estado;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Fase {
    #[default]
    Desligado,
    Baixando,
    Ligando,
    Ligado,
    Erro,
}

impl Fase {
    pub fn texto(self) -> &'static str {
        match self {
            Fase::Desligado => "desligado",
            Fase::Baixando => "baixando",
            Fase::Ligando => "ligando",
            Fase::Ligado => "ligado",
            Fase::Erro => "erro",
        }
    }
}

#[derive(Default)]
struct Interno {
    fase: Fase,
    /// os workspaces abertos aos convidados, todos pelo mesmo túnel (cada convite continua preso ao seu)
    workspaces: Vec<String>,
    url: Option<String>,
    erro: Option<String>,
    porta: Option<u16>,
    hospedagem: Option<Hospedagem>,
    ouvinte: Option<Arc<tiny_http::Server>>,
    tunel: Option<tunel::Tunel>,
    /// sobe a cada ligar e desligar: o que termina em segundo plano só vale se ainda for da mesma vez
    geracao: u64,
}

/// O estado do compartilhamento (campo `compartilhar` do `Estado`), só memória.
pub struct Compartilhar {
    interno: Mutex<Interno>,
    estado: OnceLock<Weak<Estado>>,
    /// BLUEPRINT_TUNEL_COMANDO: um programa no lugar do cloudflared (testes e e2e sem rede); recebe os mesmos argumentos e imprime a URL
    comando: Mutex<Option<PathBuf>>,
}

impl Default for Compartilhar {
    fn default() -> Self {
        Compartilhar::novo()
    }
}

impl Compartilhar {
    pub fn novo() -> Compartilhar {
        let comando = std::env::var_os("BLUEPRINT_TUNEL_COMANDO").filter(|v| !v.is_empty()).map(PathBuf::from);
        Compartilhar { interno: Mutex::default(), estado: OnceLock::new(), comando: Mutex::new(comando) }
    }

    /// O servidor entrega o próprio `Estado` (a porta dos convidados precisa dele para atender em outra thread).
    pub fn definir_estado(&self, estado: Weak<Estado>) {
        let _ = self.estado.set(estado);
    }

    /// Testes: troca o programa do túnel.
    pub fn trocar_comando(&self, programa: Option<PathBuf>) {
        *self.comando.lock().unwrap_or_else(|e| e.into_inner()) = programa;
    }

    fn travar(&self) -> MutexGuard<'_, Interno> {
        self.interno.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// `GET /api/compartilhar`: `{estado, url?, workspaces?, workspaceId?, erro?, porta?}` (`workspaceId` é o primeiro, para quem só lê um).
    pub fn json(&self) -> Value {
        let i = self.travar();
        let mut v = json!({"estado": i.fase.texto()});
        if i.fase != Fase::Desligado {
            v["workspaces"] = json!(i.workspaces);
            v["workspaceId"] = json!(i.workspaces.first());
        }
        for (chave, valor) in [("url", i.url.as_ref().map(|u| json!(u))), ("erro", i.erro.as_ref().map(|e| json!(e))), ("porta", i.porta.map(|p| json!(p)))] {
            if let Some(valor) = valor {
                v[chave] = valor;
            }
        }
        v
    }

    /// Baixando, ligando ou ligado (o Atualizar recusa).
    pub fn ligado(&self) -> bool {
        matches!(self.travar().fase, Fase::Baixando | Fase::Ligando | Fase::Ligado)
    }

    /// A URL do túnel como configuração do modo hospedado e os workspaces compartilhados, só com o compartilhamento ligado.
    pub fn porta_aberta(&self) -> Option<(Hospedagem, Vec<String>)> {
        let i = self.travar();
        Some((i.hospedagem.clone().filter(|_| i.fase == Fase::Ligado)?, i.workspaces.clone()))
    }

    fn avisar(&self) {
        if let Some(e) = self.estado.get().and_then(Weak::upgrade) {
            e.mcp.notificar("compartilhar", Some(self.json()));
        }
    }

    /// Começa a ligar (o resto acontece em segundo plano e chega por SSE). Com o túnel já ligado (ou ligando), só inclui o workspace.
    pub fn ligar(&self, workspace: &str) -> Result<(), ErroHttp> {
        let estado = self.estado.get().and_then(Weak::upgrade).ok_or_else(|| falha(503, crate::tr!("the server is not ready", "o servidor não está pronto")))?;
        let geracao = {
            let mut i = self.travar();
            if matches!(i.fase, Fase::Baixando | Fase::Ligando | Fase::Ligado) {
                if !i.workspaces.iter().any(|w| w == workspace) {
                    i.workspaces.push(workspace.to_string());
                }
                drop(i);
                self.avisar();
                return Ok(());
            }
            let g = i.geracao + 1;
            *i = Interno { fase: Fase::Ligando, workspaces: vec![workspace.to_string()], geracao: g, ..Interno::default() };
            g
        };
        self.avisar();
        let comando = self.comando.lock().unwrap_or_else(|e| e.into_inner()).clone();
        std::thread::spawn(move || {
            let c = &estado.compartilhar;
            if let Err(motivo) = c.subir(&estado, geracao, comando) {
                c.falhou(geracao, motivo);
            }
        });
        Ok(())
    }

    /// Muda a fase se ainda for a mesma vez; devolve se mudou.
    fn fase_se(&self, geracao: u64, fase: Fase) -> bool {
        let mudou = {
            let mut i = self.travar();
            let mesma = i.geracao == geracao;
            if mesma && i.fase != fase {
                i.fase = fase;
            }
            mesma
        };
        if mudou {
            self.avisar();
        }
        mudou
    }

    fn subir(&self, estado: &Arc<Estado>, geracao: u64, comando: Option<PathBuf>) -> Result<(), String> {
        let programa = match comando {
            Some(p) => p,
            None if cfg!(windows) => tunel::preparar(&tunel::pasta_bin(), tunel::SHA256, &|| {
                self.fase_se(geracao, Fase::Baixando);
            }, &tunel::baixar_curl)?,
            None => return Err(crate::tr!("sharing through the app only works on Windows for now", "compartilhar pelo app só funciona no Windows por enquanto")),
        };
        if !self.fase_se(geracao, Fase::Ligando) {
            return Ok(());
        }
        let (ouvinte, porta) = crate::http::escutar_convidados(estado).map_err(|e| e.to_string())?;
        {
            let mut i = self.travar();
            if i.geracao != geracao {
                ouvinte.unblock();
                return Ok(());
            }
            (i.ouvinte, i.porta) = (Some(ouvinte), Some(porta));
        }
        let fraco = Arc::downgrade(estado);
        let ao_cair = move || {
            if let Some(e) = fraco.upgrade() {
                e.compartilhar.falhou(geracao, crate::tr!("the tunnel stopped; turn sharing on again", "o túnel caiu; ligue o compartilhamento de novo"));
            }
        };
        let (t, url) = tunel::Tunel::abrir(&programa, porta, ao_cair)?;
        let h = Hospedagem::de(&url, "127.0.0.1", None, false)?;
        let sobra = {
            let mut i = self.travar();
            if i.geracao != geracao {
                Some(t)
            } else {
                (i.fase, i.url, i.hospedagem, i.tunel) = (Fase::Ligado, Some(url), Some(h), Some(t));
                None
            }
        };
        match sobra {
            Some(t) => drop(t), // desligaram enquanto o túnel subia
            None => self.avisar(),
        }
        Ok(())
    }

    /// Algo deu errado (ou o túnel caiu): fecha a porta e o túnel e mostra o motivo, se ainda for a mesma vez.
    fn falhou(&self, geracao: u64, motivo: String) {
        let (ouvinte, t) = {
            let mut i = self.travar();
            if i.geracao != geracao {
                return;
            }
            let (o, t) = (i.ouvinte.take(), i.tunel.take());
            (i.fase, i.erro, i.url, i.hospedagem, i.porta) = (Fase::Erro, Some(motivo), None, None, None);
            (o, t)
        };
        if let Some(o) = ouvinte {
            o.unblock();
        }
        drop(t);
        self.avisar();
    }

    /// Tira um workspace do compartilhamento; sem nenhum, derruba o túnel. Os convidados dele perdem o acesso na hora (`so_do_workspace`).
    pub fn desligar_workspace(&self, workspace: &str) {
        let vazio = {
            let mut i = self.travar();
            i.workspaces.retain(|w| w != workspace);
            i.workspaces.is_empty()
        };
        if vazio {
            self.desligar();
        } else {
            self.avisar();
        }
    }

    /// Derruba o túnel e fecha a porta dos convidados (clique em Desligar; ir para a bandeja não desliga).
    pub fn desligar(&self) {
        let (ouvinte, t, mudou) = {
            let mut i = self.travar();
            let mudou = i.fase != Fase::Desligado;
            let (o, t) = (i.ouvinte.take(), i.tunel.take());
            let g = i.geracao + 1;
            *i = Interno { geracao: g, ..Interno::default() };
            (o, t, mudou)
        };
        if let Some(o) = ouvinte {
            o.unblock();
        }
        drop(t);
        if mudou {
            self.avisar();
        }
    }
}

/// Quem chega pela porta dos convidados só alcança os workspaces compartilhados (nem o `'*'` de um dono do modo hospedado passa).
pub fn so_do_workspace(ctx: &Contexto, workspaces: &[String]) -> Contexto {
    let mut c = ctx.clone();
    c.local = false;
    c.acessos.retain(|a| workspaces.contains(&a.workspace_id));
    c.papel = c.acessos.iter().map(|a| a.papel).max().unwrap_or(Papel::Visitante);
    c
}

/// O usuário local administrando pessoas e convites enquanto compartilha: dono só dos workspaces compartilhados (convite de outro
/// workspace ou de `'*'` é recusado pelas regras de sempre).
pub fn dono_local(workspaces: &[String]) -> Contexto {
    Contexto::membro("local", "local", workspaces.iter().map(|w| Acesso::novo(w, "", Papel::Dono)).collect())
}

/// A configuração do modo hospedado que vale agora: a do `serve --hospedar` ou a do compartilhamento ligado.
pub fn hospedagem(estado: &Estado) -> Option<Hospedagem> {
    estado.config.hospedagem.clone().or_else(|| estado.compartilhar.porta_aberta().map(|(h, _)| h))
}

#[cfg(test)]
mod tests;
