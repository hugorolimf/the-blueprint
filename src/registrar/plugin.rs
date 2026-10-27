//! O plugin do Claude Code (skill, agente e MCP) gravado numa pasta que serve de marketplace local e instalado pelo `claude plugin`.
//! Os arquivos fixos vêm de `claude-plugin/` e a skill vem de `skills/` (a mesma que o guia do MCP embute); só o `.mcp.json` é gerado,
//! porque leva o caminho deste binário. Não define `settings.agent` de propósito: isso faria TODA sessão do Claude usar o agente
//! `blueprint`; quem quer o agente pede com `--agent` (o lançador `blueprint claude` faz isso).
use super::{rodar, NOME};
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;

pub const MARKETPLACE: &str = "blueprint-local";
const PLUGIN: &str = "blueprint@blueprint-local";
const PLUGIN_TERMINAL: &str = "blueprint-terminal@blueprint-local";

const ARQUIVOS: &[(&str, &str)] = &[
    (".claude-plugin/marketplace.json", include_str!("../../claude-plugin/.claude-plugin/marketplace.json")),
    ("plugins/blueprint/.claude-plugin/plugin.json", include_str!("../../claude-plugin/plugins/blueprint/.claude-plugin/plugin.json")),
    ("plugins/blueprint/agents/blueprint.md", include_str!("../../claude-plugin/plugins/blueprint/agents/blueprint.md")),
    ("plugins/blueprint/agents/blueprint-explorer.md", include_str!("../../claude-plugin/plugins/blueprint/agents/blueprint-explorer.md")),
    ("plugins/blueprint/agents/blueprint-builder.md", include_str!("../../claude-plugin/plugins/blueprint/agents/blueprint-builder.md")),
    ("plugins/blueprint/agents/blueprint-reviewer.md", include_str!("../../claude-plugin/plugins/blueprint/agents/blueprint-reviewer.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/SKILL.md", include_str!("../../skills/blueprint-planejamento/SKILL.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/planejar.md", include_str!("../../skills/blueprint-planejamento/topicos/planejar.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/editar.md", include_str!("../../skills/blueprint-planejamento/topicos/editar.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/executar.md", include_str!("../../skills/blueprint-planejamento/topicos/executar.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/prototipos.md", include_str!("../../skills/blueprint-planejamento/topicos/prototipos.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/importar.md", include_str!("../../skills/blueprint-planejamento/topicos/importar.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/apresentar.md", include_str!("../../skills/blueprint-planejamento/topicos/apresentar.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/apresentacao_ia.md", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_ia.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/apresentacao_conteudo.md", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_conteudo.md")),
    ("plugins/blueprint/skills/blueprint-planejamento/topicos/apresentacao_figuras.md", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_figuras.md")),
    ("plugins/blueprint-terminal/.claude-plugin/plugin.json", include_str!("../../claude-plugin/plugins/blueprint-terminal/.claude-plugin/plugin.json")),
    ("plugins/blueprint-terminal/hooks/hooks.json", include_str!("../../claude-plugin/plugins/blueprint-terminal/hooks/hooks.json")),
    ("plugins/blueprint-terminal/hooks/register.tsx", include_str!("../../claude-plugin/plugins/blueprint-terminal/hooks/register.tsx")),
    ("plugins/blueprint-terminal/types/index.d.ts", include_str!("../../claude-plugin/plugins/blueprint-terminal/types/index.d.ts")),
    ("plugins/blueprint-terminal/assets/marca.mjs", include_str!("../../claude-plugin/plugins/blueprint-terminal/assets/marca.mjs")),
];

fn gravar_se_mudou(caminho: &Path, conteudo: &str) -> io::Result<()> {
    if fs::read_to_string(caminho).is_ok_and(|atual| atual == conteudo) {
        return Ok(());
    }
    if let Some(pai) = caminho.parent() {
        fs::create_dir_all(pai)?;
    }
    fs::write(caminho, conteudo)
}

/// Grava o marketplace com o plugin em `pasta`. Idempotente: só reescreve o que mudou.
pub fn gravar(pasta: &Path, exe: &Path) -> io::Result<()> {
    for (relativo, conteudo) in ARQUIVOS {
        gravar_se_mudou(&pasta.join(relativo), conteudo)?;
    }
    for (nome, bytes) in [
        ("marca.png", include_bytes!("../../claude-plugin/plugins/blueprint-terminal/assets/marca.png").as_slice()),
        ("marca-32x16.cells", include_bytes!("../../claude-plugin/plugins/blueprint-terminal/assets/marca-32x16.cells").as_slice()),
        ("marca-16x8.cells", include_bytes!("../../claude-plugin/plugins/blueprint-terminal/assets/marca-16x8.cells").as_slice()),
    ] {
        let caminho = pasta.join("plugins/blueprint-terminal/assets").join(nome);
        if !fs::read(&caminho).is_ok_and(|atual| atual == bytes) { fs::write(caminho, bytes)?; }
    }
    super::subagentes::gravar(pasta, exe)?; // settings.json do plugin: o subagentStatusLine
    let exe = exe.display().to_string();
    let mcp = json!({ "mcpServers": { NOME: { "command": exe, "args": ["--mcp"] } } });
    gravar_se_mudou(&pasta.join("plugins/blueprint/.mcp.json"), &(serde_json::to_string_pretty(&mcp).expect("json") + "\n"))?;
    gravar_se_mudou(&pasta.join("plugins/blueprint/hooks/hooks.json"), &(serde_json::to_string_pretty(&hooks(&exe)).expect("json") + "\n"))?;
    let monitor = json!([{ "name": "blueprint", "command": format!("\"{}\" watch", exe.replace('\\', "/")), "description": "Avisos do blueprint desta pasta", "when": "always" }]);
    gravar_se_mudou(&pasta.join("plugins/blueprint/monitors/monitors.json"), &(serde_json::to_string_pretty(&monitor).expect("json") + "\n"))?;
    gravar_se_mudou(&pasta.join("plugins/blueprint/skills/status/SKILL.md"), &skill_status(&exe))?;
    gravar_se_mudou(&pasta.join("plugins/blueprint/skills/apresentar/SKILL.md"), SKILL_APRESENTAR)
}

/// `/blueprint:status`: o comando do usuário. O estado vem pronto do app por `!`comando`` (o Claude Code roda o comando antes de o modelo ler),
/// então não depende de o modelo achar os ids nem gasta ferramenta. Plugins sempre levam o prefixo do nome: o comando é `/blueprint:status`.
fn skill_status(exe: &str) -> String {
    let exe = exe.replace('\\', "/"); // o shell do hook é o Git Bash: barras comuns, sem escape
    format!(
        "---\nname: status\ndescription: Mostra o estado do blueprint desta pasta: a ação atual, os planos e o próximo checkpoint. Só o usuário chama, com /blueprint:status.\ndisable-model-invocation: true\nallowed-tools: Bash(\"{exe}\" hook status)\n---\n\nMostre ao usuário exatamente o texto abaixo, sem acrescentar nada, sem comentar e sem chamar ferramentas:\n\n!`\"{exe}\" hook status`\n"
    )
}

/// `/blueprint:apresentar`: o comando do usuário que abre a conversa de apresentação (contrato em docs/interno/apresentacao-ia.md). O roteiro em si vive no tópico
/// `apresentar` do guia (um só lugar para o agente e para este comando); aqui só se manda segui-lo. Texto para o AGENTE, então em inglês; ele responde no idioma do usuário.
const SKILL_APRESENTAR: &str = "---\nname: apresentar\ndescription: Conduz a conversa que gera a apresentação do blueprint desta pasta (público e nível, com figuras vivas do anatomy). Só o usuário chama, com /blueprint:apresentar.\ndisable-model-invocation: true\n---\n\nPresent the blueprint of this folder, talking with the user in their language.\n\n1. Call guia_de_planejamento with topico \"apresentacao_ia\" and follow its flow; read \"apresentacao_conteudo\" and \"apresentacao_figuras\" when you write the content and the figures.\n2. Call ler_blueprint (workspace_id and blueprint_id come from the session context or the \"Blueprint:\" line of CLAUDE.md or AGENTS.md; without them, listar_blueprints) to read the structure and the `apresentacoes` that already exist.\n3. Arguments typed by the user after the command (they may be empty, and are the audience and the level): $ARGUMENTS\n4. Ask in ONE message only what is still missing: the audience (client, a project plan without the technical development, or internal), the level (quick, standard or complete; epic only if the user asks for it by name) and, for the client audience, what the client must provide or decide. Say the time estimate of the level before starting.\n5. Generate the content and the figures and attach them with anexar_apresentacao. Tell the user where to open it: the Present screen of the blueprint in the app.\n\nNever export or print for the user: that is their click in the app.\n";

/// Os hooks do plugin. Sessão: hook de comando (o SessionStart roda antes de o MCP conectar, então `mcp_tool` não serve), em forma de execução
/// direta (`args`, sem shell, então o caminho do .exe com espaços não quebra). Edição: hook `http` direto no app, sem abrir processo a cada edição.
/// Estado do agente: PermissionRequest, PermissionDenied, StopFailure, PreCompact e PostCompact vão todos a `hook/estado` (nomes conferidos em
/// https://code.claude.com/docs/en/hooks.md); a rota responde sempre `{}`, então nenhum deles decide permissão nem segura o Claude Code.
/// `PostToolUseFailure` fica de fora: ferramenta que falha é rotina e só faria ruído.
/// shortcut: a porta é a 3000 (a padrão do app); com o app em outra porta o hook falha em silêncio, sem atrapalhar o agente.
fn hooks(exe: &str) -> serde_json::Value {
    let estado = || json!([{ "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/estado", "timeout": 5 }] }]);
    json!({ "hooks": {
        "PermissionRequest": estado(),
        "PermissionDenied": estado(),
        "StopFailure": estado(),
        "PreCompact": estado(),
        "PostCompact": estado(),
        "SessionStart": [{ "matcher": "startup|resume|clear", "hooks": [{ "type": "command", "command": exe, "args": ["hook", "sessao"], "timeout": 5 }] }],
        "SessionEnd": [{ "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/fim", "timeout": 5 }] }],
        "PostToolUse": [{ "matcher": "Edit|Write", "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/edicao", "timeout": 5 }] }],
        "SubagentStart": [{ "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/subagente", "timeout": 5 }] }],
        "SubagentStop": [{ "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/subagente", "timeout": 5 }] }],
        "Notification": [{ "hooks": [{ "type": "command", "command": exe, "args": ["hook", "notificacao"], "timeout": 5 }] }],
        "Stop": [{ "hooks": [{ "type": "http", "url": "http://127.0.0.1:3000/api/mcp/hook/parada", "timeout": 5 }] }],
    } })
}

