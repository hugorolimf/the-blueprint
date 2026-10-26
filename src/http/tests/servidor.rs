//! O servidor de verdade, numa porta livre, falando por socket: o que as rotas puras não mostram (cabeçalhos reais, corpo, SSE).
use crate::http::{iniciar, Config};
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::time::Duration;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-servidor-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn config(dir: &std::path::Path, importar: Option<PathBuf>) -> Config {
    let mut c = Config::do_ambiente();
    (c.host, c.porta, c.rede_mcp, c.assets_dir) = ("127.0.0.1".into(), 0, false, None);
    (c.banco, c.importar_de, c.prototipos) = (dir.join("blueprint.db"), importar, dir.join("prototipos"));
    c
}

/// Manda um pedido HTTP/1.1 e devolve (status, cabeçalhos, corpo).
fn pedir(porta: u16, metodo: &str, caminho: &str, corpo: Option<&str>) -> (u16, String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", porta)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    let extra = corpo.map(|c| format!("Content-Type: application/json\r\nContent-Length: {}\r\n", c.len())).unwrap_or_default();
    write!(s, "{metodo} {caminho} HTTP/1.1\r\nHost: 127.0.0.1:{porta}\r\n{extra}Connection: close\r\n\r\n{}", corpo.unwrap_or("")).unwrap();
    let mut bruto = String::new();
    s.read_to_string(&mut bruto).unwrap();
    let (cabecalho, corpo) = bruto.split_once("\r\n\r\n").unwrap_or((&bruto, ""));
    let status = cabecalho.split_whitespace().nth(1).unwrap().parse().unwrap();
    (status, cabecalho.to_string(), corpo.to_string())
}

#[test]
fn importa_o_data_json_so_na_primeira_vez_e_serve_tudo() {
    let dir = pasta("geral");
    let original = dir.join("data.json");
    let texto = r#"{"empresa":"Teste","blueprints":[{"id":"p","nome":"Importado","acoes":[]}]}"#;
    std::fs::write(&original, texto).unwrap();
    let s = iniciar(config(&dir, Some(original.clone()))).unwrap();
    assert!(s.url.starts_with("http://127.0.0.1:") && s.porta != 0);
    let (status, cab, corpo) = pedir(s.porta, "GET", "/api/dados", None);
    assert_eq!(status, 200);
    assert!(cab.to_lowercase().contains("application/json"));
    assert_eq!(serde_json::from_str::<Value>(&corpo).unwrap(), serde_json::from_str::<Value>(texto).unwrap());
    assert_eq!(std::fs::read_to_string(&original).unwrap(), texto, "o data.json original não é alterado");
    let (status, cab, corpo) = pedir(s.porta, "GET", "/", None);
    assert_eq!(status, 200);
    assert!(cab.to_lowercase().contains("text/html") && corpo.contains("id=\"login-form\""));
    assert_eq!(pedir(s.porta, "GET", "/server.mjs", None).0, 404);
    assert_eq!(pedir(s.porta, "GET", "/api/mcp/info", None).0, 200);
    let (_, _, info) = pedir(s.porta, "GET", "/api/mcp/info", None);
    assert_eq!(serde_json::from_str::<Value>(&info).unwrap()["exe"], true);
    let (status, _, corpo) = pedir(s.porta, "PUT", "/api/dados", Some(r#"{"blueprints":[{"id":"q","nome":"Gravado","acoes":[]}]}"#));
    assert_eq!((status, corpo.as_str()), (200, r#"{"ok":true}"#));
    s.parar();
    drop(s);
    // abrir de novo com o banco já preenchido não importa o arquivo outra vez
    let s2 = iniciar(config(&dir, Some(original))).unwrap();
    let (_, _, corpo) = pedir(s2.porta, "GET", "/api/dados", None);
    assert_eq!(serde_json::from_str::<Value>(&corpo).unwrap()["blueprints"][0]["id"], "q");
    s2.parar();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_primeira_execucao_sem_data_json_comeca_vazia_e_responde() {
    let dir = pasta("sem-arquivo");
    let s = iniciar(config(&dir, None)).unwrap();
    let (status, _, corpo) = pedir(s.porta, "GET", "/api/dados", None);
    assert_eq!(status, 200, "{corpo}");
    assert_eq!(serde_json::from_str::<Value>(&corpo).unwrap(), json!({"blueprints": []}));
    assert_eq!(pedir(s.porta, "GET", "/", None).0, 200);
    assert_eq!(pedir(s.porta, "GET", "/api/mcp/blueprints", None).0, 200);
    s.parar();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn corpo_grande_demais_e_recusado() {
    let dir = pasta("grande");
    let s = iniciar(config(&dir, None)).unwrap();
    let grande = format!(r#"{{"tipo":"progresso","texto":"{}"}}"#, "a".repeat(300 * 1024));
    let (status, _, corpo) = pedir(s.porta, "POST", "/api/mcp/evento", Some(&grande));
    assert_eq!(status, 413);
    assert_eq!(serde_json::from_str::<Value>(&corpo).unwrap(), json!({"erro": "grande demais", "codigo": "grande-demais"}));
    s.parar();
    let _ = std::fs::remove_dir_all(&dir);
}

/// Lê da conexão SSE até aparecer `trecho` (ou o prazo acabar).
fn ler_ate(s: &mut TcpStream, acumulado: &mut String, trecho: &str) -> bool {
    let mut buf = [0u8; 512];
    while !acumulado.contains(trecho) {
        match s.read(&mut buf) {
            Ok(0) | Err(_) => return false,
            Ok(n) => acumulado.push_str(&String::from_utf8_lossy(&buf[..n])),
        }
    }
    true
}

#[test]
fn a_fila_avisa_a_pagina_por_sse_assim_que_chega_item() {
    let dir = pasta("sse");
    let s = iniciar(config(&dir, None)).unwrap();
    let mut conexao = TcpStream::connect(("127.0.0.1", s.porta)).unwrap();
    conexao.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    write!(conexao, "GET /api/mcp/eventos HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nAccept: text/event-stream\r\n\r\n", s.porta).unwrap();
    let mut lido = String::new();
    assert!(ler_ate(&mut conexao, &mut lido, "retry: 3000"), "{lido}");
    assert!(lido.starts_with("HTTP/1.1 200") && lido.to_lowercase().contains("text/event-stream"), "{lido}");
    assert!(!lido.contains("event: fila"), "sem item novo, sem aviso");
    let (status, _, _) = pedir(s.porta, "POST", "/api/mcp/evento", Some(r#"{"tipo":"bloqueio","texto":"preciso de acesso"}"#));
    assert_eq!(status, 200);
    assert!(ler_ate(&mut conexao, &mut lido, "event: fila\ndata: nova"), "o aviso precisa chegar sem a página perguntar: {lido}");
    let (_, _, fila) = pedir(s.porta, "GET", "/api/mcp/fila", None);
    assert_eq!(serde_json::from_str::<Value>(&fila).unwrap()["itens"][0]["texto"], "preciso de acesso");
    s.parar();
    let _ = std::fs::remove_dir_all(&dir);
}
