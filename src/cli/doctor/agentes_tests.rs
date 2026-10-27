use super::*;
use crate::cli::doctor::tests::temp;

fn gravar(arquivo: &Path, texto: &str) {
    std::fs::create_dir_all(arquivo.parent().unwrap()).unwrap();
    std::fs::write(arquivo, texto).unwrap();
}

#[test]
fn plugin_instalado_e_ligado_e_ok() {
    let p = temp("claude-ok");
    gravar(&p.join("plugins/installed_plugins.json"), r#"{"version":2,"plugins":{"blueprint@blueprint-local":[{"scope":"user"}]}}"#);
    gravar(&p.join("settings.json"), r#"{"enabledPlugins":{"blueprint@blueprint-local":true}}"#);
    assert_eq!(claude(&p, &p.join(".claude.json")).estado, Estado::Ok);
}

#[test]
fn plugin_ausente_manda_registrar() {
    let p = temp("claude-ausente");
    let item = claude(&p, &p.join(".claude.json"));
    assert_eq!(item.estado, Estado::Falha);
    assert!(item.texto.contains("blueprint registrar"), "{}", item.texto);
}

#[test]
fn plugin_desligado_e_so_o_mcp_sao_diferentes() {
    let p = temp("claude-desligado");
    gravar(&p.join("plugins/installed_plugins.json"), r#"{"plugins":{"blueprint@blueprint-local":[]}}"#);
    gravar(&p.join("settings.json"), r#"{"enabledPlugins":{"blueprint@blueprint-local":false}}"#);
    let item = claude(&p, &p.join(".claude.json"));
    assert_eq!(item.estado, Estado::Falha);
    assert!(item.texto.contains("claude plugin enable"), "{}", item.texto);

    let q = temp("claude-so-mcp");
    gravar(&q.join(".claude.json"), r#"{"mcpServers":{"blueprint":{"command":"blueprint.exe","args":["--mcp"]}}}"#);
    assert_eq!(claude(&q, &q.join(".claude.json")).estado, Estado::Aviso);
}

#[test]
fn sem_claude_instalado_e_aviso() {
    let p = temp("claude-nenhum").join("nao-existe");
    assert_eq!(claude(&p, &p.join(".claude.json")).estado, Estado::Aviso);
}

#[test]
fn codex_com_sem_mcp_e_sem_codex() {
    let p = temp("codex");
    assert_eq!(codex(&p).estado, Estado::Aviso);
    gravar(&p.join("config.toml"), "[mcp_servers.outro]\ncommand = \"x\"\n");
    assert_eq!(codex(&p).estado, Estado::Falha);
    gravar(&p.join("config.toml"), "[mcp_servers.outro]\ncommand = \"x\"\n\n[mcp_servers.blueprint]\ncommand = 'blueprint.exe'\nargs = [\"--mcp\"]\n");
    assert_eq!(codex(&p).estado, Estado::Ok);
}
