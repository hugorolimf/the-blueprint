//! Todas as rotas com cada papel (docs/colaboracao.md, seção 3): o servidor decide, nunca a página.
use super::*;

struct Pessoas {
    todos: String,
    dono: String,
    colab: String,
    sup: String,
    visit: String,
}

fn pessoas(e: &Estado) -> Pessoas {
    Pessoas {
        todos: pessoa(e, Papel::Dono, "*", "", "Servidor"),
        dono: pessoa(e, Papel::Dono, "w2", "", "Dona"),
        colab: pessoa(e, Papel::Colaborador, "w2", "", "Colab"),
        sup: pessoa(e, Papel::Supervisor, "w2", "", "Sup"),
        visit: pessoa(e, Papel::Visitante, "w2", "p10", ""),
    }
}

fn evento() -> Value {
    json!({"tipo": "progresso", "workspaceId": "w2", "blueprintId": "p9", "texto": "trabalhando"})
}

#[test]
fn cada_rota_responde_conforme_o_papel() {
    let e = estado();
    let p = pessoas(&e);
    let convite_novo = json!({"workspaceId": "w2", "papel": "colaborador"});
    // (método, url, corpo, [todos, dono, colaborador, supervisor, visitante])
    let casos: Vec<(&str, &str, Option<Value>, [u16; 5])> = vec![
        ("GET", "/api/dados", None, [200, 200, 200, 200, 200]),
        ("PUT", "/api/dados", Some(fixture()), [200, 403, 403, 403, 403]),
        ("GET", "/api/sessao", None, [200, 200, 200, 200, 200]),
        ("GET", "/api/mcp/blueprints", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/blueprint?workspace=w2&blueprint=p9", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/blueprint?workspace=sm&blueprint=p1", None, [200, 404, 404, 404, 403]),
        ("GET", "/api/mcp/fila", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/agentes", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/info", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/linha-do-tempo?workspace=w2&blueprint=p9&acao=a", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/mcp/linha-do-tempo?workspace=sm&blueprint=p1&acao=f1", None, [200, 403, 403, 403, 403]),
        ("POST", "/api/mcp/evento", Some(evento()), [200, 200, 200, 403, 403]),
        ("POST", "/api/mcp/aviso", Some(json!({"tipo": "proposta-aplicada", "workspaceId": "w2", "blueprintId": "p9"})), [200, 200, 200, 403, 403]),
        ("POST", "/api/mcp/prototipo/remover", Some(json!({"prototipoId": "0123456789abcdef"})), [200, 403, 403, 403, 403]),
        ("POST", "/api/mcp/fila/ack", Some(json!({"ids": []})), [200, 200, 200, 403, 403]),
        ("GET", "/api/pessoas?workspace=w2", None, [200, 200, 403, 200, 403]),
        ("GET", "/api/pessoas/para-atribuir?workspace=w2", None, [200, 200, 200, 403, 403]),
        ("GET", "/api/pessoas/para-atribuir?workspace=sm", None, [200, 403, 403, 403, 403]),
        ("GET", "/api/convites?workspace=w2", None, [200, 200, 403, 403, 403]),
        ("POST", "/api/convites", Some(convite_novo), [201, 201, 403, 403, 403]),
        ("GET", "/api/atividade?workspace=w2", None, [200, 200, 403, 200, 403]),
        ("GET", "/api/chave", None, [200, 200, 200, 200, 403]),
        ("GET", "/api/pessoas?workspace=sm", None, [200, 403, 403, 403, 403]),
    ];
    let quem = [&p.todos, &p.dono, &p.colab, &p.sup, &p.visit];
    for (metodo, url, corpo, esperado) in casos {
        for (i, cookie) in quem.iter().enumerate() {
            let r = despachar(&e, &pedido(metodo, url, Some(cookie), corpo.clone()));
            assert_eq!(r.status, esperado[i], "{metodo} {url} papel #{i}: {}", String::from_utf8_lossy(r.bytes()));
        }
    }
    let r = despachar(&e, &pedido("PUT", "/api/dados", Some(&p.colab), Some(fixture())));
    assert_eq!(json_de(&r)["codigo"], "use-mudancas");
}

