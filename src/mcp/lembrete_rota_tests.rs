//! Testes da ROTA do hook Stop (`POST /api/mcp/hook/parada`) pelo despacho HTTP real: edição sem plano gera um lembrete no formato do Claude Code
//! (`hookSpecificOutput.additionalContext` com `hookEventName: "Stop"`), uma vez; `stop_hook_active` não repete; registrar limpa; e as regras do /api/mcp.
use super::hooks_tests::{dados, ligar, mcp};
use super::rotas::rota;
use super::Mcp;
use crate::dados::Dados;
use crate::http::pedido::Pedido;
use serde_json::{json, Value};
use std::sync::Mutex;

fn pedido(caminho: &str, host: &str, tipo: &str, origin: Option<&str>, corpo: &Value) -> Pedido {
    let mut cab = vec![("host".to_string(), host.to_string()), ("content-type".to_string(), tipo.to_string())];
    if let Some(o) = origin {
        cab.push(("origin".into(), o.into()));
    }
    Pedido::novo("POST", caminho, cab, corpo.to_string().into_bytes())
}

/// POST como o hook `http` do Claude Code manda: loopback, JSON, sem Origin. Devolve (status, corpo).
fn post(m: &Mcp, d: &Mutex<Dados>, recurso: &str, corpo: &Value) -> (u16, Value) {
    let r = rota(m, d, &pedido(&format!("/api/mcp/{recurso}"), "127.0.0.1:3000", "application/json", None, corpo));
    (r.status, serde_json::from_slice(r.bytes()).unwrap_or(Value::Null))
}

fn editar(m: &Mcp, d: &Mutex<Dados>, sessao: &str) {
    let corpo = json!({"session_id": sessao, "hook_event_name": "PostToolUse", "cwd": "C:/Proj", "tool_name": "Edit", "tool_input": {"file_path": "C:/Proj/a.rs"}});
    assert_eq!(post(m, d, "hook/edicao", &corpo).0, 200);
}

fn stop(sessao: &str, ativo: bool) -> Value {
    json!({"session_id": sessao, "hook_event_name": "Stop", "cwd": "C:/Proj", "stop_hook_active": ativo, "last_assistant_message": "Pronto."})
}

/// O formato do Claude Code para hook Stop que dá retorno sem erro: só `hookSpecificOutput` com `hookEventName: "Stop"` e `additionalContext`.
fn eh_lembrete(r: &Value) -> bool {
    let saida = &r["hookSpecificOutput"];
    r.as_object().is_some_and(|o| o.len() == 1)
        && saida["hookEventName"] == "Stop"
        && saida["additionalContext"].as_str().is_some_and(|t| t.contains("registrar_trabalho") && t.contains("workspace_id sm") && t.contains("blueprint_id p1"))
}

#[test]
fn edicao_sem_plano_faz_o_stop_devolver_um_lembrete_e_so_um() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "s1");
    let (status, r) = post(&m, &d, "hook/parada", &stop("s1", false));
    assert_eq!(status, 200);
    assert!(eh_lembrete(&r), "{r}");
    assert_eq!(post(&m, &d, "hook/parada", &stop("s1", false)), (200, json!({})), "o lembrete sai uma vez por pendência");
}

#[test]
fn stop_com_stop_hook_active_nao_repete_o_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "s1");
    assert!(eh_lembrete(&post(&m, &d, "hook/parada", &stop("s1", false)).1));
    editar(&m, &d, "s1"); // o Claude continuou por causa do lembrete e editou de novo, ainda sem plano
    assert_eq!(post(&m, &d, "hook/parada", &stop("s1", true)), (200, json!({})), "continuação de um Stop: sem lembrete, sem loop");
}

#[test]
fn registrar_trabalho_pela_rota_limpa_o_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "s1");
    let registro = json!({"workspaceId": "sm", "blueprintId": "p1", "mensagem": "Corrigi a tela", "acoes": [{"id": "f2", "planos": [
        {"texto": "Corrigir a tela", "descricao": "Escopo: x. Validação: o teste passa.", "feito": true, "evidencia": "cargo test: passa"}]}]});
    let (status, r) = post(&m, &d, "registro", &registro);
    assert_eq!(status, 200, "{r}");
    assert_eq!(post(&m, &d, "hook/parada", &stop("s1", false)), (200, json!({})));
}

#[test]
fn registro_recusado_nao_limpa_o_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "s1");
    let sem_evidencia = json!({"workspaceId": "sm", "blueprintId": "p1", "acoes": [{"id": "f2", "planos": [{"texto": "x", "descricao": "Escopo: x. Validação: y.", "feito": true}]}]});
    assert_ne!(post(&m, &d, "registro", &sem_evidencia).0, 200);
    assert!(eh_lembrete(&post(&m, &d, "hook/parada", &stop("s1", false)).1));
}

#[test]
fn registrar_progresso_com_plano_limpa_e_sem_plano_nao() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    let progresso = |plano: Option<&str>| {
        let mut e = json!({"tipo": "progresso", "texto": "Criando a tela", "workspaceId": "sm", "blueprintId": "p1", "agente": "Claude", "estado": "escrevendo", "acaoId": "f2"});
        if let Some(p) = plano {
            e["planoId"] = json!(p);
        }
        e
    };
    editar(&m, &d, "s1");
    assert_eq!(post(&m, &d, "evento", &progresso(None)).0, 200);
    assert!(eh_lembrete(&post(&m, &d, "hook/parada", &stop("s1", false)).1), "progresso sem plano não conta como registro");
    editar(&m, &d, "s2");
    assert_eq!(post(&m, &d, "evento", &progresso(Some("t2"))).0, 200);
    assert_eq!(post(&m, &d, "hook/parada", &stop("s2", false)), (200, json!({})));
}

#[test]
fn sessao_sem_edicao_nao_recebe_lembrete() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "outra"); // a pendência de outra sessão não vaza
    for corpo in [stop("s1", false), json!({"hook_event_name": "Stop"}), json!({})] {
        assert_eq!(post(&m, &d, "hook/parada", &corpo), (200, json!({})), "{corpo}");
    }
}

#[test]
fn a_rota_do_stop_segue_as_regras_do_api_mcp() {
    let (m, d) = (mcp(), dados());
    ligar(&d, "C:/Proj");
    editar(&m, &d, "s1");
    let caminho = "/api/mcp/hook/parada";
    let corpo = stop("s1", false);
    assert_eq!(rota(&m, &d, &pedido(caminho, "evil.example", "application/json", None, &corpo)).status, 403, "Host de fora");
    assert_eq!(rota(&m, &d, &pedido(caminho, "127.0.0.1:3000", "application/json", Some("http://evil.example"), &corpo)).status, 403, "Origin de fora");
    assert_eq!(rota(&m, &d, &pedido(caminho, "127.0.0.1:3000", "text/plain", None, &corpo)).status, 415, "corpo sem JSON");
    let mut quebrado = pedido(caminho, "127.0.0.1:3000", "application/json", None, &corpo);
    quebrado.corpo = b"{nao".to_vec();
    assert_eq!(rota(&m, &d, &quebrado).status, 400, "JSON inválido");
    assert!(eh_lembrete(&post(&m, &d, "hook/parada", &corpo).1), "os pedidos recusados não gastaram o lembrete");
}
