//! Testes de mcp::planejar::figuras: guardar por conteúdo, medir e servir sem dar origem a quem abre.
use crate::http::pedido::Resposta;
use crate::http::rotas::despachar;
use crate::http::tests::{estado, json_de, pedido};
use crate::http::Estado;
use serde_json::{json, Value};

/// Um PNG de 3x2: só a assinatura e o IHDR, que é o que `medidas_imagem` lê.
const PNG_3X2: &str = "iVBORw0KGgoAAAANSUhEUgAAAAMAAAACCAIAAAA=";

fn janela() -> Estado {
    let e = estado(false);
    e.mcp.ligar_janela();
    e
}

fn post(e: &Estado, rota: &str, corpo: Value) -> Resposta {
    despachar(e, &pedido("POST", &format!("/api/mcp/plano/{rota}"), &[], Some(corpo)))
}

fn abrir(e: &Estado) -> String {
    let r = post(e, "abrir", json!({"workspaceId": "w2", "blueprintId": "p9", "origem": "escrita"}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)["escritaId"].as_str().unwrap().to_string()
}

fn enviar(e: &Estado, corpo: Value) -> String {
    let r = post(e, "figura", corpo);
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)["hash"].as_str().unwrap().to_string()
}

fn servir(e: &Estado, hash: &str) -> Resposta {
    despachar(e, &pedido("GET", &format!("/api/mcp/planejar/figura?hash={hash}"), &[], None))
}

#[test]
fn figura_servida_com_csp() {
    let e = janela();
    let escrita = abrir(&e);
    // `var()` resolvido: o que fica em disco e é servido já tem a cor pronta
    let hash = enviar(&e, json!({"escritaId": escrita, "caminho": "figuras/a.svg", "tipo": "svg", "svg": "<svg viewBox=\"0 0 9 9\"><rect fill=\"var(--verde)\"/></svg>"}));
    let r = servir(&e, &hash);
    assert_eq!(r.status, 200);
    assert_eq!(r.cabecalho("content-type"), Some("image/svg+xml"), "sem charset");
    assert_eq!(r.cabecalho("x-content-type-options"), Some("nosniff"));
    assert_eq!(r.cabecalho("content-security-policy"), Some("sandbox; default-src 'none'; style-src 'unsafe-inline'"));
    let texto = String::from_utf8_lossy(r.bytes()).into_owned();
    assert!(texto.contains("#8be3b5") && !texto.contains("var("), "{texto}");
    assert_eq!(crate::plano::ids::sha256_hex(r.bytes()), hash, "o hash é do resultado");

    let png = enviar(&e, json!({"escritaId": escrita, "caminho": "b.png", "tipo": "png", "base64": PNG_3X2}));
    assert_eq!(servir(&e, &png).cabecalho("content-type"), Some("image/png"));

    assert_eq!(servir(&e, &"0".repeat(64)).status, 404, "hash que não existe");
    for ruim in ["abc", "../../etc/passwd", &"Z".repeat(64), &"A".repeat(64)] {
        assert_eq!(servir(&e, ruim).status, 400, "{ruim}");
    }
    assert_eq!(despachar(&e, &pedido("POST", "/api/mcp/planejar/figura", &[], Some(json!({})))).status, 404, "só GET");
}

#[test]
fn medidas_gravadas() {
    let e = janela();
    let escrita = abrir(&e);
    let svg = enviar(&e, json!({"escritaId": escrita, "caminho": "a.svg", "tipo": "svg", "svg": "<svg viewBox=\"0 0 900 460\"></svg>"}));
    let png = enviar(&e, json!({"escritaId": escrita, "caminho": "b.png", "tipo": "png", "base64": PNG_3X2}));
    let d = e.dados.lock().unwrap();
    let numero = d.plano_escrita(&escrita).unwrap().unwrap().numero;
    let a = d.figura_de("p9", numero, "a.svg").unwrap().unwrap();
    assert_eq!((a.hash.as_str(), a.tipo.as_str(), a.largura, a.altura), (svg.as_str(), "svg", Some(900), Some(460)));
    let b = d.figura_de("p9", numero, "b.png").unwrap().unwrap();
    assert_eq!((b.hash.as_str(), b.tipo.as_str(), b.largura, b.altura, b.tamanho), (png.as_str(), "png", Some(3), Some(2), 29));
    drop(d);

    // reenvio no mesmo caminho substitui, e a medida vai para o detalhe ao fechar
    let novo = enviar(&e, json!({"escritaId": escrita, "caminho": "a.svg", "tipo": "svg", "svg": "<svg viewBox=\"0 0 10 20\"></svg>"}));
    assert_ne!(novo, svg);
    let r = post(&e, "escrever", json!({"escritaId": escrita, "acao": {"id": "1", "nome": "N", "detalhes": [{"id": "d", "figura": "a.svg", "figura_info": {"hash": "forjado", "tipo": "svg", "largura": 1, "altura": 1}}]}}));
    assert_eq!(r.status, 200);
    assert_eq!(post(&e, "fechar", json!({"escritaId": escrita})).status, 200);
    let mut d = e.dados.lock().unwrap();
    let tudo = d.ler_tudo(&crate::contexto::Contexto::local()).unwrap();
    let info = &tudo["workspaces"][0]["blueprints"][0]["acoes"][0]["detalhes"][0]["figura_info"];
    assert_eq!(info, &json!({"hash": novo, "tipo": "svg", "largura": 10, "altura": 20}), "nunca o que o agente escreveu");
}

#[test]
fn limite_de_figuras_na_versao() {
    let e = janela();
    let escrita = abrir(&e);
    {
        let mut d = e.dados.lock().unwrap();
        let v = d.plano_escrita(&escrita).unwrap().unwrap();
        for i in 0..crate::plano::svg::MAX_FIGURAS {
            d.figura_registrar(&crate::dados::planejar::FiguraLinha { blueprint_id: v.blueprint_id.clone(), numero: v.numero, caminho: format!("figuras/{i}.png"), hash: "0".repeat(64), tipo: "png".into(), tamanho: 1, largura: None, altura: None }).unwrap();
        }
    }
    let r = post(&e, "figura", json!({"escritaId": escrita, "caminho": "figuras/nova.png", "tipo": "png", "base64": PNG_3X2}));
    assert_eq!(r.status, 422, "acima do limite não grava: {}", String::from_utf8_lossy(r.bytes()));
    enviar(&e, json!({"escritaId": escrita, "caminho": "figuras/0.png", "tipo": "png", "base64": PNG_3X2})); // reenvio do mesmo caminho substitui
}
