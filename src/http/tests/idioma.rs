//! O idioma chega pelo Accept-Language: o mesmo erro sai em inglês ou em português, sempre com o mesmo código estável.
use super::{estado, json_de, pedido, texto_de};
use crate::http::rotas::despachar;
use crate::http::Estado;
use serde_json::{json, Value};

fn post(e: &Estado, rota: &str, idioma: &str, corpo: Value) -> Value {
    json_de(&despachar(e, &pedido("POST", &format!("/api/mcp/{rota}"), &[("Accept-Language", idioma)], Some(corpo))))
}

#[test]
fn erro_de_rota_no_idioma_do_pedido_com_o_mesmo_codigo() {
    let e = estado(false);
    let corpo = json!({"tipo": "inventado", "texto": "x", "workspaceId": "sm", "blueprintId": "p1"});
    let en = post(&e, "evento", "en-US,en;q=0.9", corpo.clone());
    let pt = post(&e, "evento", "pt-BR,pt;q=0.9", corpo);
    assert_eq!(en, json!({"erro": "invalid event type", "codigo": "tipo-invalido"}));
    assert_eq!(pt, json!({"erro": "tipo de evento inválido", "codigo": "tipo-invalido"}));
    let fr = post(&e, "evento", "fr-FR", json!({"tipo": "inventado"}));
    assert_eq!(fr["erro"], "invalid event type", "idioma sem catálogo: inglês");
}

#[test]
fn erro_com_parametros_traz_os_valores_fora_do_texto() {
    let e = estado(false);
    let r = post(&e, "proposta", "en", json!({"workspaceId": "sm", "blueprintId": "p1", "ajustar": 7, "acoes": [{"nome": "x"}]}));
    assert_eq!((r["codigo"].clone(), r["params"]["numero"].clone()), (json!("proposta-nao-encontrada"), json!(7)));
    assert!(r["erro"].as_str().unwrap().starts_with("There is no proposal no. 7 in the queue."), "{r}");
}

#[test]
fn validacao_da_proposta_no_idioma_do_pedido() {
    let e = estado(false);
    let en = post(&e, "proposta", "en", json!({"workspaceId": "sm", "blueprintId": "p1", "acoes": []}));
    let pt = post(&e, "proposta", "pt-BR", json!({"workspaceId": "sm", "blueprintId": "p1", "acoes": []}));
    assert!(en["erros"][0].as_str().unwrap().starts_with("`acoes` is required"), "{en}");
    assert!(pt["erros"][0].as_str().unwrap().starts_with("`acoes` é obrigatório"), "{pt}");
}

#[test]
fn recusas_do_iniciar_e_textos_simples_tambem_seguem_o_idioma() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/mcp/iniciar", &[("Accept-Language", "en")], Some(json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "nada"}))));
    assert_eq!((r.status, json_de(&r)), (404, json!({"erro": "Action not found in this blueprint.", "codigo": "acao-nao-encontrada"})));
    let r = despachar(&e, &pedido("GET", "/api/mcp/nada", &[("Accept-Language", "en")], None));
    assert_eq!((r.status, texto_de(&r)), (404, "not found".to_string()));
    let r = despachar(&e, &pedido("GET", "/api/mcp/nada", &[("Accept-Language", "pt-BR")], None));
    assert_eq!(texto_de(&r), "não encontrado");
}

#[test]
fn sem_accept_language_vale_o_idioma_do_usuario() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/mcp/evento", &[], Some(json!({"tipo": "inventado"}))));
    assert_eq!(json_de(&r)["erro"], crate::tr!("invalid event type", "tipo de evento inválido"));
}
