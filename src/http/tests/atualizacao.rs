use super::{estado, json_de, pedido, texto_de};
use crate::http::rotas::despachar;
use serde_json::json;

#[test]
fn o_estado_da_atualizacao_so_vai_para_esta_maquina() {
    let e = estado(false);
    let r = despachar(&e, &pedido("GET", "/api/atualizacao", &[], None));
    assert_eq!(r.status, 200);
    let j = json_de(&r);
    assert_eq!((j["fase"].as_str(), j["atual"].as_str()), (Some("parada"), Some(crate::atualizador::VERSAO)));
    assert_eq!(r.cabecalho("cache-control"), Some("no-store"));
    // Host de outra máquina e Origin de outro site: recusados, como em /api/mcp
    let fora = crate::http::pedido::Pedido::novo("GET", "/api/atualizacao", vec![("Host".into(), "evil.example".into())], Vec::new());
    assert_eq!(despachar(&e, &fora).status, 403);
    let site = pedido("POST", "/api/atualizacao/instalar", &[("Origin", "http://evil.example")], Some(json!({})));
    assert_eq!(despachar(&e, &site).status, 403);
}

#[test]
fn todo_post_exige_json_e_o_metodo_certo() {
    let e = estado(false);
    let sem_tipo = crate::http::pedido::Pedido::novo("POST", "/api/atualizacao/verificar", vec![("Host".into(), "127.0.0.1:3000".into())], b"{}".to_vec());
    assert_eq!(despachar(&e, &sem_tipo).status, 415);
    assert_eq!(despachar(&e, &pedido("DELETE", "/api/atualizacao", &[], None)).status, 405);
    assert_eq!(despachar(&e, &pedido("GET", "/api/atualizacao/instalar", &[], None)).status, 404, "instalar não é GET: um link não instala");
    assert_eq!(despachar(&e, &pedido("POST", "/api/atualizacao/nada", &[], Some(json!({})))).status, 404);
}

#[test]
fn sem_atualizacao_pronta_o_clique_nao_instala_nada() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/atualizacao/instalar", &[], Some(json!({"caminho": "C:/qualquer.exe"}))));
    assert_eq!(r.status, 409);
    assert_eq!(json_de(&r)["codigo"], "sem-atualizacao", "{}", texto_de(&r));
}

#[test]
fn a_preferencia_automatica_exige_um_booleano() {
    let e = estado(false);
    for ruim in [json!({}), json!({"ligada": "sim"}), json!({"ligada": 1})] {
        assert_eq!(despachar(&e, &pedido("POST", "/api/atualizacao/automatica", &[], Some(ruim))).status, 400);
    }
}
