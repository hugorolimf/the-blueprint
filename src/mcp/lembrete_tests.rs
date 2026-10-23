//! Testes do lembrete de registro: edição sem plano gera um aviso único no Stop; registrar apaga; sessão sem edição não gera nada.
use super::acoes;
use super::hooks::{edicao, travado};
use super::hooks_tests::{dados, ligar, mcp};
use super::lembrete::{parada, registrado};
use crate::contexto::Contexto;
use serde_json::{json, Value};

fn editar(m: &super::Mcp, d: &std::sync::Mutex<crate::dados::Dados>, sessao: &str) {
    let corpo = json!({"session_id": sessao, "cwd": "C:\\Proj", "tool_name": "Edit", "tool_input": {"file_path": "C:\\Proj\\a.rs"}});
    edicao(m, d, &corpo).unwrap();
}

fn parar(m: &super::Mcp, sessao: &str, ativo: bool) -> Value {
    parada(m, &json!({"session_id": sessao, "hook_event_name": "Stop", "stop_hook_active": ativo}))
}

fn tem_lembrete(r: &Value) -> bool {
    r["hookSpecificOutput"]["hookEventName"] == "Stop" && r["hookSpecificOutput"]["additionalContext"].as_str().is_some_and(|t| t.contains("registrar_trabalho"))
}

#[test]
fn edicao_sem_plano_gera_um_lembrete_no_stop_e_depois_some() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    editar(&m, &d, "s1");
    let r = parar(&m, "s1", false);
    assert!(tem_lembrete(&r), "{r}");
    assert!(r["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("blueprint_id p1"));
    assert_eq!(parar(&m, "s1", false), json!({}), "um lembrete por pendência");
}

#[test]
fn registrar_apaga_o_lembrete_do_blueprint() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    editar(&m, &d, "s1");
    registrado(&m, "sm", "p1");
    assert_eq!(parar(&m, "s1", false), json!({}));
}

#[test]
fn registro_de_outro_blueprint_nao_apaga_o_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    editar(&m, &d, "s1");
    registrado(&m, "sm", "outro");
    assert!(tem_lembrete(&parar(&m, "s1", false)));
}

#[test]
fn sessao_sem_edicao_nao_gera_nada() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    assert_eq!(parar(&m, "s2", false), json!({}));
    assert_eq!(parar(&m, "", false), json!({}));
}

#[test]
fn stop_hook_active_nao_repete_e_nao_gasta_o_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    editar(&m, &d, "s1");
    // o Claude já está continuando por causa de um Stop: não devolve nada e mantém a pendência
    assert_eq!(parar(&m, "s1", true), json!({}));
    assert!(tem_lembrete(&parar(&m, "s1", false)));
}

#[test]
fn edicao_com_plano_herdado_nao_marca_a_sessao() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    {
        let mut g = travado(&d);
        let aviso = json!({"tipo": "progresso", "texto": "Criar tela", "workspaceId": "sm", "blueprintId": "p1", "agente": "Claude", "papel": "construtor", "estado": "escrevendo", "acaoId": "f2", "planoId": "t2", "onde": ["x"]});
        acoes::evento(&m, g.ler_tudo(&Contexto::local()).unwrap(), &aviso).unwrap();
    }
    editar(&m, &d, "s1");
    assert_eq!(parar(&m, "s1", false), json!({}));
}
