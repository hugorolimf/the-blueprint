//! Testes do estado do blueprint (comando /blueprint:status e status line).
use super::hooks_tests::{dados, ligar, mcp};
use super::status::status;
use serde_json::json;

#[test]
fn o_status_mostra_a_acao_atual_os_planos_e_o_proximo_checkpoint() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let r = status(&m, &d, &json!({"cwd": "C:/Proj/sub"})).unwrap();
    assert_eq!(r["vinculado"], true);
    let t = r["texto"].as_str().unwrap();
    assert!(t.contains("Ação 1.2: Tela (01/12 a 05/12) · 0/1 planos"), "{t}");
    assert!(t.contains("[ ] Criar tela") && !t.contains("Feito"), "{t}");
    assert!(t.contains("Sem checkpoints pendentes."), "{t}");
    assert_eq!(r["linha"], "Blueprint · ação 1.2 · Tela · 0/1");
}

#[test]
fn pasta_solta_nao_tem_estado_e_a_linha_cabe_numa_linha() {
    let (m, d) = (mcp(), dados());
    assert_eq!(status(&m, &d, &json!({"cwd": "C:/Solta"})).unwrap(), json!({"vinculado": false}));
    ligar(&d, "C:/Proj");
    let linha = status(&m, &d, &json!({"cwd": "C:/Proj"})).unwrap()["linha"].as_str().unwrap().to_string();
    assert!(!linha.contains('\n') && linha.chars().count() < 80, "{linha}");
}
