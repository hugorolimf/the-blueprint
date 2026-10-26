//! Os segmentos da status line que vêm só da entrada do Claude Code (contexto usado, limites de uso, worktree e agente), sem perguntar nada ao app.
//! Função pura: campo ausente ou `null` é omitido (`rate_limits` só existe para assinantes Pro e Max, depois da primeira resposta; `worktree` e `agent` só quando
//! há worktree ou `--agent`). Sem nenhum campo, devolve texto vazio e a status line fica como era. Sem `refreshInterval`: a linha atualiza pelos eventos do Claude.
use serde_json::Value;

const NOME_MAX: usize = 16;

/// Percentual de 0 a 100, arredondado; ausente, `null`, texto, negativo ou NaN: None.
fn percentual(v: &Value) -> Option<u32> {
    v.as_f64().filter(|p| p.is_finite() && *p >= 0.0).map(|p| p.min(100.0).round() as u32)
}

/// Nome vindo de fora: sem controle (vai direto ao terminal), curto e não vazio.
fn nome(v: &Value) -> Option<String> {
    let n: String = v.as_str()?.chars().filter(|c| !c.is_control()).take(NOME_MAX).collect();
    Some(n.trim().to_string()).filter(|n| !n.is_empty())
}

/// `contexto 42% · 5h 23% · 7d 41% · worktree x · agente y`, só com o que veio.
pub fn segmentos(entrada: &Value) -> String {
    let mut s = Vec::new();
    if let Some(p) = percentual(&entrada["context_window"]["used_percentage"]) {
        s.push(crate::tr!("context {p}%", "contexto {p}%"));
    }
    if let Some(p) = percentual(&entrada["rate_limits"]["five_hour"]["used_percentage"]) {
        s.push(format!("5h {p}%"));
    }
    if let Some(p) = percentual(&entrada["rate_limits"]["seven_day"]["used_percentage"]) {
        s.push(format!("7d {p}%"));
    }
    if let Some(n) = nome(&entrada["worktree"]["name"]) {
        s.push(format!("worktree {n}"));
    }
    if let Some(n) = nome(&entrada["agent"]["name"]) {
        s.push(crate::tr!("agent {n}", "agente {n}"));
    }
    s.join(" · ")
}

#[cfg(test)]
#[path = "hook_extras_tests.rs"]
mod tests;
