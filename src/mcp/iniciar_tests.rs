//! Testes de `POST /api/mcp/iniciar`: com um lançador falso (nenhum terminal de verdade abre), os argumentos exatos e todas as recusas.
use super::hooks_tests::{dados, mcp};
use super::iniciar::{iniciar, limpo};
use crate::contexto::Contexto;
use crate::http::pedido::Pedido;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

fn pasta_real() -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-iniciar-{}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn corpo(acao: &str) -> Value {
    json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": acao})
}

#[test]
fn abre_o_terminal_na_pasta_com_o_prompt_como_um_so_argumento() {
    let d = dados();
    let pasta = pasta_real();
    super::hooks::travado(&d).vincular_pasta(&Contexto::local(), pasta.to_str().unwrap(), "sm", "p1").unwrap();
    let chamadas: RefCell<Vec<(PathBuf, PathBuf, String)>> = RefCell::new(vec![]);
    let falso = |p: &Path, e: &Path, a: &str| {
        chamadas.borrow_mut().push((p.into(), e.into(), a.into()));
        Ok("wt")
    };
    let (status, resposta) = iniciar(&d, &corpo("f2"), &falso).unwrap();
    assert_eq!(status, 200, "{resposta}");
    assert_eq!((resposta["ok"].clone(), resposta["terminal"].clone()), (json!(true), json!("wt")));
    let c = chamadas.borrow();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].0.to_str().unwrap(), pasta.to_str().unwrap().replace('\\', "/").to_lowercase(), "a pasta como o banco a guarda");
    assert_eq!(c[0].1, std::env::current_exe().unwrap());
    assert_eq!(c[0].2, "Execute a ação 1.2 «Tela» do blueprint «Portal» (workspace_id sm, blueprint_id p1, acao_id f2): leia com ler_blueprint, registre o progresso com registrar_progresso e conclua cada plano com marcar_concluido e a evidência.");
}

#[test]
fn as_recusas_nao_abrem_terminal() {
    let d = dados();
    let nunca = |_: &Path, _: &Path, _: &str| -> Result<&'static str, String> { panic!("não podia abrir terminal") };
    assert_eq!(iniciar(&d, &json!({}), &nunca).unwrap_err().status, 400);
    assert_eq!(iniciar(&d, &json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f\u{0}2"}), &nunca).unwrap_err().status, 400, "id com controle");
    assert_eq!(iniciar(&d, &json!({"workspaceId": "nao", "blueprintId": "p1", "acaoId": "f2"}), &nunca).unwrap_err().status, 404);
    assert_eq!(iniciar(&d, &json!({"workspaceId": "sm", "blueprintId": "nao", "acaoId": "f2"}), &nunca).unwrap_err().status, 404);
    assert_eq!(iniciar(&d, &corpo("nao"), &nunca).unwrap_err().status, 404, "ação que não existe");
    let (status, r) = iniciar(&d, &corpo("f2"), &nunca).unwrap();
    assert_eq!((status, r["codigo"].clone()), (409, json!("sem-pasta")), "blueprint sem pasta ligada");
    super::hooks::travado(&d).vincular_pasta(&Contexto::local(), "C:/pasta/que/nao/existe/mais", "sm", "p1").unwrap();
    assert_eq!(iniciar(&d, &corpo("f2"), &nunca).unwrap().1["codigo"], "sem-pasta", "pasta ligada mas apagada do disco");
}

#[test]
fn sem_nenhum_terminal_devolve_501() {
    let d = dados();
    super::hooks::travado(&d).vincular_pasta(&Contexto::local(), pasta_real().to_str().unwrap(), "sm", "p1").unwrap();
    let (status, r) = iniciar(&d, &corpo("f2"), &|_: &Path, _: &Path, _: &str| Err("nada abriu".to_string())).unwrap();
    assert_eq!((status, r["codigo"].clone(), r["erro"].clone()), (501, json!("sem-terminal"), json!("nada abriu")));
}

#[test]
fn os_nomes_nao_levam_separador_do_windows_terminal_nem_controle() {
    assert_eq!(limpo("a; wt new-tab\ncmd /c calc"), "a, wt new-tabcmd /c calc");
    assert_eq!(limpo(&"x".repeat(200)).chars().count(), 80);
}

fn post(host: &str, origem: Option<&str>, tipo: &str) -> Pedido {
    let cab = |k: &str, v: &str| (k.to_string(), v.to_string());
    let mut h = vec![cab("host", host), cab("content-type", tipo)];
    if let Some(o) = origem {
        h.push(cab("origin", o));
    }
    Pedido::novo("POST", "/api/mcp/iniciar", h, corpo("f2").to_string().into_bytes())
}

#[test]
fn a_rota_so_responde_em_loopback_com_json_e_o_mcp_rede_nao_libera() {
    let d = dados();
    let m = mcp();
    let r = |p: &Pedido| super::rotas::rota(&m, &d, p).status;
    assert_eq!(r(&post("evil.example", None, "application/json")), 403);
    assert_eq!(r(&post("127.0.0.1:3000", Some("http://evil.example"), "application/json")), 403);
    assert_eq!(r(&post("127.0.0.1:3000", None, "text/plain")), 415);
    // MCP_REDE=1 libera as outras rotas, mas não esta
    let rede = super::Mcp::novo("0.0.0.0", true, "blueprint.exe", "http://0.0.0.0:3000", std::env::temp_dir());
    assert_eq!(super::rotas::rota(&rede, &d, &post("192.168.0.5:3000", None, "application/json")).status, 403);
    assert_eq!(super::rotas::rota(&rede, &d, &post("127.0.0.1:3000", None, "application/json")).status, 403, "o servidor escuta em 0.0.0.0, não em loopback");
    // loopback de verdade, sem pasta ligada: passa pelas portas e para na recusa de negócio (nenhum terminal abre)
    assert_eq!(r(&post("127.0.0.1:3000", None, "application/json")), 409);
}
