//! `mover`: ação de outro blueprint do workspace. Só valida; mover de verdade é da página.
use super::mover_acoes;
use serde_json::{json, Value};

fn blueprints() -> Vec<Value> {
    vec![
        json!({"id": "a", "nome": "Atual", "acoes": [{"id": "a1", "nome": "Daqui", "planos": [], "checkpoints": []}]}),
        json!({"id": "b", "nome": "Outro", "acoes": [{"id": "b1", "nome": "Lá", "planos": [{"id": "t", "feito": true}, {"id": "u", "feito": false}], "checkpoints": [{"id": "m"}]}]}),
    ]
}

fn roda(pedidos: Value, atual: Option<usize>, apagando: &[&str]) -> (Vec<Value>, Vec<String>, Vec<String>) {
    let bps = blueprints();
    let (mut erros, mut avisos) = (Vec::new(), Vec::new());
    let ids: Vec<String> = apagando.iter().map(|s| s.to_string()).collect();
    let r = mover_acoes(pedidos.as_array().unwrap(), atual.map(|i| &bps[i]), &bps, &ids, &mut erros, &mut avisos);
    (r, erros, avisos)
}

#[test]
fn descreve_a_acao_de_outro_blueprint() {
    let (r, erros, _) = roda(json!([{"acao_id": "b1", "motivo": "unir"}]), Some(0), &[]);
    assert!(erros.is_empty(), "{erros:?}");
    assert_eq!((r[0]["deBlueprintId"].as_str(), r[0]["planos"].as_u64(), r[0]["feitos"].as_u64(), r[0]["checkpoints"].as_u64()), (Some("b"), Some(2), Some(1), Some(1)));
}

#[test]
fn recusa_id_desconhecido_o_do_proprio_blueprint_e_repetido() {
    let (_, erros, _) = roda(json!([{"acao_id": "zz"}, {"acao_id": "a1"}, {"acao_id": "b1"}, {"acao_id": "b1"}]), Some(0), &[]);
    assert_eq!(erros.len(), 3, "{erros:?}");
    assert!(erros[0].contains("não existe") && erros[1].contains("já está neste blueprint") && erros[2].contains("duas vezes"));
}

#[test]
fn nao_move_e_apaga_a_mesma_acao() {
    let (r, erros, _) = roda(json!([{"acao_id": "b1"}]), Some(0), &["b1"]);
    assert!(r.is_empty() && erros[0].contains("em `mover` e em `remover`"), "{erros:?}");
}

#[test]
fn blueprint_novo_ignora_o_movimento() {
    let (r, erros, avisos) = roda(json!([{"acao_id": "b1"}]), None, &[]);
    assert!(r.is_empty() && erros.is_empty() && avisos[0].contains("ignorado"));
}
