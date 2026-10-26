//! A API do compartilhamento, só na porta local e só desta máquina (Host e Origin em loopback, como /api/atualizacao); todo POST exige
//! JSON. Pela porta dos convidados e no modo hospedado ela não existe (`acesso::so_local`). O estado novo chega à página pelo SSE do MCP,
//! evento `compartilhar` (o mesmo JSON do GET).
use crate::contexto::Contexto;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use crate::mcp::permissao::so_loopback;
use serde_json::Value;

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    resultado(estado, pedido).unwrap_or_else(|e| e.resposta())
}

fn resultado(estado: &Estado, pedido: &Pedido) -> Result<Resposta, ErroHttp> {
    if !so_loopback(&estado.mcp, pedido) {
        return Err(falha(403, crate::tr!("sharing is only controlled from this machine", "o compartilhamento só se controla nesta máquina")).codigo("so-local"));
    }
    let c = &estado.compartilhar;
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/compartilhar") => Ok(Resposta::json(200, &c.json())),
        ("POST", _) if !pedido.eh_json() => Err(falha(415, crate::tr!("send application/json", "envie application/json")).codigo("json")),
        ("POST", "/api/compartilhar/ligar") => {
            let corpo: Value = serde_json::from_slice(&pedido.corpo).map_err(|_| falha(400, crate::tr!("invalid JSON", "JSON inválido")).codigo("json-invalido"))?;
            let ws = corpo.get("workspaceId").and_then(Value::as_str).unwrap_or("");
            if ws.is_empty() || ws == "*" {
                return Err(falha(400, crate::tr!("send the workspaceId to share", "envie o workspaceId a compartilhar")).codigo("falta-workspace"));
            }
            let existe = crate::acesso::filtro::nome_do_workspace(estado.dados.lock().unwrap_or_else(|e| e.into_inner()).ler_tudo(&Contexto::local())?, ws).is_some();
            if !existe {
                return Err(falha(404, crate::tr!("workspace not found", "workspace não encontrado")).codigo("workspace-inexistente"));
            }
            c.ligar(ws)?;
            Ok(Resposta::json(202, &c.json()))
        }
        ("POST", "/api/compartilhar/desligar") => {
            // com workspaceId, tira só esse (o túnel cai quando não sobra nenhum); sem, desliga tudo
            match serde_json::from_slice::<Value>(&pedido.corpo).ok().as_ref().and_then(|v| v.get("workspaceId")?.as_str()) {
                Some(ws) => c.desligar_workspace(ws),
                None => c.desligar(),
            }
            Ok(Resposta::json(200, &c.json()))
        }
        ("GET" | "POST", _) => Err(falha(404, crate::tr!("not found", "não encontrado"))),
        _ => Err(falha(405, crate::tr!("method not allowed", "método não permitido")).codigo("metodo")),
    }
}
