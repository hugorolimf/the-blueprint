//! As rotas /api/mcp/* (porte de `rota` do mcp.mjs): só esta máquina, JSON em todo POST, e erros sempre como `{ "erro": "..." }`.
use super::acoes::{self, id_prototipo_valido};
use super::hooks;
use super::permissao::{permitido, so_loopback};
use super::{FluxoSse, Mcp};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, Corpo, ErroHttp, Pedido, Resposta};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Mutex;

fn nao_encontrado() -> Resposta {
    Resposta::texto(404, "text/plain", "não encontrado")
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido) -> Resposta {
    if !permitido(mcp, pedido) {
        return Resposta::texto(403, "text/plain", "o MCP do The Blueprint só funciona nesta máquina (MCP_REDE=1 libera a rede)");
    }
    let recurso = pedido.caminho.trim_start_matches("/api/mcp/");
    if recurso == "iniciar" && !so_loopback(mcp, pedido) {
        return Resposta::texto(403, "text/plain", "iniciar uma ação só funciona nesta máquina (nem MCP_REDE=1 libera)");
    }
    match executar(mcp, dados, pedido, recurso) {
        Ok(resposta) => resposta,
        Err(e) => Resposta::json(e.status, &json!({"erro": e.mensagem})),
    }
}

/// Reenvia um registro guardado com o app fechado (`pendentes.rs`) à mesma rota POST que o agente usaria ao vivo, com a mesma validação. Não confere Host nem
/// Origin: quem escreveu o arquivo já tinha acesso à pasta de dados do usuário.
pub fn reenviar(mcp: &Mcp, dados: &Mutex<Dados>, recurso: &str, corpo: &Value) -> (u16, Value) {
    let pedido = Pedido::novo("POST", &format!("/api/mcp/{recurso}"), vec![("content-type".into(), "application/json".into())], corpo.to_string().into_bytes());
    let resposta = executar(mcp, dados, &pedido, recurso).unwrap_or_else(|e| Resposta::json(e.status, &json!({"erro": e.mensagem})));
    (resposta.status, serde_json::from_slice(resposta.bytes()).unwrap_or(Value::Null))
}

/// Roda `f` com o data.json inteiro em mãos (a cópia em memória da camada de dados).
fn com_dados<T>(dados: &Mutex<Dados>, f: impl FnOnce(&Value) -> Result<T, ErroHttp>) -> Result<T, ErroHttp> {
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner());
    f(d.ler_tudo(&Contexto::local())?)
}

