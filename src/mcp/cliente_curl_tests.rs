use super::cliente_curl::{config, requisitar, separar};
use std::io::{Read, Write};
use std::net::TcpListener;

#[test]
fn a_chave_e_o_corpo_vao_no_config_com_aspas_escapadas() {
    let c = config("https://equipe.teste/api/mcp/evento", "POST", Some(r#"{"texto":"a \"b\" \\ c"}"#), Some("bpk_x"), "en");
    assert!(c.contains("url = \"https://equipe.teste/api/mcp/evento\"\n"));
    assert!(c.contains("header = \"Authorization: Bearer bpk_x\"\n"));
    assert!(c.contains(r#"data-binary = "{\"texto\":\"a \\\"b\\\" \\\\ c\"}""#), "{c}");
    assert!(c.contains("write-out = \"\\n%{http_code}\""));
    let sem = config("https://e/api/mcp/fila", "GET", None, None, "en");
    assert!(!sem.contains("Authorization") && !sem.contains("data-binary"));
}

#[test]
fn separa_o_status_da_ultima_linha() {
    assert_eq!(separar("{\"ok\":true}\n200"), Some((200, "{\"ok\":true}".into())));
    assert_eq!(separar("\n401"), Some((401, String::new())));
    assert_eq!(separar("\n000"), None, "000 = o curl não conseguiu falar com o servidor");
    assert_eq!(separar(""), None);
}

#[test]
fn o_curl_entrega_a_chave_no_cabecalho_e_o_corpo() {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let servidor = std::thread::spawn(move || {
        let (mut s, _) = ouvinte.accept().unwrap();
        s.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        let mut pedido = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = s.read(&mut buf) {
            pedido.extend_from_slice(&buf[..n]);
            if n == 0 || String::from_utf8_lossy(&pedido).contains("\"fim\":1}") {
                break;
            }
        }
        s.write_all(b"HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}").unwrap();
        String::from_utf8_lossy(&pedido).into_owned()
    });
    let url = format!("http://127.0.0.1:{porta}/api/mcp/evento");
    let r = requisitar(&url, "POST", Some("{\"texto\":\"oi\",\"fim\":1}"), Some("bpk_teste"), || "sem resposta".into());
    let recebido = servidor.join().unwrap();
    assert_eq!(r, Ok((201, "{\"ok\":true}".into())));
    assert!(recebido.starts_with("POST /api/mcp/evento HTTP/1.1"), "{recebido}");
    assert!(recebido.contains("Authorization: Bearer bpk_teste"), "{recebido}");
    assert!(recebido.contains("Content-Type: application/json"));
}
