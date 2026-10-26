//! As rotas /api/terminal/*: só esta máquina (Host e Origin em loopback; nem MCP_REDE=1 libera, porque elas abrem processo e digitam nele) e
//! JSON em todo POST (página de outro site não manda `application/json` sem o preflight do CORS, que aqui nunca é aceito). Sem token na URL.
//! Nada aqui abre sessão sozinho: `iniciar` só vem do clique confirmado no diálogo "Iniciar a ação" (web/js/iniciar-acao.js).
use super::fluxo::FluxoTerminal;
use super::{citar, Info};
use crate::http::pedido::{falha, Corpo, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::Path;

/// Troca a linha de comando do `claude` por outra (só para os testes de ponta a ponta, que não podem gastar tokens).
pub const VAR_COMANDO: &str = "BLUEPRINT_TERMINAL_COMANDO";

pub fn rota(estado: &Estado, pedido: &Pedido) -> Resposta {
    if !crate::mcp::permissao::so_loopback(&estado.mcp, pedido) {
        return Resposta::texto(403, "text/plain", crate::tr!("the app's terminal only works on this machine", "o terminal do app só funciona nesta máquina"));
    }
    let recurso = pedido.caminho.trim_start_matches("/api/terminal/");
    executar(estado, pedido, recurso).unwrap_or_else(|e| e.resposta())
}

fn texto<'a>(corpo: &'a Value, chave: &str) -> &'a str {
    corpo.get(chave).and_then(Value::as_str).unwrap_or("")
}

fn numero(corpo: &Value, chave: &str, padrao: i16) -> i16 {
    corpo.get(chave).and_then(Value::as_i64).map_or(padrao, |n| n.clamp(1, 1000) as i16)
}

fn sessao(estado: &Estado, id: &str) -> Result<std::sync::Arc<super::Sessao>, ErroHttp> {
    estado.terminais.achar(id).ok_or_else(|| falha(404, crate::tr!("terminal session not found", "sessão do terminal não encontrada")).codigo("sessao-nao-encontrada"))
}

fn executar(estado: &Estado, pedido: &Pedido, recurso: &str) -> Result<Resposta, ErroHttp> {
    if pedido.metodo == "GET" {
        return Ok(match recurso {
            "sessoes" => Resposta::json(200, &json!({"sessoes": estado.terminais.listar()})),
            "fluxo" => {
                let (historico, rx, codigo) = sessao(estado, &pedido.parametro("id").unwrap_or_default())?.assinar();
                let fluxo = FluxoTerminal::novo(historico, rx, codigo);
                Resposta { status: 200, cabecalhos: vec![("content-type".into(), "text/event-stream; charset=utf-8".into()), ("cache-control".into(), "no-store".into())], corpo: Corpo::Fluxo(Box::new(fluxo)) }
            }
            _ => Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado")),
        });
    }
    if pedido.metodo != "POST" {
        return Ok(Resposta::texto(405, "text/plain", crate::tr!("method not allowed", "método não permitido")));
    }
    if !pedido.eh_json() {
        return Ok(Resposta::texto(415, "text/plain", crate::tr!("send application/json", "envie application/json")));
    }
    let corpo: Value = serde_json::from_slice(&pedido.corpo).map_err(|e| falha(400, crate::tr!("invalid JSON: {e}", "JSON inválido: {e}")).codigo("json-invalido"))?;
    let id = texto(&corpo, "id");
    Ok(match recurso {
        "iniciar" => iniciar(estado, &corpo)?,
        "escrever" => {
            let ok = sessao(estado, id)?.escrever(texto(&corpo, "dados").as_bytes());
            Resposta::json(if ok { 200 } else { 410 }, &json!({"ok": ok}))
        }
        "tamanho" => Resposta::json(200, &json!({"ok": sessao(estado, id)?.redimensionar(numero(&corpo, "colunas", 100), numero(&corpo, "linhas", 30))})),
        "fechar" => Resposta::json(200, &json!({"ok": estado.terminais.fechar(id)})),
        _ => Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado")),
    })
}

/// O mesmo "Iniciar com o Claude" de /api/mcp/iniciar (a pasta do blueprint e o prompt da ação vêm de `mcp::iniciar`), mas o lançador abre
/// `blueprint claude "<prompt>" --canal` num pseudoconsole do app em vez de um terminal externo.
fn iniciar(estado: &Estado, corpo: &Value) -> Result<Resposta, ErroHttp> {
    let info = Info {
        titulo: texto(corpo, "titulo").chars().filter(|c| !c.is_control()).take(80).collect(),
        workspace_id: texto(corpo, "workspaceId").into(),
        blueprint_id: texto(corpo, "blueprintId").into(),
        acao_id: texto(corpo, "acaoId").into(),
    };
    let (colunas, linhas) = (numero(corpo, "colunas", 100), numero(corpo, "linhas", 30));
    let aberta = RefCell::new(None);
    let lancar = |pasta: &Path, exe: &Path, prompt: &str| -> Result<&'static str, String> {
        let linha = std::env::var(VAR_COMANDO).ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| format!("{} claude {} --canal", citar(&exe.to_string_lossy()), citar(prompt)));
        let s = estado.terminais.abrir(&linha, pasta, info.clone(), colunas, linhas)?;
        *aberta.borrow_mut() = Some(s.json());
        Ok("app")
    };
    let (status, mut resposta) = crate::mcp::iniciar::iniciar(&estado.dados, corpo, &lancar)?;
    if let Some(s) = aberta.take() {
        resposta["sessao"] = s;
    }
    Ok(Resposta::json(status, &resposta))
}
