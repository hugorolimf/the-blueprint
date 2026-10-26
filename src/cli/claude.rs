use serde_json::{json, Value};
use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::process::Command;

const AGENTE: &str = "blueprint:blueprint";

/// O canal que o botão "Iniciar com o Claude" liga nas sessões que abre, para elas poderem receber depois o pedido de iniciar outra ação (docs/claude-na-folha.md,
/// "Escolher a sessão"). `@inline` porque este comando carrega o plugin por `--plugin-dir` (verificado com um Claude real em 2026-10-26, scripts/agente-real-canal.mjs:
/// com `@blueprint-local` o Claude Code avisa que o plugin instalado é `blueprint@inline` e a mensagem não chega).
const FLAG_CANAL: [&str; 2] = ["--dangerously-load-development-channels", "plugin:blueprint@inline"];

/// `--canal` (em qualquer posição) não é do Claude: liga o canal. A flag do canal vai no FIM, depois do prompt, porque pode aceitar vários valores e engoliria o prompt.
fn separa_canal(extras: &[String]) -> (bool, Vec<String>) {
    let canal = extras.iter().any(|e| e == "--canal");
    let mut resto: Vec<String> = extras.iter().filter(|e| *e != "--canal").cloned().collect();
    if canal {
        resto.extend(FLAG_CANAL.map(String::from));
    }
    (canal, resto)
}

/// `--equipe` (em qualquer posição) também não é do Claude: abre a sessão do convidado, que usa o plugin `blueprint-equipe` já instalado pelo `blueprint conectar`
/// (docs/app-no-projeto-compartilhado.md). Sem o plugin local nem o canal: eles falam com o app desta máquina, que não é o dono dos dados do workspace compartilhado.
fn separa_equipe(extras: &[String]) -> (bool, Vec<String>) {
    (extras.iter().any(|e| e == "--equipe"), extras.iter().filter(|e| *e != "--equipe").cloned().collect())
}

/// Os argumentos do `claude` do convidado: só o nome da sessão e o que veio na linha (o prompt).
fn argumentos_da_equipe(extras: &[String]) -> Vec<OsString> {
    ["--name", "The Blueprint"].into_iter().map(OsString::from).chain(extras.iter().map(OsString::from)).collect()
}

pub(super) fn executar(extras: &[String]) -> i32 {
    let (equipe, extras) = separa_equipe(extras);
    let (canal, extras) = separa_canal(&extras);
    let extras = extras.as_slice();
    let Some(cli) = crate::registrar::procurar("claude") else {
        eprintln!("erro: Claude Code não encontrado no PATH");
        return 1;
    };
    if equipe {
        return match Command::new(cli).args(argumentos_da_equipe(extras)).status() {
            Ok(status) => status.code().unwrap_or(1),
            Err(e) => {
                eprintln!("erro: não consegui iniciar Claude Code: {e}");
                1
            }
        };
    }
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("erro: não sei onde este programa está: {e}");
            return 1;
        }
    };
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("erro: não consegui ler a pasta atual: {e}");
            return 1;
        }
    };
    let (pasta, exe) = match plugin_e_exe(&crate::http::config::pasta_do_app(), &exe) {
        Ok(par) => par,
        Err(e) => {
            eprintln!("erro: não consegui preparar o plugin: {e}");
            return 1;
        }
    };
    if let Err(e) = crate::registrar::plugin::gravar(&pasta, &exe) {
        eprintln!("erro: não consegui preparar o plugin: {e}");
        return 1;
    }
    let ligado = blueprint_ligado(&cwd);
    if !ligado {
        eprintln!("Aviso: esta pasta não está ligada a um blueprint; abrindo Claude com o plugin, sem o agente Blueprint.");
    }
    let argumentos = argumentos(&pasta.join("plugins/blueprint"), &exe, ligado, extras);
    let mut comando = Command::new(cli);
    comando.args(argumentos);
    if canal {
        comando.env("BLUEPRINT_CHANNEL", "1");
    }
    match comando.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("erro: não consegui iniciar Claude Code: {e}");
            1
        }
    }
}

/// Onde gravar o plugin e qual .exe ele chama. Um build de desenvolvimento (dentro de `target`) não pode tomar o plugin do dia a dia
/// (toda sessão nova do Claude passaria a usar o .exe de teste) nem prender o .exe que o cargo recompila ("Acesso negado"):
/// usa `claude-plugin-dev` e uma cópia do .exe, uma por compilação; as cópias velhas saem quando nenhuma sessão as segura mais.
fn plugin_e_exe(app: &Path, exe: &Path) -> std::io::Result<(std::path::PathBuf, std::path::PathBuf)> {
    if !exe.components().any(|c| c.as_os_str() == "target") {
        return Ok((app.join("claude-plugin"), exe.to_path_buf()));
    }
    let pasta = app.join("claude-plugin-dev");
    let carimbo = fs::metadata(exe)?.modified()?.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let bin = pasta.join("bin");
    let copia = bin.join(format!("blueprint-{carimbo}.exe"));
    if !copia.exists() {
        fs::create_dir_all(&bin)?;
        for velha in fs::read_dir(&bin)?.flatten() {
            let _ = fs::remove_file(velha.path()); // a que ainda está em uso fica para a próxima vez
        }
        fs::copy(exe, &copia)?;
    }
    Ok((pasta, copia))
}

