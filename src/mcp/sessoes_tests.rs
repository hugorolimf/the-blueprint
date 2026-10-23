//! Testes do registro das sessões abertas: SessionStart registra, SessionEnd tira, a lista é por blueprint.
use super::hooks::sessao;
use super::hooks_tests::{dados, ligar, mcp};
use super::sessoes::fim;
use serde_json::json;

fn ids(m: &super::Mcp) -> Vec<String> {
    m.sessoes_do_blueprint("sm", "p1")["sessoes"].as_array().unwrap().iter().map(|s| s["id"].as_str().unwrap().to_string()).collect()
}

#[test]
fn o_session_start_registra_a_sessao_com_a_pasta_e_o_session_end_a_tira() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    sessao(&m, &d, &json!({"session_id": "s2", "cwd": "C:\\Proj\\sub"})).unwrap();
    sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Proj"})).unwrap();
    let lista = m.sessoes_do_blueprint("sm", "p1");
    assert_eq!(lista["sessoes"], json!([{"id": "s1", "pasta": "C:\\Proj", "canal": false}, {"id": "s2", "pasta": "C:\\Proj\\sub", "canal": false}]));
    fim(&m, &json!({"session_id": "s1"}));
    assert_eq!(ids(&m), vec!["s2"]);
    assert_eq!(m.sessao("s1"), None, "a lembrança do blueprint da sessão sai junto");
}

#[test]
fn retomar_a_sessao_nao_duplica_e_outro_blueprint_nao_a_ve() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    for _ in 0..2 {
        sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Proj"})).unwrap();
    }
    assert_eq!(ids(&m), vec!["s1"]);
    assert_eq!(m.sessoes_do_blueprint("sm", "outro")["sessoes"], json!([]));
}

#[test]
fn pasta_sem_blueprint_sem_id_ou_com_pasta_invalida_nao_registra() {
    let (m, d) = (mcp(), dados());
    sessao(&m, &d, &json!({"session_id": "s1", "cwd": "C:\\Solta"})).unwrap();
    ligar(&d, "C:\\Proj");
    sessao(&m, &d, &json!({"cwd": "C:\\Proj"})).unwrap();
    m.abrir_sessao("s3", "a\nb", "sm", "p1");
    assert_eq!(ids(&m), Vec::<String>::new());
    fim(&m, &json!({})); // sem session_id não faz nada
}

#[test]
fn passou_do_limite_recomeca_a_lista() {
    let m = mcp();
    for i in 0..=super::sessoes::MAX_ABERTAS {
        m.abrir_sessao(&format!("s{i}"), "C:\\Proj", "sm", "p1");
    }
    assert_eq!(ids(&m), vec![format!("s{}", super::sessoes::MAX_ABERTAS)]);
}
