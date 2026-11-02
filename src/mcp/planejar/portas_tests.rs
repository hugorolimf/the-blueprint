//! Quem alcança o Planejar: convidado nunca, e com a janela aberta as escritas da árvore e os pedidos que abrem processo só com o token do gesto (D7, D14).
use crate::contexto::{Acesso, Contexto, Papel};
use crate::http::pedido::{Pedido, Resposta};
use crate::http::rotas::{despachar, despachar_convidado};
use crate::http::tests::{estado, fixture, json_de, pedido};
use crate::http::Estado;
use crate::mcp::rotas::rota_com;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

const URL: &str = "https://teste-falso-tunel.trycloudflare.com";
const HOST: &str = "teste-falso-tunel.trycloudflare.com";
const ROTAS: [(&str, &str); 4] = [("POST", "/api/mcp/plano/abrir"), ("GET", "/api/mcp/planejar/pulso?workspace=w2&blueprint=p9"), ("POST", "/api/mcp/gesto/marcar"), ("POST", "/api/mcp/execucao/sinal")];

fn codigo(r: &Resposta) -> String {
    serde_json::from_slice::<Value>(r.bytes()).ok().and_then(|v| v["codigo"].as_str().map(String::from)).unwrap_or_default()
}

fn com_token(metodo: &str, url: &str, token: &str, corpo: Option<Value>) -> Pedido {
    pedido(metodo, url, &[("X-Blueprint-Gesto", token)], corpo)
}

fn do_tunel(metodo: &str, url: &str, cookie: Option<&str>, corpo: Option<Value>) -> Pedido {
    let mut cab: Vec<(String, String)> = vec![("Host".into(), HOST.into()), ("X-Forwarded-Proto".into(), "https".into()), ("X-Forwarded-For".into(), "203.0.113.9".into()), ("Origin".into(), URL.into())];
    if let Some(c) = cookie {
        cab.push(("Cookie".into(), format!("bp_sessao={c}")));
    }
    if corpo.is_some() {
        cab.push(("Content-Type".into(), "application/json".into()));
    }
    let mut p = Pedido::novo(metodo, url, cab, corpo.map(|c| c.to_string().into_bytes()).unwrap_or_default());
    p.ip = "127.0.0.1".into();
    p
}

