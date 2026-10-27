//! Doctor: o plugin no Claude Code e o MCP no Codex, lidos dos arquivos de configuração deles (sem rodar as CLIs, que podem gravar).
use super::{Estado, Item};
use crate::registrar::plugin::MARKETPLACE;
use serde_json::Value;
use std::path::Path;

fn json(arquivo: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(arquivo).ok()?).ok()
}

/// `config`: a pasta do Claude Code (CLAUDE_CONFIG_DIR ou ~/.claude); `claude_json`: onde fica o MCP registrado à parte (`claude mcp add --scope user`).
pub fn claude(config: &Path, claude_json: &Path) -> Item {
    let nome = crate::tr!("Claude Code plugin", "Plugin do Claude Code");
    let plugin = format!("blueprint@{MARKETPLACE}");
    let instalado = json(&config.join("plugins").join("installed_plugins.json")).is_some_and(|v| v["plugins"].get(&plugin).is_some());
    // o Claude grava `true` ao instalar; só `false` explícito é desligado
    let desligado = json(&config.join("settings.json")).is_some_and(|v| v["enabledPlugins"][&plugin] == Value::Bool(false));
    let (estado, texto) = match (instalado, desligado) {
        (true, false) => (Estado::Ok, crate::tr!("{plugin} is installed and enabled", "{plugin} está instalado e ligado")),
        (true, true) => (Estado::Falha, crate::tr!("{plugin} is installed but disabled: run claude plugin enable {plugin}", "{plugin} está instalado, mas desligado: rode claude plugin enable {plugin}")),
        (false, _) if json(claude_json).is_some_and(|v| v["mcpServers"]["blueprint"].is_object()) => (Estado::Aviso, crate::tr!(
            "only the blueprint MCP is registered, without the plugin (no hooks: the agent does not learn the folder's blueprint). Run blueprint registrar.",
            "só o MCP blueprint está registrado, sem o plugin (sem os hooks: o agente não fica sabendo o blueprint da pasta). Rode blueprint registrar."
        )),
        (false, _) if !config.is_dir() => (Estado::Aviso, crate::tr!(
            "Claude Code was not found ({} does not exist). If you use it, install it and run blueprint registrar.",
            "o Claude Code não foi encontrado ({} não existe). Se você usa, instale e rode blueprint registrar.",
            config.display()
        )),
        (false, _) => (Estado::Falha, crate::tr!("{plugin} is not installed: run blueprint registrar", "{plugin} não está instalado: rode blueprint registrar")),
    };
    Item::novo(nome, estado, texto)
}

/// `pasta`: CODEX_HOME ou ~/.codex. O Codex não tem plugin: basta o `[mcp_servers.blueprint]` no config.toml.
pub fn codex(pasta: &Path) -> Item {
    let nome = crate::tr!("Codex MCP", "MCP do Codex");
    let Ok(config) = std::fs::read_to_string(pasta.join("config.toml")) else {
        return Item::novo(nome, Estado::Aviso, crate::tr!(
            "Codex was not found ({} has no config.toml). If you use it, run blueprint registrar after installing it.",
            "o Codex não foi encontrado ({} não tem config.toml). Se você usa, rode blueprint registrar depois de instalar.",
            pasta.display()
        ));
    };
    // shortcut: confere a seção pelo texto, sem ler TOML; uma tabela inline (`mcp_servers = { blueprint = … }`) passaria por ausente
    let registrado = config.lines().map(str::trim).any(|l| l == "[mcp_servers.blueprint]" || l == "[mcp_servers.\"blueprint\"]");
    if registrado {
        Item::novo(nome, Estado::Ok, crate::tr!("the blueprint MCP is registered in {}", "o MCP blueprint está registrado em {}", pasta.join("config.toml").display()))
    } else {
        Item::novo(nome, Estado::Falha, crate::tr!("Codex has no blueprint MCP: run blueprint registrar", "o Codex não tem o MCP blueprint: rode blueprint registrar"))
    }
}

#[cfg(test)]
#[path = "agentes_tests.rs"]
mod tests;
