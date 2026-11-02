//! Abre o invólucro do agente num terminal à vista: porte de `iniciar::abrir_terminal_com` com lista de argumentos. O texto do plano nunca chega aqui (D6):
//! `lancar` só aceita `["agente-plano", <16 hex>]`, e o invólucro busca o resto por HTTP. O `;`, separador do `wt`, e as aspas de cada terminal deixam de ser assunto.
#![allow(dead_code)] // quem usa é o `motor`, de outra fatia
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::Command;

const SUBCOMANDO: &str = "agente-plano";

/// Os únicos argumentos que um terminal recebe.
pub fn args_do_agente(execucao_id: &str) -> Vec<OsString> {
    vec![SUBCOMANDO.into(), execucao_id.into()]
}

fn id_valido(id: &OsStr) -> bool {
    id.to_str().is_some_and(|s| s.len() == 16 && s.bytes().all(|c| c.is_ascii_hexdigit()))
}

/// O lançador do motor (`Ambiente::lancar`).
pub fn lancar(pasta: &Path, exe: &Path, args: &[OsString]) -> Result<&'static str, String> {
    lancar_com(&crate::registrar::procurar, pasta, exe, args)
}

/// `lancar` com a busca de programas à parte, para o teste usar um terminal falso sem mexer no `PATH` do processo.
pub(crate) fn lancar_com(procurar: &dyn Fn(&str) -> Option<PathBuf>, pasta: &Path, exe: &Path, args: &[OsString]) -> Result<&'static str, String> {
    match args {
        [sub, id] if sub == SUBCOMANDO && id_valido(id) => abrir(procurar, pasta, exe, args),
        _ => Err("argumentos-recusados".into()),
    }
}

/// `xdg-terminal-exec` (o do Omarchy) ou `x-terminal-emulator -e`, já na pasta. Sem terminal é erro: rodar sem janela deixaria o agente invisível, e o usuário não teria como responder a ele.
#[cfg(not(windows))]
fn abrir(procurar: &dyn Fn(&str) -> Option<PathBuf>, pasta: &Path, exe: &Path, args: &[OsString]) -> Result<&'static str, String> {
    use std::process::Stdio;
    for (terminal, antes) in [("xdg-terminal-exec", &[][..]), ("x-terminal-emulator", &["-e"][..])] {
        let Some(caminho) = procurar(terminal) else { continue };
        let mut c = Command::new(caminho);
        c.args(antes).arg(exe).args(args).current_dir(pasta).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        let filho = c.spawn().map_err(|e| format!("{terminal} em {}: {e}", pasta.display()))?;
        recolher(filho);
        return Ok(terminal);
    }
    Err("sem-terminal".into())
}

/// `wt -d <pasta> <exe> ...` ou um console novo, direto no processo. O `wt` lê `;` como separador em qualquer argumento: pasta assim vai ao console novo.
#[cfg(windows)]
fn abrir(procurar: &dyn Fn(&str) -> Option<PathBuf>, pasta: &Path, exe: &Path, args: &[OsString]) -> Result<&'static str, String> {
    use std::os::windows::process::CommandExt;
    let tem_separador = |p: &Path| p.to_string_lossy().contains(';');
    let mut falha_wt = String::new();
    if let Some(wt) = procurar("wt").filter(|_| !tem_separador(pasta) && !tem_separador(exe)) {
        let mut c = Command::new(&wt);
        c.arg("-d").arg(pasta).arg(exe).args(args).creation_flags(0x0800_0000); // CREATE_NO_WINDOW: o wt abre a janela dele
        match c.spawn() {
            Ok(filho) => {
                recolher(filho);
                return Ok("wt");
            }
            Err(e) => falha_wt = format!("wt ({}): {e}; ", wt.display()),
        }
    }
    let mut c = Command::new(exe);
    c.args(args).current_dir(pasta).creation_flags(0x0000_0010); // CREATE_NEW_CONSOLE
    let filho = c.spawn().map_err(|e| format!("{falha_wt}console novo ({}) em {}: {e}", exe.display(), pasta.display()))?;
    recolher(filho);
    Ok("cmd")
}

/// Não espera o terminal, mas colhe o fim dele: sem isso, cada agente deixaria um processo defunto até o app fechar.
fn recolher(mut filho: std::process::Child) {
    std::thread::spawn(move || {
        let _ = filho.wait();
    });
}
