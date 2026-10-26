//! Monitor do Claude: SSE local, sem sondagem em repouso, sem imprimir conteúdo livre do usuário.
use crate::mcp::{cliente::requisitar, stdio::base};
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

const LINHA_MAX: usize = 2048;
const POR_MINUTO: usize = 20;

fn linha<R: BufRead>(leitor: &mut R) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let n = leitor.take((LINHA_MAX + 1) as u64).read_until(b'\n', &mut bytes)?;
    if n > LINHA_MAX { return Err(io::Error::new(io::ErrorKind::InvalidData, "linha SSE grande demais")); }
    if n == 0 { return Ok(None); }
    Ok(Some(String::from_utf8_lossy(&bytes).trim_end_matches(['\r', '\n']).into()))
}

/// Percent-encoding mínimo para a pasta ir na query (letras, números e `-_.~/` passam; o resto vira %XX).
fn codificar(t: &str) -> String {
    t.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~/".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

/// `canal` = (id, pasta) de uma sessão com o canal ligado: o servidor a lista como destinatário enquanto esta conexão estiver aberta.
fn conectar(url: &str, canal: Option<(&str, &str)>) -> io::Result<BufReader<TcpStream>> {
    let endereco = url.strip_prefix("http://").ok_or_else(|| io::Error::other("use http local"))?.trim_end_matches('/');
    // O monitor não aceita destinos remotos: o que chega aqui entra no contexto de uma sessão.
    let host = endereco.split(':').next().unwrap_or("");
    if !matches!(host, "127.0.0.1" | "localhost") { return Err(io::Error::other("use loopback")); }
    let destino = if endereco.contains(':') { endereco.to_owned() } else { format!("{endereco}:80") };
    let addr = destino.to_socket_addrs()?.next().ok_or_else(|| io::Error::other("sem endereço"))?;
    let mut tcp = TcpStream::connect_timeout(&addr, Duration::from_secs(2))?;
    tcp.set_read_timeout(Some(Duration::from_secs(45)))?;
    tcp.set_write_timeout(Some(Duration::from_secs(2)))?;
    // HTTP/1.0 evita transfer-encoding chunked e permite ler SSE diretamente.
    let consulta = canal.map(|(id, pasta)| format!("?canal={id}&cwd={}", codificar(pasta))).unwrap_or_default();
    write!(tcp, "GET /api/mcp/eventos{consulta} HTTP/1.0\r\nHost: {endereco}\r\nAccept: text/event-stream\r\n\r\n")?;
    let mut leitor = BufReader::new(tcp);
    let status = linha(&mut leitor)?.unwrap_or_default();
    if status.split_whitespace().nth(1) != Some("200") { return Err(io::Error::other("SSE indisponível")); }
    let mut tamanho = 0;
    let mut sse = false;
    while let Some(l) = linha(&mut leitor)? {
        tamanho += l.len();
        if tamanho > 8192 { return Err(io::Error::other("cabeçalho grande demais")); }
        if l.to_ascii_lowercase().starts_with("content-type: text/event-stream") { sse = true; }
        if l.is_empty() { break; }
    }
    if !sse { return Err(io::Error::other("resposta não é SSE")); }
    Ok(leitor)
}

fn alvo(url: &str, pasta: &str) -> Option<(String, String)> {
    let corpo = json!({"cwd": pasta}).to_string();
    let (200, r) = requisitar(url, "POST", "hook/sessao", Some(&corpo)).ok()? else { return None };
    let r: Value = serde_json::from_str(&r).ok()?;
    Some((r["workspaceId"].as_str()?.into(), r["blueprintId"].as_str()?.into()))
}

/// Os eventos endereçados a UMA sessão com canal (texto montado pelo app depois do clique do usuário) e o prefixo de cada um. Os dois "pedido do usuário"
/// são ordens; a "conversa do usuário" não é (as instruções do MCP dizem como responder). Contrato: docs/claude-na-folha.md.
fn prefixo(nome: &str) -> Option<&'static str> {
    match nome {
        "iniciar-acao" => Some("The Blueprint · pedido do usuário (ele clicou em Iniciar no app): "),
        "comentario-plano" => Some("The Blueprint · pedido do usuário (ele clicou em Enviar ao agente num comentário de plano): "),
        "conversa-usuario" => Some("The Blueprint · conversa do usuário (responda com responder_conversa; ajustes só por propor_blueprint): "),
        _ => None,
    }
}