/// O dono local convida e a pessoa entra pela porta dos convidados: devolve o cookie dela.
fn convidado_dono(e: &Estado) -> String {
    let r = despachar(e, &pedido("POST", "/api/convites", &[], Some(json!({"workspaceId": "w2", "papel": "dono"}))));
    assert_eq!(r.status, 201, "{}", String::from_utf8_lossy(r.bytes()));
    let link = json_de(&r)["link"].as_str().unwrap().to_string();
    let segredo = link.split_once('#').unwrap().1;
    let r = despachar_convidado(e, &do_tunel("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo, "nome": "Ana"}))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    r.cabecalho("set-cookie").unwrap().trim_start_matches("bp_sessao=").split(';').next().unwrap().to_string()
}

#[test]
fn convidado_nao_alcanca_planejar() {
    let e = estado(false);
    e.compartilhar.ligado_para_teste(URL, "w2");
    let token = e.mcp.ligar_janela();
    let cookie = convidado_dono(&e);
    for (metodo, url) in ROTAS {
        let corpo = (metodo != "GET").then(|| json!({"workspaceId": "w2", "blueprintId": "p9"}));
        for c in [None, Some(cookie.as_str())] {
            let mut p = do_tunel(metodo, url, c, corpo.clone());
            p.cabecalhos.push(("X-Blueprint-Gesto".into(), token.clone())); // nem com o token
            let r = despachar_convidado(&e, &p);
            assert_eq!((r.status, codigo(&r).as_str()), (404, "so-local"), "{metodo} {url}");
        }
        // o caminho de `POST /mcp`, que chega a `rota_com` sem passar por `so_local`: membro autenticado, dono do workspace
        let membro = Contexto::membro("m1", "Ana", vec![Acesso::novo("w2", "*", Papel::Dono)]);
        let p = com_token(metodo, url, &token, corpo);
        let r = rota_com(&e.mcp, &e.dados, &p, &membro);
        assert_eq!((r.status, codigo(&r).as_str()), (404, "so-local"), "membro em {metodo} {url}");
    }
}

#[test]
fn put_com_janela_403() {
    let e = estado(false);
    let token = e.mcp.ligar_janela();
    let r = despachar(&e, &pedido("PUT", "/api/dados", &[], Some(fixture())));
    assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"));
    let r = despachar(&e, &com_token("PUT", "/api/dados", "errado", Some(fixture())));
    assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"), "token errado");
    let mudancas = json!({"base": 0, "cliente": "c1", "ops": []});
    let r = despachar(&e, &pedido("POST", "/api/mudancas", &[], Some(mudancas.clone())));
    assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"));
    assert_eq!(despachar(&e, &pedido("GET", "/api/dados", &[], None)).status, 200, "ler continua sem token");
    let r = despachar(&e, &com_token("PUT", "/api/dados", &token, Some(fixture())));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_ne!(codigo(&despachar(&e, &com_token("POST", "/api/mudancas", &token, Some(mudancas)))), "gesto-negado");
}

#[test]
fn put_sem_janela_passa() {
    let e = estado(false);
    let r = despachar(&e, &pedido("PUT", "/api/dados", &[], Some(fixture())));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let r = despachar(&e, &pedido("POST", "/api/mudancas", &[], Some(json!({"base": 0, "cliente": "c1", "ops": []}))));
    assert_ne!(codigo(&r), "gesto-negado");
}

#[test]
fn iniciar_com_janela_403() {
    let e = estado(false);
    // sem janela: segue para a validação de sempre (corpo vazio: nenhum terminal abre)
    assert_eq!(codigo(&despachar(&e, &pedido("POST", "/api/mcp/iniciar", &[], Some(json!({}))))), "faltam-ids");
    let token = e.mcp.ligar_janela();
    for url in ["/api/mcp/iniciar", "/api/mcp/apresentar", "/api/mcp/conversa"] {
        let r = despachar(&e, &pedido("POST", url, &[], Some(json!({}))));
        assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"), "{url}");
    }
    // a resposta vem da ferramenta do agente, que não tem token: com a janela aberta ela continua chegando
    assert_eq!(codigo(&despachar(&e, &pedido("POST", "/api/mcp/conversa/resposta", &[], Some(json!({}))))), "faltam-ids");
    for url in ["/api/mcp/iniciar", "/api/mcp/apresentar", "/api/mcp/conversa"] {
        assert_eq!(codigo(&despachar(&e, &com_token("POST", url, &token, Some(json!({}))))), "faltam-ids", "{url} com o token");
    }
    assert_eq!(despachar(&e, &pedido("GET", "/api/mcp/conversa?workspace=w2&blueprint=p9&acao=a", &[], None)).status, 200, "ler não pede token");
}

#[test]
fn versao_espelho_acompanha() {
    let e = estado(false);
    let antes = e.versao_dados.load(Ordering::SeqCst);
    assert_eq!(despachar(&e, &pedido("PUT", "/api/dados", &[], Some(fixture()))).status, 200);
    let depois = e.versao_dados.load(Ordering::SeqCst);
    assert!(depois > antes, "o PUT sobe a versão");
    assert_ne!(e.mcp.versao_espelho.load(Ordering::SeqCst), depois, "só `rota_estado` copia a versão");
    despachar(&e, &pedido("GET", "/api/mcp/planejar/pulso?workspace=w2&blueprint=p9", &[], None));
    assert_eq!(e.mcp.versao_espelho.load(Ordering::SeqCst), depois, "atualizado antes de atender");
}

#[test]
fn plano_sem_janela_409() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/mcp/plano/abrir", &[], Some(json!({"workspaceId": "w2"}))));
    assert_eq!((r.status, codigo(&r).as_str()), (409, "sem-janela"));
    assert!(json_de(&r)["erro"].as_str().unwrap().contains("propor_blueprint"));
    e.mcp.ligar_janela();
    assert_ne!(codigo(&despachar(&e, &pedido("POST", "/api/mcp/plano/abrir", &[], Some(json!({"workspaceId": "w2"}))))), "sem-janela");
}

#[test]
fn gesto_exige_token_e_loopback() {
    let e = estado(false);
    let r = despachar(&e, &pedido("POST", "/api/mcp/gesto/marcar", &[], Some(json!({}))));
    assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"), "sem janela, nenhum token vale");
    let token = e.mcp.ligar_janela();
    assert_eq!(token.len(), 32);
    assert!(token.bytes().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(e.mcp.ligar_janela(), token, "o mesmo durante a abertura");
    assert_eq!(codigo(&despachar(&e, &pedido("POST", "/api/mcp/gesto/marcar", &[], Some(json!({}))))), "gesto-negado");
    let fora = pedido("POST", "/api/mcp/gesto/marcar", &[("X-Blueprint-Gesto", &token), ("Origin", "http://exemplo.invalid")], Some(json!({})));
    let r = despachar(&e, &fora);
    assert_eq!((r.status, codigo(&r).as_str()), (403, "gesto-negado"), "origem de fora, mesmo com o token");
    assert_ne!(codigo(&despachar(&e, &com_token("POST", "/api/mcp/gesto/marcar", &token, Some(json!({}))))), "gesto-negado");
    let r = despachar(&e, &com_token("POST", "/api/mcp/gesto/nao-existe", &token, Some(json!({}))));
    assert_eq!(r.status, 404);
}

#[test]
fn rotas_do_planejar_so_nesta_maquina_mesmo_com_mcp_rede() {
    let e = estado(true); // MCP_REDE=1 libera o resto do MCP
    e.mcp.ligar_janela();
    let de_fora = |url: &str| {
        let mut p = pedido("GET", url, &[], None);
        p.cabecalhos.retain(|(k, _)| k != "Host");
        p.cabecalhos.push(("Host".into(), "192.0.2.10:3000".into()));
        p
    };
    assert_eq!(despachar(&e, &de_fora("/api/mcp/blueprints")).status, 200);
    assert_eq!(despachar(&e, &de_fora("/api/mcp/planejar/config")).status, 403);
    assert_eq!(despachar(&e, &de_fora("/api/mcp/plano/ler")).status, 403);
}

#[test]
fn limites_de_corpo_do_planejar() {
    use crate::http::rotas::limite_corpo;
    assert_eq!(limite_corpo("POST", "/api/mcp/plano/figura"), 6 << 20);
    assert_eq!(limite_corpo("POST", "/api/mcp/plano/abrir"), 2 << 20);
    assert_eq!(limite_corpo("POST", "/api/mcp/plano/escrever"), 2 << 20);
}

#[test]
fn pasta_das_figuras_fica_ao_lado_dos_prototipos() {
    let e = estado(false);
    assert_eq!(e.mcp.pasta_figuras(), e.mcp.prototipos.join("figuras"), "pasta de teste: dentro dela");
    let m = crate::mcp::Mcp::novo("127.0.0.1", false, "b", "http://127.0.0.1:3000", std::path::PathBuf::from("/dados/prototipos"));
    assert_eq!(m.pasta_figuras(), std::path::PathBuf::from("/dados/figuras"));
}
