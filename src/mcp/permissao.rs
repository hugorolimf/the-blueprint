//! Quem pode falar com /api/mcp. No app local: só esta máquina (Host e Origin em loopback), salvo MCP_REDE=1, sem token. No modo hospedado
//! a sessão ou a chave do membro substituem o loopback e cada recurso confere o papel (docs/colaboracao.md, seção 3).
use super::Mcp;
use crate::contexto::{Acao, Contexto};
use crate::http::pedido::{falha, ErroHttp, Pedido};
use serde_json::Value;

const LOOPBACK: [&str; 4] = ["127.0.0.1", "localhost", "::1", "[::1]"];

/// `host:porta` → `host` (o `replace(/:\d+$/, "")` do Node; endereços IPv6 entre colchetes ficam inteiros)
fn sem_porta(host: &str) -> &str {
    match host.rsplit_once(':') {
        Some((h, porta)) if !porta.is_empty() && porta.bytes().all(|c| c.is_ascii_digit()) => h,
        _ => host,
    }
}

pub fn permitido(mcp: &Mcp, pedido: &Pedido) -> bool {
    if mcp.rede {
        return true;
    }
    so_loopback(mcp, pedido)
}

/// Host e Origin em loopback, com ou sem MCP_REDE: as rotas que abrem processo (iniciar) nunca saem desta máquina.
pub(crate) fn so_loopback(mcp: &Mcp, pedido: &Pedido) -> bool {
    let host = pedido.cabecalho("host").unwrap_or("");
    if !LOOPBACK.contains(&sem_porta(host)) || !LOOPBACK.contains(&mcp.host.as_str()) {
        return false;
    }
    pedido.cabecalho("origin").is_none_or(|o| o == format!("http://{host}"))
}

fn proibido() -> ErroHttp {
    falha(403, crate::tr!("you do not have permission for this", "você não tem permissão para isso"))
}

/// `blueprint` vazio = o workspace inteiro (proposta de blueprint novo).
pub fn alvo<'a>(ws: &'a str, bp: &'a str) -> (&'a str, Option<&'a str>) {
    (ws, (!bp.is_empty()).then_some(bp))
}

/// Modo hospedado: o papel alcança o recurso? `corpo` é o JSON do POST (os ids do alvo vêm dele); `dados` é o arquivo inteiro, para achar
/// de qual blueprint é um protótipo. A fila e os agentes são filtrados depois (`pode_ler_item`); o resto lê dos dados já filtrados.
pub fn exige_hospedado(ctx: &Contexto, recurso: &str, pedido: &Pedido, corpo: &Value, dados: &Value) -> Result<(), ErroHttp> {
    // visitante não usa o MCP: nem fila, nem agentes, nem linha do tempo
    if ctx.visitante || !ctx.autenticado() {
        return Err(proibido());
    }
    let texto = |k: &str| corpo.get(k).and_then(Value::as_str).unwrap_or("");
    let pode = match (pedido.metodo.as_str(), recurso) {
        ("GET", "linha-do-tempo") => {
            let (ws, bp) = (pedido.parametro("workspace").unwrap_or_default(), pedido.parametro("blueprint").unwrap_or_default());
            let (ws, bp) = alvo(&ws, &bp);
            ctx.pode(Acao::Ler, ws, bp)
        }
        ("GET", _) => true,
        ("POST", "proposta" | "registro" | "evento" | "aviso" | "prototipo") => {
            let (ws, bp) = alvo(texto("workspaceId"), texto("blueprintId"));
            ctx.pode(Acao::Gravar, ws, bp)
        }
        ("POST", "prototipo/remover") => match crate::acesso::filtro::blueprint_do_prototipo(dados, texto("prototipoId")) {
            Some((ws, bp)) => ctx.pode(Acao::Gravar, &ws, Some(&bp)),
            // já saiu dos dados (a página apaga antes de pedir o arquivo): basta gravar em algum lugar
            None => ctx.pode_editar(),
        },
        // `fila/ack` confere item por item (`pode_gravar_item`)
        ("POST", "fila/ack") => ctx.pode_editar(),
        _ => true,
    };
    if pode { Ok(()) } else { Err(proibido()) }
}

/// Um item da fila (ou um agente) tem `workspaceId` e `blueprintId`: o contexto lê esse alvo?
pub fn pode_ler_item(ctx: &Contexto, item: &Value) -> bool {
    let texto = |k: &str| item.get(k).and_then(Value::as_str).unwrap_or("");
    let (ws, bp) = alvo(texto("workspaceId"), texto("blueprintId"));
    ctx.local || (!ctx.visitante && ctx.pode(Acao::Ler, ws, bp))
}

/// O SSE do MCP para quem não é o usuário local (modo hospedado e convidados do compartilhamento): nunca o texto que o usuário local
/// manda às sessões dele (`iniciar-acao`, `comentario-plano`, `conversa-usuario`) nem o estado do app (atualização, compartilhamento);
/// o que traz `workspaceId`, só se o papel lê esse alvo.
pub fn evento_visivel(ctx: &Contexto, e: &super::Evento) -> bool {
    if matches!(e.nome, "iniciar-acao" | "comentario-plano" | "conversa-usuario" | "atualizacao" | "compartilhar") {
        return false;
    }
    e.dados.as_ref().filter(|d| d.get("workspaceId").is_some()).is_none_or(|d| pode_ler_item(ctx, d))
}

pub fn pode_gravar_item(ctx: &Contexto, item: &Value) -> bool {
    let texto = |k: &str| item.get(k).and_then(Value::as_str).unwrap_or("");
    let (ws, bp) = alvo(texto("workspaceId"), texto("blueprintId"));
    ctx.pode(Acao::Gravar, ws, bp)
}
