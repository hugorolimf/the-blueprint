//! Testes do comentário do plano e da conversa com o plano (nenhum Claude de verdade: o evento é lido do SSE em memória e a resposta vem da rota).
use super::conversa::{enviar, ferramenta, responder};
use super::hooks::{sessao, travado};
use super::hooks_tests::{dados, mcp};
use super::{Evento, Mcp};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::Pedido;
use serde_json::{json, Map, Value};
use std::cell::RefCell;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

/// Pasta ligada ao blueprint p1, uma sessão aberta nela e o canal dela conectado (a guarda precisa viver durante o teste).
fn com_canal() -> (Mcp, Mutex<Dados>, super::sessoes::Guarda) {
    let (m, d) = (mcp(), dados());
    let pasta = std::env::temp_dir().join(format!("blueprint-conversa-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let cwd = pasta.to_str().unwrap().to_string();
    travado(&d).vincular_pasta(&Contexto::local(), &cwd, "sm", "p1").unwrap();
    sessao(&m, &d, &json!({"session_id": "s1", "cwd": cwd})).unwrap();
    let guarda = m.abrir_canal("canal1", &cwd).unwrap();
    (m, d, guarda)
}

fn corpo(extra: Value) -> Value {
    let mut c = json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "sessaoId": "canal1", "tipo": "conversa", "texto": "Divida o plano em dois"});
    c.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    c
}

fn eventos(rx: &Receiver<Evento>) -> Vec<Evento> {
    rx.try_iter().collect()
}

#[test]
fn o_comentario_vai_so_ao_canal_escolhido_com_o_plano_e_nao_entra_na_conversa() {
    let (m, d, _g) = com_canal();
    let rx = m.assinar();
    let (status, r) = enviar(&m, &d, &corpo(json!({"tipo": "comentario", "planoId": "t2", "texto": "Use a cor\ndo tema"}))).unwrap();
    assert_eq!((status, r["entregue"].clone()), (200, json!(true)), "{r}");
    let e = eventos(&rx);
    assert_eq!(e.len(), 1, "só o evento do canal; nada de conversa");
    assert_eq!(e[0].nome, "comentario-plano");
    let dados = e[0].dados.as_ref().unwrap();
    assert_eq!(dados["canalId"], "canal1");
    let texto = dados["texto"].as_str().unwrap();
    assert!(texto.starts_with("Comentário no plano «Criar tela» (plano_id t2) da ação 1.2 «Tela» do blueprint «Portal» (workspace_id sm, blueprint_id p1, acao_id f2): "), "{texto}");
    assert!(texto.ends_with("Use a cor do tema"), "a quebra de linha vira espaço: {texto}");
    assert_eq!(m.conversa("sm", "p1", "f2")["mensagens"], json!([]));
}

#[test]
fn a_conversa_entrega_guarda_a_mensagem_e_avisa_a_pagina() {
    let (m, d, _g) = com_canal();
    let rx = m.assinar();
    assert_eq!(enviar(&m, &d, &corpo(json!({}))).unwrap().0, 200);
    let nomes: Vec<&str> = eventos(&rx).iter().map(|e| e.nome).collect();
    assert_eq!(nomes, ["conversa-usuario", "conversa"]);
    let msgs = m.conversa("sm", "p1", "f2");
    assert_eq!((msgs["mensagens"][0]["de"].clone(), msgs["mensagens"][0]["texto"].clone()), (json!("usuario"), json!("Divida o plano em dois")));
    // sem canal: nada vai ao canal, a página copia o prompt e a mensagem fica na conversa
    let (status, r) = enviar(&m, &d, &corpo(json!({"semCanal": true, "sessaoId": "", "texto": "Outra"}))).unwrap();
    assert_eq!((status, r["entregue"].clone()), (200, json!(false)));
    assert!(r["prompt"].as_str().unwrap().starts_with("Ação 1.2 «Tela» do blueprint «Portal»"), "{r}");
    assert_eq!(eventos(&rx).iter().map(|e| e.nome).collect::<Vec<_>>(), ["conversa"]);
    assert_eq!(m.conversa("sm", "p1", "f2")["mensagens"].as_array().unwrap().len(), 2);
}

#[test]
fn recusas_de_quem_envia() {
    let (m, d, guarda) = com_canal();
    let status = |c: Value| match enviar(&m, &d, &c) { Ok((s, _)) => s, Err(e) => e.status };
    assert_eq!(status(corpo(json!({"tipo": "comentario"}))), 400, "comentário sem plano");
    assert_eq!(status(corpo(json!({"tipo": "comentario", "planoId": "t1"}))), 404, "plano de outra ação");
    assert_eq!(status(corpo(json!({"acaoId": "nao-existe"}))), 404);
    assert_eq!(status(corpo(json!({"tipo": "ordem"}))), 400);
    assert_eq!(status(corpo(json!({"texto": " \n "}))), 400);
    assert_eq!(status(corpo(json!({"texto": "x".repeat(801)}))), 413);
    assert_eq!(status(corpo(json!({"texto": "€".repeat(800)}))), 413, "cabe em 800 caracteres, mas não na linha do canal");
    assert_eq!(status(corpo(json!({"sessaoId": ""}))), 400);
    drop(guarda);
    assert_eq!(status(corpo(json!({}))), 410, "a sessão caiu");
    assert_eq!(m.conversa("sm", "p1", "f2")["mensagens"], json!([]), "nada recusado entra na conversa");
}