fn executar(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str) -> Result<Resposta, ErroHttp> {
    if pedido.metodo == "GET" {
        return Ok(match recurso {
            "blueprints" => Resposta::json(200, &com_dados(dados, |d| Ok(acoes::blueprints(d)))?),
            "blueprint" => {
                let p = |nome: &str| pedido.parametro(nome).unwrap_or_default();
                Resposta::json(200, &com_dados(dados, |d| acoes::blueprint(d, &p("workspace"), &p("blueprint"), pedido.parametro("acao").as_deref()))?)
            }
            "pastas" => Resposta::json(200, &json!({"pastas": hooks::travado(dados).listar_pastas(&Contexto::local())?})),
            "fila" => Resposta::json(200, &json!({"itens": mcp.itens()})),
            "sessoes" => Resposta::json(200, &mcp.sessoes_do_blueprint(&pedido.parametro("workspace").unwrap_or_default(), &pedido.parametro("blueprint").unwrap_or_default())), // as sessões do Claude abertas no blueprint
            "agentes" => Resposta::json(200, &mcp.agentes()), // quem está trabalhando agora (presença, só memória)
            "linha-do-tempo" => Resposta::json(200, &super::linha_do_tempo::listar(dados, pedido)?), // o que o agente fez na ação, do mais recente ao mais antigo
            "info" => Resposta::json(200, &json!({"arquivo": mcp.arquivo, "url": mcp.url, "exe": true})), // a página mostra como registrar o servidor MCP no agente
            "eventos" => {
                let mut fluxo = FluxoSse::novo(mcp.assinar());
                // `?canal=&cwd=`: a conexão é de uma sessão com o canal ligado e ela é destinatária enquanto durar
                if let (Some(id), Some(cwd)) = (pedido.parametro("canal"), pedido.parametro("cwd")) {
                    if let Some(guarda) = mcp.abrir_canal(&id, &cwd) {
                        fluxo = fluxo.com_guarda(guarda);
                    }
                }
                Resposta { status: 200, cabecalhos: vec![("content-type".into(), "text/event-stream; charset=utf-8".into()), ("cache-control".into(), "no-store".into())], corpo: Corpo::Fluxo(Box::new(fluxo)) }
            }
            _ => nao_encontrado(),
        });
    }
    if pedido.metodo != "POST" {
        return Ok(Resposta::texto(405, "text/plain", "método não permitido"));
    }
    if !pedido.eh_json() {
        return Ok(Resposta::texto(415, "text/plain", "envie application/json"));
    }
    let corpo: Value = if pedido.corpo.is_empty() { json!({}) } else { serde_json::from_slice(&pedido.corpo).map_err(|e| falha(400, e.to_string()))? };
    if recurso.starts_with("hook/") {
        mcp.sinal_sessao(hooks::texto(&corpo, "session_id")); // qualquer hook da sessão prova que ela está viva (expiração em sessoes.rs)
    }
    Ok(match recurso {
        "proposta" => {
            let (status, resposta) = com_dados(dados, |d| acoes::proposta(mcp, d, &corpo))?;
            Resposta::json(status, &resposta)
        }
        "registro" => {
            let (status, resposta) = com_dados(dados, |d| super::registro::registro(mcp, d, &corpo))?;
            if status == 200 {
                super::lembrete::registrado(mcp, &hooks::texto(&corpo, "workspaceId"), &hooks::texto(&corpo, "blueprintId"));
            }
            Resposta::json(status, &resposta)
        }
        "evento" => {
            let r = super::linha_do_tempo::evento(mcp, dados, &corpo)?; // acoes::evento e, se for de uma ação, a linha do tempo
            // um progresso com plano também conta como registro: some o lembrete da sessão
            if corpo.get("planoId").and_then(Value::as_str).is_some_and(|p| !p.is_empty()) {
                super::lembrete::registrado(mcp, &hooks::texto(&corpo, "workspaceId"), &hooks::texto(&corpo, "blueprintId"));
            }
            Resposta::json(200, &r)
        }
        "hook/parada" => Resposta::json(200, &super::lembrete::parada(mcp, &corpo)), // o Stop do turno (ver lembrete.rs)
        "aviso" => Resposta::json(200, &com_dados(dados, |d| super::eventos::aviso(mcp, d, &corpo))?),
        "prototipo" => Resposta::json(200, &com_dados(dados, |d| acoes::prototipo(mcp, d, &corpo))?),
        // os hooks do Claude Code e a ligação pasta → blueprint (ver hooks.rs)
        "hook/sessao" => Resposta::json(200, &hooks::sessao(mcp, dados, &corpo)?),
        "hook/fim" => Resposta::json(200, &super::sessoes::fim(mcp, &corpo)), // o SessionEnd tira a sessão da lista
        "hook/edicao" => Resposta::json(200, &hooks::edicao(mcp, dados, &corpo)?),
        "hook/subagente" => Resposta::json(200, &hooks::subagente(mcp, dados, &corpo)?),
        "hook/status" => Resposta::json(200, &super::status::status(mcp, dados, &corpo)?),
        "pasta/desvincular" => Resposta::json(200, &hooks::desvincular(dados, &corpo)?),
        "pasta/vincular" => Resposta::json(200, &hooks::vincular(dados, &corpo)?),
        "iniciar" => {
            let (status, resposta) = if corpo.get("sessaoId").is_some() {
                super::iniciar::em_sessao(mcp, dados, &corpo)? // entrega o pedido a uma sessão já aberta, pelo canal dela
            } else {
                super::iniciar::iniciar(dados, &corpo, &super::iniciar::abrir_terminal)? // abre o terminal de verdade; os testes usam um lançador falso
            };
            Resposta::json(status, &resposta)
        }
        "prototipo/remover" => {
            // chamado pela página ao excluir; só mexe em arquivos com id gerado por aqui
            let id = corpo.get("prototipoId").and_then(Value::as_str).filter(|i| id_prototipo_valido(i)).ok_or_else(|| falha(400, "prototipoId inválido"))?;
            let _ = std::fs::remove_file(mcp.prototipos.join(format!("{id}.html")));
            Resposta::json(200, &json!({"ok": true}))
        }
        "fila/ack" => {
            let ids: HashSet<i64> = corpo.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            mcp.ack(&ids);
            corpo.get("propostas").and_then(Value::as_array).into_iter().flatten().for_each(|p| mcp.ack_proposta(p)); // propostas pela chave: ver `Fila::ack_proposta`
            corpo.get("registros").and_then(Value::as_array).into_iter().flatten().for_each(|r| mcp.ack_registro(r)); // registros: a contagem e a chave são próprias
            Resposta::json(200, &json!({"ok": true}))
        }
        _ => nao_encontrado(),
    })
}
