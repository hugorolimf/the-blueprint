//! `blueprint conectar`: endereço e chave validados, conexão e plugin gravados numa pasta temporária, CLIs falsas (anotam os argumentos num
//! arquivo) no lugar do `claude` e do `codex`, trocar o endereço, cair para o `mcp add` e remover.
use super::conectar::*;
use std::fs;
use std::path::{Path, PathBuf};

const CHAVE: &str = "bpk_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ";

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-conectar-{nome}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

/// Uma CLI falsa que anota cada chamada em `log.txt`; com `falha_plugin`, todo `<cli> plugin …` sai com erro.
fn cli_falsa(dir: &Path, nome: &str, falha_plugin: bool) -> PathBuf {
    let log = dir.join("log.txt");
    #[cfg(windows)]
    let (caminho, texto) = (dir.join(format!("{nome}.cmd")), format!("@echo off\r\necho {nome} %*>> \"{}\"\r\n{}", log.display(), if falha_plugin { "if \"%1\"==\"plugin\" exit /b 1\r\n" } else { "" }));
    #[cfg(not(windows))]
    let (caminho, texto) = (dir.join(nome), format!("#!/bin/sh\necho {nome} \"$@\" >> '{}'\n{}", log.display(), if falha_plugin { "[ \"$1\" = plugin ] && exit 1\nexit 0\n" } else { "" }));
    fs::write(&caminho, texto).unwrap();
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&caminho, fs::Permissions::from_mode(0o755)).unwrap();
    }
    caminho
}

#[test]
fn so_aceita_endereco_https_sem_caminho_e_http_local_so_em_teste() {
    assert_eq!(validar_endereco("https://abc-def.trycloudflare.com/", false).unwrap(), "https://abc-def.trycloudflare.com");
    assert_eq!(validar_endereco("https://equipe.exemplo.com:8443", false).unwrap(), "https://equipe.exemplo.com:8443");
    for ruim in ["http://abc.trycloudflare.com", "https://abc.com/caminho", "https://", "https://a b.com", "https://u@a.com", "ftp://a.com", "abc.trycloudflare.com", "http://127.0.0.1:3000"] {
        assert!(validar_endereco(ruim, false).is_err(), "{ruim}");
    }
    assert_eq!(validar_endereco("http://127.0.0.1:4100", true).unwrap(), "http://127.0.0.1:4100");
    assert!(validar_endereco("http://10.0.0.2:4100", true).is_err(), "http só em loopback");
}

#[test]
fn a_chave_tem_a_forma_da_tela() {
    assert!(validar_chave(CHAVE).is_ok());
    for ruim in ["", "bpk_curta", &CHAVE[4..], &format!("{CHAVE}x"), "bpk_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNO\""] {
        assert!(validar_chave(ruim).is_err(), "{ruim}");
    }
}

#[test]
fn conecta_troca_o_endereco_e_remove_sem_a_chave_em_argumento_nem_no_plugin() {
    let (app, bin) = (pasta("app"), pasta("bin"));
    let (claude, codex) = (cli_falsa(&bin, "claude", false), cli_falsa(&bin, "codex", false));
    let achar = |nome: &str| Some(if nome == "claude" { claude.clone() } else { codex.clone() });
    let exe = Path::new("C:/Apps/Blueprint/blueprint.exe");
    let (texto, codigo) = conectar_em(&app, exe, "https://um.trycloudflare.com", CHAVE, achar);
    assert_eq!(codigo, 0, "{texto}");
    let conexao: serde_json::Value = serde_json::from_str(&fs::read_to_string(app.join("conexao-equipe.json")).unwrap()).unwrap();
    assert_eq!((conexao["url"].as_str(), conexao["chave"].as_str()), (Some("https://um.trycloudflare.com"), Some(CHAVE)));
    let raiz = app.join("claude-plugin-equipe/plugins/blueprint-equipe");
    let mcp: serde_json::Value = serde_json::from_str(&fs::read_to_string(raiz.join(".mcp.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["blueprint-equipe"]["args"], serde_json::json!(["--mcp", "--equipe"]));
    let hooks: serde_json::Value = serde_json::from_str(&fs::read_to_string(raiz.join("hooks/hooks.json")).unwrap()).unwrap();
    for (evento, nome) in [("SessionStart", "sessao"), ("SessionEnd", "fim"), ("PostToolUse", "edicao"), ("SubagentStart", "subagente"), ("SubagentStop", "subagente"), ("Stop", "parada")] {
        let h = &hooks["hooks"][evento][0]["hooks"][0];
        assert_eq!((h["type"].as_str(), h["args"].clone()), (Some("command"), serde_json::json!(["hook", nome, "--equipe"])), "{evento}");
    }
    assert_eq!(hooks["hooks"]["PostToolUse"][0]["matcher"], "Edit|Write");
    let log = fs::read_to_string(bin.join("log.txt")).unwrap();
    assert!(log.contains("claude plugin install blueprint-equipe@blueprint-equipe-local --scope user"), "{log}");
    assert!(log.contains("codex mcp add blueprint-equipe -- C:/Apps/Blueprint/blueprint.exe --mcp --equipe"), "{log}");
    assert!(!log.contains(" blueprint-local") && !log.contains("blueprint@blueprint-local"), "não mexe no plugin local: {log}");
    for arquivo in [raiz.join(".mcp.json"), raiz.join("hooks/hooks.json"), raiz.join(".claude-plugin/plugin.json")] {
        assert!(!fs::read_to_string(&arquivo).unwrap().contains("bpk_"), "a chave não vai para o plugin ({})", arquivo.display());
    }
    assert!(!log.contains("bpk_"), "a chave não vai em argumento de processo");
    // o túnel mudou: rodar de novo troca o endereço
    assert_eq!(conectar_em(&app, exe, "https://dois.trycloudflare.com", CHAVE, achar).1, 0);
    assert!(fs::read_to_string(app.join("conexao-equipe.json")).unwrap().contains("https://dois.trycloudflare.com"));
    let (_, codigo) = remover_em(&app, achar);
    assert_eq!(codigo, 0);
    assert!(!app.join("conexao-equipe.json").exists() && !app.join("claude-plugin-equipe").exists(), "a chave sai do disco");
    let log = fs::read_to_string(bin.join("log.txt")).unwrap();
    assert!(log.contains("claude plugin uninstall blueprint-equipe@blueprint-equipe-local --scope user") && log.contains("codex mcp remove blueprint-equipe"), "{log}");
    let _ = (fs::remove_dir_all(app), fs::remove_dir_all(bin));
}

#[test]
fn sem_o_plugin_fica_o_mcp_e_sem_cli_so_avisa() {
    let (app, bin) = (pasta("app2"), pasta("bin2"));
    let claude = cli_falsa(&bin, "claude", true);
    let (texto, codigo) = conectar_em(&app, Path::new("/x/blueprint"), "https://um.trycloudflare.com", CHAVE, |n| (n == "claude").then(|| claude.clone()));
    assert_eq!(codigo, 0, "{texto}");
    assert!(texto.contains("Codex"), "{texto}");
    let log = fs::read_to_string(bin.join("log.txt")).unwrap();
    assert!(log.contains("claude mcp add --scope user blueprint-equipe -- /x/blueprint --mcp --equipe"), "{log}");
    let _ = (fs::remove_dir_all(app), fs::remove_dir_all(bin));
}
