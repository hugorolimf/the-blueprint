//! Testes do terminal do app: o texto UTF-8 partido, a citação da linha de comando, o SSE e (só no Windows) um pseudoconsole de verdade com
//! programas simples (cmd), nunca o claude.
use super::fluxo::FluxoTerminal;
use super::*;
use std::io::Read;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

#[test]
fn utf8_partido_entre_dois_pedacos_espera_o_resto() {
    let mut resto = Vec::new();
    let e = "é".as_bytes();
    assert_eq!(texto_utf8(&mut resto, &[b'a', e[0]]), "a");
    assert_eq!(resto, [e[0]]);
    assert_eq!(texto_utf8(&mut resto, &[e[1], b'b']), "éb");
    assert!(resto.is_empty());
    assert_eq!(texto_utf8(&mut resto, &[0xff, b'c']), "\u{FFFD}c", "byte inválido vira o caractere de troca, não trava");
}

#[test]
fn cita_como_o_windows_le() {
    assert_eq!(citar("claude"), "claude");
    assert_eq!(citar(""), "\"\"");
    assert_eq!(citar("C:\\Program Files\\b.exe"), "\"C:\\Program Files\\b.exe\"");
    assert_eq!(citar("diz \"oi\""), "\"diz \\\"oi\\\"\"");
    assert_eq!(citar("pasta com fim\\"), "\"pasta com fim\\\\\"", "barra antes da aspa final dobra");
}

fn ler_tudo(mut f: FluxoTerminal) -> String {
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    s
}

#[test]
fn sse_manda_o_historico_primeiro_e_o_fim_da_sessao_que_terminou() {
    let s = ler_tudo(FluxoTerminal::novo("\u{1b}[1mOi".into(), None, Some(3)));
    assert!(s.starts_with("retry: 3000\nevent: historico\ndata: \"\\u001b[1mOi\"\n\n"), "{s}");
    assert!(s.ends_with("event: fim\ndata: {\"codigo\":3}\n\n"), "{s}");
}

#[test]
fn sse_junta_pedacos_e_termina_no_fim() {
    let (tx, rx) = std::sync::mpsc::sync_channel(8);
    tx.send(Msg::Dados("a".into())).unwrap();
    tx.send(Msg::Dados("b".into())).unwrap();
    tx.send(Msg::Fim(Some(0))).unwrap();
    let s = ler_tudo(FluxoTerminal::novo(String::new(), Some(rx), None));
    assert!(s.contains("\ndata: \"ab\"\n\nevent: fim\ndata: {\"codigo\":0}\n\n"), "{s}");
}

#[cfg(not(windows))]
#[test]
fn fora_do_windows_recusa_com_motivo() {
    let e = Terminais::novo().abrir("sh", Path::new("."), Info::default(), 80, 24).err().unwrap();
    assert!(e.contains("Windows"));
}

/// Junta o que a sessão escreve até `parar` dizer chega (ou o prazo vencer); devolve o texto e o fim, se veio.
#[cfg(windows)]
fn ler_ate(rx: &Receiver<Msg>, inicial: String, segundos: u64, parar: impl Fn(&str) -> bool) -> (String, Option<Option<u32>>) {
    let limite = Instant::now() + Duration::from_secs(segundos);
    let mut t = inicial;
    while !parar(&t) {
        let Some(resta) = limite.checked_duration_since(Instant::now()) else { break };
        match rx.recv_timeout(resta) {
            Ok(Msg::Dados(d)) => t.push_str(&d),
            Ok(Msg::Fim(c)) => return (t, Some(c)),
            Err(_) => break,
        }
    }
    (t, None)
}

#[cfg(windows)]
fn cmd(args: &str) -> String {
    format!("{} {args}", citar(&std::env::var("ComSpec").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".into())))
}

#[cfg(windows)]
#[test]
fn programa_curto_escreve_e_termina_com_o_codigo() {
    let t = Terminais::novo();
    let s = t.abrir(&cmd("/c echo blueprint-oi&& exit 7"), &std::env::temp_dir(), Info { titulo: "teste".into(), ..Info::default() }, 80, 24).unwrap();
    let (historico, rx, _) = s.assinar();
    let (texto, fim) = match rx {
        Some(rx) => ler_ate(&rx, historico, 15, |_| false),
        None => (historico, Some(s.json()["codigo"].as_u64().map(|c| c as u32))),
    };
    assert!(texto.contains("blueprint-oi"), "{texto:?}");
    assert_eq!(fim, Some(Some(7)), "o código de saída chega no fim");
    assert!(!s.viva());
    assert_eq!(t.listar()[0]["codigo"], 7);
    assert!(t.fechar(&s.id) && t.listar().as_array().unwrap().is_empty());
}

#[cfg(windows)]
#[test]
fn sessao_interativa_le_teclas_redimensiona_e_fecha_sem_orfao() {
    // um neto com nome próprio (cópia do cmd) para conferir no tasklist que a árvore inteira cai ao fechar
    let pasta = std::env::temp_dir().join(format!("blueprint-terminal-{}", std::process::id()));
    std::fs::create_dir_all(&pasta).unwrap();
    let neto = format!("bporfao{}.exe", std::process::id());
    std::fs::copy(std::env::var("ComSpec").unwrap_or_else(|_| "C:\\Windows\\System32\\cmd.exe".into()), pasta.join(&neto)).unwrap();
    let rodando = || String::from_utf8_lossy(&std::process::Command::new("tasklist").args(["/FI", &format!("IMAGENAME eq {neto}")]).output().unwrap().stdout).contains(&neto);
    let t = Terminais::novo();
    let s = t.abrir(&cmd("/q /k"), &pasta, Info::default(), 80, 24).unwrap();
    let (historico, rx, _) = s.assinar();
    let rx = rx.expect("viva");
    assert!(s.escrever(b"set /a 6*7\r"));
    let (texto, fim) = ler_ate(&rx, historico, 15, |t| t.contains("42"));
    assert!(texto.contains("42") && fim.is_none(), "a tecla chegou e a resposta voltou: {texto:?}");
    assert!(s.redimensionar(120, 40));
    assert!(s.escrever(format!(".\\{neto} /c ping -n 60 127.0.0.1\r").as_bytes()));
    let comeco = Instant::now();
    while !rodando() && comeco.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(rodando(), "o neto subiu: {:?}", ler_ate(&rx, String::new(), 1, |_| false).0);
    assert!(t.fechar(&s.id));
    let (_, fim) = ler_ate(&rx, String::new(), 10, |_| false);
    assert!(fim.is_some(), "fechar avisa o fim às páginas");
    std::thread::sleep(Duration::from_millis(300));
    assert!(!rodando(), "nenhum processo órfão depois de fechar");
    assert!(!s.escrever(b"x"), "sessão fechada não aceita tecla");
    let _ = std::fs::remove_dir_all(&pasta);
}

#[cfg(windows)]
#[test]
fn programa_que_nao_existe_e_recusado_com_o_motivo() {
    let e = Terminais::novo().abrir("nao-existe-blueprint-xyz.exe", &std::env::temp_dir(), Info::default(), 80, 24).err().unwrap();
    assert!(e.contains("CreateProcessW"), "{e}");
}
