//! Conferir se esta máquina alcança o repositório: `git ls-remote` com o git e a credencial que a pessoa já tem. Nada pergunta nem abre janela
//! (`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never`), nenhum token é lido ou guardado e a saída do git não vai a lugar nenhum.
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Troca o programa `git` por outro (só para os testes, que não têm rede).
pub const VAR_GIT: &str = "BLUEPRINT_GIT_COMANDO";
const LIMITE: Duration = Duration::from_secs(20);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Acesso {
    Com,
    Sem,
    GitAusente,
    SemResposta,
}

impl Acesso {
    /// O código estável que a página lê (docs/repositorio-github.md, "Estados do acesso").
    pub fn codigo(self) -> &'static str {
        match self {
            Acesso::Com => "com-acesso",
            Acesso::Sem => "sem-acesso",
            Acesso::GitAusente => "git-ausente",
            Acesso::SemResposta => "sem-resposta",
        }
    }
}

pub fn conferir(url: &str) -> Acesso {
    let git = std::env::var(VAR_GIT).ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| "git".into());
    conferir_com(&git, url, LIMITE)
}

pub fn conferir_com(git: &str, url: &str, limite: Duration) -> Acesso {
    let mut c = Command::new(git);
    // `--` impede que a URL seja lida como opção; a URL já passou por `repositorio_valido`
    c.args(["ls-remote", "--exit-code", "--", url, "HEAD"]).env("GIT_TERMINAL_PROMPT", "0").env("GCM_INTERACTIVE", "never").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    sem_janela(&mut c);
    let Ok(mut filho) = c.spawn() else { return Acesso::GitAusente };
    let fim = Instant::now() + limite;
    loop {
        match filho.try_wait() {
            Ok(Some(estado)) => return if estado.success() { Acesso::Com } else { Acesso::Sem },
            Ok(None) if Instant::now() >= fim => {
                let _ = filho.kill();
                let _ = filho.wait();
                return Acesso::SemResposta;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(_) => return Acesso::Sem,
        }
    }
}

#[cfg(windows)]
fn sem_janela(c: &mut Command) {
    use std::os::windows::process::CommandExt;
    c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: o app não prende nem pisca um console
}
#[cfg(not(windows))]
fn sem_janela(_: &mut Command) {}
