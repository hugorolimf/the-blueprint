//! O acesso do modo hospedado (`blueprint serve --hospedar`): resolve o `Contexto` de cada pedido (cookie da sessão ou chave do MCP), confere
//! HTTPS, Host e Origin, e guarda na memória o que não precisa de banco (tentativas inválidas, sessões encerradas). No modo local nada daqui
//! roda: o despacho entrega `Contexto::local()`. Contrato: docs/colaboracao.md, seções 3 a 5.
pub mod cabecalhos;
pub mod chaves;
pub mod convites;
pub mod filtro;
pub mod limite;
pub mod rotas;
mod rotas_pessoas;
pub mod segredo;
pub mod sessoes;

use crate::contexto::Contexto;
use crate::http::config::Hospedagem;
use crate::http::pedido::{falha, Pedido, Resposta};
use crate::http::Estado;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

/// O estado em memória do acesso (campo `acesso` do `Estado`).
#[derive(Default)]
pub struct Acesso {
    /// quem foi removido ou teve o convite revogado: o SSE dessa pessoa fecha (`sessao_ativa`)
    encerrados: Mutex<HashSet<String>>,
    /// tentativas com segredo inválido (convite)
    pub(crate) limite: Mutex<limite::Limite>,
    /// hash da sessão ou da chave → última vez em que `visto_em`/`usada_em` foi gravado
    vistos: Mutex<HashMap<String, i64>>,
}

impl Acesso {
    pub fn novo() -> Acesso {
        Acesso::default()
    }

    /// A sessão de `ctx` ainda vale? Lida da memória, para um fluxo SSE aberto fechar quem foi removido. No modo local, sempre.
    pub fn sessao_ativa(&self, ctx: &Contexto) -> bool {
        ctx.local || (ctx.autenticado() && !self.encerrados.lock().unwrap_or_else(|e| e.into_inner()).contains(&ctx.usuario))
    }

    /// `usuario` (id do membro ou "visitante:<convite>") perdeu o acesso: as conexões abertas dele fecham.
    pub fn encerrar(&self, usuario: &str) {
        self.encerrados.lock().unwrap_or_else(|e| e.into_inner()).insert(usuario.to_string());
    }

    /// Hora de gravar que a sessão (ou a chave) foi vista? No máximo uma vez a cada 10 min por hash.
    pub(crate) fn anotar_visto(&self, hash: &str, agora_ms: i64) -> bool {
        let mut v = self.vistos.lock().unwrap_or_else(|e| e.into_inner());
        if v.get(hash).is_some_and(|t| agora_ms - t < sessoes::VISTO_A_CADA_MS) {
            return false;
        }
        if v.len() > 10_000 {
            v.clear();
        }
        v.insert(hash.to_string(), agora_ms);
        true
    }
}

/// As rotas que só existem no app local (abrem processo, falam com a janela ou com os hooks do Claude Code desta máquina, ligam o
/// compartilhamento). Valem no modo hospedado e na porta dos convidados do compartilhamento.
pub fn so_local(caminho: &str) -> bool {
    let mcp = caminho.strip_prefix("/api/mcp/").unwrap_or("");
    caminho.starts_with("/api/terminal/")
        || caminho.starts_with("/api/repositorio/")
        || caminho.starts_with("/api/atualizacao")
        || caminho.starts_with("/api/compartilhar")
        || caminho == "/api/equipes"
        || caminho.starts_with("/api/equipes/")
        || caminho == "/api/app/mostrar"
        || matches!(mcp, "iniciar" | "apresentar" | "apresentacao/pedido" | "sessoes" | "pastas")
        || mcp.starts_with("conversa")
        || mcp.starts_with("hook/")
        || mcp.starts_with("pasta/")
}

/// Os hooks do Claude Code que o Claude de um membro manda pelo `blueprint hook <evento> --equipe` (progresso, subagentes, lembrete do Stop).
/// Só existem com a chave do membro (`Authorization`); sem ela continuam só locais. O status e as sessões nunca saem do app local.
pub fn hook_de_membro(caminho: &str) -> bool {
    matches!(caminho.strip_prefix("/api/mcp/").unwrap_or(""), "hook/sessao" | "hook/edicao" | "hook/subagente" | "hook/parada" | "hook/fim")
}

/// O despacho do modo hospedado: confere HTTPS e Host, resolve o Contexto, confere a Origin e entrega a `proximo` (o roteador de sempre);
/// toda resposta sai com os cabeçalhos de segurança.
pub fn despachar(estado: &Estado, pedido: &Pedido, h: &Hospedagem, proximo: impl Fn(&Contexto) -> Resposta) -> Resposta {
    let caminho = pedido.caminho.as_str();
    let resposta = (|| {
        cabecalhos::conferir(h, pedido)?;
        let autorizacao = pedido.cabecalho("authorization").filter(|_| caminho.starts_with("/api/mcp/") || caminho == "/mcp");
        if so_local(caminho) && !(hook_de_membro(caminho) && autorizacao.is_some()) {
            return Err(falha(404, crate::tr!("this route only exists in the local app", "esta rota só existe no app local")).codigo("so-local"));
        }
        let agora = crate::tempo::agora_ms();
        let ctx = match autorizacao {
            Some(valor) => {
                let ctx = match chaves::do_cabecalho(valor) {
                    Some(chave) => chaves::contexto(&mut estado.dados.lock().unwrap_or_else(|e| e.into_inner()), &estado.acesso, chave, agora)?,
                    None => None,
                };
                ctx.ok_or_else(|| falha(401, crate::tr!("invalid or revoked key", "chave inválida ou revogada")).codigo("chave-invalida"))?
            }
            None => {
                let ctx = match pedido.cookie(sessoes::COOKIE).filter(|v| segredo::forma_valida(v)) {
                    Some(valor) => sessoes::contexto(&mut estado.dados.lock().unwrap_or_else(|e| e.into_inner()), &estado.acesso, &segredo::hash(valor), agora)?,
                    None => None,
                };
                // navegador: todo pedido que muda algo vem da página pública (a chave do agente não é mandada pelo navegador sozinho)
                if !matches!(pedido.metodo.as_str(), "GET" | "HEAD") && caminho != "/mcp" && !cabecalhos::origem_ok(h, pedido) { // /mcp não é de navegador: a rota recusa qualquer Origin
                    return Err(falha(403, crate::tr!("invalid origin", "origem inválida")).codigo("origem-invalida"));
                }
                ctx.unwrap_or_else(Contexto::anonimo)
            }
        };
        let publica = !caminho.starts_with("/api/") || caminho == "/api/sessao" || caminho.starts_with("/api/convite/");
        if !publica && !ctx.autenticado() {
            return Err(falha(401, crate::tr!("sign in through an invite link", "entre por um link de convite")).codigo("sem-sessao"));
        }
        Ok(proximo(&ctx))
    })()
    .unwrap_or_else(|e| e.resposta());
    cabecalhos::proteger(h, caminho, resposta)
}

#[cfg(test)]
mod tests;