fn args(partes: &[&str]) -> Vec<String> {
    partes.iter().map(|p| (*p).to_string()).collect()
}

/// Instala o plugin no Claude Code (escopo user) e tira o `mcp add` antigo, que o plugin passa a fornecer (senão o servidor apareceria duas vezes).
/// Se algo falhar, o chamador volta ao `mcp add`: a pessoa nunca fica sem o MCP.
pub fn instalar(cli: &Path, pasta: &Path) -> Result<(), String> {
    // idempotente: tira a versão anterior (falha se não existia, e tudo bem) e instala de novo
    let _ = rodar(cli, &args(&["plugin", "marketplace", "remove", MARKETPLACE]));
    rodar(cli, &args(&["plugin", "marketplace", "add", &pasta.display().to_string()]))?;
    rodar(cli, &args(&["plugin", "install", PLUGIN, "--scope", "user"]))?;
    rodar(cli, &args(&["plugin", "install", PLUGIN_TERMINAL, "--scope", "user"]))?;
    // versões antigas do Claude Code deixam o mod de terminal desativado depois do install (sem isto o `claude` puro não o carrega); as novas já o ligam e
    // respondem "already enabled" com erro: isso é sucesso, não falha
    if let Err(e) = rodar(cli, &args(&["plugin", "enable", PLUGIN_TERMINAL, "--scope", "user"])) {
        if !ja_ligado(&e) {
            return Err(e);
        }
    }
    let _ = rodar(cli, &args(&["mcp", "remove", "--scope", "user", NOME]));
    Ok(())
}

/// A mensagem do `claude plugin enable` quando o plugin já estava ligado (não é erro de verdade).
pub(super) fn ja_ligado(erro: &str) -> bool {
    erro.to_lowercase().contains("already enabled")
}

/// Desinstala o plugin (e o marketplace, que leva o plugin junto). Quem sobrou do `mcp add` antigo é removido pelo chamador.
pub fn remover(cli: &Path) {
    let _ = rodar(cli, &args(&["plugin", "uninstall", PLUGIN, "--scope", "user"]));
    let _ = rodar(cli, &args(&["plugin", "uninstall", PLUGIN_TERMINAL, "--scope", "user"]));
    let _ = rodar(cli, &args(&["plugin", "marketplace", "remove", MARKETPLACE]));
}
