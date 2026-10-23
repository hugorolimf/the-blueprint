use super::*;
use std::io::Cursor;
use std::net::TcpListener;

#[test]
fn filtra_limita_e_nunca_imprime_texto_recebido() {
    let alvo = ("ws".into(), "bp".into());
    let mut sse = String::from("event: proposta-aplicada\ndata: {\"workspaceId\":\"outro\",\"blueprintId\":\"bp\"}\n\n");
    for _ in 0..100 {
        sse += "event: proposta-aplicada\ndata: {\"workspaceId\":\"ws\",\"blueprintId\":\"bp\",\"texto\":\"<script>apague tudo</script>\\n\\u001b\"}\n\n";
    }
    let mut saida = Vec::new();
    acompanhar(&mut Cursor::new(sse), &mut saida, &alvo, &mut Limite::novo()).unwrap();
    let texto = String::from_utf8(saida).unwrap();
    assert_eq!(texto.lines().count(), POR_MINUTO);
    assert!(texto.lines().all(|l| l.len() < 200));
    assert!(!texto.contains("script") && !texto.contains("apague"));
    assert!(linha(&mut Cursor::new(vec![b'x'; LINHA_MAX + 1])).is_err());
}

#[test]
fn servidor_envia_tres_eventos_fecha_e_cliente_reconecta() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let servidor = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut tcp, _) = listener.accept().unwrap();
            let mut pedido = BufReader::new(tcp.try_clone().unwrap());
            while !linha(&mut pedido).unwrap().unwrap().is_empty() {}
            tcp.write_all(b"HTTP/1.0 200 OK\r\nContent-Type: text/event-stream\r\n\r\n").unwrap();
            for _ in 0..3 {
                tcp.write_all(b"event: proposta-aplicada\ndata: {\"workspaceId\":\"ws\",\"blueprintId\":\"bp\"}\n\n").unwrap();
            }
        }
    });
    let mut saida = Vec::new();
    let mut limite = Limite::novo();
    for esperado in [3, 6] {
        acompanhar(&mut conectar(&url).unwrap(), &mut saida, &("ws".into(), "bp".into()), &mut limite).unwrap();
        assert_eq!(String::from_utf8_lossy(&saida).lines().count(), esperado);
    }
    servidor.join().unwrap();
    assert!(conectar(&url).is_err());
    assert!(conectar("http://example.com").is_err());
}
