//! Testes do painel de subagentes (hook/subagentes): a linha de cada subagente, os estados falhou e cancelado, os tokens e a ausência de duplicados.
use super::hooks::subagente;
use super::hooks_tests::{dados, ligar, mcp};
use super::subagentes_status::{linha, nome_do, subagentes, tokens_curtos};
use super::Mcp;
use crate::idioma::{com, Idioma};
use serde_json::{json, Value};
use std::sync::Mutex;

use crate::dados::Dados;

fn comeca(m: &Mcp, d: &Mutex<Dados>, id: &str) {
    subagente(m, d, &json!({"hook_event_name": "SubagentStart", "cwd": "C:\\Proj", "agent_id": id, "agent_type": "Explore"})).unwrap();
}

fn agente(m: &Mcp, nome: &str) -> Option<Value> {
    m.agentes()["agentes"].as_array().unwrap().iter().find(|a| a["agente"] == nome).cloned()
}

fn painel(m: &Mcp, d: &Mutex<Dados>, tarefa: Value) -> Value {
    let mut t = json!({"id": "d4e5f6a7", "agentType": "Explore", "status": "running", "description": "Explorar", "label": "Lendo src", "tokenCount": 1200});
    t.as_object_mut().unwrap().extend(tarefa.as_object().unwrap().clone());
    subagentes(m, d, &json!({"cwd": "C:\\Proj", "session_id": "s1", "columns": 80, "tasks": [t]})).unwrap()
}

#[test]
fn a_linha_traz_a_acao_o_rotulo_e_os_tokens_do_agente_que_os_hooks_criaram() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    let mut aviso = serde_json::Map::new();
    for (k, v) in [("blueprintId", "p1"), ("acaoId", "f2"), ("planoId", "t2"), ("estado", "escrevendo")] {
        aviso.insert(k.into(), json!(v));
    }
    m.registrar_atividade("Claude", aviso);
    comeca(&m, &d, "d4e5f6a7");
    let r = painel(&m, &d, json!({}));
    assert_eq!(r["linhas"][0]["id"], "d4e5f6a7");
    assert_eq!(r["linhas"][0]["content"], "Explore d4e5 · ação 1.2 · Lendo src · 1.2k tokens");
    let a = agente(&m, "Explore d4e5").unwrap();
    assert_eq!((a["mensagem"].as_str(), a["tokens"].clone(), a["rotulo"].as_str(), a["planoId"].as_str()), (Some("Lendo src"), json!(1200), Some("Lendo src"), Some("t2")));
    assert_eq!(m.agentes()["agentes"].as_array().unwrap().len(), 2, "o Claude e o subagente: nenhum agente a mais");
}

#[test]
fn o_tick_repetido_nao_vira_evento_e_os_tokens_so_mudam_com_um_salto() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    comeca(&m, &d, "d4e5f6a7");
    painel(&m, &d, json!({}));
    let itens = m.itens().len();
    painel(&m, &d, json!({"tokenCount": 1300}));
    painel(&m, &d, json!({"tokenCount": 1300}));
    assert_eq!((m.itens().len(), agente(&m, "Explore d4e5").unwrap()["tokens"].clone()), (itens, json!(1200)), "mesmo rótulo e salto pequeno: nada novo");
    painel(&m, &d, json!({"tokenCount": 1800}));
    assert_eq!(agente(&m, "Explore d4e5").unwrap()["tokens"], 1800);
    assert_eq!(m.itens().len(), itens, "só os tokens mudaram: sem evento");
    painel(&m, &d, json!({"tokenCount": 1800, "label": "Escrevendo o relatório"}));
    assert_eq!((m.itens().len(), agente(&m, "Explore d4e5").unwrap()["mensagem"].clone()), (itens + 1, json!("Escrevendo o relatório")));
}

#[test]
fn falhou_e_cancelado_deixam_de_parecer_concluido() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    comeca(&m, &d, "d4e5f6a7");
    comeca(&m, &d, "9a8b7c6d");
    let corpo = json!({"hook_event_name": "SubagentStop", "cwd": "C:\\Proj", "agent_id": "d4e5f6a7", "agent_type": "Explore", "last_assistant_message": "Tudo certo"});
    subagente(&m, &d, &corpo).unwrap(); // o hook concluiu...
    let r = painel(&m, &d, json!({"status": "failed"})); // ...mas o painel diz que falhou
    let a = agente(&m, "Explore d4e5").unwrap();
    assert_eq!((a["estado"].as_str(), a["mensagem"].as_str()), (Some("bloqueado"), Some("o subagente falhou")));
    assert!(r["linhas"][0]["content"].as_str().unwrap().contains("falhou"));
    let itens = m.itens().len();
    painel(&m, &d, json!({"status": "failed"}));
    assert_eq!(m.itens().len(), itens, "o mesmo estado no tick seguinte não repete o evento");
    painel(&m, &d, json!({"id": "9a8b7c6d", "status": "killed"}));
    let b = agente(&m, "Explore 9a8b").unwrap();
    assert_eq!((b["estado"].as_str(), b["mensagem"].as_str()), (Some("bloqueado"), Some("o subagente foi cancelado")));
}

