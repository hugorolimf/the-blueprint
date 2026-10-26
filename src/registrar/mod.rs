//! `blueprint registrar [--remover]`: liga (ou desliga) o `blueprint` no Claude Code (plugin com skill, agente e MCP) e no Codex (só o MCP).
//! O instalador chama isto ao instalar e ao desinstalar, então ninguém precisa digitar `claude mcp add …` nem `codex mcp add …`.
//! Usa as próprias CLIs (elas sabem onde e como guardam a configuração) e só mexe no servidor chamado `blueprint`; o que não achar no PATH
//! é dito com o comando a rodar à mão. shortcut: no PATH só procura .exe, .cmd e .bat (o `codex` do npm é .cmd); sem PATHEXT inteiro.
pub mod conectar;
#[cfg(test)]
mod conectar_tests;
pub(crate) mod plugin;
pub mod statusline;
mod subagentes;
#[cfg(test)]
mod tests;

use crate::http::config::pasta_do_app;
use std::path::{Path, PathBuf};
use std::process::Command;

const NOME: &str = "blueprint";

/// Cada agente que aceita MCP: o executável e como ele registra e remove (`{exe}` é o caminho deste binário).
struct Agente {
    titulo: &'static str,
    cli: &'static str,
    registrar: &'static [&'static str],
    remover: &'static [&'static str],
}

const AGENTES: [Agente; 2] = [
    Agente {
        titulo: "Claude Code",
        cli: "claude",
        // escopo user: vale em qualquer pasta, não só na que estava aberta na instalação
        registrar: &["mcp", "add", "--scope", "user", NOME, "--", "{exe}", "--mcp"],
        remover: &["mcp", "remove", "--scope", "user", NOME],
    },
    Agente { titulo: "Codex", cli: "codex", registrar: &["mcp", "add", NOME, "--", "{exe}", "--mcp"], remover: &["mcp", "remove", NOME] },
];

fn montar(modelo: &[&str], exe: &Path) -> Vec<String> {
    modelo.iter().map(|p| if *p == "{exe}" { exe.display().to_string() } else { (*p).to_string() }).collect()
}

/// O arquivo da CLI dentro de `dirs`, na ordem em que o Windows tentaria (`claude.exe` antes de `claude.cmd`).
pub(crate) fn procurar_em(cli: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let extensoes: &[&str] = if cfg!(windows) { &["exe", "cmd", "bat"] } else { &[""] };
    dirs.iter().flat_map(|d| extensoes.iter().map(move |e| if e.is_empty() { d.join(cli) } else { d.join(format!("{cli}.{e}")) })).find(|p| p.is_file())
}

pub(crate) fn procurar(cli: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    procurar_em(cli, &std::env::split_paths(&path).collect::<Vec<_>>())
}

/// Roda a CLI (pelo caminho completo: o Rust sabe chamar .cmd com segurança) e devolve a saída de erro se falhar.
fn rodar(cli: &Path, args: &[String]) -> Result<(), String> {
    let saida = Command::new(cli).args(args).output().map_err(|e| format!("não consegui executar {}: {e}", cli.display()))?;
    if saida.status.success() {
        return Ok(());
    }
    let texto = String::from_utf8_lossy(if saida.stderr.is_empty() { &saida.stdout } else { &saida.stderr }).trim().to_string();
    Err(if texto.is_empty() { format!("saiu com {}", saida.status) } else { texto })
}

pub(crate) fn dentro_de_target(exe: &Path) -> bool {
    exe.components().any(|c| c.as_os_str() == "target")
}

/// Um .exe de `target\` não pode ser o que o agente abre: com o MCP e o monitor rodando nele, o `cargo build`/`cargo dev` não consegue
/// sobrescrevê-lo ("Acesso negado"). Nesse caso o agente passa a usar uma cópia em `%APPDATA%\Blueprint\bin\`.
pub(crate) fn exe_estavel(exe: PathBuf) -> Result<PathBuf, String> {
    if !dentro_de_target(&exe) {
        return Ok(exe);
    }
    let destino = pasta_do_app().join("bin").join("blueprint.exe");
    let copiar = || std::fs::create_dir_all(destino.parent().unwrap()).and_then(|()| std::fs::copy(&exe, &destino));
    copiar().map(|_| destino.clone()).map_err(|e| format!("não consegui copiar para {} ({e}); feche as sessões do agente que usam o blueprint e tente de novo", destino.display()))
}

/// Registra (ou remove) em cada agente instalado. Devolve o texto para mostrar e o código de saída: 1 só se um agente existe e falhou.
pub fn executar(remover: bool) -> (String, i32) {
    let exe = match std::env::current_exe() {
        Ok(e) if remover => e,
        Ok(e) => match exe_estavel(e) {
            Ok(e) => e,
            Err(e) => return (format!("erro: {e}"), 1),
        },
        Err(e) => return (format!("erro: não sei onde este programa está: {e}"), 1),
    };
    let (mut linhas, mut codigo) = (Vec::new(), 0);
    for agente in &AGENTES {
        let Some(cli) = procurar(agente.cli) else {
            let dica = if remover { String::new() } else { format!(" (depois de instalar, rode: {} {})", agente.cli, montar(agente.registrar, &exe).join(" ")) };
            linhas.push(format!("{}: não encontrado no PATH{dica}", agente.titulo));
            continue;
        };
        // idempotente: tira o que já existe com o mesmo nome (falha se não existia, e tudo bem) antes de registrar de novo
        let _ = rodar(&cli, &montar(agente.remover, &exe));
        let e_claude = agente.cli == "claude";
        if remover {
            if e_claude {
                plugin::remover(&cli);
            }
            linhas.push(format!("{}: {} removido", agente.titulo, if e_claude { "plugin e MCP" } else { "MCP" }));
            continue;
        }
        if e_claude {
            let pasta = pasta_do_app().join("claude-plugin");
            match plugin::gravar(&pasta, &exe).map_err(|e| e.to_string()).and_then(|()| plugin::instalar(&cli, &pasta)) {
                Ok(()) => {
                    linhas.push(format!("{}: plugin `{NOME}` instalado (skill, agente e MCP)", agente.titulo));
                    continue;
                }
                // sem o plugin a pessoa ainda fica com o MCP
                Err(e) => linhas.push(format!("{}: plugin não instalado ({e}); registrando só o MCP", agente.titulo)),
            }
        }
        match rodar(&cli, &montar(agente.registrar, &exe)) {
            Ok(()) => linhas.push(format!("{}: MCP `{NOME}` registrado", agente.titulo)),
            Err(e) => {
                codigo = 1;
                linhas.push(format!("{}: não registrou ({e})", agente.titulo));
            }
        }
    }
    (linhas.join("\n"), codigo)
}
