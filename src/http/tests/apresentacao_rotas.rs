//! Testes das rotas das apresentações geradas pela IA: anexar, substituir, listar, remover, a página isolada em sandbox e a recusa de fora desta máquina.
use super::{estado, json_de, pedido, texto_de};
use crate::http::pedido::{Pedido, Resposta};
use crate::http::rotas::{despachar, limite_corpo};
use crate::http::Estado;
use serde_json::{json, Value};

fn post(e: &Estado, rota: &str, corpo: Value) -> Resposta {
    despachar(e, &pedido("POST", &format!("/api/mcp/{rota}"), &[], Some(corpo)))
}

fn get(e: &Estado, url: &str) -> Resposta {
    despachar(e, &pedido("GET", url, &[], None))
}

fn anexo(extra: Value) -> Value {
    let mut c = json!({"workspaceId": "sm", "blueprintId": "p1",
        "conteudo": {"versao": 1, "publico": "cliente", "nivel": "padrao", "idioma": "pt-BR", "titulo": "Plano do Portal", "resumo": ["Você recebe o portal."]},
        "figuras": [{"id": "fichas", "titulo": "Fichas", "impressao": "abc", "codigo": "function (L) { return { svg: '<svg></svg>' }; }"}]});
    for (k, v) in extra.as_object().unwrap() {
        c[k] = v.clone();
    }
    c
}

fn limpar(e: &Estado) {
    let _ = std::fs::remove_dir_all(e.mcp.pasta_apresentacoes());
    let _ = std::fs::remove_dir_all(&e.mcp.prototipos);
}

#[test]
fn anexar_substituir_listar_e_remover() {
    let e = estado(false);
    assert_eq!(post(&e, "apresentacao", anexo(json!({"conteudo": null}))).status, 400);
    assert_eq!(post(&e, "apresentacao", anexo(json!({"workspaceId": "nao"}))).status, 404);
    assert_eq!(post(&e, "apresentacao", anexo(json!({"blueprintId": "nao"}))).status, 404);
    assert_eq!(post(&e, "apresentacao", anexo(json!({"blueprintId": ""}))).status, 400, "precisa de um blueprint");
    assert_eq!(post(&e, "apresentacao", anexo(json!({"apresentacaoId": "0123456789abcdef"}))).status, 404, "só substitui uma que o blueprint já tem");
    let r = post(&e, "apresentacao", anexo(json!({})));
    assert_eq!(r.status, 200, "{}", texto_de(&r));
    let resposta = json_de(&r);
    let id = resposta["apresentacaoId"].as_str().unwrap().to_string();
    assert_eq!((resposta["titulo"].clone(), resposta["caminho"].clone(), resposta["atualizado"].clone(), resposta["avisos"].clone()), (json!("Plano do Portal"), json!(format!("/apresentacao/{id}")), json!(false), json!([])));
    assert!(e.mcp.pasta_apresentacoes().join(format!("{id}.json")).is_file(), "fora do banco, em arquivo");

    let lista = json_de(&get(&e, "/api/mcp/apresentacoes?workspace=sm&blueprint=p1"));
    assert_eq!(lista["apresentacoes"][0]["id"], json!(id));
    assert_eq!((lista["apresentacoes"][0]["publico"].clone(), lista["apresentacoes"][0]["nivel"].clone(), lista["apresentacoes"][0]["figuras"].clone()), (json!("cliente"), json!("padrao"), json!([{"id": "fichas", "impressao": "abc"}])));
    assert!(json_de(&get(&e, "/api/mcp/apresentacoes?workspace=sm&blueprint=outro"))["apresentacoes"].as_array().unwrap().is_empty());
    let lido = json_de(&get(&e, "/api/mcp/blueprint?workspace=sm&blueprint=p1"));
    assert_eq!(lido["apresentacoes"][0]["id"], json!(id), "ler_blueprint mostra o que já existe, com as impressões");
    assert!(json_de(&get(&e, "/api/mcp/blueprint?workspace=sm&blueprint=p1&acao=f1")).get("apresentacoes").is_none(), "na ação inteira, não");

    // reenviar com o id substitui (mesmo id, mesmo arquivo)
    let novo = json_de(&post(&e, "apresentacao", anexo(json!({"apresentacaoId": id, "conteudo": {"versao": 1, "publico": "interno", "nivel": "rapido", "titulo": "Outro título"}}))));
    assert_eq!((novo["apresentacaoId"].clone(), novo["atualizado"].clone()), (json!(id), json!(true)));
    let lista = json_de(&get(&e, "/api/mcp/apresentacoes?workspace=sm&blueprint=p1"));
    assert_eq!((lista["apresentacoes"].as_array().unwrap().len(), lista["apresentacoes"][0]["titulo"].clone()), (1, json!("Outro título")));
    // o id de outro blueprint não deixa trocar o conteúdo dele
    let de_outro = json_de(&post(&e, "apresentacao", anexo(json!({"workspaceId": "w2", "blueprintId": "p9"}))));
    assert_eq!(post(&e, "apresentacao", anexo(json!({"apresentacaoId": de_outro["apresentacaoId"]}))).status, 404);

    assert_eq!(post(&e, "apresentacao/remover", json!({"apresentacaoId": "../../server"})).status, 400);
    assert_eq!(post(&e, "apresentacao/remover", json!({"apresentacaoId": id})).status, 200);
    assert!(!e.mcp.pasta_apresentacoes().join(format!("{id}.json")).exists(), "o arquivo também sai do disco");
    assert_eq!(post(&e, "apresentacao/remover", json!({"apresentacaoId": id})).status, 200, "idempotente");
    limpar(&e);
}

