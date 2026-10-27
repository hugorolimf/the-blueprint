//! Testes do servidor HTTP: as rotas sem socket (funções de Pedido para Resposta) e um servidor de verdade numa porta livre.
mod api_dados;
mod apresentacao_rotas;
mod atualizacao;
mod equipes;
mod exemplo;
mod idioma;
mod mcp_rotas;
mod servidor;
mod sincronia_rotas;
mod estaticos_rotas;
mod terminal_rotas;

use super::estaticos::Assets;
use super::pedido::{Pedido, Resposta};
use super::{Config, Estado};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::mcp::Mcp;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

static CONTADOR: AtomicU32 = AtomicU32::new(0);

pub fn fixture() -> Value {
    json!({"blueprints": [{"id": "p1", "nome": "Portal", "descricao": "d", "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "planejado",
         "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "x"}, {"id": "t2", "texto": "Já feita", "descricao": "y", "feito": true}],
         "checkpoints": [{"id": "m1", "nome": "Rota no ar", "data": "2026-11-20"}, {"id": "m2", "nome": "Já feito", "data": "2026-11-25", "feito": true}]}]}],
        "workspaces": [{"id": "w2", "nome": "Outro", "blueprints": [{"id": "p9", "nome": "Dele", "acoes": []}]}]})
}

pub fn estado(rede: bool) -> Estado {
    let mut dados = Dados::em_memoria().unwrap();
    dados.gravar_tudo(&Contexto::local(), &fixture()).unwrap();
    let prototipos = std::env::temp_dir().join(format!("blueprint-http-{}-{}", std::process::id(), CONTADOR.fetch_add(1, Ordering::SeqCst)));
    let mut config = Config::do_ambiente();
    (config.rede_mcp, config.prototipos, config.assets_dir) = (rede, prototipos.clone(), None);
    let (versao_dados, sincronia) = crate::http::estado::versao_e_sincronia(false);
    Estado { ao_mostrar: Mutex::new(None), ao_sair: Mutex::new(None), atualizador: crate::atualizador::Atualizador::do_ambiente(&config.banco), assets: Assets::novo(None), mcp: Mcp::novo("127.0.0.1", rede, "C:/app/blueprint.exe", "http://127.0.0.1:3000", prototipos), dados: Mutex::new(dados), dados_json: Mutex::new(None), versao_dados, sincronia, terminais: crate::terminal::Terminais::novo(), acesso: crate::acesso::Acesso::novo(), compartilhar: crate::compartilhar::Compartilhar::novo(), config }
}

/// Um pedido vindo do próprio computador (Host em loopback), com os cabeçalhos extras que o teste quiser.
pub fn pedido(metodo: &str, url: &str, extra: &[(&str, &str)], corpo: Option<Value>) -> Pedido {
    let mut cabecalhos: Vec<(String, String)> = vec![("Host".into(), "127.0.0.1:3000".into())];
    if corpo.is_some() {
        cabecalhos.push(("Content-Type".into(), "application/json".into()));
    }
    cabecalhos.extend(extra.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    Pedido::novo(metodo, url, cabecalhos, corpo.map(|c| c.to_string().into_bytes()).unwrap_or_default())
}

pub fn json_de(r: &Resposta) -> Value {
    serde_json::from_slice(r.bytes()).unwrap_or_else(|e| panic!("corpo não é JSON ({e}): {}", String::from_utf8_lossy(r.bytes())))
}

pub fn texto_de(r: &Resposta) -> String {
    String::from_utf8_lossy(r.bytes()).into_owned()
}
