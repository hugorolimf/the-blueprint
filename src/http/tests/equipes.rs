use super::{estado, json_de, pedido};
use crate::http::rotas::despachar;
use serde_json::json;

#[test]
fn a_lista_de_equipes_so_vai_para_esta_maquina() {
    let e = estado(false);
    let r = despachar(&e, &pedido("GET", "/api/equipes", &[], None));
    assert_eq!(r.status, 200);
    assert!(json_de(&r)["equipes"].is_array());
    let fora = crate::http::pedido::Pedido::novo("GET", "/api/equipes", vec![("Host".into(), "evil.example".into())], Vec::new());
    assert_eq!(despachar(&e, &fora).status, 403);
    let site = pedido("POST", "/api/equipes/abrir", &[("Origin", "http://evil.example")], Some(json!({"origem": "https://x.trycloudflare.com"})));
    assert_eq!(despachar(&e, &site).status, 403);
}

#[test]
fn so_abre_o_que_o_app_ja_lembra_e_so_por_post_json() {
    let e = estado(false);
    // uma origem que nunca entrou pela janela da equipe não abre nada: o corpo não escolhe URL nova
    for rota in ["/api/equipes/abrir", "/api/equipes/esquecer"] {
        let r = despachar(&e, &pedido("POST", rota, &[], Some(json!({"origem": "https://nunca-entrei.example.com"}))));
        assert_eq!((r.status, json_de(&r)["codigo"].as_str().map(String::from)), (404, Some("equipe-desconhecida".into())), "{rota}");
    }
    let sem_tipo = crate::http::pedido::Pedido::novo("POST", "/api/equipes/abrir", vec![("Host".into(), "127.0.0.1:3000".into())], b"{}".to_vec());
    assert_eq!(despachar(&e, &sem_tipo).status, 415);
    assert_eq!(despachar(&e, &pedido("GET", "/api/equipes/abrir", &[], None)).status, 404, "abrir não é GET: um link não abre");
    assert_eq!(despachar(&e, &pedido("DELETE", "/api/equipes", &[], None)).status, 405);
}
