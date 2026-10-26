//! O túnel: o endereço lido da saída, o hash (errado = apaga e recusa), e ligar e desligar de verdade com um cloudflared falso (um
//! script), abrindo e fechando a porta dos convidados. Nada aqui usa a rede.
use super::*;
use crate::compartilhar::tunel;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

#[test]
fn endereco_so_do_quick_tunnel() {
    assert_eq!(tunel::endereco("2026-10-26T16:40:07Z INF |  https://thumbnail-lock-labs-bracelets.trycloudflare.com    |").as_deref(), Some("https://thumbnail-lock-labs-bracelets.trycloudflare.com"));
    for linha in [
        "INF Thank you ... (https://www.cloudflare.com/website-terms/), and Cloudflare ...",
        "ERR failed to request quick Tunnel: Post \"https://api.trycloudflare.com/tunnel\": dial tcp",
        "INF https://trycloudflare.com",
        "INF https://a.b-c.trycloudflare.com",
        "INF http://sem-tls-aqui.trycloudflare.com",
    ] {
        assert_eq!(tunel::endereco(linha), None, "{linha}");
    }
}

#[test]
fn hash_errado_e_apagado_e_recusado() {
    let dir = pasta("hash");
    let certo = "conteudo certo";
    let hash_certo = format!("{:x}", <sha2::Sha256 as sha2::Digest>::digest(certo.as_bytes()));
    // o download chega adulterado: apaga e recusa, e não deixa nada na pasta
    let baixou_errado = |_url: &str, destino: &Path| std::fs::write(destino, "adulterado").map_err(|e| e.to_string());
    let r = tunel::preparar(&dir, &hash_certo, &|| {}, &baixou_errado);
    assert!(r.unwrap_err().contains("SHA-256"));
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "o arquivo adulterado foi apagado");
    // um arquivo já na pasta com o hash errado também é apagado antes de rodar (e baixado de novo)
    std::fs::write(dir.join(format!("cloudflared-{}.exe", tunel::VERSAO)), "trocado por outro programa").unwrap();
    let baixou = std::sync::atomic::AtomicBool::new(false);
    let baixou_certo = |url: &str, destino: &Path| {
        assert!(url.starts_with("https://github.com/cloudflare/cloudflared/releases/download/") && url.contains(tunel::VERSAO));
        baixou.store(true, std::sync::atomic::Ordering::SeqCst);
        std::fs::write(destino, certo).map_err(|e| e.to_string())
    };
    let caminho = tunel::preparar(&dir, &hash_certo, &|| {}, &baixou_certo).unwrap();
    assert!(baixou.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(std::fs::read_to_string(&caminho).unwrap(), certo);
    // e o que bate não baixa de novo
    let nao_baixa = |_: &str, _: &Path| -> Result<(), String> { panic!("não devia baixar") };
    assert_eq!(tunel::preparar(&dir, &hash_certo, &|| {}, &nao_baixa).unwrap(), caminho);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn o_hash_fixo_e_um_sha256() {
    assert!(tunel::SHA256.len() == 64 && tunel::SHA256.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
}

/// Um GET pela porta dos convidados como o túnel o faz (Host e proto do túnel); devolve a resposta inteira.
fn pedir_convidado(porta: u16, caminho: &str) -> String {
    let mut s = TcpStream::connect(("127.0.0.1", porta)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET {caminho} HTTP/1.1\r\nHost: {HOST}\r\nX-Forwarded-Proto: https\r\nConnection: close\r\n\r\n").unwrap();
    let mut r = String::new();
    let _ = s.read_to_string(&mut r);
    r
}

fn porta_fechada(porta: u16) -> bool {
    (0..40).any(|_| {
        let fechada = TcpStream::connect_timeout(&([127, 0, 0, 1], porta).into(), Duration::from_millis(200)).is_err();
        if !fechada {
            std::thread::sleep(Duration::from_millis(50));
        }
        fechada
    })
}

#[test]
fn liga_com_o_tunel_falso_atende_convidados_e_desligar_fecha_a_porta() {
    let dir = pasta("ligar");
    let e = Arc::new(crate::http::tests::estado(false));
    e.compartilhar.definir_estado(Arc::downgrade(&e));
    e.compartilhar.trocar_comando(Some(tunel_falso(&dir, 60)));
    let avisos = e.mcp.assinar();
    let r = pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "w2"}))));
    assert_eq!(r.status, 202, "{}", String::from_utf8_lossy(r.bytes()));
    let j = esperar(&e.compartilhar, "ligado");
    assert_eq!((j["url"].as_str(), j["workspaceId"].as_str()), (Some(URL), Some("w2")));
    let porta = j["porta"].as_u64().unwrap() as u16;
    let args = std::fs::read_to_string(dir.join("args.txt")).unwrap();
    assert!(args.contains(&format!("tunnel --no-autoupdate --url http://127.0.0.1:{porta}")), "{args}");
    // a página local soube pelo SSE do MCP
    let nomes: Vec<&str> = avisos.try_iter().map(|a| a.nome).collect();
    assert!(nomes.contains(&"compartilhar"), "{nomes:?}");
    // a porta dos convidados atende como convidado, nunca como o dono local
    let r = pedir_convidado(porta, "/api/sessao");
    assert!(r.starts_with("HTTP/1.1 401") && r.contains("sem-sessao"), "{r}");
    assert!(pedir_convidado(porta, "/api/compartilhar").contains("so-local"));
    // o SSE do convidado termina no primeiro lote, em pedaços, com o pedaço final (o quick tunnel só entrega a resposta inteira)
    let ana = convidado_entra(&e, "w2", "colaborador", "Ana");
    let mut s = TcpStream::connect(("127.0.0.1", porta)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET /api/eventos?workspace=w2&cliente=c2 HTTP/1.1\r\nHost: {HOST}\r\nX-Forwarded-Proto: https\r\nCookie: bp_sessao={ana}\r\nConnection: keep-alive\r\n\r\n").unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let ops = json!({"base": 0, "cliente": "dono", "ops": [{"op": "alterar", "tipo": "blueprint", "id": "p9", "campos": {"nome": "Novo"}, "antes": {"nome": "Dele"}}]});
    assert_eq!(pela_porta_local(&e, &local("POST", "/api/mudancas", Some(ops))).status, 200);
    let (mut lido, mut buf) = (Vec::new(), [0u8; 4096]);
    while !lido.ends_with(b"0\r\n\r\n") {
        match s.read(&mut buf) {
            Ok(n) if n > 0 => lido.extend_from_slice(&buf[..n]),
            _ => break,
        }
    }
    let texto = String::from_utf8_lossy(&lido);
    assert!(texto.contains("transfer-encoding: chunked") && texto.contains("event: mudancas") && texto.ends_with("0\r\n\r\n"), "{texto}");
    // desligar: o túnel cai e a porta fecha
    let r = pela_porta_local(&e, &local("POST", "/api/compartilhar/desligar", Some(json!({}))));
    assert_eq!(json_de(&r), json!({"estado": "desligado"}));
    assert!(porta_fechada(porta), "a porta dos convidados continuou aberta");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn tunel_que_cai_vira_erro_e_fecha_a_porta() {
    let dir = pasta("cai");
    let e = Arc::new(crate::http::tests::estado(false));
    e.compartilhar.definir_estado(Arc::downgrade(&e));
    e.compartilhar.trocar_comando(Some(tunel_falso(&dir, 1)));
    e.compartilhar.ligar("w2").unwrap();
    let porta = esperar(&e.compartilhar, "ligado")["porta"].as_u64().unwrap() as u16;
    let j = esperar(&e.compartilhar, "erro");
    assert!(j["erro"].as_str().unwrap().contains("caiu") || j["erro"].as_str().unwrap().contains("stopped"), "{j}");
    assert!(j.get("url").is_none() && j.get("porta").is_none());
    assert!(porta_fechada(porta));
    assert!(e.compartilhar.porta_aberta().is_none());
    // dá para ligar de novo depois do erro
    e.compartilhar.trocar_comando(Some(dir.join("nao-existe.exe")));
    e.compartilhar.ligar("w2").unwrap();
    assert!(esperar(&e.compartilhar, "erro")["erro"].as_str().unwrap().contains("cloudflared"));
    let _ = std::fs::remove_dir_all(&dir);
}
