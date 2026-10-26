//! /api/terminal/*: só loopback (nem MCP_REDE libera), JSON em todo POST e o "Iniciar" abrindo a sessão no pseudoconsole do app.
use super::{estado, json_de, pedido};
use crate::http::rotas::despachar;
use serde_json::json;

#[test]
fn so_esta_maquina_mesmo_com_mcp_rede() {
    let e = estado(true);
    let de_fora = crate::http::pedido::Pedido::novo("GET", "/api/terminal/sessoes", vec![("Host".into(), "192.168.0.9:3000".into())], vec![]);
    assert_eq!(despachar(&e, &de_fora).status, 403);
    let outro_site = pedido("POST", "/api/terminal/escrever", &[("Origin", "http://exemplo.com")], Some(json!({"id": "t1", "dados": "x"})));
    assert_eq!(despachar(&e, &outro_site).status, 403, "página de outro site não digita no terminal");
}

#[test]
fn post_sem_json_e_sessao_desconhecida() {
    let e = estado(false);
    let mut p = pedido("POST", "/api/terminal/fechar", &[], None);
    p.cabecalhos.push(("content-type".into(), "text/plain".into()));
    assert_eq!(despachar(&e, &p).status, 415);
    assert_eq!(despachar(&e, &pedido("GET", "/api/terminal/fluxo?id=t99", &[], None)).status, 404);
    assert_eq!(despachar(&e, &pedido("POST", "/api/terminal/escrever", &[], Some(json!({"id": "t99", "dados": "x"})))).status, 404);
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/terminal/sessoes", &[], None))), json!({"sessoes": []}));
}

#[test]
fn iniciar_sem_pasta_ligada_recusa_como_o_terminal_externo() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/terminal/iniciar", &[("Origin", "http://127.0.0.1:3000")], Some(json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f1"}))));
    assert_eq!((r.status, json_de(&r)["codigo"].clone()), (409, json!("sem-pasta")));
}

#[cfg(windows)]
#[test]
fn iniciar_abre_a_sessao_no_terminal_do_app() {
    use crate::contexto::Contexto;
    let e = estado(false);
    let pasta = std::env::temp_dir().join(format!("blueprint-terminal-rota-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    e.dados.lock().unwrap().vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    std::env::set_var(crate::terminal::rotas::VAR_COMANDO, "cmd.exe /q /k"); // só esta rota lê a variável: nada de claude (nem tokens) no teste
    let r = despachar(&e, &pedido("POST", "/api/terminal/iniciar", &[], Some(json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f1", "titulo": "1.1 · Base\u{7}", "colunas": 90, "linhas": 20}))));
    let j = json_de(&r);
    assert_eq!((r.status, j["terminal"].clone()), (200, json!("app")), "{j}");
    let id = j["sessao"]["id"].as_str().unwrap().to_string();
    assert_eq!((j["sessao"]["titulo"].clone(), j["sessao"]["acaoId"].clone(), j["sessao"]["viva"].clone()), (json!("1.1 · Base"), json!("f1"), json!(true)), "título sem controle");
    let escrever = despachar(&e, &pedido("POST", "/api/terminal/escrever", &[], Some(json!({"id": id, "dados": "exit\r"}))));
    assert_eq!(escrever.status, 200);
    assert_eq!(despachar(&e, &pedido("POST", "/api/terminal/tamanho", &[], Some(json!({"id": id, "colunas": 100, "linhas": 30})))).status, 200);
    assert_eq!(json_de(&despachar(&e, &pedido("POST", "/api/terminal/fechar", &[], Some(json!({"id": id}))))), json!({"ok": true}));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/terminal/sessoes", &[], None))), json!({"sessoes": []}));
    let _ = std::fs::remove_dir_all(&pasta);
}