#[test]
fn concluido_pelo_hook_mantem_a_ultima_mensagem_e_sem_o_hook_o_painel_conclui() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    comeca(&m, &d, "d4e5f6a7");
    comeca(&m, &d, "9a8b7c6d");
    subagente(&m, &d, &json!({"hook_event_name": "SubagentStop", "cwd": "C:\\Proj", "agent_id": "d4e5f6a7", "agent_type": "Explore", "last_assistant_message": "Achei 3 arquivos"})).unwrap();
    painel(&m, &d, json!({"status": "completed"}));
    assert_eq!(agente(&m, "Explore d4e5").unwrap()["mensagem"], "Achei 3 arquivos", "a mensagem do SubagentStop vale mais que o rótulo");
    painel(&m, &d, json!({"id": "9a8b7c6d", "status": "completed", "label": "Resumo pronto"}));
    let b = agente(&m, "Explore 9a8b").unwrap();
    assert_eq!((b["estado"].as_str(), b["mensagem"].as_str()), (Some("concluido"), Some("Resumo pronto")));
}

#[test]
fn quem_os_hooks_nao_conhecem_fica_sem_linha_e_nao_vira_agente() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    assert_eq!(painel(&m, &d, json!({})), json!({"linhas": []}));
    assert!(m.agentes()["agentes"].as_array().unwrap().is_empty(), "o painel nunca cria agente");
    comeca(&m, &d, "d4e5f6a7");
    for tarefa in [json!({"id": ""}), json!({"id": "!!!"}), json!({"id": "zzzz9999"})] {
        assert_eq!(painel(&m, &d, tarefa)["linhas"], json!([]));
    }
    for corpo in [json!({}), Value::Null, json!({"cwd": "C:\\Solta", "tasks": [{"id": "d4e5f6a7", "agentType": "Explore"}]}), json!({"cwd": "C:\\Proj", "tasks": "x"}), json!({"cwd": "C:\\Proj", "tasks": []})] {
        assert_eq!(subagentes(&m, &d, &corpo).unwrap(), json!({"linhas": []}));
    }
    assert_eq!(m.agentes()["agentes"].as_array().unwrap().len(), 1);
}

#[test]
fn a_linha_cabe_nas_colunas_e_nunca_leva_controle_do_terminal() {
    let longo = "x".repeat(300);
    let l = linha("Explore d4e5", "running", Some("1.2"), &longo, Some(12_345), 60);
    assert!(l.chars().count() <= 60 && l.starts_with("Explore d4e5 · ação 1.2 · x") && l.contains('…') && l.ends_with("12.3k tokens"), "{l}");
    assert_eq!(linha("Explore d4e5", "running", None, "", None, 60), "Explore d4e5");
    assert_eq!(linha("A", "killed", None, "x", Some(950), 80), "A · cancelado · x · 950 tokens");
    assert_eq!(com(Idioma::En, || linha("A", "failed", Some("2.9"), "x", None, 80)), "A · failed · action 2.9 · x");
    // o rótulo vem do modelo: escape ANSI, quebra de linha e BEL não passam
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:\\Proj");
    comeca(&m, &d, "d4e5f6a7");
    let r = painel(&m, &d, json!({"label": "ok\u{1b}[31m vermelho\n\u{7}fim", "name": "bom\u{1b}nome"}));
    let c = r["linhas"][0]["content"].as_str().unwrap();
    assert!(!c.chars().any(char::is_control) && c.starts_with("bomnome") && c.contains("ok[31m vermelhofim"), "{c:?}");
}

#[test]
fn o_nome_e_o_mesmo_que_os_hooks_dao_e_os_tokens_saem_curtos() {
    assert_eq!(nome_do("a9cff85187b11bee4", "Explore").as_deref(), Some("Explore a9cf"));
    assert_eq!(nome_do("z9\n", "a;b\u{7}c").as_deref(), Some("abc z9"));
    assert_eq!(nome_do("x1", "").as_deref(), Some("subagente x1"));
    assert_eq!(nome_do("--", "Explore"), None);
    assert_eq!((tokens_curtos(950).as_str(), tokens_curtos(12_300).as_str(), tokens_curtos(1_000).as_str()), ("950", "12.3k", "1.0k"));
}
