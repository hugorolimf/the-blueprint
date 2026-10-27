//! A linha do tempo da execução por ação: cada evento que passa por `acoes::evento` e tem ação é anotado no banco (`dados::linha_do_tempo`) e avisado por SSE
//! (`linha-do-tempo`, só ids). Aqui fica o filtro do que entra: a evidência das marcações nunca entra (já vira comentário do plano), nem conteúdo de arquivo
//! (o servidor nem o recebe) ou comando; o texto do progresso e do bloqueio vai cortado. Regra completa: docs/claude-na-folha.md.
use super::acoes;
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::{Dados, EventoLinha, LIMITE_POR_ACAO};
use crate::http::pedido::{ErroHttp, Pedido};
use serde_json::{json, Value};
use std::sync::Mutex;

const TEXTO_MAX: usize = 200;
const ONDE_MAX: usize = 120;

/// O evento da linha do tempo para um item da fila já validado; None se ele não for de uma ação.
pub(super) fn da_fila(item: &Value) -> Option<EventoLinha> {
    let s = |k: &str| item.get(k).and_then(Value::as_str).filter(|v| !v.is_empty()).map(str::to_string);
    let (tipo, acao_id) = (s("tipo")?, s("acaoId")?);
    let com_texto = tipo == "progresso" || tipo == "bloqueio";
    Some(EventoLinha {
        workspace_id: s("workspaceId").unwrap_or_default(),
        blueprint_id: s("blueprintId").unwrap_or_default(),
        acao_id,
        alvo: if tipo.starts_with("checkpoint") { s("checkpointId") } else { s("planoId") },
        agente: s("agente"),
        estado: s("estado"),
        texto: if com_texto { item["texto"].as_str().unwrap_or("").chars().take(TEXTO_MAX).collect() } else { String::new() },
        onde: item["onde"].as_array().and_then(|l| l.first()).and_then(Value::as_str).map(|o| o.chars().take(ONDE_MAX).collect()),
        tipo,
    })
}

/// `acoes::evento` e, se ele aceitou e o evento é de uma ação, a anotação na linha do tempo. Falhar ao anotar não recusa o evento (ele já está na fila;
/// recusar faria o agente reenviar e duplicar).
pub(super) fn evento(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<Value, ErroHttp> {
    let ctx = Contexto::local();
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner());
    let item = acoes::evento(mcp, d.ler_tudo(&ctx)?, corpo)?;
    if let (Some(plano), Some(tipo), Some(em)) = (item["planoId"].as_str(), item["tipo"].as_str(), item["em"].as_str()) {
        let ja_feito = if tipo == "progresso" {
            let dados = d.ler_tudo(&ctx)?;
            let bp = acoes::localizar(dados, item["workspaceId"].as_str().unwrap_or(""), item["blueprintId"].as_str().unwrap_or(""))?.1;
            bp.and_then(|p| p["acoes"].as_array())
                .and_then(|acoes| acoes.iter().find(|a| a["id"] == item["acaoId"]))
                .and_then(|a| a["planos"].as_array())
                .and_then(|planos| planos.iter().find(|t| t["id"] == plano))
                .is_some_and(|t| t["feito"] == true)
        } else { false };
        if !ja_feito { d.registrar_tempo_plano(plano, tipo, em)?; }
    }
    if let Some(e) = da_fila(&item) {
        match d.gravar_linha_do_tempo(&ctx, crate::tempo::agora_ms(), &e) {
            Ok(()) => mcp.notificar("linha-do-tempo", Some(json!({"workspaceId": e.workspace_id, "blueprintId": e.blueprint_id, "acaoId": e.acao_id}))),
            Err(erro) => eprintln!("linha do tempo: não foi possível anotar o evento: {erro}"),
        }
    }
    Ok(item)
}

/// `GET /api/mcp/linha-do-tempo?workspace=&blueprint=&acao=`: os eventos da ação, do mais recente ao mais antigo.
pub(super) fn listar(dados: &Mutex<Dados>, pedido: &Pedido) -> Result<Value, ErroHttp> {
    let p = |n: &str| pedido.parametro(n).unwrap_or_default();
    let eventos = dados.lock().unwrap_or_else(|e| e.into_inner()).linha_do_tempo(&Contexto::local(), &p("workspace"), &p("blueprint"), &p("acao"))?;
    Ok(json!({"eventos": eventos, "limite": LIMITE_POR_ACAO}))
}
