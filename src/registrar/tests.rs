use super::*;
use std::fs;

#[test]
fn comando_do_claude_vale_em_qualquer_pasta_e_aponta_para_o_exe() {
    let args = montar(AGENTES[0].registrar, Path::new("C:\\Apps\\Blueprint\\blueprint.exe"));
    assert_eq!(args, ["mcp", "add", "--scope", "user", "blueprint", "--", "C:\\Apps\\Blueprint\\blueprint.exe", "--mcp"]);
}

#[test]
fn comando_do_codex_e_a_remocao_usam_so_o_nome_blueprint() {
    let exe = Path::new("/x/blueprint");
    assert_eq!(montar(AGENTES[1].registrar, exe), ["mcp", "add", "blueprint", "--", "/x/blueprint", "--mcp"]);
    assert_eq!(montar(AGENTES[1].remover, exe), ["mcp", "remove", "blueprint"]);
    assert_eq!(montar(AGENTES[0].remover, exe), ["mcp", "remove", "--scope", "user", "blueprint"]);
}

#[test]
fn grava_o_plugin_completo_com_o_exe_no_mcp_e_sem_agente_padrao() {
    let pasta = std::env::temp_dir().join(format!("blueprint-plugin-{}", std::process::id()));
    plugin::gravar(&pasta, Path::new("C:\\Apps\\Blueprint\\blueprint.exe")).unwrap();
    let raiz = pasta.join("plugins/blueprint");
    for f in [".claude-plugin/plugin.json", "agents/blueprint.md", "skills/blueprint-planejamento/SKILL.md", "skills/blueprint-planejamento/topicos/executar.md"] {
        assert!(raiz.join(f).is_file(), "faltou {f}");
    }
    let mcp: serde_json::Value = serde_json::from_str(&fs::read_to_string(raiz.join(".mcp.json")).unwrap()).unwrap();
    assert_eq!(mcp["mcpServers"]["blueprint"]["command"], "C:\\Apps\\Blueprint\\blueprint.exe");
    assert_eq!(mcp["mcpServers"]["blueprint"]["args"][0], "--mcp");
    let h: serde_json::Value = serde_json::from_str(&fs::read_to_string(raiz.join("hooks/hooks.json")).unwrap()).unwrap();
    let sessao = &h["hooks"]["SessionStart"][0]["hooks"][0];
    assert_eq!((sessao["command"].as_str(), sessao["args"][1].as_str()), (Some("C:\\Apps\\Blueprint\\blueprint.exe"), Some("sessao")));
    assert_eq!(h["hooks"]["PostToolUse"][0]["hooks"][0]["type"], "http");
    for evento in ["SubagentStart", "SubagentStop"] {
        assert_eq!(h["hooks"][evento][0]["hooks"][0]["type"], "http", "{evento} avisa o app por http");
        assert!(h["hooks"][evento][0]["hooks"][0]["url"].as_str().unwrap().ends_with("/api/mcp/hook/subagente"));
    }
    assert_eq!(h["hooks"]["Notification"][0]["hooks"][0]["args"][1], "notificacao");
    let terminal = pasta.join("plugins/blueprint-terminal");
    for nome in [".claude-plugin/plugin.json", "hooks/hooks.json", "hooks/register.tsx", "types/index.d.ts", "assets/marca.png", "assets/marca.mjs", "assets/marca-32x16.cells", "assets/marca-16x8.cells"] {
        assert!(terminal.join(nome).is_file(), "faltou {nome}");
    }
    let skill = fs::read_to_string(raiz.join("skills/status/SKILL.md")).unwrap();
    assert!(skill.contains("name: status") && skill.contains("disable-model-invocation: true"));
    assert!(skill.contains("!`\"C:/Apps/Blueprint/blueprint.exe\" hook status`"), "{skill}");
    let manifesto: serde_json::Value = serde_json::from_str(&fs::read_to_string(raiz.join(".claude-plugin/plugin.json")).unwrap()).unwrap();
    assert!(manifesto.get("settings").is_none(), "settings.agent faria toda sessão do Claude usar o agente");
    // idempotente: gravar de novo não reescreve o que não mudou
    let antes = fs::metadata(raiz.join(".mcp.json")).unwrap().modified().unwrap();
    plugin::gravar(&pasta, Path::new("C:\\Apps\\Blueprint\\blueprint.exe")).unwrap();
    assert_eq!(fs::metadata(raiz.join(".mcp.json")).unwrap().modified().unwrap(), antes);
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn procura_a_cli_na_ordem_das_pastas() {
    let base = std::env::temp_dir().join(format!("blueprint-registrar-{}", std::process::id()));
    let (a, b) = (base.join("a"), base.join("b"));
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    let nome = if cfg!(windows) { "agente.cmd" } else { "agente" };
    fs::write(b.join(nome), "").unwrap();
    assert_eq!(procurar_em("agente", &[a.clone(), b.clone()]), Some(b.join(nome)));
    assert_eq!(procurar_em("outro", &[a, b]), None);
    fs::remove_dir_all(base).unwrap();
}

#[test]
fn plugin_ja_ligado_nao_e_falha_do_registrar() {
    use super::plugin::ja_ligado;
    assert!(ja_ligado("✘ Failed to enable plugin \"blueprint-terminal@blueprint-local\": Plugin \"blueprint-terminal@blueprint-local\" is already enabled at user scope"));
    assert!(!ja_ligado("✘ Failed to enable plugin: plugin not found") && !ja_ligado(""));
}