#[test]
fn cada_um_le_so_o_que_o_papel_alcanca() {
    let e = estado();
    let p = pessoas(&e);
    let colab = json_de(&despachar(&e, &pedido("GET", "/api/dados", Some(&p.colab), None)));
    assert_eq!(colab["blueprints"], json!([]), "sem acesso à raiz, nenhum blueprint dela");
    assert_eq!(colab["empresa"], "Raiz");
    assert!(colab.get("usuario").is_none() && colab.get("chats").is_none());
    assert_eq!(colab["workspaces"][0]["blueprints"].as_array().unwrap().len(), 2);
    let visit = json_de(&despachar(&e, &pedido("GET", "/api/dados", Some(&p.visit), None)));
    assert_eq!(visit["workspaces"][0]["blueprints"], json!([{"id": "p10", "nome": "Público", "acoes": []}]));
    let todos = json_de(&despachar(&e, &pedido("GET", "/api/dados", Some(&p.todos), None)));
    assert_eq!(todos, fixture(), "o dono de todos lê tudo");
    let lista = json_de(&despachar(&e, &pedido("GET", "/api/mcp/blueprints", Some(&p.sup), None)));
    assert!(!lista.to_string().contains("Portal"), "o MCP também lê só o que o papel alcança: {lista}");
}

#[test]
fn visitante_nao_le_comentarios_do_agente() {
    let mut v = fixture();
    crate::acesso::filtro::para_visitante(&mut v);
    assert!(v["blueprints"][0]["acoes"][0]["planos"][0]["agente"].get("comentarios").is_none());
    assert_eq!(v["blueprints"][0]["acoes"][0]["planos"][0]["agente"]["estado"], "ok");
    let ctx = Contexto::visitante("c", "sm", "p1", "folha");
    let lido = crate::acesso::filtro::filtrar(&fixture(), &ctx).into_owned();
    assert!(!lido.to_string().contains("segredo do agente") && lido.get("chats").is_none() && lido.get("usuario").is_none());
    assert_eq!(lido["blueprints"].as_array().unwrap().len(), 1);
}

#[test]
fn a_fila_leva_o_autor_e_so_aparece_para_quem_le_o_alvo() {
    let e = estado();
    let p = pessoas(&e);
    let raiz = pessoa(&e, Papel::Colaborador, "sm", "", "Só raiz");
    assert_eq!(despachar(&e, &pedido("POST", "/api/mcp/evento", Some(&p.colab), Some(evento()))).status, 200);
    let fila = json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", Some(&p.sup), None)));
    let item = &fila["itens"][0];
    assert_eq!(item["autor"]["nome"], "Colab");
    assert!(item["autor"]["id"].as_str().is_some_and(|id| id.len() == 16));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", Some(&raiz), None)))["itens"], json!([]));
    // supervisor não aplica nem descarta; o ack de quem não grava o alvo não tira nada
    let id = item["id"].as_i64().unwrap();
    assert_eq!(despachar(&e, &pedido("POST", "/api/mcp/fila/ack", Some(&raiz), Some(json!({"ids": [id]})))).status, 200);
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", Some(&p.sup), None)))["itens"].as_array().unwrap().len(), 1);
    assert_eq!(despachar(&e, &pedido("POST", "/api/mcp/fila/ack", Some(&p.colab), Some(json!({"ids": [id]})))).status, 200);
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", Some(&p.sup), None)))["itens"], json!([]));
}