struct Limite { inicio: Instant, linhas: usize, canal: Option<String> }
impl Limite {
    fn novo() -> Self { Self { inicio: Instant::now(), linhas: 0, canal: None } }
    fn mensagem(&mut self, nome: &str, dados: &str, alvo: &(String, String)) -> Option<String> {
        let texto = match nome {
            "proposta-aplicada" => "The Blueprint: o usuário aplicou uma proposta no blueprint desta pasta. Consulte ler_blueprint para ver o estado atual.",
            "checkpoint-marcado" => "The Blueprint: o usuário marcou um checkpoint no blueprint desta pasta. Consulte ler_blueprint para ver o estado atual.",
            _ if prefixo(nome).is_some() => "", // o texto vem do evento, abaixo; só a sessão com canal endereçada o recebe
            _ => return None,
        };
        let dados: Value = serde_json::from_str(dados).ok()?;
        if dados["workspaceId"] != alvo.0 || dados["blueprintId"] != alvo.1 { return None; }
        let texto = if let Some(prefixo) = prefixo(nome) {
            if dados["canalId"].as_str()? != self.canal.as_deref()? { return None; }
            let pedido: String = dados["texto"].as_str()?.chars().filter(|c| !c.is_control()).take(1200).collect();
            format!("{prefixo}{pedido}")
        } else { texto.to_string() };
        if self.inicio.elapsed() >= Duration::from_secs(60) { self.inicio = Instant::now(); self.linhas = 0; }
        if self.linhas >= POR_MINUTO { return None; }
        self.linhas += 1;
        Some(texto)
    }
}

#[cfg(test)]
fn acompanhar<R: BufRead, W: Write>(leitor: &mut R, saida: &mut W, alvo: &(String, String), limite: &mut Limite) -> io::Result<()> {
    acompanhar_com(leitor, alvo, limite, &mut |texto| { writeln!(saida, "{texto}")?; saida.flush() })
}

fn acompanhar_com<R: BufRead>(leitor: &mut R, alvo: &(String, String), limite: &mut Limite, avisar: &mut impl FnMut(&str) -> io::Result<()>) -> io::Result<()> {
    let (mut evento, mut dados) = (String::new(), String::new());
    while let Some(l) = linha(leitor)? {
        if l.is_empty() {
            if let Some(texto) = limite.mensagem(&evento, &dados, alvo) { avisar(&texto)?; }
            evento.clear(); dados.clear();
        } else if let Some(e) = l.strip_prefix("event:") { evento = e.trim().into(); }
        else if let Some(d) = l.strip_prefix("data:") {
            if dados.len() + d.len() > LINHA_MAX { return Err(io::Error::other("evento grande demais")); }
            dados.push_str(d.trim_start());
        }
    }
    Ok(())
}

pub fn executar() -> i32 {
    if std::env::var("BLUEPRINT_CHANNEL").as_deref() == Ok("1") { return 0; }
    observar(None, |texto| { let mut saida = io::stdout().lock(); writeln!(saida, "{texto}")?; saida.flush() });
    0
}

/// `canal`: o id desta sessão com o canal ligado (só o `--mcp` com `BLUEPRINT_CHANNEL=1` passa); sem ele, o pedido de iniciar uma ação nunca chega aqui.
pub(crate) fn observar(canal: Option<String>, mut avisar: impl FnMut(&str) -> io::Result<()>) {
    let url = base();
    let pasta = std::env::current_dir().unwrap_or_default().display().to_string();
    let mut espera = 1;
    let mut limite = Limite::novo();
    limite.canal = canal.clone();
    loop {
        if let Some(alvo) = alvo(&url, &pasta) {
            if let Ok(mut leitor) = conectar(&url, canal.as_deref().map(|id| (id, pasta.as_str()))) {
                let inicio = Instant::now();
                let _ = acompanhar_com(&mut leitor, &alvo, &mut limite, &mut avisar);
                if inicio.elapsed() >= Duration::from_secs(30) { espera = 1; }
            }
        }
        std::thread::sleep(Duration::from_secs(espera));
        espera = (espera * 2).min(30);
    }
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
