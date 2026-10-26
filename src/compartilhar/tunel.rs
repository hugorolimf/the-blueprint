//! O cloudflared (quick tunnel da Cloudflare, sem conta). Versão e SHA-256 fixos; baixado por curl.exe para %APPDATA%\Blueprint\bin só no
//! primeiro uso e conferido pelo hash antes de cada execução (hash errado: apaga e recusa). Roda sem autoupdate num Job Object com
//! KILL_ON_JOB_CLOSE (cai junto com o app) e o endereço `https://<nome>.trycloudflare.com` é lido da saída de erro dele, que continua
//! sendo lida até o fim (cano cheio travaria o cloudflared) e avisa quando o processo sai sozinho.
//! Fonte do hash: o `digest` do asset `cloudflared-windows-amd64.exe` da release oficial 2026.10.4 em
//! api.github.com/repos/cloudflare/cloudflared/releases, conferido em 2026-10-26 baixando o arquivo (sha256sum) e com o `Checksum` que o
//! próprio cloudflared imprime ao subir. Trocar de versão: os três valores abaixo juntos.
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

pub const VERSAO: &str = "2026.10.4";
pub const SHA256: &str = "411b16d40c8d872243efeb52c550053d00af0a8a97fbb32e730fb8aba429eaa8";
const URL: &str = "https://github.com/cloudflare/cloudflared/releases/download/2026.10.4/cloudflared-windows-amd64.exe";
/// o de hoje tem 53 MB
const MAX: u64 = 100 * 1024 * 1024;
const ESPERA_URL: Duration = Duration::from_secs(60);

pub fn pasta_bin() -> PathBuf {
    crate::http::config::pasta_do_app().join("bin")
}

fn nome() -> String {
    format!("cloudflared-{VERSAO}.exe")
}

/// O arquivo bate com o hash? Se não, apaga e recusa.
pub fn conferir_ou_apagar(caminho: &Path, esperado: &str) -> Result<(), String> {
    if crate::atualizador::hash::confere(caminho, esperado) {
        return Ok(());
    }
    let _ = std::fs::remove_file(caminho);
    Err(crate::tr!("the cloudflared file does not match the expected SHA-256 and was deleted", "o arquivo do cloudflared não bate com o SHA-256 esperado e foi apagado"))
}

/// O cloudflared pronto para rodar: o que já está em `pasta` (se o hash bate) ou um baixado agora. `baixando` avisa antes do download e
/// `baixar(url, destino)` é o download (o curl no app; os testes passam outro, sem rede).
pub fn preparar(pasta: &Path, esperado: &str, baixando: &dyn Fn(), baixar: &dyn Fn(&str, &Path) -> Result<(), String>) -> Result<PathBuf, String> {
    let caminho = pasta.join(nome());
    if caminho.is_file() && conferir_ou_apagar(&caminho, esperado).is_ok() {
        return Ok(caminho);
    }
    std::fs::create_dir_all(pasta).map_err(|e| e.to_string())?;
    baixando();
    let parcial = caminho.with_extension("parcial");
    baixar(URL, &parcial)?;
    conferir_ou_apagar(&parcial, esperado)?;
    std::fs::rename(&parcial, &caminho).map_err(|e| e.to_string())?;
    // a versão anterior (o nome leva a versão) não serve mais
    for antigo in std::fs::read_dir(pasta).into_iter().flatten().flatten().map(|e| e.path()) {
        let n = antigo.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if n.starts_with("cloudflared-") && antigo != caminho {
            let _ = std::fs::remove_file(&antigo);
        }
    }
    Ok(caminho)
}

/// O download de verdade: o curl do atualizador, só https e só os hosts de release do GitHub.
pub fn baixar_curl(url: &str, destino: &Path) -> Result<(), String> {
    let origem = crate::atualizador::rede::Origem { api: String::new(), teste: false };
    crate::atualizador::rede::baixar(&origem, url, destino, MAX, &|_| {})
}

