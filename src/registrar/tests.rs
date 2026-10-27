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
    assert!(h["hooks"]["SessionEnd"][0]["hooks"][0]["url"].as_str().unwrap().ends_with("/api/mcp/hook/fim"), "o SessionEnd tira a sessão da lista");
    assert_eq!(h["hooks"]["Notification"][0]["hooks"][0]["args"][1], "notificacao");
    assert_eq!(h["hooks"]["Stop"][0]["hooks"][0]["type"], "http", "o Stop avisa o app por http");
    assert!(h["hooks"]["Stop"][0]["hooks"][0]["url"].as_str().unwrap().ends_with("/api/mcp/hook/parada"));
    for evento in ["PermissionRequest", "PermissionDenied", "StopFailure", "PreCompact", "PostCompact"] {
        let gancho = &h["hooks"][evento][0]["hooks"][0];
        assert_eq!(gancho["type"], "http", "{evento} avisa o app por http");
        assert!(gancho["url"].as_str().unwrap().ends_with("/api/mcp/hook/estado"), "{evento}");
        assert_eq!(gancho["timeout"], 5, "{evento}");
        assert!(h["hooks"][evento][0].get("matcher").is_none(), "{evento} vale para qualquer ferramenta e causa");
    }
    assert!(h["hooks"].get("PostToolUseFailure").is_none(), "falha de ferramenta é rotina: sem hook");
    for agente in ["blueprint", "blueprint-explorer", "blueprint-builder", "blueprint-reviewer"] {
        assert!(raiz.join(format!("agents/{agente}.md")).is_file(), "faltou o agente {agente}");
    }
    let terminal = pasta.join("plugins/blueprint-terminal");
    for nome in [".claude-plugin/plugin.json", "hooks/hooks.json", "hooks/register.tsx", "types/index.d.ts", "assets/marca.png", "assets/marca.mjs", "assets/marca-32x16.cells", "assets/marca-16x8.cells"] {
        assert!(terminal.join(nome).is_file(), "faltou {nome}");
    }
    let skill = fs::read_to_string(raiz.join("skills/status/SKILL.md")).unwrap();
    assert!(skill.contains("name: status") && skill.contains("disable-model-invocation: true"));
    assert!(skill.contains("!`\"C:/Apps/Blueprint/blueprint.exe\" hook status`"), "{skill}");
    let apresentar = fs::read_to_string(raiz.join("skills/apresentar/SKILL.md")).unwrap();
    assert!(apresentar.contains("name: apresentar") && apresentar.contains("disable-model-invocation: true"), "só o usuário chama /blueprint:apresentar");
    for trecho in ["guia_de_planejamento", "topico \"apresentacao_ia\"", "ler_blueprint", "anexar_apresentacao", "$ARGUMENTS", "epic only if the user asks for it by name"] {
        assert!(apresentar.contains(trecho), "falta no comando: {trecho}");
    }
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

#[test]
fn exe_de_target_nao_e_o_que_o_agente_abre() {
    // barras normais: `\` só separa pastas no Windows e o CI também roda no Ubuntu
    assert!(dentro_de_target(Path::new("C:/x/Blueprint/target/debug/blueprint.exe")));
    assert!(dentro_de_target(Path::new("C:/x/Blueprint/target/release/blueprint.exe")));
    assert!(!dentro_de_target(Path::new("C:/Users/x/AppData/Local/Programs/Blueprint/blueprint.exe")));
}

#[test]
fn o_plugin_leva_o_settings_so_com_o_subagent_status_line_apontando_para_o_exe() {
    let pasta = std::env::temp_dir().join(format!("blueprint-plugin-subagentes-{}", std::process::id()));
    plugin::gravar(&pasta, Path::new(r"C:\Meus Apps\blueprint.exe")).unwrap();
    let caminho = pasta.join("plugins/blueprint/settings.json");
    let s: serde_json::Value = serde_json::from_str(&fs::read_to_string(&caminho).unwrap()).unwrap();
    assert_eq!(s.as_object().unwrap().len(), 1, "o manual só aceita agent e subagentStatusLine; agent mudaria toda sessão");
    assert_eq!(s["subagentStatusLine"], serde_json::json!({"type": "command", "command": "\"C:/Meus Apps/blueprint.exe\" hook subagentes"}));
    let antes = fs::metadata(&caminho).unwrap().modified().unwrap();
    plugin::gravar(&pasta, Path::new(r"C:\Meus Apps\blueprint.exe")).unwrap();
    assert_eq!(fs::metadata(&caminho).unwrap().modified().unwrap(), antes, "idempotente");
    fs::remove_dir_all(pasta).unwrap();
}

#[test]
fn os_agentes_de_papel_usam_so_o_frontmatter_documentado_e_cada_um_o_seu() {
    let campos = |nome: &str| -> Vec<(String, String)> {
        let texto = fs::read_to_string(format!("{}/claude-plugin/plugins/blueprint/agents/{nome}.md", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let cabecalho = texto.strip_prefix("---\n").and_then(|t| t.split_once("\n---\n")).expect("frontmatter").0.to_string();
        cabecalho.lines().filter_map(|l| l.split_once(": ")).map(|(k, v)| (k.to_string(), v.to_string())).collect()
    };
    let valor = |c: &[(String, String)], k: &str| c.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    for (nome, cor, modelo) in [("blueprint-explorer", "cyan", Some("haiku")), ("blueprint-builder", "orange", None), ("blueprint-reviewer", "green", None)] {
        let c = campos(nome);
        assert_eq!(valor(&c, "name").as_deref(), Some(nome));
        assert_eq!(valor(&c, "color").as_deref(), Some(cor), "{nome}");
        assert_eq!(valor(&c, "model").as_deref(), modelo, "{nome}");
        assert!(valor(&c, "description").is_some_and(|d| !d.is_empty()), "{nome}");
        // o que o Claude Code ignora em agente de plugin, ou muda a sessão do usuário de forma surpreendente, não entra
        for proibido in ["hooks", "permissionMode", "mcpServers", "background", "isolation", "initialPrompt"] {
            assert!(valor(&c, proibido).is_none(), "{nome} não usa {proibido}");
        }
    }
    let leitura = "mcp__plugin_blueprint_blueprint__ler_blueprint";
    let explorador = valor(&campos("blueprint-explorer"), "tools").unwrap();
    assert!(explorador.contains("Read") && explorador.contains("Grep") && explorador.contains("Glob") && explorador.contains(leitura));
    for escrita in ["Edit", "Write", "Bash", "propor_blueprint", "marcar_", "registrar_"] {
        assert!(!explorador.contains(escrita), "o explorador só lê: {escrita}");
    }
    let revisor = valor(&campos("blueprint-reviewer"), "tools").unwrap();
    assert!(revisor.contains("Bash") && revisor.contains(leitura) && !revisor.contains("Edit") && !revisor.contains("Write"));
    assert!(valor(&campos("blueprint-builder"), "tools").is_none(), "o construtor usa as ferramentas padrão");
}
