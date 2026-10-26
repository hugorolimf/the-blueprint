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

pub(super) fn executar(extras: &[String]) -> i32 {
    let (canal, extras) = separa_canal(extras);
    let extras = extras.as_slice();
    let Some(cli) = crate::registrar::procurar("claude") else {
        eprintln!("erro: Claude Code não encontrado no PATH");
        return 1;
    };
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
    let pasta = crate::http::config::pasta_do_app().join("claude-plugin");
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
    cwd.ancestors().any(|pasta| {
        ["CLAUDE.md", "AGENTS.md"].iter().any(|nome| {
            fs::read_to_string(pasta.join(nome))
                .is_ok_and(|texto| texto.lines().any(linha_blueprint_valida))
        })
    })
}

fn linha_blueprint_valida(linha: &str) -> bool {
    let Some(resto) = linha.strip_prefix("Blueprint: workspace_id `") else {
        return false;
    };
    let Some((workspace, resto)) = resto.split_once("`, blueprint_id `") else {
        return false;
    };
    let Some((blueprint, _)) = resto.split_once('`') else {
        return false;
    };
    [workspace, blueprint].iter().all(|id| {
        !id.is_empty()
            && id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
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
    fn exige_ids_reais_na_linha_de_vinculo() {
        assert!(linha_blueprint_valida(
            "Blueprint: workspace_id `sm`, blueprint_id `ia-sm` (MCP `blueprint`)."
        ));
        assert!(!linha_blueprint_valida("Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`)."));
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