#[test]
fn a_resposta_da_ia_volta_a_conversa_e_so_cita_proposta_que_esta_na_fila() {
    let (m, d) = (mcp(), dados());
    let rx = m.assinar();
    let r = |c: Value| match responder(&m, &d, &c) { Ok((s, _)) => s, Err(e) => e.status };
    let base = json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "texto": "Proponho dividir."});
    assert_eq!(r(base.clone()), 200);
    assert_eq!(eventos(&rx).iter().map(|e| e.nome).collect::<Vec<_>>(), ["conversa"]);
    let mut com_proposta = base.clone();
    com_proposta["proposta"] = json!(1);
    assert_eq!(r(com_proposta.clone()), 409, "a fila está vazia");
    let item: Map<String, Value> = json!({"tipo": "proposta", "workspaceId": "sm", "blueprintId": "p1"}).as_object().unwrap().clone();
    m.empurra_proposta(item, json!({}), None);
    assert_eq!(r(com_proposta), 200);
    let msgs = m.conversa("sm", "p1", "f2")["mensagens"].clone();
    assert_eq!((msgs[1]["de"].clone(), msgs[1]["proposta"].clone()), (json!("agente"), json!(1)));
    let mut ruim = base.clone();
    ruim["texto"] = json!("x".repeat(4001));
    assert_eq!(r(ruim), 400);
    ruim = base.clone();
    ruim["acaoId"] = json!("nao-existe");
    assert_eq!(r(ruim), 404);
    // só as últimas 40 mensagens ficam
    for _ in 0..45 {
        r(base.clone());
    }
    assert_eq!(m.conversa("sm", "p1", "f2")["mensagens"].as_array().unwrap().len(), 40);
}

#[test]
fn a_ferramenta_monta_o_corpo_e_exige_texto() {
    let enviado = RefCell::new(None);
    let servidor = |rota: &str, metodo: &str, corpo: Option<&Value>| -> Result<Value, String> {
        *enviado.borrow_mut() = Some((rota.to_string(), metodo.to_string(), corpo.cloned()));
        Ok(json!({"ok": true}))
    };
    assert!(ferramenta(&json!({"workspace_id": "sm"}), &servidor).is_err());
    let ok = ferramenta(&json!({"workspace_id": "sm", "blueprint_id": "p1", "acao_id": "f2", "texto": " Feito ", "proposta": 3}), &servidor).unwrap();
    assert!(ok.contains("propor_blueprint"));
    let (rota, metodo, corpo) = enviado.into_inner().unwrap();
    assert_eq!((rota.as_str(), metodo.as_str()), ("conversa/resposta", "POST"));
    assert_eq!(corpo.unwrap(), json!({"workspaceId": "sm", "blueprintId": "p1", "acaoId": "f2", "texto": "Feito", "proposta": 3}));
}

#[test]
fn as_instrucoes_ensinam_o_prefixo_da_conversa_e_a_ferramenta_de_resposta() {
    let r = super::stdio::tratar(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})).unwrap();
    let instrucoes = r["result"]["instructions"].as_str().unwrap();
    assert!(instrucoes.contains("«The Blueprint · conversa do usuário»") && instrucoes.contains("responder_conversa") && instrucoes.contains("Send to agent"), "{instrucoes}");
}

fn pedido(metodo: &str, caminho: &str, host: &str, origem: Option<&str>) -> Pedido {
    let mut h = vec![("host".to_string(), host.to_string()), ("content-type".to_string(), "application/json".to_string())];
    if let Some(o) = origem {
        h.push(("origin".into(), o.into()));
    }
    Pedido::novo(metodo, caminho, h, corpo(json!({})).to_string().into_bytes())
}

#[test]
fn as_rotas_da_conversa_so_respondem_em_loopback_e_o_mcp_rede_nao_libera() {
    let (m, d, _g) = com_canal();
    let r = |p: &Pedido| super::rotas::rota(&m, &d, p).status;
    assert_eq!(r(&pedido("POST", "/api/mcp/conversa", "evil.example", None)), 403);
    assert_eq!(r(&pedido("POST", "/api/mcp/conversa", "127.0.0.1:3000", Some("http://evil.example"))), 403);
    assert_eq!(r(&pedido("POST", "/api/mcp/conversa", "127.0.0.1:3000", Some("http://127.0.0.1:3000"))), 200);
    assert_eq!(r(&pedido("GET", "/api/mcp/conversa?workspace=sm&blueprint=p1&acao=f2", "127.0.0.1:3000", None)), 200);
    let rede = Mcp::novo("0.0.0.0", true, "blueprint.exe", "http://0.0.0.0:3000", std::env::temp_dir());
    for (metodo, rota) in [("POST", "/api/mcp/conversa"), ("POST", "/api/mcp/conversa/resposta"), ("GET", "/api/mcp/conversa")] {
        assert_eq!(super::rotas::rota(&rede, &d, &pedido(metodo, rota, "192.168.0.5:3000", None)).status, 403, "{rota}");
    }
}