#[test]
fn os_avisos_voltam_na_resposta_e_o_corpo_grande_passa_ate_o_limite_do_contrato() {
    let e = estado(false);
    let r = json_de(&post(&e, "apresentacao", anexo(json!({"figuras": [{"id": "outra"}], "conteudo": {"versao": 1, "publico": "cliente", "nivel": "padrao", "titulo": "T", "etapas": [{"id": "e1", "figura": "fichas"}]}}))));
    let avisos = r["avisos"].to_string();
    assert!(avisos.contains("has no codigo") && avisos.contains("points to figure"), "{avisos}");
    assert_eq!(limite_corpo("POST", "/api/mcp/apresentacao"), 2 * 1024 * 1024 + 256 * 1024);
    assert_eq!(limite_corpo("POST", "/mcp"), 2 * 1024 * 1024 + 256 * 1024, "o MCP por HTTP leva a mesma ferramenta");
    assert_eq!(limite_corpo("POST", "/api/mcp/proposta"), 256 * 1024);
    limpar(&e);
}

#[test]
fn a_pagina_sai_isolada_em_sandbox_e_baixa_como_arquivo() {
    let e = estado(false);
    let id = json_de(&post(&e, "apresentacao", anexo(json!({}))))["apresentacaoId"].as_str().unwrap().to_string();
    let r = get(&e, &format!("/apresentacao/{id}"));
    if crate::apresentacao::arquivo("modelo.html").is_none() {
        assert_eq!(r.status, 503, "sem a biblioteca na compilação a rota avisa");
    } else {
        assert_eq!(r.status, 200, "{}", texto_de(&r));
        let cab = |r: &Resposta, n: &str| r.cabecalhos.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
        assert_eq!(cab(&r, "content-security-policy").unwrap(), "sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:");
        assert!(!cab(&r, "content-security-policy").unwrap().contains("allow-same-origin"), "nunca allow-same-origin");
        assert_eq!((cab(&r, "x-content-type-options").unwrap(), cab(&r, "cache-control").unwrap()), ("nosniff".into(), "no-store".into()));
        assert!(cab(&r, "content-type").unwrap().starts_with("text/html") && cab(&r, "content-disposition").is_none());
        assert!(texto_de(&r).contains("Plano do Portal"), "o conteúdo guardado entra na página");
        let baixar = get(&e, &format!("/apresentacao/{id}?baixar=1"));
        assert_eq!(baixar.status, 200);
        assert_eq!(cab(&baixar, "content-disposition").unwrap(), format!("attachment; filename=\"apresentacao-{id}.html\""));
    }
    for url in ["/apresentacao/ffffffffffffffff", "/apresentacao/curto", "/apresentacao/../server", "/apresentacao/0123456789abcdef.html"] {
        assert_eq!(get(&e, url).status, 404, "{url}");
    }
    assert_eq!(despachar(&e, &pedido("POST", &format!("/apresentacao/{id}"), &[], Some(json!({})))).status, 405);
    limpar(&e);
}

