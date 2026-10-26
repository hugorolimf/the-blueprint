//! `hook/estado`: os hooks `http` do Claude Code que só mudam o ESTADO do agente na folha (aguardando, bloqueado, escrevendo), sem estado novo. Uma rota,
//! despachada por `hook_event_name` (nomes e campos conferidos em https://code.claude.com/docs/en/hooks.md):
//!   PermissionRequest  tool_name                 -> aguardando «waiting for permission: <tool_name>»
//!   PermissionDenied   tool_name, reason         -> bloqueado com o motivo (só dispara no modo auto)
//!   StopFailure        error, error_details      -> bloqueado com o erro de API
//!   PreCompact                                   -> aguardando «compacting the conversation»
//!   PostCompact                                  -> escrevendo (volta ao trabalho)
//! Só presença: grava na atividade do agente (a folha mostra a figura no estado), sem item na fila e sem linha do tempo, porque um pedido de permissão
//! é comum e encheria o limite de 50 eventos da ação. A resposta é sempre `{}`: sem `decision`, o hook nunca decide permissão nem segura o Claude Code.
//! Sessão sem blueprint ou evento que não é destes: não faz nada. `PostToolUseFailure` fica de fora de propósito (uma ferramenta que falha é rotina).
use super::hooks::{alvo, evento_de, identidade, texto};
use super::Mcp;
use crate::dados::Dados;
use crate::http::pedido::ErroHttp;
use serde_json::{json, Value};
use std::sync::Mutex;

type R<T> = Result<T, ErroHttp>;

/// A frase da folha: curta e sem caracteres de controle (o texto vem de fora).
const FRASE_MAX: usize = 200;

fn limpo(s: &str, max: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(max).collect::<String>().trim().to_string()
}

/// O estado e a frase do evento, ou `None` se não é um dos cinco.
fn transicao(corpo: &Value) -> Option<(&'static str, String)> {
    let campo = |c: &str, max: usize| limpo(texto(corpo, c), max);
    let ferramenta = || Some(campo("tool_name", 60)).filter(|f| !f.is_empty()).unwrap_or_else(|| "tool".to_string());
    let (estado, frase) = match texto(corpo, "hook_event_name") {
        "PermissionRequest" => {
            let ferramenta = ferramenta();
            ("aguardando", crate::tr!("waiting for permission: {ferramenta}", "aguardando permissão: {ferramenta}"))
        }
        "PermissionDenied" => {
            let (ferramenta, motivo) = (ferramenta(), campo("reason", 120));
            ("bloqueado", if motivo.is_empty() { crate::tr!("permission denied: {ferramenta}", "permissão negada: {ferramenta}") } else { crate::tr!("permission denied: {ferramenta} ({motivo})", "permissão negada: {ferramenta} ({motivo})") })
        }
        "StopFailure" => {
            let (erro, detalhe) = (campo("error", 40), campo("error_details", 120));
            let erro = if erro.is_empty() { "unknown".to_string() } else { erro };
            ("bloqueado", if detalhe.is_empty() { crate::tr!("API error: {erro}", "erro da API: {erro}") } else { crate::tr!("API error: {erro} ({detalhe})", "erro da API: {erro} ({detalhe})") })
        }
        "PreCompact" => ("aguardando", crate::tr!("compacting the conversation", "compactando a conversa")),
        "PostCompact" => ("escrevendo", crate::tr!("conversation compacted, back to work", "conversa compactada, de volta ao trabalho")),
        _ => return None,
    };
    Some((estado, frase.chars().take(FRASE_MAX).collect()))
}

pub fn estado(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let Some((estado, frase)) = transicao(corpo) else { return Ok(json!({})) };
    let Some((ws, bp)) = alvo(mcp, dados, corpo)? else { return Ok(json!({})) };
    let (nome, papel, _) = identidade(corpo);
    let evento = evento_de(mcp, &nome, papel, &ws, &bp, &frase, estado);
    // o mesmo mapa que `acoes::evento` entrega à atividade: o nome vai à parte e o texto é a mensagem
    let mut atividade = evento.as_object().cloned().unwrap_or_default();
    atividade.remove("agente");
    atividade.remove("texto");
    atividade.insert("mensagem".into(), json!(frase));
    mcp.registrar_atividade(&nome, atividade);
    Ok(json!({}))
}
