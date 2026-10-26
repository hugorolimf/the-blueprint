//! Testes dos segmentos extras da status line: entrada completa, parcial, vazia e torta, nos dois idiomas.
use super::segmentos;
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};

fn completa() -> Value {
    json!({
        "context_window": {"used_percentage": 42.4, "remaining_percentage": 57.6},
        "rate_limits": {"five_hour": {"used_percentage": 23.5, "resets_at": 1}, "seven_day": {"used_percentage": 41.2, "resets_at": 2}, "spend_limit": {"used_percentage": 99}},
        "worktree": {"name": "meu-recurso", "path": "/x", "branch": "worktree-meu-recurso"},
        "agent": {"name": "security-reviewer"}
    })
}

#[test]
fn entrada_completa_em_portugues_e_em_ingles() {
    assert_eq!(segmentos(&completa()), "contexto 42% · 5h 24% · 7d 41% · worktree meu-recurso · agente security-reviewe");
    assert_eq!(com(Idioma::En, || segmentos(&completa())), "context 42% · 5h 24% · 7d 41% · worktree meu-recurso · agent security-reviewe");
}

#[test]
fn campo_ausente_ou_nulo_e_omitido_sem_deixar_separador_sobrando() {
    assert_eq!(segmentos(&json!({"context_window": {"used_percentage": 8}})), "contexto 8%");
    assert_eq!(segmentos(&json!({"rate_limits": {"seven_day": {"used_percentage": 3}}, "agent": {"name": "x"}})), "7d 3% · agente x");
    // início de sessão: o Claude Code manda null no percentual e rate_limits só vem depois da primeira resposta
    assert_eq!(segmentos(&json!({"context_window": {"used_percentage": null, "current_usage": null}, "rate_limits": {}})), "");
    assert_eq!(segmentos(&json!({"rate_limits": {"five_hour": {"resets_at": 1}}, "worktree": {}, "agent": {"name": "  "}})), "");
}

#[test]
fn entrada_vazia_ou_torta_devolve_vazio_e_nunca_falha() {
    for e in [json!({}), Value::Null, json!([]), json!("texto"), json!(7), json!({"context_window": "x", "rate_limits": 3, "worktree": [], "agent": null})] {
        assert_eq!(segmentos(&e), "", "{e}");
    }
    assert_eq!(segmentos(&json!({"context_window": {"used_percentage": "42"}, "rate_limits": {"five_hour": {"used_percentage": -1}}})), "", "texto e negativo não são percentual");
    assert_eq!(segmentos(&json!({"context_window": {"used_percentage": 140}})), "contexto 100%");
}

#[test]
fn nomes_de_fora_nao_levam_controle_do_terminal_e_nao_alargam_a_linha() {
    let s = segmentos(&json!({"worktree": {"name": "a\u{1b}[31mb\nc"}, "agent": {"name": "x".repeat(200)}}));
    assert!(!s.chars().any(char::is_control), "{s:?}");
    assert_eq!(s, format!("worktree a[31mbc · agente {}", "x".repeat(16)));
    assert!(segmentos(&completa()).chars().count() < 100);
}
