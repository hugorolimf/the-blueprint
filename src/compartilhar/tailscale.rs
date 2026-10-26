//! O endereço fixo pelo Tailscale Funnel (docs/compartilhar.md, "A fixed address"): o app roda `tailscale funnel --https=<porta> http://127.0.0.1:<porta dos convidados>`
//! em primeiro plano, num Job Object como o cloudflared. Em primeiro plano o Funnel só existe enquanto o processo vive: se o app cair, o endereço público para de apontar para
//! a porta (que outro programa poderia reaproveitar), em vez de ficar configurado no Tailscale como com `--bg`. O endereço (`https://<máquina>.<tailnet>.ts.net`) é fixo; o login
//! e a conta são do próprio Tailscale (o app não guarda senha nem chave). Antes de ligar: `tailscale status --json` (conectado? qual o nome?) e `tailscale serve status --json`
//! (qual das portas 443, 8443 e 10000 está livre, para nunca trocar a configuração de quem já usa o Serve ou o Funnel).
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// BLUEPRINT_TAILSCALE_COMANDO troca o programa (testes e e2e sem o Tailscale de verdade); `Compartilhar::novo` a lê.
pub const VAR_COMANDO: &str = "BLUEPRINT_TAILSCALE_COMANDO";
/// As únicas portas em que o Funnel escuta (documentação do Tailscale).
const PORTAS: [u16; 3] = [443, 8443, 10000];

/// O `tailscale` desta máquina: o trocado para teste, o do PATH ou o da instalação padrão do Windows.
pub fn programa(trocado: Option<PathBuf>) -> Option<PathBuf> {
    if trocado.is_some() {
        return trocado;
    }
    crate::registrar::procurar("tailscale").or_else(|| ["C:\\Program Files\\Tailscale\\tailscale.exe", "C:\\Program Files (x86)\\Tailscale\\tailscale.exe"].into_iter().map(PathBuf::from).find(|p| p.is_file()))
}

/// Roda o programa e devolve o que ele imprimiu na saída padrão (até 20 s).
fn rodar(programa: &Path, args: &[&str]) -> Result<String, String> {
    let mut c = Command::new(programa);
    c.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000);
    }
    let mut filho = c.spawn().map_err(|e| crate::tr!("could not run tailscale: {e}", "não consegui rodar o tailscale: {e}"))?;
    let mut saida = filho.stdout.take();
    let leitor = std::thread::spawn(move || {
        let mut t = String::new();
        if let Some(s) = saida.as_mut() {
            let _ = s.read_to_string(&mut t);
        }
        t
    });
    let fim = Instant::now() + Duration::from_secs(20);
    loop {
        match filho.try_wait() {
            Ok(Some(_)) => return Ok(leitor.join().unwrap_or_default()),
            Ok(None) if Instant::now() >= fim => {
                let _ = filho.kill();
                let _ = filho.wait();
                return Err(crate::tr!("tailscale did not answer in 20 s", "o tailscale não respondeu em 20 s"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return Err(e.to_string()),
        }
    }
}

/// O nome da máquina na tailnet (`maquina.tailnet.ts.net`, sem o ponto final) de um `tailscale status --json`, só se o Tailscale está conectado.
pub fn nome_da_maquina(status_json: &str) -> Result<String, String> {
    let v: serde_json::Value = serde_json::from_str(status_json).map_err(|_| crate::tr!("tailscale status did not return JSON", "o tailscale status não devolveu JSON"))?;
    let estado = v["BackendState"].as_str().unwrap_or("");
    if estado != "Running" {
        return Err(crate::tr!("Tailscale is not connected (state: {estado}). Open Tailscale and sign in, then try again.", "O Tailscale não está conectado (estado: {estado}). Abra o Tailscale, entre na sua conta e tente de novo."));
    }
    let nome = v["Self"]["DNSName"].as_str().unwrap_or("").trim_end_matches('.').to_ascii_lowercase();
    let valido = nome.ends_with(".ts.net") && nome.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.');
    if !valido {
        return Err(crate::tr!("This device has no .ts.net name: turn on MagicDNS and HTTPS certificates for your tailnet in the Tailscale admin console.", "Este dispositivo não tem nome .ts.net: ligue o MagicDNS e os certificados HTTPS da sua tailnet no painel do Tailscale."));
    }
    Ok(nome)
}

/// A primeira das portas do Funnel que o Serve ainda não usa, lida de `tailscale serve status --json` (`{}` ou vazio = nenhuma em uso).
pub fn porta_livre(serve_json: &str) -> Result<u16, String> {
    let v: serde_json::Value = if serve_json.trim().is_empty() { serde_json::json!({}) } else { serde_json::from_str(serve_json).map_err(|_| crate::tr!("tailscale serve status did not return JSON", "o tailscale serve status não devolveu JSON"))? };
    let em_uso: Vec<u16> = v["TCP"].as_object().map(|m| m.keys().filter_map(|k| k.parse().ok()).collect()).unwrap_or_default();
    PORTAS.into_iter().find(|p| !em_uso.contains(p)).ok_or_else(|| crate::tr!("The three Funnel ports (443, 8443 and 10000) are already in use by your Tailscale Serve or Funnel; free one and try again.", "As três portas do Funnel (443, 8443 e 10000) já estão em uso pelo seu Tailscale Serve ou Funnel; libere uma e tente de novo."))
}

/// O endereço que o `tailscale funnel` imprime (`https://maquina.tailnet.ts.net/` ou com `:8443`); só nomes `.ts.net`.
pub fn endereco(linha: &str) -> Option<String> {
    linha.match_indices("https://").find_map(|(i, _)| {
        let resto = &linha[i + 8..];
        let host: String = resto.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.').collect::<String>().to_ascii_lowercase();
        let nome = host.strip_suffix(".ts.net")?;
        if nome.is_empty() {
            return None;
        }
        let porta: String = resto[host.len()..].strip_prefix(':').map(|p| p.chars().take_while(char::is_ascii_digit).collect()).unwrap_or_default();
        Some(if porta.is_empty() || porta == "443" { format!("https://{host}") } else { format!("https://{host}:{porta}") })
    })
}

/// Liga o Funnel: confere o Tailscale, escolhe a porta livre e deixa o processo de pé. Devolve o túnel e o endereço fixo.
pub fn abrir(trocado: Option<PathBuf>, porta_local: u16, ao_cair: impl FnOnce() + Send + 'static) -> Result<(super::tunel::Tunel, String), String> {
    let programa = programa(trocado).ok_or_else(|| crate::tr!("Tailscale is not installed on this computer (tailscale.com/download).", "O Tailscale não está instalado neste computador (tailscale.com/download)."))?;
    let maquina = nome_da_maquina(&rodar(&programa, &["status", "--json"])?)?;
    let porta = porta_livre(&rodar(&programa, &["serve", "status", "--json"]).unwrap_or_default())?;
    let args = [String::from("funnel"), format!("--https={porta}"), format!("http://127.0.0.1:{porta_local}")];
    let (tunel, impresso) = super::tunel::Tunel::abrir_com(&programa, &args, endereco, "tailscale", ao_cair)?;
    // o nome impresso tem de ser o desta máquina: nada de aceitar o endereço de outra coisa que apareça na saída
    let esperado = if porta == 443 { format!("https://{maquina}") } else { format!("https://{maquina}:{porta}") };
    if impresso != esperado {
        return Err(crate::tr!("tailscale printed {impresso}, not the address of this device ({esperado})", "o tailscale imprimiu {impresso}, e não o endereço deste dispositivo ({esperado})"));
    }
    Ok((tunel, esperado))
}
