//! Testes do acesso: um servidor hospedado sem socket (o despacho de verdade, com pedidos montados como o proxy os entrega).
mod chaves;
mod convites;
mod hooks;
mod hospedado;
mod limite;
mod papeis;
mod segredo;
mod sincronia;

use super::*;
use crate::contexto::Papel;
use crate::dados::Dados;
use crate::http::config::Config;
use crate::http::estaticos::Assets;
use crate::http::pedido::Resposta;
use crate::mcp::Mcp;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, Ordering};

static CONTADOR: AtomicU32 = AtomicU32::new(0);
pub const URL: &str = "https://equipe.teste";

pub fn fixture() -> Value {
    json!({"empresa": "Raiz", "usuario": {"nome": "Local"}, "chats": [{"id": "x"}], "blueprints": [{"id": "p1", "nome": "Portal", "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "planejado", "prototipos": [{"id": "0123456789abcdef", "titulo": "t"}],
         "planos": [{"id": "t1", "texto": "Criar rota", "agente": {"estado": "ok", "comentarios": [{"texto": "segredo do agente"}]}}], "checkpoints": []}]}],
        "workspaces": [{"id": "w2", "nome": "Outro", "blueprints": [{"id": "p9", "nome": "Dele", "acoes": []}, {"id": "p10", "nome": "Público", "acoes": []}]}]})
}

pub fn estado() -> Estado {
    let mut dados = Dados::em_memoria().unwrap();
    dados.gravar_tudo(&Contexto::local(), &fixture()).unwrap();
    let prototipos = std::env::temp_dir().join(format!("blueprint-acesso-{}-{}", std::process::id(), CONTADOR.fetch_add(1, Ordering::SeqCst)));
    let mut config = Config::do_ambiente();
    (config.prototipos, config.assets_dir) = (prototipos.clone(), None);
    config.hospedagem = Some(Hospedagem::de(URL, "127.0.0.1", None, true).unwrap());
    Estado { ao_mostrar: Mutex::new(None), ao_sair: Mutex::new(None), atualizador: crate::atualizador::Atualizador::do_ambiente(&config.banco), assets: Assets::novo(None), mcp: Mcp::novo("127.0.0.1", false, "C:/app/blueprint.exe", "http://127.0.0.1:3000", prototipos), dados: Mutex::new(dados), dados_json: Mutex::new(None), versao_dados: std::sync::atomic::AtomicU64::new(1), sincronia: crate::sincronia::Sincronia::nova(1, true), terminais: crate::terminal::Terminais::novo(), acesso: Acesso::novo(), compartilhar: crate::compartilhar::Compartilhar::novo(), config }
}

/// Um pedido como o proxy o entrega: par em loopback, `X-Forwarded-Proto: https`, Host e Origin públicos.
pub fn pedido(metodo: &str, url: &str, cookie: Option<&str>, corpo: Option<Value>) -> Pedido {
    let mut cab: Vec<(String, String)> = vec![("Host".into(), "equipe.teste".into()), ("X-Forwarded-Proto".into(), "https".into()), ("X-Forwarded-For".into(), "203.0.113.7".into()), ("Origin".into(), URL.into())];
    if let Some(c) = cookie {
        cab.push(("Cookie".into(), format!("outro=1; bp_sessao={c}")));
    }
    if corpo.is_some() {
        cab.push(("Content-Type".into(), "application/json".into()));
    }
    let mut p = Pedido::novo(metodo, url, cab, corpo.map(|c| c.to_string().into_bytes()).unwrap_or_default());
    p.ip = "127.0.0.1".into();
    p
}

pub fn despachar(e: &Estado, p: &Pedido) -> Resposta {
    crate::http::rotas::despachar(e, p)
}

pub fn json_de(r: &Resposta) -> Value {
    serde_json::from_slice(r.bytes()).unwrap_or(Value::Null)
}

/// O valor do cookie `bp_sessao` de um `Set-Cookie`.
pub fn cookie_de(r: &Resposta) -> String {
    let v = r.cabecalho("set-cookie").expect("set-cookie");
    v.trim_start_matches("bp_sessao=").split(';').next().unwrap().to_string()
}

/// Cria um convite direto na camada de dados e devolve o segredo.
pub fn convite(e: &Estado, papel: Papel, ws: &str, bp: &str, usos: Option<i64>) -> String {
    let n = super::convites::NovoConvite { papel, workspace_id: ws, blueprint_id: bp, abre: "folha", validade_horas: 24, usos_max: usos };
    e.dados.lock().unwrap().transacao(|t| super::convites::criar(t, None, &n, crate::tempo::agora_ms())).unwrap().1
}

/// Entra pelo convite com o nome e devolve o cookie.
pub fn entrar(e: &Estado, segredo: &str, nome: &str) -> String {
    let r = despachar(e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": segredo, "nome": nome}))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    cookie_de(&r)
}

/// Uma pessoa com esse papel no escopo: o cookie da sessão dela.
pub fn pessoa(e: &Estado, papel: Papel, ws: &str, bp: &str, nome: &str) -> String {
    let s = convite(e, papel, ws, bp, Some(1));
    if papel == Papel::Visitante {
        let r = despachar(e, &pedido("POST", "/api/convite/entrar", None, Some(json!({"segredo": s}))));
        return cookie_de(&r);
    }
    entrar(e, &s, nome)
}

#[test]
fn sessao_ativa_ate_ser_encerrada() {
    let a = Acesso::novo();
    let ana = Contexto::membro("m1", "Ana", vec![]);
    assert!(a.sessao_ativa(&Contexto::local()));
    assert!(a.sessao_ativa(&ana));
    assert!(!a.sessao_ativa(&Contexto::anonimo()), "sem sessão não há o que manter aberto");
    a.encerrar("m1");
    assert!(!a.sessao_ativa(&ana));
    assert!(a.sessao_ativa(&Contexto::local()), "o local nunca é encerrado");
}