#[test]
fn apresentar_e_so_desta_maquina_nem_mcp_rede_libera() {
    let e = estado(true);
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "publico": "cliente", "nivel": "padrao"});
    let cabecalhos = vec![("Host".to_string(), "10.0.0.5:3000".to_string()), ("Content-Type".to_string(), "application/json".to_string())];
    let de_fora = despachar(&e, &Pedido::novo("POST", "/api/mcp/apresentar", cabecalhos, corpo.to_string().into_bytes()));
    assert_eq!(de_fora.status, 403, "{}", texto_de(&de_fora));
    assert_eq!(post(&e, "apresentar", json!({"workspaceId": "sm", "blueprintId": "p1", "publico": "todos", "nivel": "padrao"})).status, 400, "de dentro chega à validação");
    assert!(crate::acesso::so_local("/api/mcp/apresentar") && !crate::acesso::so_local("/api/mcp/apresentacao"), "no modo hospedado e para convidados o pedido de abrir terminal não existe");
    limpar(&e);
}

#[test]
fn ler_e_editar_pela_rota_com_operacoes_e_avisos() {
    let e = estado(false);
    let id = json_de(&post(&e, "apresentacao", anexo(json!({}))))["apresentacaoId"].as_str().unwrap().to_string();
    let lido = json_de(&get(&e, &format!("/api/mcp/apresentacao?id={id}")));
    assert_eq!((lido["titulo"].clone(), lido["conteudo"]["titulo"].clone(), lido["figuras"][0]["codigo"].is_string(), lido.get("editadoEm").is_none()), (json!("Plano do Portal"), json!("Plano do Portal"), true, true), "traz o código das figuras e nenhuma marca de edição");
    assert!(lido.get("workspaceId").is_none(), "só o que o contrato descreve");
    assert_eq!(get(&e, "/api/mcp/apresentacao?id=ffffffffffffffff").status, 404);
    assert_eq!(get(&e, "/api/mcp/apresentacao?id=../x").status, 404);

    let r = post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "operacoes": [{"op": "definir", "caminho": "titulo", "valor": "Plano revisado"}, {"op": "inserir", "caminho": "riscos", "valor": {"titulo": "Prazo", "texto": "Depende da conta."}}]}));
    assert_eq!(r.status, 200, "{}", texto_de(&r));
    let j = json_de(&r);
    assert_eq!((j["ok"].clone(), j["apresentacaoId"].clone(), j["avisos"].clone()), (json!(true), json!(id), json!([])));
    let em = j["editadoEm"].as_str().unwrap().to_string();
    let depois = json_de(&get(&e, &format!("/api/mcp/apresentacao?id={id}")));
    assert_eq!((depois["titulo"].clone(), depois["conteudo"]["riscos"][0]["titulo"].clone(), depois["editadoEm"].clone(), depois["editadoPor"].clone()), (json!("Plano revisado"), json!("Prazo"), json!(em), json!("usuario")));
    let lista = json_de(&get(&e, "/api/mcp/apresentacoes?workspace=sm&blueprint=p1"));
    assert_eq!((lista["apresentacoes"][0]["titulo"].clone(), lista["apresentacoes"][0]["editadoEm"].clone()), (json!("Plano revisado"), json!(em)), "a lista e o ler_blueprint trazem editadoEm");
    let lido_bp = json_de(&get(&e, "/api/mcp/blueprint?workspace=sm&blueprint=p1"));
    assert_eq!(lido_bp["apresentacoes"][0]["editadoEm"], json!(em));

    // o agente edita o código da figura em um trecho; o usuário e o agente editam campos diferentes
    let agente = json_de(&post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "autor": "agente", "operacoes": [{"op": "figura_trecho", "figura": "fichas", "de": "<svg></svg>", "para": "<svg id=\"x\"></svg>"}]})));
    assert_eq!(agente["ok"], json!(true));
    let ultimo = json_de(&get(&e, &format!("/api/mcp/apresentacao?id={id}")));
    assert_eq!((ultimo["editadoPor"].clone(), ultimo["titulo"].clone(), ultimo["figuras"][0]["codigo"].as_str().unwrap().contains("<svg id=\"x\"></svg>")), (json!("agente"), json!("Plano revisado"), true), "a edição do agente não desfez a do usuário");

    // erros: o mesmo contrato do anexar, e nada muda
    let r = post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "operacoes": [{"op": "figura_trecho", "figura": "fichas", "de": "nao-existe", "para": "x"}]}));
    assert_eq!((r.status, json_de(&r)["codigo"].clone()), (422, json!("trecho-ausente")));
    assert_eq!(json_de(&post(&e, "apresentacao/editar", json!({"apresentacaoId": id}))) ["codigo"], json!("nada-a-editar"));
    assert_eq!(post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "operacoes": [{"op": "definir", "caminho": "publico", "valor": "todos"}]})).status, 400);
    assert_eq!(post(&e, "apresentacao/editar", json!({"apresentacaoId": "ffffffffffffffff", "operacoes": []})).status, 404);
    assert_eq!(post(&e, "apresentacao/editar", json!({"apresentacaoId": "../x", "operacoes": []})).status, 400);
    assert_eq!(json_de(&get(&e, &format!("/api/mcp/apresentacao?id={id}")))["titulo"], json!("Plano revisado"), "as recusadas não gravam");
    // conteúdo e figuras inteiros (o "restaurar" do editor) e o limite do corpo da rota nova
    assert_eq!(post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "conteudo": anexo(json!({}))["conteudo"].clone()})).status, 200);
    assert_eq!(limite_corpo("POST", "/api/mcp/apresentacao/editar"), 2 * 1024 * 1024 + 256 * 1024);
    // a página servida já mostra o texto editado
    if crate::apresentacao::arquivo("modelo.html").is_some() {
        let r = post(&e, "apresentacao/editar", json!({"apresentacaoId": id, "operacoes": [{"op": "definir", "caminho": "titulo", "valor": "Titulo Editado Unico"}]}));
        assert_eq!(r.status, 200);
        assert!(texto_de(&get(&e, &format!("/apresentacao/{id}"))).contains("Titulo Editado Unico"));
    }
    limpar(&e);
}

#[test]
fn o_pedido_de_ajuste_e_so_desta_maquina_e_valida_o_texto() {
    let e = estado(true);
    let corpo = json!({"workspaceId": "sm", "blueprintId": "p1", "apresentacaoId": "0123456789abcdef", "texto": "mude"});
    let de_fora = despachar(&e, &Pedido::novo("POST", "/api/mcp/apresentacao/pedido", vec![("Host".into(), "10.0.0.5:3000".into()), ("Content-Type".into(), "application/json".into())], corpo.to_string().into_bytes()));
    assert_eq!(de_fora.status, 403, "{}", texto_de(&de_fora));
    let r = post(&e, "apresentacao/pedido", json!({"workspaceId": "sm", "blueprintId": "p1", "apresentacaoId": "0123456789abcdef", "texto": "mude"}));
    assert_eq!((r.status, json_de(&r)["codigo"].clone()), (404, json!("apresentacao-nao-encontrada")), "de dentro chega à validação");
    assert!(crate::acesso::so_local("/api/mcp/apresentacao/pedido") && !crate::acesso::so_local("/api/mcp/apresentacao/editar"), "no modo hospedado o pedido de abrir terminal não existe; a edição vale");
    limpar(&e);
}
