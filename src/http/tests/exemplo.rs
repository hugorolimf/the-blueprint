//! O exemplo do primeiro uso (`GET /api/exemplo` + o workspace novo por `/api/mudancas`, como web/js/primeiro-uso.js faz) num banco
//! já preenchido: carregar e remover o exemplo deixa os dados antigos idênticos.
use super::{estado, json_de, pedido};
use crate::http::rotas::despachar;
use serde_json::{json, Value};

fn dados(e: &crate::http::Estado) -> Value {
    json_de(&despachar(e, &pedido("GET", "/api/dados", &[], None)))
}

/// O que a página faz com o modelo: os blueprints da raiz num workspace novo, com todo id trocado (aqui, com um prefixo).
fn como_workspace(modelo: &Value, id: &str) -> Value {
    let mut blueprints = modelo["blueprints"].clone();
    fn trocar(v: &mut Value, id: &str) {
        if let Some(o) = v.as_object_mut() {
            if let Some(Value::String(s)) = o.get_mut("id") {
                *s = format!("{id}-{s}");
            }
            for lista in ["acoes", "planos", "checkpoints"] {
                o.get_mut(lista).and_then(Value::as_array_mut).into_iter().flatten().for_each(|f| trocar(f, id));
            }
        }
    }
    blueprints.as_array_mut().unwrap().iter_mut().for_each(|b| trocar(b, id));
    json!({"nome": modelo["empresa"], "empresa": modelo["empresa"], "icone": "blueprint", "exemplo": true, "blueprints": blueprints})
}

fn mudar(e: &crate::http::Estado, ops: Value) -> Value {
    let r = despachar(e, &pedido("POST", "/api/mudancas", &[], Some(json!({"cliente": "teste", "ops": ops}))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)
}

#[test]
fn o_modelo_vem_no_idioma_do_pedido() {
    let e = estado(false);
    let en = json_de(&despachar(&e, &pedido("GET", "/api/exemplo", &[("accept-language", "en")], None)));
    let pt = json_de(&despachar(&e, &pedido("GET", "/api/exemplo", &[("accept-language", "pt-BR")], None)));
    assert_eq!((en["empresa"].as_str(), pt["empresa"].as_str()), (Some("Aurora Studio"), Some("Estúdio Aurora")));
    assert!(pt["blueprints"].as_array().is_some_and(|b| !b.is_empty()));
    assert_eq!(despachar(&e, &pedido("POST", "/api/exemplo", &[], Some(json!({})))).status, 405);
}

#[test]
fn carregar_e_remover_o_exemplo_nao_mexe_nos_dados_antigos() {
    let e = estado(false);
    let antes = dados(&e);
    let modelo = json_de(&despachar(&e, &pedido("GET", "/api/exemplo", &[], None)));
    let ws = como_workspace(&modelo, "ex1");
    mudar(&e, json!([{"op": "criar", "tipo": "workspace", "id": "ex1", "posicao": 99, "valor": ws}]));
    let depois = dados(&e);
    let novos = depois["workspaces"].as_array().unwrap();
    assert_eq!(novos.len(), antes["workspaces"].as_array().unwrap().len() + 1);
    let ex = novos.iter().find(|w| w["id"] == "ex1").expect("o workspace do exemplo");
    assert_eq!((ex["exemplo"].as_bool(), ex["blueprints"].as_array().map(Vec::len)), (Some(true), modelo["blueprints"].as_array().map(Vec::len)));
    assert_eq!(depois["blueprints"], antes["blueprints"], "a raiz continua igual");
    assert_eq!(&depois["workspaces"][0], &antes["workspaces"][0], "o outro workspace continua igual");
    // de novo, com outros ids: dois exemplos convivem e o primeiro não muda
    mudar(&e, json!([{"op": "criar", "tipo": "workspace", "id": "ex2", "valor": como_workspace(&modelo, "ex2")}]));
    assert_eq!(dados(&e)["workspaces"].as_array().unwrap().iter().find(|w| w["id"] == "ex1"), Some(ex));
    // remover: os dados voltam a ser exatamente os de antes
    mudar(&e, json!([{"op": "apagar", "tipo": "workspace", "id": "ex1"}, {"op": "apagar", "tipo": "workspace", "id": "ex2"}]));
    assert_eq!(dados(&e), antes);
}
