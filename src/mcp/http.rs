//! `POST /mcp`: o MCP por HTTP ("streamable HTTP", especificação 2025-06-18) para quem foi convidado e não instalou o app (docs/compartilhar.md, "Remote MCP over HTTP").
//! Só nas portas dos convidados e no modo hospedado: no app local não existe (404). As mesmas ferramentas e instruções do `--mcp`, atendidas dentro deste processo
//! pelas rotas `/api/mcp`, com o Contexto do membro da chave (`Authorization: Bearer bpk_…`) e o papel dele.
//! Decisões: sem sessão (`Mcp-Session-Id` é opcional na especificação) e sem fluxo SSE (GET e DELETE dão 405); sem OAuth e sem `WWW-Authenticate`, para o cliente
//! usar o cabeçalho; `initialize`, `ping` e `tools/list` respondem sem chave (a descrição do servidor, que é pública: o pré-teste de conexão do Claude Code não manda o
//! cabeçalho e trata 401 como fatal), mas `tools/call` exige chave (401). Chave inválida é 401 antes de chegar aqui (acesso/mod.rs). Origin presente = navegador: 403.
use super::stdio::{tratar_via, VERSOES};
use crate::contexto::Contexto;
use crate::http::pedido::{Pedido, Resposta};
use crate::http::Estado;
use serde_json::{json, Value};

fn erro(status: u16, codigo: i32, mensagem: &str) -> Resposta {
    Resposta::json(status, &json!({"jsonrpc": "2.0", "id": null, "error": {"code": codigo, "message": mensagem}}))
}

pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    if ctx.local {
        return Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado"));
    }
    if pedido.cabecalho("origin").is_some() {
        return erro(403, -32000, "browsers are not MCP clients: this endpoint takes no Origin header");
    }
    if pedido.metodo != "POST" {
        return erro(405, -32000, "use POST").com("allow", "POST");
    }
    if pedido.cabecalho("mcp-protocol-version").is_some_and(|v| !VERSOES.contains(&v.trim())) {
        return erro(400, -32600, &format!("unsupported MCP-Protocol-Version (use one of: {})", VERSOES.join(", ")));
    }
    let Ok(msg) = serde_json::from_slice::<Value>(&pedido.corpo) else { return erro(400, -32700, "invalid JSON") };
    if !msg.is_object() {
        return erro(400, -32600, "send one JSON-RPC message per request (no batches)");
    }
    if msg["method"] == "tools/call" && !ctx.autenticado() {
        return erro(401, -32001, "authentication required: send Authorization: Bearer <your key> (the key is in the My key screen)");
    }
    // as ferramentas falam direto com as rotas deste processo, com o contexto do membro; o texto é para o agente, então em inglês
    let direto = |metodo: &str, rota: &str, corpo: Option<&str>| -> Result<(u16, String), String> {
        let cabecalhos = if corpo.is_some() { vec![("content-type".to_string(), "application/json".to_string())] } else { vec![] };
        let interno = Pedido::novo(metodo, &format!("/api/mcp/{rota}"), cabecalhos, corpo.map(|c| c.as_bytes().to_vec()).unwrap_or_default());
        let r = super::rotas::rota_com(&estado.mcp, &estado.dados, &interno, ctx);
        Ok((r.status, String::from_utf8_lossy(r.bytes()).into_owned()))
    };
    match crate::idioma::com(crate::idioma::Idioma::En, || tratar_via(&msg, Some(&direto))) {
        Some(resposta) => Resposta::json(200, &resposta),
        None => Resposta::texto(202, "text/plain", ""), // notificação: aceita, sem corpo
    }
}
