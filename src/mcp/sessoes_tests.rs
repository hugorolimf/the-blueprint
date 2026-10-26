//! Testes do registro das sessões abertas: SessionStart registra, SessionEnd tira, a lista é por blueprint.
use super::hooks::sessao;
use super::hooks_tests::{dados, ligar, mcp};
use super::sessoes::{fim, PRAZO};
use serde_json::json;
use std::time::{Duration, Instant};

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

fn ids_em(m: &super::Mcp, agora: Instant) -> Vec<String> {
    m.sessoes_do_blueprint_em("sm", "p1", agora)["sessoes"].as_array().unwrap().iter().map(|s| s["id"].as_str().unwrap().to_string()).collect()
}

#[test]
fn sessao_sem_sinal_por_um_prazo_sai_da_lista_e_um_hook_renova() {
    let m = mcp();
    m.abrir_sessao("s1", "C:\\Proj", "sm", "p1");
    m.abrir_sessao("s2", "C:\\Proj", "sm", "p1");
    let t = Instant::now();
    assert_eq!(ids_em(&m, t + PRAZO - Duration::from_secs(60)), vec!["s1", "s2"], "horas parada ainda vale");
    m.sinal_sessao_em("s2", t + Duration::from_secs(3600));
    m.sinal_sessao_em("fantasma", t); // sessão fora da lista: nada
    assert_eq!(ids_em(&m, t + PRAZO), vec!["s2"], "s1 passou do prazo; s2 teve um hook depois");
    assert_eq!(ids_em(&m, t), vec!["s2"], "a que expirou foi esquecida, não só escondida");
    assert_eq!(ids_em(&m, t + PRAZO + Duration::from_secs(3600)), Vec::<String>::new());
}

#[test]
fn canal_vivo_segura_as_sessoes_da_pasta_e_ao_cair_volta_o_prazo() {
    let m = mcp();
    m.abrir_sessao("s1", "C:\\Proj", "sm", "p1");
    m.abrir_sessao("s2", "C:\\Outra", "sm", "p1");
    let guarda = m.abrir_canal("abc123", "c:/proj/").unwrap();
    let longe = Instant::now() + PRAZO * 3;
    assert_eq!(ids_em(&m, longe), vec!["abc123"], "a pasta com canal não expira (o canal mostra a sessão); a outra, sem canal, expira");
    drop(guarda);
    let t = Instant::now();
    assert_eq!(ids_em(&m, t + PRAZO - Duration::from_secs(60)), vec!["s1"], "a queda do canal conta como o último sinal");
    assert_eq!(ids_em(&m, t + PRAZO), Vec::<String>::new());
}
