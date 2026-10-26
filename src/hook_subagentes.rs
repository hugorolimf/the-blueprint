//! `blueprint hook subagentes`: o comando do `subagentStatusLine` do Claude Code (o painel de subagentes). Lê no stdin o JSON do painel (`tasks[]`, `columns`),
//! manda ao app aberto (`/api/mcp/hook/subagentes`) e imprime uma linha JSON `{"id", "content"}` por subagente que o app conhece; o resto fica sem linha e o painel
//! usa a padrão. Sem o app, ou com qualquer erro: não imprime nada (como os outros `hook`), nunca atrapalha a sessão.
use crate::mcp::cliente::requisitar;
use serde_json::{json, Value};

/// As linhas que o Claude Code lê, a partir da resposta do app; só entram as que têm `id` e `content` em texto.
fn linhas(resposta: &Value) -> Vec<String> {
    resposta["linhas"].as_array().into_iter().flatten()
        .filter(|l| l["id"].is_string() && l["content"].is_string())
        .map(|l| json!({"id": l["id"], "content": l["content"]}).to_string())
        .collect()
}

pub fn saida(base: &str, entrada: &str) -> Option<String> {
    let (status, resposta) = requisitar(base, "POST", "hook/subagentes", Some(entrada)).ok()?;
    let linhas = linhas(&serde_json::from_str::<Value>(&resposta).ok().filter(|_| status == 200)?);
    (!linhas.is_empty()).then(|| linhas.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imprime_so_as_linhas_validas_uma_por_linha() {
        let r = json!({"linhas": [{"id": "a1", "content": "Explore a1 · x"}, {"id": 7, "content": "x"}, {"id": "b2"}, {"id": "c3", "content": ""}]});
        assert_eq!(linhas(&r), [r#"{"content":"Explore a1 · x","id":"a1"}"#, r#"{"content":"","id":"c3"}"#]);
        assert!(linhas(&json!({})).is_empty() && linhas(&json!({"linhas": "x"})).is_empty());
    }

    #[test]
    fn sem_o_app_aberto_nao_imprime_nada() {
        assert!(saida("http://127.0.0.1:9", r#"{"tasks":[]}"#).is_none());
    }
}