/// O endereço do quick tunnel numa linha da saída (`https://<palavras-com-hífen>.trycloudflare.com`); ignora `api.trycloudflare.com` e
/// as outras URLs que o cloudflared imprime (termos de uso, documentação).
pub fn endereco(linha: &str) -> Option<String> {
    linha.match_indices("https://").find_map(|(i, _)| {
        let resto = &linha[i + 8..];
        let host: String = resto.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.').collect::<String>().to_ascii_lowercase();
        let nome = host.strip_suffix(".trycloudflare.com")?;
        (!nome.is_empty() && nome.contains('-') && nome.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')).then(|| format!("https://{host}"))
    })
}

/// O processo do túnel. Soltar o `Tunel` mata o processo (e o job derruba o que ele tiver aberto).
pub struct Tunel {
    filho: Child,
    #[cfg(windows)]
    _job: job::Job,
}

impl Tunel {
    /// O quick tunnel: roda `programa tunnel --no-autoupdate --url http://127.0.0.1:<porta>` e espera o endereço público (até 60 s). `ao_cair` roda se o
    /// processo sair depois de dar o endereço (inclusive quando o próprio app o derruba: quem chama confere se ainda é o túnel atual).
    pub fn abrir(programa: &Path, porta: u16, ao_cair: impl FnOnce() + Send + 'static) -> Result<(Tunel, String), String> {
        let args = ["tunnel", "--no-autoupdate", "--url"].map(String::from).into_iter().chain([format!("http://127.0.0.1:{porta}")]).collect::<Vec<_>>();
        Tunel::abrir_com(programa, &args, endereco, "cloudflared", ao_cair)
    }

    /// Qualquer programa que publica a porta e imprime o endereço (cloudflared, tailscale funnel): lê a saída padrão e a de erro, e `extrai` acha o endereço numa linha.
    /// `nome` é só para as mensagens. As duas saídas são lidas até o fim (cano cheio travaria o programa).
    pub fn abrir_com(programa: &Path, args: &[String], extrai: fn(&str) -> Option<String>, nome: &str, ao_cair: impl FnOnce() + Send + 'static) -> Result<(Tunel, String), String> {
        let mut c = Command::new(programa);
        c.args(args).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            c.creation_flags(0x0800_0000); // sem janela de console
        }
        let mut filho = c.spawn().map_err(|e| crate::tr!("could not run {nome}: {e}", "não consegui rodar o {nome}: {e}"))?;
        let (so, se) = (filho.stdout.take(), filho.stderr.take());
        // shortcut: o processo entra no job logo depois de nascer (o `Command` não cria suspenso); o programa não abre filhos nesse instante
        #[cfg(windows)]
        let tunel = match job::Job::prender(&filho) {
            Some(j) => Tunel { filho, _job: j },
            None => {
                let _ = filho.kill();
                let _ = filho.wait();
                return Err(crate::tr!("could not put {nome} in a Job Object", "não consegui pôr o {nome} num Job Object"));
            }
        };
        #[cfg(not(windows))]
        let tunel = Tunel { filho };
        let (linhas_tx, linhas_rx) = mpsc::channel::<String>();
        let fontes: Vec<Box<dyn Read + Send>> = [so.map(|f| Box::new(f) as Box<dyn Read + Send>), se.map(|f| Box::new(f) as Box<dyn Read + Send>)].into_iter().flatten().collect();
        for fonte in fontes {
            let tx = linhas_tx.clone();
            std::thread::spawn(move || {
                let (mut leitor, mut linha) = (BufReader::new(fonte), Vec::new());
                while leitor.read_until(10u8, &mut linha).is_ok_and(|n| n > 0) {
                    let _ = tx.send(String::from_utf8_lossy(&linha).trim().chars().take(300).collect());
                    linha.clear();
                }
            });
        }
        drop(linhas_tx);
        let (tx, rx) = mpsc::channel::<Result<String, String>>();
        let ultima = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let ultima_do_leitor = ultima.clone();
        std::thread::spawn(move || {
            let mut tx = Some(tx);
            for texto in linhas_rx {
                if let Some(url) = tx.as_ref().and(extrai(&texto)) {
                    let _ = tx.take().map(|t| t.send(Ok(url)));
                }
                *ultima_do_leitor.lock().unwrap_or_else(|e| e.into_inner()) = texto;
            }
            match tx {
                Some(t) => drop(t.send(Err(ultima_do_leitor.lock().unwrap_or_else(|e| e.into_inner()).clone()))),
                None => ao_cair(),
            }
        });
        match rx.recv_timeout(ESPERA_URL) {
            Ok(Ok(url)) => Ok((tunel, url)),
            Ok(Err(ultima)) => Err(crate::tr!("{nome} stopped before giving an address: {ultima}", "o {nome} parou antes de dar o endereço: {ultima}")),
            Err(_) => {
                let ultima = ultima.lock().unwrap_or_else(|e| e.into_inner()).clone();
                Err(crate::tr!("{nome} did not give an address in 60 s. Its last message: {ultima}", "o {nome} não deu o endereço em 60 s. Última mensagem dele: {ultima}"))
            }
        }
    }
}

impl Drop for Tunel {
    fn drop(&mut self) {
        let _ = self.filho.kill();
        let _ = self.filho.wait();
    }
}

#[cfg(windows)]
mod job {
    use std::ffi::c_void;
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use std::ptr::null;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE};

    /// Fechar o handle (no Drop, ou o app saindo de qualquer jeito) derruba os processos do job.
    pub struct Job(HANDLE);
    // SAFETY: um handle de job vale em qualquer thread; só é fechado no Drop.
    unsafe impl Send for Job {}

    impl Job {
        pub fn prender(filho: &Child) -> Option<Job> {
            // SAFETY: handles válidos (o do filho vive enquanto `filho` existir); o job é fechado no Drop ou aqui mesmo em falha.
            unsafe {
                let job = CreateJobObjectW(null(), null());
                if job.is_null() {
                    return None;
                }
                let j = Job(job);
                let mut limite: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                limite.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limite as *const _ as *const c_void, std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32) != 0
                    && AssignProcessToJobObject(job, filho.as_raw_handle() as HANDLE) != 0;
                ok.then_some(j)
            }
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            // SAFETY: handle próprio, fechado uma vez.
            unsafe { CloseHandle(self.0) };
        }
    }
}
