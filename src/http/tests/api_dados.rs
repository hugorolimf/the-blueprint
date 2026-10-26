use super::{estado, fixture, json_de, pedido, texto_de};
use crate::http::pedido::Pedido;
use crate::http::rotas::{despachar, limite_corpo};
use serde_json::json;

#[test]
fn ler_devolve_os_dados_e_gravar_troca_tudo() {
    let e = estado(false);
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None))), fixture());
    let novo = json!({"empresa": "Outra", "blueprints": [{"id": "x", "nome": "Novo", "acoes": []}]});
    let r = despachar(&e, &pedido("PUT", "/api/dados", &[], Some(novo.clone())));
    assert_eq!((r.status, texto_de(&r)), (200, r#"{"ok":true}"#.to_string()));
    assert_eq!(r.cabecalho("content-type"), Some("application/json; charset=utf-8"));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None))), novo);
}

#[test]
fn gravar_recusa_o_que_o_node_recusa() {
    let e = estado(false);
    let sem_tipo = Pedido::novo("PUT", "/api/dados", vec![("Host".into(), "127.0.0.1:3000".into())], b"{}".to_vec());
    assert_eq!(despachar(&e, &sem_tipo).status, 415);
    let ruim = Pedido::novo("PUT", "/api/dados", vec![("Content-Type".into(), "application/json".into())], b"{nao e json".to_vec());
    let r = despachar(&e, &ruim);
    assert_eq!((r.status, texto_de(&r)), (400, "erro".to_string()));
    let r = despachar(&e, &pedido("PUT", "/api/dados", &[], Some(json!({"empresa": "sem blueprints"}))));
    assert_eq!((r.status, texto_de(&r)), (400, "formato inválido: falta blueprints[]".to_string()));
    assert_eq!(despachar(&e, &pedido("DELETE", "/api/dados", &[], None)).status, 405);
    // o que foi recusado não mexeu em nada
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None))), fixture());
}

#[test]
fn um_plano_sem_id_e_recusado_sem_perder_os_dados() {
    let e = estado(false);
    let r = despachar(&e, &pedido("PUT", "/api/dados", &[], Some(json!({"blueprints": [{"id": "x", "acoes": [{"id": "f", "planos": [{"texto": "sem id"}]}]}]}))));
    assert_eq!(r.status, 400);
    assert!(texto_de(&r).contains("sem id"), "{}", texto_de(&r));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None))), fixture());
}

#[test]
fn os_limites_de_corpo_sao_os_do_node() {
    assert_eq!(limite_corpo("PUT", "/api/dados"), 10 * 1024 * 1024);
    assert_eq!(limite_corpo("POST", "/api/mcp/prototipo"), 2 * 1024 * 1024 + 64 * 1024);
    assert_eq!(limite_corpo("POST", "/api/mcp/proposta"), 256 * 1024);
}

fn gravar_na_versao(e: &crate::http::Estado, versao: Option<&str>, corpo: serde_json::Value) -> crate::http::pedido::Resposta {
    let mut cab = vec![("Content-Type".to_string(), "application/json".to_string())];
    if let Some(v) = versao { cab.push(("If-Match".into(), v.into())); }
    despachar(e, &Pedido::novo("PUT", "/api/dados", cab, serde_json::to_vec(&corpo).unwrap()))
}

#[test]
fn gravar_sobre_versao_velha_e_recusado_e_cada_gravacao_avisa_as_janelas() {
    let e = estado(false);
    let janela = e.mcp.assinar();
    let v1 = despachar(&e, &pedido("GET", "/api/dados", &[], None)).cabecalho("x-dados-versao").unwrap().to_string();
    let primeiro = json!({"blueprints": [{"id": "a", "nome": "A", "acoes": []}]});
    let r = gravar_na_versao(&e, Some(&v1), primeiro.clone());
    assert_eq!(r.status, 200);
    let v2 = r.cabecalho("x-dados-versao").unwrap().to_string();
    assert_ne!(v1, v2, "cada gravação sobe a versão");
    let ev = janela.recv_timeout(std::time::Duration::from_secs(1)).expect("a gravação avisa as janelas");
    assert_eq!((ev.nome, ev.dados), ("dados", Some(json!({"versao": v2.parse::<u64>().unwrap()}))));
    // uma janela que ainda tem a versão velha não apaga o que a outra gravou
    let r = gravar_na_versao(&e, Some(&v1), json!({"blueprints": [{"id": "velho", "nome": "Velho", "acoes": []}]}));
    assert_eq!(r.status, 409);
    assert_eq!(json_de(&r), json!({"erro": "desatualizado", "codigo": "desatualizado", "versao": v2.parse::<u64>().unwrap()}));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None))), primeiro);
    assert!(janela.try_recv().is_err(), "a gravação recusada não avisa ninguém");
    // com aspas, como um ETag, vale igual; sem If-Match segue valendo como antes (importadores e scripts simples)
    assert_eq!(gravar_na_versao(&e, Some(&format!("\"{v2}\"")), primeiro.clone()).status, 200);
    assert_eq!(gravar_na_versao(&e, None, primeiro).status, 200);
    assert_eq!(gravar_na_versao(&e, Some("lixo"), json!({"blueprints": []})).status, 409);
}
