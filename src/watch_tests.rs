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
        acompanhar(&mut conectar(&url, None).unwrap(), &mut saida, &("ws".into(), "bp".into()), &mut limite).unwrap();
        assert_eq!(String::from_utf8_lossy(&saida).lines().count(), esperado);
    }
    servidor.join().unwrap();
    assert!(conectar(&url, None).is_err());
    assert!(conectar("http://example.com", None).is_err());
}

fn iniciar_acao(canal: &str, texto: &str) -> String {
    format!("event: iniciar-acao\ndata: {{\"workspaceId\":\"ws\",\"blueprintId\":\"bp\",\"canalId\":\"{canal}\",\"texto\":\"{texto}\"}}\n\n")
}

#[test]
fn o_pedido_de_iniciar_so_chega_a_sessao_enderecada_e_so_com_canal() {
    let alvo = ("ws".into(), "bp".into());
    let sse = iniciar_acao("minha", "Execute a ação 1.1\\u001b") + &iniciar_acao("outra", "não é para mim");
    let (mut com_canal, mut sem_canal) = (Limite::novo(), Limite::novo());
    com_canal.canal = Some("minha".into());
    let (mut a, mut b) = (Vec::new(), Vec::new());
    acompanhar(&mut Cursor::new(sse.clone()), &mut a, &alvo, &mut com_canal).unwrap();
    acompanhar(&mut Cursor::new(sse), &mut b, &alvo, &mut sem_canal).unwrap();
    assert_eq!(String::from_utf8(a).unwrap(), "The Blueprint · pedido do usuário (ele clicou em Iniciar no app): Execute a ação 1.1\n");
    assert!(b.is_empty(), "o monitor e a sessão sem canal ignoram o pedido");
}

#[test]
fn a_pasta_vai_codificada_na_query_do_canal() {
    assert_eq!(codificar("C:\\Meus Projetos\\ação"), "C%3A%5CMeus%20Projetos%5Ca%C3%A7%C3%A3o".to_string());
    assert_eq!(codificar("a/b-c_d.e~"), "a/b-c_d.e~");
}
