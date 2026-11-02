//! Os argumentos das ferramentas MCP: lê os campos que o agente mandou e monta o corpo das rotas /api/mcp (usado por `stdio.rs`, que atende o `--mcp` e o MCP por HTTP).
use serde_json::{json, Map, Value};

pub(super) fn texto(a: &Value, chave: &str) -> Result<String, String> {
    match a.get(chave).and_then(Value::as_str).map(str::trim) {
        Some(t) if !t.is_empty() => Ok(t.to_string()),
        _ => Err(format!("{chave} is required.")),
    }
}

pub(super) fn opcional(a: &Value, chave: &str) -> String {
    a.get(chave).and_then(Value::as_str).unwrap_or("").to_string()
}

/// Um campo que deve chegar como objeto ou lista; alguns clientes mandam o JSON já em texto, e isso também vale. Ausente vira `null` (a validação responde).
pub(super) fn estruturado(a: &Value, chave: &str) -> Value {
    match a.get(chave) {
        Some(Value::String(t)) => serde_json::from_str(t).unwrap_or(Value::Null),
        Some(v) => v.clone(),
        None => Value::Null,
    }
}

pub(super) fn alvo(a: &Value) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("workspaceId".into(), json!(opcional(a, "workspace_id")));
    m.insert("blueprintId".into(), json!(opcional(a, "blueprint_id")));
    m
}

pub(super) fn com(mut m: Map<String, Value>, extra: &[(&str, Value)]) -> Map<String, Value> {
    for (k, v) in extra {
        m.insert((*k).into(), v.clone());
    }
    m
}

pub(super) fn plano(a: &Value) -> Result<Map<String, Value>, String> {
    Ok(com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("planoId", json!(texto(a, "plano_id")?))]))
}

pub(super) fn checkpoint(a: &Value) -> Result<Map<String, Value>, String> {
    Ok(com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("checkpointId", json!(texto(a, "checkpoint_id")?))]))
}

/// Os campos opcionais de atividade (painel de agentes) que a ferramenta recebeu; o servidor valida e recusa o que estiver fora do contrato.
pub(super) fn com_atividade(mut m: Map<String, Value>, a: &Value) -> Map<String, Value> {
    for campo in ["agente", "papel", "estado", "onde", "percentual"] {
        if let Some(v) = a.get(campo) {
            m.insert(campo.into(), v.clone());
        }
    }
    m
}

pub(super) fn plano_se_houver(a: &Value) -> Result<Map<String, Value>, String> {
    if a.get("plano_id").is_some_and(|v| crate::blueprint::verdade(Some(v))) { plano(a) } else { Ok(alvo(a)) }
}

pub(super) fn ambiente(chave: &str) -> Option<String> {
    std::env::var(chave).ok()
}

/// O que o invólucro deixa no ambiente do agente de um plano (`BLUEPRINT_EXECUCAO*`, seção 7) entra em todo evento; `execucao_segredo` é para quem não repassa o ambiente ao MCP, como pode ser o Codex.
pub(super) fn com_execucao(mut m: Map<String, Value>, a: &Value, env: &dyn Fn(&str) -> Option<String>) -> Map<String, Value> {
    let de_env = |chave: &str| env(chave).filter(|v| !v.trim().is_empty());
    let segredo = a.get("execucao_segredo").and_then(Value::as_str).filter(|s| !s.trim().is_empty()).map(String::from).or_else(|| de_env("BLUEPRINT_EXECUCAO_SEGREDO"));
    if let Some(s) = segredo {
        m.insert("execucaoSegredo".into(), json!(s));
    }
    if let Some(id) = de_env("BLUEPRINT_EXECUCAO") {
        m.insert("execucaoId".into(), json!(id));
    }
    if !m.contains_key("agente") {
        if let Some(nome) = de_env("BLUEPRINT_AGENTE_NOME") {
            m.insert("agente".into(), json!(nome));
        }
    }
    m
}
