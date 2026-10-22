use super::{estado, json_de, pedido, texto_de};
use crate::http::pedido::{Pedido, Resposta};
use crate::http::rotas::despachar;
use crate::http::Estado;
use serde_json::{json, Value};

fn post(e: &Estado, rota: &str, corpo: Value) -> Resposta {
    despachar(e, &pedido("POST", &format!("/api/mcp/{rota}"), &[], Some(corpo)))
}

fn com(mut base: Value, extra: Value) -> Value {
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn proposta_valida() -> Value {
    let planos: Vec<Value> = [1, 2, 3].iter().map(|n| json!({"texto": format!("Criar item {n}"), "descricao": "Validação: roda."})).collect();
    json!({"workspaceId": "sm", "blueprintId": "p1", "mensagem": "pelo teste", "acoes": [{"nome": "Nova", "inicio": "2030-01-01", "fim": "2030-01-31", "planos": planos, "checkpoints": [{"nome": "Pronto", "data": "2030-01-30"}]}]})
}

fn com_cabecalhos(metodo: &str, url: &str, cabecalhos: &[(&str, &str)]) -> Pedido {
    Pedido::novo(metodo, url, cabecalhos.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(), vec![])
}

#[test]
fn so_esta_maquina_fala_com_o_mcp() {
    let e = estado(false);
    for cabecalhos in [
        vec![("Host", "evil.example.com")],
        vec![("Host", "127.0.0.1.evil.com:3000")],
        vec![("Host", "127.0.0.1:3000"), ("Origin", "http://evil.example.com")],
        vec![("Host", "127.0.0.1:3000"), ("Origin", "http://127.0.0.1:9999")],
        vec![],
    ] {
        let r = despachar(&e, &com_cabecalhos("GET", "/api/mcp/blueprints", &cabecalhos));
        assert_eq!(r.status, 403, "{cabecalhos:?}");
        assert!(texto_de(&r).contains("MCP_REDE=1"));
    }
    for host in ["127.0.0.1:3000", "localhost:8080", "[::1]:3000", "127.0.0.1"] {
        assert_eq!(despachar(&e, &com_cabecalhos("GET", "/api/mcp/blueprints", &[("Host", host)])).status, 200, "{host}");
    }
    let mesma_origem = com_cabecalhos("GET", "/api/mcp/blueprints", &[("Host", "127.0.0.1:3000"), ("Origin", "http://127.0.0.1:3000")]);
    assert_eq!(despachar(&e, &mesma_origem).status, 200);
}

#[test]
fn mcp_rede_libera_tudo() {
    let e = estado(true);
    let p = com_cabecalhos("GET", "/api/mcp/blueprints", &[("Host", "meu-servidor.lan:3000"), ("Origin", "http://outro")]);
    assert_eq!(despachar(&e, &p).status, 200);
}

#[test]
fn todo_post_exige_json_e_json_valido() {
    let e = estado(false);
    let sem_tipo = Pedido::novo("POST", "/api/mcp/evento", vec![("Host".into(), "127.0.0.1:3000".into())], b"{}".to_vec());
    let r = despachar(&e, &sem_tipo);
    assert_eq!((r.status, texto_de(&r)), (415, "envie application/json".to_string()));
    let torto = Pedido::novo("POST", "/api/mcp/evento", vec![("Host".into(), "127.0.0.1:3000".into()), ("Content-Type".into(), "application/json; charset=utf-8".into())], b"{torto".to_vec());
    let r = despachar(&e, &torto);
    assert_eq!(r.status, 400);
    assert!(json_de(&r)["erro"].is_string());
    assert_eq!(despachar(&e, &pedido("PUT", "/api/mcp/evento", &[], Some(json!({})))).status, 405);
    assert_eq!(despachar(&e, &pedido("GET", "/api/mcp/nada", &[], None)).status, 404);
    assert_eq!(post(&e, "nada", json!({})).status, 404);
}

#[test]
fn listar_e_ler_blueprints() {
    let e = estado(false);
    let lista = json_de(&despachar(&e, &pedido("GET", "/api/mcp/blueprints", &[], None)));
    assert_eq!(lista["workspaces"][0]["id"], "sm");
    assert_eq!(lista["workspaces"][0]["blueprints"][0], json!({"id": "p1", "nome": "Portal", "descricao": "d", "acoes": 1}));
    assert_eq!(lista["workspaces"][1]["id"], "w2");
    assert_eq!(lista["hoje"].as_str().unwrap().len(), 10);
    let estrutura = json_de(&despachar(&e, &pedido("GET", "/api/mcp/blueprint?workspace=sm&blueprint=p1", &[], None)));
    assert_eq!(estrutura["blueprint"]["acoes"][0]["planos"][1], json!({"id": "t2", "texto": "Já feita", "feito": true}));
    assert!(estrutura["blueprint"]["acoes"][0].get("observacao").is_none(), "a estrutura é enxuta");
    assert_eq!(estrutura["outros"], json!([]));
    let acao = json_de(&despachar(&e, &pedido("GET", "/api/mcp/blueprint?workspace=sm&blueprint=p1&acao=f1", &[], None)));
    assert_eq!(acao["acao"]["planos"][0]["descricao"], "x");
    assert_eq!(acao["blueprintId"], "p1");
    for (url, status) in [
        ("/api/mcp/blueprint?workspace=sm&blueprint=p1&acao=nao", 404),
        ("/api/mcp/blueprint?workspace=sm&blueprint=nao", 404),
        ("/api/mcp/blueprint?workspace=nao&blueprint=p1", 404),
        ("/api/mcp/blueprint?workspace=sm", 400),
    ] {
        assert_eq!(despachar(&e, &pedido("GET", url, &[], None)).status, status, "{url}");
    }
    let outro = json_de(&despachar(&e, &pedido("GET", "/api/mcp/blueprint?workspace=w2&blueprint=p9", &[], None)));
    assert_eq!(outro["blueprint"]["nome"], "Dele");
}

#[test]
fn proposta_invalida_volta_com_erros_e_nao_entra_na_fila() {
    let e = estado(false);
    let r = post(&e, "proposta", json!({"workspaceId": "sm", "blueprintId": "p1", "acoes": [{"nome": "x", "inicio": "2026-12-01", "fim": "2026-11-01", "planos": [], "checkpoints": []}]}));
    assert_eq!(r.status, 422);
    assert!(json_de(&r)["erros"][0].as_str().unwrap().contains("anterior"));
    assert_eq!(post(&e, "proposta", json!({"workspaceId": "nao", "blueprintId": "p1", "acoes": []})).status, 404);
    assert_eq!(post(&e, "proposta", json!({"workspaceId": "sm", "blueprintId": "", "acoes": proposta_valida()["acoes"]})).status, 422, "blueprint novo exige nome");
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", &[], None)))["itens"], json!([]));
}

#[test]
fn reenviar_a_proposta_substitui_a_pendente_e_o_ack_limpa() {
    let e = estado(false);
    assert_eq!(post(&e, "proposta", proposta_valida()).status, 200);
    assert_eq!(post(&e, "proposta", com(proposta_valida(), json!({"mensagem": "Versão 2"}))).status, 200);
    let fila = json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", &[], None)));
    assert_eq!(fila["itens"].as_array().unwrap().len(), 1);
    assert_eq!(fila["itens"][0]["proposta"]["mensagem"], "Versão 2");
    assert_eq!(fila["itens"][0]["tipo"], "proposta");
    let novo = post(&e, "proposta", json!({"workspaceId": "sm", "blueprintId": "", "blueprint": {"nome": "Novo", "descricao": "d"}, "acoes": proposta_valida()["acoes"]}));
    assert_eq!(novo.status, 200);
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", &[], None)))["itens"].as_array().unwrap().len(), 2);
    assert_eq!(json_de(&post(&e, "fila/ack", json!({"ids": [2, 3, 99]}))), json!({"ok": true}));
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/fila", &[], None)))["itens"].as_array().unwrap().len(), 0);
}

#[test]
fn eventos_de_andamento_e_suas_regras() {
    let e = estado(false);
    let base = |tipo: &str, texto: &str| json!({"tipo": tipo, "texto": texto, "workspaceId": "sm", "blueprintId": "p1"});
    assert_eq!(post(&e, "evento", json!({"tipo": "inventado", "texto": "x"})).status, 400);
    assert_eq!(post(&e, "evento", base("progresso", "   ")).status, 400);
    assert_eq!(post(&e, "evento", base("progresso", "lendo")).status, 200);
    assert_eq!(post(&e, "evento", base("bloqueio", "preciso de acesso")).status, 200);
    let ok = json_de(&post(&e, "evento", com(base("concluir", "cargo test passou"), json!({"acaoId": "f1", "planoId": "t1"}))));
    assert_eq!((ok["planoId"].as_str(), ok["plano"].as_str(), ok["acaoId"].as_str()), (Some("t1"), Some("Criar rota"), Some("f1")));
    assert_eq!(post(&e, "evento", com(base("concluir", "x"), json!({"acaoId": "f1", "planoId": "t2"}))).status, 409, "já está concluída");
    assert_eq!(post(&e, "evento", com(base("concluir", "x"), json!({"acaoId": "f1", "planoId": "nao"}))).status, 404);
    assert_eq!(post(&e, "evento", com(base("progresso", "x"), json!({"acaoId": "f1", "planoId": "t2"}))).status, 200, "progresso pode apontar um plano feito");
    let cp = json_de(&post(&e, "evento", com(base("checkpoint-feito", "demonstrado"), json!({"acaoId": "f1", "checkpointId": "m1"}))));
    assert_eq!((cp["checkpointId"].as_str(), cp["checkpoint"].as_str()), (Some("m1"), Some("Rota no ar")));
    assert_eq!(post(&e, "evento", com(base("checkpoint-feito", "x"), json!({"acaoId": "f1", "checkpointId": "m2"}))).status, 409);
    assert_eq!(post(&e, "evento", com(base("checkpoint-nao-feito", "x"), json!({"acaoId": "f1", "checkpointId": "nao"}))).status, 404);
    let cortado = json_de(&post(&e, "evento", base("progresso", &"a".repeat(5000))));
    assert_eq!(cortado["texto"].as_str().unwrap().chars().count(), 2000);
    assert!(cortado["texto"].as_str().unwrap().ends_with('…'));
}

#[test]
fn prototipos_anexar_substituir_e_remover() {
    let e = estado(false);
    let html = "<!doctype html><title>t</title><svg></svg>";
    let alvo = |extra: Value| com(json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f1", "titulo": "Tela", "html": html}), extra);
    assert_eq!(post(&e, "prototipo", alvo(json!({"html": "sem tag"}))).status, 400);
    assert_eq!(post(&e, "prototipo", alvo(json!({"titulo": ""}))).status, 400);
    assert_eq!(post(&e, "prototipo", alvo(json!({"acaoId": "nao"}))).status, 404);
    assert_eq!(post(&e, "prototipo", alvo(json!({"html": format!("<p>{}</p>", "x".repeat(2 * 1024 * 1024))}))).status, 413);
    assert_eq!(post(&e, "prototipo", alvo(json!({"prototipoId": "0123456789abcdef"}))).status, 404, "só substitui um que a ação já tem");
    let r = json_de(&post(&e, "prototipo", alvo(json!({}))));
    let id = r["prototipoId"].as_str().unwrap().to_string();
    assert!(id.len() == 16 && id.bytes().all(|c| c.is_ascii_hexdigit()));
    assert_eq!((r["atualizado"].as_bool(), r["titulo"].as_str()), (Some(false), Some("Tela")));
    assert_eq!(std::fs::read_to_string(e.mcp.prototipos.join(format!("{id}.html"))).unwrap(), html);
    // a página registra o protótipo na ação (é ela que guarda a referência); aí o mesmo id substitui
    let mut dados = super::fixture();
    dados["blueprints"][0]["acoes"][0]["prototipos"] = json!([{"id": id, "titulo": "Tela"}]);
    assert_eq!(despachar(&e, &pedido("PUT", "/api/dados", &[], Some(dados))).status, 200);
    let novo = json_de(&post(&e, "prototipo", alvo(json!({"prototipoId": id, "html": "<p>v2</p>"}))));
    assert_eq!((novo["prototipoId"].as_str(), novo["atualizado"].as_bool()), (Some(id.as_str()), Some(true)));
    assert_eq!(std::fs::read_to_string(e.mcp.prototipos.join(format!("{id}.html"))).unwrap(), "<p>v2</p>");
    assert_eq!(post(&e, "prototipo/remover", json!({"prototipoId": "../../server"})).status, 400);
    assert_eq!(post(&e, "prototipo/remover", json!({"prototipoId": id})).status, 200);
    assert!(!e.mcp.prototipos.join(format!("{id}.html")).exists());
    let _ = std::fs::remove_dir_all(&e.mcp.prototipos);
}

#[test]
fn o_prototipo_sai_isolado_em_sandbox() {
    let e = estado(false);
    std::fs::create_dir_all(&e.mcp.prototipos).unwrap();
    std::fs::write(e.mcp.prototipos.join("0123456789abcdef.html"), "<script>1</script>").unwrap();
    let r = despachar(&e, &pedido("GET", "/prototipos/0123456789abcdef.html", &[], None));
    assert_eq!(r.status, 200);
    let csp = r.cabecalho("content-security-policy").unwrap();
    assert!(csp.starts_with("sandbox allow-scripts;") && csp.contains("default-src 'none'") && !csp.contains("allow-same-origin"), "{csp}");
    assert_eq!(r.cabecalho("x-content-type-options"), Some("nosniff"));
    assert_eq!(r.cabecalho("cache-control"), Some("no-store"));
    for url in ["/prototipos/ffffffffffffffff.html", "/prototipos/curto.html", "/prototipos/0123456789abcdef.txt", "/prototipos/../server.html"] {
        assert_eq!(despachar(&e, &pedido("GET", url, &[], None)).status, 404, "{url}");
    }
    let _ = std::fs::remove_dir_all(&e.mcp.prototipos);
}

fn agentes(e: &Estado) -> Vec<Value> {
    json_de(&despachar(e, &pedido("GET", "/api/mcp/agentes", &[], None)))["agentes"].as_array().unwrap().clone()
}

#[test]
fn o_contrato_de_atividade_aceita_o_antigo_e_recusa_o_invalido() {
    let e = estado(false);
    let base = |extra: Value| com(json!({"tipo": "progresso", "texto": "lendo o código", "workspaceId": "sm", "blueprintId": "p1"}), extra);
    // chamada antiga, só com texto: vale, e o agente aparece com o nome padrão
    assert_eq!(post(&e, "evento", base(json!({}))).status, 200);
    assert_eq!(agentes(&e)[0]["agente"], "agente");
    // chamada nova, completa
    let ok = post(&e, "evento", base(json!({"agente": "  construtor-1 ", "papel": "construtor", "estado": "escrevendo", "onde": ["src/mcp/", "src/http/rotas.rs"], "percentual": 41.6, "acaoId": "f1", "planoId": "t1"})));
    assert_eq!(ok.status, 200);
    let item = json_de(&ok);
    assert_eq!((item["agente"].as_str(), item["percentual"].as_i64(), item["plano"].as_str()), (Some("construtor-1"), Some(42), Some("Criar rota")));
    // recusas, cada uma com o motivo
    for (extra, trecho) in [
        (json!({"estado": "dormindo"}), "estado inválido"),
        (json!({"papel": "chefe"}), "papel inválido"),
        (json!({"estado": "ocioso"}), "estado inválido"),
        (json!({"onde": "a".repeat(121)}), "onde inválido"),
        (json!({"onde": ["a", "b", "c", "d", "e", "f"]}), "onde inválido"),
        (json!({"onde": ["linha\nquebrada"]}), "onde inválido"),
        (json!({"onde": [42]}), "onde inválido"),
        (json!({"agente": "nome\u{7}com controle"}), "agente inválido"),
        (json!({"agente": "x".repeat(41)}), "agente inválido"),
        (json!({"agente": 7}), "agente inválido"),
        (json!({"percentual": 101}), "percentual inválido"),
        (json!({"percentual": -1}), "percentual inválido"),
        (json!({"percentual": "metade"}), "percentual inválido"),
    ] {
        let r = post(&e, "evento", base(extra.clone()));
        assert_eq!(r.status, 400, "{extra}");
        assert!(json_de(&r)["erro"].as_str().unwrap().contains(trecho), "{extra}: {}", texto_de(&r));
    }
    assert_eq!(agentes(&e).len(), 2, "o que foi recusado não entra no painel");
    // os campos só valem em progresso e bloqueio: nos outros tipos são ignorados, sem erro
    assert_eq!(post(&e, "evento", json!({"tipo": "concluir", "texto": "ok", "workspaceId": "sm", "blueprintId": "p1", "acaoId": "f1", "planoId": "t1", "estado": "dormindo"})).status, 200);
}

#[test]
fn tres_agentes_aparecem_na_lista_com_estado_onde_e_plano() {
    let e = estado(false);
    let avisa = |agente: &str, papel: &str, estado: &str, onde: &str| {
        let r = post(&e, "evento", json!({"tipo": "progresso", "texto": format!("{agente} trabalhando"), "workspaceId": "sm", "blueprintId": "p1", "agente": agente, "papel": papel, "estado": estado, "onde": [onde], "acaoId": "f1", "planoId": "t1"}));
        assert_eq!(r.status, 200);
    };
    avisa("explorador", "explorador", "lendo", "kit/");
    avisa("construtor", "construtor", "escrevendo", "src/mcp/acoes.rs");
    avisa("revisor", "revisor", "testando", "src/");
    let lista = agentes(&e);
    let nomes: Vec<&str> = lista.iter().map(|a| a["agente"].as_str().unwrap()).collect();
    assert_eq!(nomes, ["revisor", "construtor", "explorador"], "o mais recente primeiro");
    assert_eq!((lista[1]["papel"].as_str(), lista[1]["estado"].as_str(), lista[1]["onde"].clone(), lista[1]["plano"].as_str()), (Some("construtor"), Some("escrevendo"), json!(["src/mcp/acoes.rs"]), Some("Criar rota")));
    assert_eq!(lista[0]["mensagem"], "revisor trabalhando");
    assert_eq!(json_de(&despachar(&e, &pedido("GET", "/api/mcp/agentes", &[], None)))["vidaMs"], 600_000);
    // o bloqueio também identifica o agente e vira "bloqueado"
    assert_eq!(post(&e, "evento", json!({"tipo": "bloqueio", "texto": "preciso de acesso", "workspaceId": "sm", "blueprintId": "p1", "agente": "construtor"})).status, 200);
    let c = agentes(&e).into_iter().find(|a| a["agente"] == "construtor").unwrap();
    assert_eq!((c["estado"].as_str(), c["papel"].as_str(), c["tipo"].as_str()), (Some("bloqueado"), Some("construtor"), Some("bloqueio")));
}

#[test]
fn a_lista_de_agentes_segue_a_regra_de_seguranca_do_mcp() {
    let e = estado(false);
    let r = despachar(&e, &com_cabecalhos("GET", "/api/mcp/agentes", &[("Host", "evil.example.com")]));
    assert_eq!(r.status, 403);
}