fn blueprint_ligado(cwd: &Path) -> bool {
    let corpo = json!({ "cwd": cwd.display().to_string() }).to_string();
    let resposta = crate::mcp::cliente::requisitar(
        &crate::mcp::stdio::base(),
        "POST",
        "hook/status",
        Some(&corpo),
    )
    .ok()
    .and_then(|(status, texto)| interpretar_status(status, &texto));
    resposta.unwrap_or_else(|| tem_linha_blueprint(cwd))
}

fn interpretar_status(status: u16, texto: &str) -> Option<bool> {
    if status != 200 {
        return None;
    }
    serde_json::from_str::<Value>(texto)
        .ok()?
        .get("vinculado")?
        .as_bool()
}

fn tem_linha_blueprint(cwd: &Path) -> bool {
    blueprint_da_pasta(cwd).is_some()
}

/// Os ids da linha `Blueprint: workspace_id …, blueprint_id …` do CLAUDE.md/AGENTS.md da pasta ou de uma pasta acima (o `blueprint hook --equipe`
/// do convidado também usa: o app remoto não conhece as pastas desta máquina).
pub(crate) fn blueprint_da_pasta(cwd: &Path) -> Option<(String, String)> {
    cwd.ancestors().find_map(|pasta| {
        ["CLAUDE.local.md", "CLAUDE.md", "AGENTS.md"].iter().find_map(|nome| fs::read_to_string(pasta.join(nome)).ok()?.lines().find_map(ids_da_linha))
    })
}

/// Aceita os ids com ou sem crase (`Blueprint: workspace_id `a`, blueprint_id `b`` ou `Blueprint: workspace_id a, blueprint_id b`).
pub(crate) fn ids_da_linha(linha: &str) -> Option<(String, String)> {
    let resto = linha.trim().strip_prefix("Blueprint: workspace_id ")?;
    let (workspace, resto) = resto.split_once(", blueprint_id ")?;
    let limpo = |s: &str| s.trim().trim_matches('`').to_string();
    let blueprint = resto.trim_start().strip_prefix('`').map_or_else(|| resto.split([',', ' ', '.']).next().unwrap_or(""), |r| r.split('`').next().unwrap_or(""));
    let (workspace, blueprint) = (limpo(workspace), limpo(blueprint));
    let valido = |id: &str| !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    (valido(&workspace) && valido(&blueprint)).then_some((workspace, blueprint))
}

