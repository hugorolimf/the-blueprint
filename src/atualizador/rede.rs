//! O acesso à rede do updater: um `curl.exe` (já vem no Windows 10 1803+), sem biblioteca HTTP nem TLS dentro do app (o binário tem meta de < 5 MB).
//! Só faz GET, sem token e sem enviar nada além do `User-Agent`. shortcut: o curl segue redirecionamentos (o GitHub manda os assets para outro host),
//! então a lista de hosts vale para a URL de partida e o SHA-256 (hash.rs) vale para o que chegou; se o curl faltar, trocar por WinHTTP (`windows-sys`).
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// Os hosts que o GitHub usa para servir o arquivo de uma release (a URL de partida e os redirecionamentos dela).
const HOSTS_ASSETS: [&str; 3] = ["github.com", "objects.githubusercontent.com", "release-assets.githubusercontent.com"];
const REPO_API: &str = "https://api.github.com/repos/hugorolimf/the-blueprint";
/// Teto do instalador (o de hoje tem ~3 MB) e da resposta da API.
pub const MAX_INSTALADOR: u64 = 100 * 1024 * 1024;
const MAX_TEXTO: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Origem {
    pub api: String,
    /// `BLUEPRINT_ATUALIZACAO_URL` ligada (testes): aceita http em 127.0.0.1 em vez de só os hosts do GitHub
    pub teste: bool,
}

impl Origem {
    pub fn do_ambiente() -> Origem {
        match std::env::var("BLUEPRINT_ATUALIZACAO_URL").ok().filter(|v| !v.is_empty()) {
            Some(url) => Origem { api: url.trim_end_matches('/').to_string(), teste: true },
            None => Origem { api: REPO_API.to_string(), teste: false },
        }
    }

    /// Pode baixar desta URL? Só https nos hosts de release do GitHub; no modo de teste, só o http local.
    pub fn permite(&self, url: &str) -> bool {
        if self.teste {
            return ["http://127.0.0.1", "http://localhost"].iter().any(|p| url.strip_prefix(p).is_some_and(|r| r.starts_with([':', '/'])));
        }
        let Some(resto) = url.strip_prefix("https://") else { return false };
        let host = resto.split(['/', ':', '?']).next().unwrap_or("").to_ascii_lowercase();
        HOSTS_ASSETS.contains(&host.as_str())
    }
}

/// O curl do Windows (System32), senão o do PATH; se não houver nenhum, o erro aparece ao rodar ("não consegui rodar o curl").
fn curl() -> PathBuf {
    let sistema = std::env::var_os("SystemRoot").map(|r| Path::new(&r).join("System32").join("curl.exe")).filter(|p| p.is_file());
    sistema.or_else(|| crate::registrar::procurar("curl")).unwrap_or_else(|| PathBuf::from("curl"))
}

fn comando(origem: &Origem, url: &str, max_segundos: u32) -> Command {
    let mut c = Command::new(curl());
    c.args(["-sS", "-L", "--fail", "--connect-timeout", "15", "--max-time", &max_segundos.to_string(), "-A", concat!("The-Blueprint-updater/", env!("CARGO_PKG_VERSION"))]);
    if !origem.teste {
        c.args(["--proto", "=https", "--proto-redir", "=https"]);
    }
    c.arg(url).stdin(Stdio::null());
    baixa_prioridade(&mut c);
    c
}

#[cfg(windows)]
fn baixa_prioridade(c: &mut Command) {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000 | 0x0000_4000); // sem janela de console, prioridade abaixo do normal: o download não atrapalha o trabalho
}
#[cfg(not(windows))]
fn baixa_prioridade(_c: &mut Command) {}

fn erro_do_curl(saida: &str) -> String {
    let linha = saida.trim().lines().last().unwrap_or("sem resposta");
    format!("não consegui falar com o GitHub: {linha}")
}

/// O corpo (texto) de uma URL da API, até 2 MB. A API pede o `Accept` próprio.
pub fn obter_texto(origem: &Origem, url: &str) -> Result<String, String> {
    let mut c = comando(origem, url, 30);
    let saida = c.args(["-H", "Accept: application/vnd.github+json"]).output().map_err(|e| format!("não consegui rodar o curl (ele vem no Windows 10 e 11): {e}"))?;
    if !saida.status.success() {
        return Err(erro_do_curl(&String::from_utf8_lossy(&saida.stderr)));
    }
    if saida.stdout.len() > MAX_TEXTO {
        return Err("a resposta do GitHub veio grande demais".into());
    }
    String::from_utf8(saida.stdout).map_err(|_| "a resposta do GitHub não é texto".to_string())
}

/// Baixa `url` para `destino` (que o chamador quer como `.parcial`). `progresso(bytes)` é chamado a cada 0,4 s enquanto baixa; passou de `limite`, o download
/// é cancelado. O arquivo só existe se deu certo: em falha ele é apagado.
pub fn baixar(origem: &Origem, url: &str, destino: &Path, limite: u64, progresso: &dyn Fn(u64)) -> Result<(), String> {
    let mut c = comando(origem, url, 900);
    let mut filho = c.arg("-o").arg(destino).stderr(Stdio::piped()).spawn().map_err(|e| format!("não consegui rodar o curl: {e}"))?;
    let resultado = loop {
        std::thread::sleep(Duration::from_millis(400));
        let tamanho = std::fs::metadata(destino).map_or(0, |m| m.len());
        progresso(tamanho);
        if tamanho > limite {
            let _ = filho.kill();
            let _ = filho.wait();
            break Err(format!("o arquivo passou do limite de {} MB e foi cancelado", limite / 1024 / 1024));
        }
        match filho.try_wait() {
            Ok(Some(estado)) if estado.success() => break Ok(()),
            Ok(Some(_)) => {
                let mut msg = String::new();
                if let Some(mut e) = filho.stderr.take() {
                    let _ = e.read_to_string(&mut msg);
                }
                break Err(erro_do_curl(&msg));
            }
            Ok(None) => {}
            Err(e) => break Err(format!("o download falhou: {e}")),
        }
    };
    if resultado.is_err() {
        let _ = std::fs::remove_file(destino);
    }
    resultado
}
