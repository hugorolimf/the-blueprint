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
fn comentario_e_conversa_seguem_o_mesmo_enderecamento_com_o_prefixo_de_cada_um() {
    let alvo = ("ws".into(), "bp".into());
    let evento = |nome: &str, canal: &str| iniciar_acao(canal, "Comentário no plano").replacen("iniciar-acao", nome, 1);
    let sse = evento("comentario-plano", "minha") + &evento("conversa-usuario", "minha") + &evento("conversa-usuario", "outra") + &evento("conversa", "minha");
    let mut limite = Limite::novo();
    limite.canal = Some("minha".into());
    let mut saida = Vec::new();
    acompanhar(&mut Cursor::new(sse), &mut saida, &alvo, &mut limite).unwrap();
    let saida = String::from_utf8(saida).unwrap();
    let linhas: Vec<&str> = saida.lines().collect();
    assert_eq!(linhas, [
        "The Blueprint · pedido do usuário (ele clicou em Enviar ao agente num comentário de plano): Comentário no plano",
        "The Blueprint · conversa do usuário (responda com responder_conversa; ajustes só por propor_blueprint): Comentário no plano",
    ], "a outra sessão e o aviso `conversa` (da página) não chegam");
}

#[test]
fn a_pasta_vai_codificada_na_query_do_canal() {
    assert_eq!(codificar("C:\\Meus Projetos\\ação"), "C%3A%5CMeus%20Projetos%5Ca%C3%A7%C3%A3o".to_string());
    assert_eq!(codificar("a/b-c_d.e~"), "a/b-c_d.e~");
}

#[test]
fn o_aviso_diz_qual_item_e_quem_aplicou() {
    use crate::idioma::{com, Idioma};
    let alvo = ("ws".into(), "bp".into());
    let rodar = |idioma: Idioma, eventos: &[&str]| -> String {
        let sse: String = eventos.iter().map(|d| format!("event: {}\ndata: {}\n\n", d.split_once('|').unwrap().0, d.split_once('|').unwrap().1)).collect();
        let mut saida = Vec::new();
        com(idioma, || acompanhar(&mut Cursor::new(sse), &mut saida, &alvo, &mut Limite::novo()).unwrap());
        String::from_utf8(saida).unwrap()
    };
    let base = r#""workspaceId":"ws","blueprintId":"bp""#;
    let em = rodar(Idioma::En, &[
        &format!(r#"proposta-aplicada|{{{base},"numero":3,"origem":"usuario"}}"#),
        &format!(r#"proposta-aplicada|{{{base},"numero":4,"origem":"automatica"}}"#),
        &format!(r#"registro-adicionado|{{{base},"numero":2,"origem":"usuario"}}"#),
        &format!(r#"registro-adicionado|{{{base},"numero":5,"origem":"automatica"}}"#),
    ]);
    let linhas: Vec<&str> = em.lines().collect();
    assert!(linhas[0].contains("approved proposal no. 3") && linhas[0].contains("explicit approval"), "{}", linhas[0]);
    assert!(linhas[1].contains("proposal no. 4 was applied AUTOMATICALLY") && linhas[1].contains("not an explicit approval"), "{}", linhas[1]);
    assert!(linhas[2].contains("added record no. 2") && linhas[2].contains("explicit approval"), "{}", linhas[2]);
    assert!(linhas[3].contains("record no. 5 was added AUTOMATICALLY") && linhas[3].contains("not an explicit approval"), "{}", linhas[3]);
    // em português, e sem número
    let pt = rodar(Idioma::Pt, &[&format!(r#"proposta-aplicada|{{{base},"numero":7,"origem":"automatica"}}"#), &format!(r#"proposta-aplicada|{{{base},"origem":"usuario"}}"#)]);
    let p: Vec<&str> = pt.lines().collect();
    assert!(p[0].contains("proposta nº 7 foi aplicada AUTOMATICAMENTE") && p[0].contains("não é uma aprovação explícita"), "{}", p[0]);
    assert!(p[1].contains("aprovou a proposta ao clicar") && !p[1].contains("{n}"), "{}", p[1]);
    // página antiga (sem origem) ou origem inventada: o texto genérico de sempre, nunca o de aprovação
    let velho = rodar(Idioma::En, &[&format!(r#"proposta-aplicada|{{{base}}}"#), &format!(r#"proposta-aplicada|{{{base},"numero":1,"origem":"o-agente"}}"#)]);
    assert!(velho.lines().all(|l| l.contains("applied a proposal") && !l.contains("explicit approval")), "{velho}");
}
