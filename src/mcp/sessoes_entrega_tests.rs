//! Testes do canal das sessões e da entrega do pedido de iniciar uma ação a uma sessão aberta (nenhum Claude de verdade; o evento é lido do SSE em memória).
use super::hooks::{sessao, travado};
use super::hooks_tests::{dados, mcp};
use super::iniciar::em_sessao;
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::Dados;
use serde_json::{json, Value};
use std::sync::Mutex;

/// Uma pasta real ligada ao blueprint p1 e uma sessão (hook) aberta nela.
fn com_sessao_na_pasta() -> (Mcp, Mutex<Dados>, String) {
    let (m, d) = (mcp(), dados());
    let pasta = std::env::temp_dir().join(format!("blueprint-sessoes-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let cwd = pasta.to_str().unwrap().to_string();
    travado(&d).vincular_pasta(&Contexto::local(), &cwd, "sm", "p1").unwrap();
    sessao(&m, &d, &json!({"session_id": "s1", "cwd": cwd})).unwrap();
    (m, d, cwd)
}

fn pedir(sessao_id: &str, extra: Value) -> Value {
    let mut c = json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "sessaoId": sessao_id});
    c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    c
}

#[test]
fn o_canal_so_e_destinatario_enquanto_a_conexao_vive_e_a_lista_o_separa_das_sem_canal() {
    let (m, _d, cwd) = com_sessao_na_pasta();
    sessao(&m, &_d, &json!({"session_id": "s2", "cwd": cwd})).unwrap();
    assert_eq!(m.sessoes_do_blueprint("sm", "p1")["sessoes"].as_array().unwrap().iter().filter(|s| s["canal"] == true).count(), 0);
    let guarda = m.abrir_canal("abc123", &cwd).unwrap();
    let lista = m.sessoes_do_blueprint("sm", "p1");
    assert_eq!(lista["sessoes"], json!([{"id": "abc123", "pasta": cwd, "canal": true}, {"id": "s1", "pasta": cwd, "canal": false}]), "2 sessões na pasta, 1 com canal: sobra 1 sem canal");
    drop(guarda);
    assert!(m.sessoes_do_blueprint("sm", "p1")["sessoes"].as_array().unwrap().iter().all(|s| s["canal"] == false), "a conexão caiu");
    assert!(m.abrir_canal("com espaço", &cwd).is_none() && m.abrir_canal("", &cwd).is_none() && m.abrir_canal("ok", "a\nb").is_none());
}

#[test]
fn o_pedido_vai_so_a_sessao_escolhida_pelo_evento_iniciar_acao() {
    let (m, d, cwd) = com_sessao_na_pasta();
    let _guarda = m.abrir_canal("abc123", &cwd).unwrap();
    let eventos = m.assinar();
    let (status, r) = em_sessao(&m, &d, &pedir("abc123", json!({}))).unwrap();
    assert_eq!((status, r["terminal"].clone(), r["ok"].clone()), (200, json!("canal"), json!(true)), "{r}");
    let prompt = r["prompt"].as_str().unwrap();
    assert!(prompt.starts_with("Execute a ação 1.2 «Tela» do blueprint «Portal»"), "{prompt}");
    let e = eventos.try_recv().unwrap();
    assert_eq!(e.nome, "iniciar-acao");
    assert_eq!(e.dados.unwrap(), json!({"workspaceId": "sm", "blueprintId": "p1", "canalId": "abc123", "texto": prompt}));
}

#[test]
fn as_recusas_da_entrega_nao_mandam_evento_e_ainda_devolvem_o_prompt() {
    let (m, d, cwd) = com_sessao_na_pasta();
    let eventos = m.assinar();
    let (status, r) = em_sessao(&m, &d, &pedir("nao-existe", json!({}))).unwrap();
    assert_eq!((status, r["codigo"].clone()), (410, json!("sessao-caiu")), "canal que não está conectado");
    assert!(r["prompt"].as_str().unwrap().starts_with("Execute a ação"), "dá para copiar");
    let _de_fora = m.abrir_canal("fora1", "C:\\outra\\pasta").unwrap();
    let (status, r) = em_sessao(&m, &d, &pedir("fora1", json!({}))).unwrap();
    assert_eq!((status, r["codigo"].clone()), (409, json!("sessao-de-outra-pasta")), "canal de pasta que não é do blueprint");
    let (status, r) = em_sessao(&m, &d, &pedir("s1", json!({"semCanal": true}))).unwrap();
    assert_eq!((status, r["terminal"].clone()), (200, json!("copiar")), "sem canal só devolve o prompt");
    assert_eq!(em_sessao(&m, &d, &json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "sessaoId": ""})).unwrap_err().status, 400);
    assert!(eventos.try_recv().is_err(), "nenhuma recusa nem cópia emite evento");
    let _ = cwd;
}