#[test]
fn prototipo_so_para_quem_le_o_blueprint() {
    let e = estado();
    let p = pessoas(&e);
    std::fs::create_dir_all(&e.mcp.prototipos).unwrap();
    std::fs::write(e.mcp.prototipos.join("0123456789abcdef.html"), "<p>x</p>").unwrap();
    let url = "/prototipos/0123456789abcdef.html";
    let r = despachar(&e, &pedido("GET", url, Some(&p.todos), None));
    assert_eq!((r.status, r.cabecalho("x-frame-options")), (200, Some("SAMEORIGIN")));
    assert_eq!(despachar(&e, &pedido("GET", url, Some(&p.colab), None)).status, 404);
    assert_eq!(despachar(&e, &pedido("GET", url, None, None)).status, 404);
}

#[test]
fn papel_e_remocao_nunca_deixam_o_workspace_sem_dono() {
    let e = estado();
    let p = pessoas(&e);
    let membros = json_de(&despachar(&e, &pedido("GET", "/api/pessoas?workspace=w2", Some(&p.dono), None)))["membros"].as_array().unwrap().clone();
    let id = |nome: &str| membros.iter().find(|m| m["nome"] == nome).unwrap()["id"].as_str().unwrap().to_string();
    assert!(membros.iter().any(|m| m["nome"] == "Dona" && m["voce"] == true));
    let (dona, colab, servidor) = (id("Dona"), id("Colab"), id("Servidor"));
    let papel = |cookie: &str, membro: &str, papel: &str| despachar(&e, &pedido("POST", "/api/pessoas/papel", Some(cookie), Some(json!({"workspaceId": "w2", "membroId": membro, "papel": papel}))));
    let r = papel(&p.dono, &dona, "colaborador");
    assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (200, None), "o dono de todos também é dono de w2");
    let r = papel(&p.todos, &dona, "dono");
    assert_eq!(r.status, 200);
    assert_eq!(papel(&p.dono, &servidor, "supervisor").status, 403, "o dono de todos só muda pelo terminal do servidor");
    assert_eq!(json_de(&papel(&p.dono, &colab, "visitante"))["codigo"], "papel-invalido");
    assert_eq!(papel(&p.colab, &dona, "supervisor").status, 403);
    let remover = |cookie: &str, membro: &str| despachar(&e, &pedido("POST", "/api/pessoas/remover", Some(cookie), Some(json!({"workspaceId": "w2", "membroId": membro}))));
    assert_eq!(remover(&p.dono, &colab).status, 200);
    assert_eq!(despachar(&e, &pedido("GET", "/api/dados", Some(&p.colab), None)).status, 401, "removido perde a sessão");
    assert!(!e.acesso.sessao_ativa(&Contexto::membro(&colab, "Colab", vec![])));
    assert_eq!(remover(&p.dono, &colab).status, 404);
    // sem o dono de todos, a Dona é a última dona de w2
    let e2 = estado();
    let dono2 = pessoa(&e2, Papel::Dono, "w2", "", "Única");
    let lista = json_de(&despachar(&e2, &pedido("GET", "/api/pessoas?workspace=w2", Some(&dono2), None)));
    let unica = lista["membros"][0]["id"].as_str().unwrap().to_string();
    for r in [despachar(&e2, &pedido("POST", "/api/pessoas/papel", Some(&dono2), Some(json!({"workspaceId": "w2", "membroId": unica, "papel": "colaborador"})))),
              despachar(&e2, &pedido("POST", "/api/pessoas/remover", Some(&dono2), Some(json!({"workspaceId": "w2", "membroId": unica}))))] {
        assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (409, Some("ultimo-dono")));
    }
    let atividade = json_de(&despachar(&e, &pedido("GET", "/api/atividade?workspace=w2&limite=50", Some(&p.sup), None)));
    let tipos: Vec<&str> = atividade["itens"].as_array().unwrap().iter().filter_map(|i| i["tipo"].as_str()).collect();
    assert!(tipos.contains(&"papel-mudado") && tipos.contains(&"membro-removido") && tipos.contains(&"entrou"), "{tipos:?}");
}