fn argumentos(plugin: &Path, exe: &Path, ligado: bool, extras: &[String]) -> Vec<OsString> {
    let mut args = vec![
        OsString::from("--plugin-dir"),
        plugin.as_os_str().to_owned(),
        OsString::from("--plugin-dir"),
        plugin.parent().expect("pasta de plugins").join("blueprint-terminal").into_os_string(),
    ];
    if ligado {
        args.extend([OsString::from("--agent"), OsString::from(AGENTE)]);
    }
    let config = crate::hook::trecho_da_status_line(&exe.to_string_lossy());
    args.extend([
        OsString::from("--name"),
        OsString::from("The Blueprint"),
        OsString::from("--settings"),
        OsString::from(serde_json::to_string(&config).expect("json")),
    ]);
    args.extend(extras.iter().map(OsString::from));
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pasta_temporaria() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("blueprint-claude-{}-{id}", std::process::id()))
    }

    #[test]
    fn a_flag_equipe_abre_o_claude_do_convidado_sem_plugin_local_nem_canal() {
        let entrada: Vec<String> = ["Execute a ação 1", "--equipe"].map(String::from).to_vec();
        let (equipe, resto) = separa_equipe(&entrada);
        assert!(equipe);
        assert_eq!(resto, vec!["Execute a ação 1".to_string()]);
        let args: Vec<String> = argumentos_da_equipe(&resto).iter().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, ["--name", "The Blueprint", "Execute a ação 1"]);
        assert!(!args.iter().any(|a| a.contains("plugin") || a.contains("channel")), "{args:?}");
        assert_eq!(separa_equipe(&["x".to_string()]), (false, vec!["x".to_string()]));
    }

    #[test]
    fn build_de_desenvolvimento_usa_plugin_proprio_e_copia_do_exe() {
        let raiz = pasta_temporaria();
        let app = raiz.join("app");
        let instalado = raiz.join("Programs").join("blueprint.exe");
        assert_eq!(plugin_e_exe(&app, &instalado).unwrap(), (app.join("claude-plugin"), instalado));
        let dev = raiz.join("target").join("dev").join("debug").join("blueprint.exe");
        fs::create_dir_all(dev.parent().unwrap()).unwrap();
        fs::write(&dev, b"exe").unwrap();
        let (pasta, copia) = plugin_e_exe(&app, &dev).unwrap();
        assert_eq!(pasta, app.join("claude-plugin-dev"), "o plugin do dia a dia não é tocado");
        assert!(copia.starts_with(pasta.join("bin")) && fs::read(&copia).unwrap() == b"exe", "o plugin chama a cópia, não o .exe do cargo");
        assert_eq!(plugin_e_exe(&app, &dev).unwrap().1, copia, "mesma compilação, mesma cópia");
        let _ = fs::remove_dir_all(raiz);
    }

    #[test]
    fn exige_ids_reais_na_linha_de_vinculo() {
        assert_eq!(ids_da_linha("Blueprint: workspace_id `sm`, blueprint_id `ia-sm` (MCP `blueprint`)."), Some(("sm".into(), "ia-sm".into())));
        assert!(ids_da_linha("Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).").is_none());
        assert_eq!(ids_da_linha("Blueprint: workspace_id yuzgakl, blueprint_id mn9su6k."), Some(("yuzgakl".into(), "mn9su6k".into())), "sem crase também");
        assert_eq!(ids_da_linha("Blueprint: workspace_id …, blueprint_id …"), None, "o modelo da linha não vale");
    }

    #[test]
    fn canal_sai_dos_argumentos_do_claude_e_a_flag_vai_depois_do_prompt() {
        let (canal, resto) = separa_canal(&["Execute a ação".into(), "--canal".into()]);
        assert!(canal);
        assert_eq!(resto, ["Execute a ação", "--dangerously-load-development-channels", "plugin:blueprint@inline"]);
        let (canal, resto) = separa_canal(&["--continue".into()]);
        assert!(!canal);
        assert_eq!(resto, ["--continue"], "sem --canal nada muda");
    }

    #[test]
    fn sem_vinculo_nao_forca_o_agente() {
        let args = argumentos(Path::new("plugin"), Path::new("blueprint.exe"), false, &[]);
        assert!(!args.iter().any(|arg| arg == "--agent"));
    }

    #[test]
    fn consulta_so_ativa_o_agente_com_resposta_valida_do_app() {
        assert_eq!(interpretar_status(200, r#"{"vinculado":true}"#), Some(true));
        assert_eq!(
            interpretar_status(200, r#"{"vinculado":false}"#),
            Some(false)
        );
        assert_eq!(interpretar_status(403, r#"{"vinculado":true}"#), None);
        assert_eq!(interpretar_status(200, "{}"), None);
    }

    #[test]
    fn manda_flags_para_um_claude_falso_encontrado_no_path() {
        let dir = pasta_temporaria();
        fs::create_dir_all(&dir).unwrap();
        let fake = if cfg!(windows) {
            dir.join("claude.cmd")
        } else {
            dir.join("claude")
        };
        #[cfg(windows)]
        fs::write(
            &fake,
            "@echo off\r\n>\"%BLUEPRINT_FAKE_OUTPUT%\" echo %*\r\n",
        )
        .unwrap();
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::write(
                &fake,
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$BLUEPRINT_FAKE_OUTPUT\"\n",
            )
            .unwrap();
            fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let cli = crate::registrar::procurar_em("claude", &[dir.clone()]).unwrap();
        let plugin = dir.join("app data/plugins/blueprint");
        let exe = dir.join("blueprint.exe");
        let args = argumentos(&plugin, &exe, true, &["--continue".into()]);
        let captura = dir.join("args.txt");
        let status = Command::new(cli)
            .args(args)
            .env("BLUEPRINT_FAKE_OUTPUT", &captura)
            .status()
            .unwrap();
        assert!(status.success());
        let enviados = fs::read_to_string(captura).unwrap();
        for parte in [
            "--plugin-dir",
            "blueprint",
            "--agent",
            AGENTE,
            "--name",
            "The Blueprint",
            "--settings",
            "statusLine",
            "--continue",
        ] {
            assert!(enviados.contains(parte), "faltou {parte}: {enviados}");
        }
        assert_eq!(enviados.matches("--plugin-dir").count(), 2);
        assert!(enviados.contains("blueprint-terminal"));
        fs::remove_dir_all(dir).unwrap();
    }
}
