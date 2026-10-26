//! As rotas da frente SINCRONIA pelo despacho: /api/mudancas, /api/eventos, /api/presenca e /api/historico.
use super::{estado, json_de, pedido};
use crate::http::pedido::Corpo;
use crate::http::rotas::{despachar, limite_corpo};
use serde_json::{json, Value};
use std::io::Read;

fn mudar(e: &crate::http::Estado, corpo: Value) -> crate::http::pedido::Resposta {
    despachar(e, &pedido("POST", "/api/mudancas", &[], Some(corpo)))
}

fn versao(e: &crate::http::Estado) -> u64 {
    despachar(e, &pedido("GET", "/api/dados", &[], None)).cabecalho("x-dados-versao").unwrap().parse().unwrap()
}

#[test]
fn mudancas_aplica_sobe_a_versao_e_a_leitura_ve_o_novo() {
    let e = estado(false);
    let v0 = versao(&e);
    let r = mudar(&e, json!({"base": v0, "cliente": "c1", "ops": [{"op": "alterar", "tipo": "plano", "id": "t1", "campos": {"texto": "Novo"}, "antes": {"texto": "Criar rota"}}]}));
    assert_eq!(r.status, 200);
    assert_eq!(json_de(&r), json!({"versao": v0 + 1, "conflitos": [], "ignoradas": []}));
    assert_eq!(r.cabecalho("x-dados-versao"), Some((v0 + 1).to_string().as_str()));
    let dados = json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None)));
    assert_eq!(dados["blueprints"][0]["acoes"][0]["planos"][0]["texto"], "Novo");
    assert_eq!(versao(&e), v0 + 1);
    // nada mudou de fato: a versão fica
    let r = mudar(&e, json!({"ops": [{"op": "alterar", "tipo": "plano", "id": "t1", "campos": {"texto": "Novo"}}]}));
    assert_eq!(json_de(&r)["versao"], v0 + 1);
}

#[test]
fn mudancas_recusa_lote_malformado_sem_mexer_em_nada() {
    let e = estado(false);
    let r = mudar(&e, json!({"ops": [{"op": "alterar", "tipo": "plano", "id": "t1", "campos": {"texto": "x"}}, {"op": "apagar", "tipo": "nada", "id": "t2"}]}));
    assert_eq!(r.status, 400);
    let j = json_de(&r);
    assert_eq!((j["codigo"].as_str(), j["params"]["indice"].as_u64()), (Some("operacao-invalida"), Some(1)));
    let r = mudar(&e, json!({"ops": [{"op": "alterar", "tipo": "plano", "id": "t1", "campos": {"texto": "x"}}, {"op": "alterar", "tipo": "acao", "id": "f1", "campos": {"planos": []}}]}));
    assert_eq!((r.status, json_de(&r)["params"]["indice"].as_u64()), (400, Some(1)));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/dados", &[], None)))["blueprints"][0]["acoes"][0]["planos"][0]["texto"], "Criar rota");
    assert_eq!(despachar(&e, &pedido("GET", "/api/mudancas", &[], None)).status, 405);
    let sem_tipo = crate::http::pedido::Pedido::novo("POST", "/api/mudancas", vec![], b"{}".to_vec());
    assert_eq!(despachar(&e, &sem_tipo).status, 415);
    assert_eq!(limite_corpo("POST", "/api/mudancas"), 2 * 1024 * 1024);
}

/// Lê do fluxo SSE até achar `ate` (o fluxo dorme no canal; só se lê o que já foi enviado).
fn ler_ate(r: crate::http::pedido::Resposta, ate: &str) -> String {
    let Corpo::Fluxo(mut leitor) = r.corpo else { panic!("não é fluxo") };
    let mut texto = String::new();
    let mut trecho = [0u8; 4096];
    while !texto.contains(ate) {
        let n = leitor.read(&mut trecho).unwrap();
        texto.push_str(&String::from_utf8_lossy(&trecho[..n]));
    }
    texto
}

#[test]
fn eventos_leva_o_lote_a_quem_le_o_workspace_e_retoma_pelo_last_event_id() {
    let e = estado(false);
    let v0 = versao(&e);
    let ao_vivo = despachar(&e, &pedido("GET", &format!("/api/eventos?workspace=sm&desde={v0}"), &[], None));
    assert_eq!((ao_vivo.status, ao_vivo.cabecalho("content-type")), (200, Some("text/event-stream; charset=utf-8")));
    mudar(&e, json!({"cliente": "c-ana", "ops": [{"op": "criar", "tipo": "plano", "id": "t9", "pai": "f1", "posicao": 0, "valor": {"texto": "Novo"}}]}));
    let texto = ler_ate(ao_vivo, "event: mudancas");
    assert!(texto.contains(&format!("id: {}\nevent: mudancas\n", v0 + 1)), "{texto}");
    let dados: Value = serde_json::from_str(texto.split("data: ").nth(1).unwrap().lines().next().unwrap()).unwrap();
    assert_eq!((dados["blueprintId"].as_str(), dados["cliente"].as_str(), dados["autor"]["id"].as_str()), (Some("p1"), Some("c-ana"), Some("local")));
    // reconectar com o Last-Event-ID antigo reenvia o lote; com um id de antes de um PUT, recarregar
    let r = despachar(&e, &pedido("GET", "/api/eventos?workspace=sm", &[("Last-Event-ID", &v0.to_string())], None));
    assert!(ler_ate(r, "event: mudancas").contains("\"t9\""));
    despachar(&e, &pedido("PUT", "/api/dados", &[], Some(super::fixture())));
    let r = despachar(&e, &pedido("GET", "/api/eventos?workspace=sm", &[("Last-Event-ID", &v0.to_string())], None));
    assert!(ler_ate(r, "event: recarregar").contains("recarregar"));
    assert_eq!(despachar(&e, &pedido("GET", "/api/eventos", &[], None)).status, 400, "falta o workspace");
}

#[test]
fn presenca_no_modo_local_so_responde_ok_e_historico_lista_quem_mudou() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/presenca", &[], Some(json!({"workspaceId": "sm", "blueprintId": "p1", "editando": {"tipo": "plano", "id": "t1"}, "cliente": "c1"}))));
    assert_eq!((r.status, json_de(&r)), (200, json!({"ok": true})));
    mudar(&e, json!({"ops": [{"op": "alterar", "tipo": "plano", "id": "t1", "campos": {"texto": "A"}}]}));
    let h = json_de(&despachar(&e, &pedido("GET", "/api/historico?workspace=sm&item=t1", &[], None)));
    let l = &h["itens"][0];
    assert_eq!((l["campo"].as_str(), l["antes"].as_str(), l["depois"].as_str(), l["membro"]["id"].as_str()), (Some("texto"), Some("Criar rota"), Some("A"), Some("local")));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/historico?workspace=w2", &[], None)))["itens"], json!([]));
}
