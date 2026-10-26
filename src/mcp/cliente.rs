//! Cliente HTTP mínimo para o modo `--mcp` falar com o servidor (http://127.0.0.1:3000 ou BLUEPRINT_URL), sem biblioteca de rede.
//! `http://` vai direto pelo socket; `https://` (servidor hospedado) vai pelo curl do sistema (`cliente_curl`). Com BLUEPRINT_CHAVE, todo
//! pedido leva `Authorization: Bearer` (a chave de MCP do membro).
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const PRAZO: Duration = Duration::from_secs(15);

/// `encodeURIComponent`
pub fn codificar(s: &str) -> String {
    s.bytes().fold(String::new(), |mut saida, b| {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) { saida.push(b as char) } else { saida.push_str(&format!("%{b:02X}")) }
        saida
    })
}

fn dividir(url: &str) -> Result<(String, String), String> {
    let resto = url.strip_prefix("http://").ok_or_else(|| crate::tr!("BLUEPRINT_URL must start with http:// or https:// (got \"{url}\").", "BLUEPRINT_URL precisa começar com http:// ou https:// (recebido \"{url}\")."))?;
    let host = resto.trim_end_matches('/').split('/').next().unwrap_or("");
    let com_porta = if host.rsplit_once(':').is_some_and(|(_, p)| p.bytes().all(|c| c.is_ascii_digit()) && !p.is_empty()) { host.to_string() } else { format!("{host}:80") };
    Ok((host.to_string(), com_porta))
}

fn desfazer_chunks(mut corpo: &[u8]) -> Vec<u8> {
    let mut saida = Vec::new();
    while let Some(fim) = corpo.windows(2).position(|w| w == b"\r\n") {
        let tamanho = std::str::from_utf8(&corpo[..fim]).ok().and_then(|t| usize::from_str_radix(t.split(';').next().unwrap_or("").trim(), 16).ok()).unwrap_or(0);
        if tamanho == 0 || corpo.len() < fim + 2 + tamanho {
            break;
        }
        saida.extend_from_slice(&corpo[fim + 2..fim + 2 + tamanho]);
        corpo = corpo.get(fim + 2 + tamanho + 2..).unwrap_or(&[]);
    }
    saida
}

/// Uma rota de /api/mcp. Devolve (status, corpo); erro de rede vira texto pronto para o agente.
pub fn requisitar(base: &str, metodo: &str, rota: &str, corpo: Option<&str>) -> Result<(u16, String), String> {
    requisitar_em(base, metodo, &format!("/api/mcp/{rota}"), corpo)
}

/// Qualquer caminho do servidor local.
pub fn requisitar_em(base: &str, metodo: &str, caminho: &str, corpo: Option<&str>) -> Result<(u16, String), String> {
    let equipe = super::equipe::conexao();
    if super::equipe::ligado() && equipe.is_none() {
        return Err(crate::tr!("Not connected to a shared workspace: run blueprint conectar <address> <key> (the command is in the My key screen).", "Sem conexão com um workspace compartilhado: rode blueprint conectar <endereço> <chave> (o comando está na tela Minha chave)."));
    }
    let sem_resposta = || crate::tr!("Could not reach The Blueprint at {base}. Is it running (blueprint serve)? If it uses another port, set BLUEPRINT_URL.", "Não consegui falar com o The Blueprint em {base}. Ele está rodando (blueprint serve)? Se usa outra porta, defina BLUEPRINT_URL.");
    let chave = equipe.map(|c| c.chave).or_else(|| std::env::var("BLUEPRINT_CHAVE").ok()).filter(|c| !c.trim().is_empty());
    // BLUEPRINT_MCP_CURL=1 força o curl também em http:// (é como os testes cobrem o caminho do https sem certificado)
    if base.starts_with("https://") || std::env::var("BLUEPRINT_MCP_CURL").as_deref() == Ok("1") {
        return super::cliente_curl::requisitar(&format!("{base}{caminho}"), metodo, corpo, chave.as_deref().map(str::trim), sem_resposta);
    }
    let (host, com_porta) = dividir(base)?;
    let endereco = com_porta.to_socket_addrs().ok().and_then(|mut a| a.next()).ok_or_else(sem_resposta)?;
    let mut conexao = TcpStream::connect_timeout(&endereco, PRAZO).map_err(|_| sem_resposta())?;
    let _ = (conexao.set_read_timeout(Some(PRAZO)), conexao.set_write_timeout(Some(PRAZO)));
    let dados = corpo.unwrap_or("");
    let mut envio = if corpo.is_some() { format!("Content-Type: application/json\r\nContent-Length: {}\r\n", dados.len()) } else { String::new() };
    if let Some(c) = chave.as_deref().map(str::trim).filter(|c| !c.contains(['\r', '\n'])) {
        envio.push_str(&format!("Authorization: Bearer {c}\r\n"));
    }
    // o idioma desta thread: inglês no `--mcp` (texto para o agente), o do usuário no `blueprint hook` (texto para a pessoa)
    let idioma = crate::idioma::atual().codigo();
    let pedido = format!("{metodo} {caminho} HTTP/1.1\r\nHost: {host}\r\n{envio}Accept: application/json\r\nAccept-Language: {idioma}\r\nConnection: close\r\n\r\n{dados}");
    conexao.write_all(pedido.as_bytes()).map_err(|_| sem_resposta())?;
    let mut bruto = Vec::new();
    conexao.read_to_end(&mut bruto).map_err(|_| sem_resposta())?;
    let fim_cabecalho = bruto.windows(4).position(|w| w == b"\r\n\r\n").ok_or_else(sem_resposta)?;
    let cabecalho = String::from_utf8_lossy(&bruto[..fim_cabecalho]).to_string();
    let status: u16 = cabecalho.split_whitespace().nth(1).and_then(|s| s.parse().ok()).ok_or_else(sem_resposta)?;
    let em_chunks = cabecalho.to_lowercase().contains("transfer-encoding: chunked");
    let resto = &bruto[fim_cabecalho + 4..];
    let corpo = if em_chunks { desfazer_chunks(resto) } else { resto.to_vec() };
    Ok((status, String::from_utf8_lossy(&corpo).into_owned()))
}
