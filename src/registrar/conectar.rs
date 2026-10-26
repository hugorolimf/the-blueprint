//! `blueprint conectar <endereço> <chave>`: liga o Claude Code (plugin `blueprint-equipe`, com MCP e hooks) e o Codex (só o MCP) do
//! convidado a um workspace compartilhado (o túnel do Compartilhar ou um `serve --hospedar`). O plugin fica numa pasta e num marketplace
//! próprios (`claude-plugin-equipe`, `blueprint-equipe-local`), para não brigar com o plugin local `blueprint`, e não muda de um endereço
//! para outro: o endereço e a chave ficam em `conexao-equipe.json` (`mcp::equipe`), que o `--mcp --equipe` e o `hook <evento> --equipe`
//! leem a cada pedido. Rodar de novo com outro endereço só regrava esse arquivo (e reinstala, idempotente); `--remover` desfaz tudo.
use super::{procurar, rodar};
use crate::mcp::equipe;
use serde_json::json;
use std::path::{Path, PathBuf};

pub const NOME: &str = "blueprint-equipe";
const MARKETPLACE: &str = "blueprint-equipe-local";
const PLUGIN: &str = "blueprint-equipe@blueprint-equipe-local";

fn args(partes: &[&str]) -> Vec<String> {
    partes.iter().map(|p| (*p).to_string()).collect()
}

/// A origem do servidor (`https://host[:porta]`, sem caminho). `http://127.0.0.1` ou `http://localhost` só com `http_local` (os testes).
pub fn validar_endereco(endereco: &str, http_local: bool) -> Result<String, String> {
    let e = endereco.trim().trim_end_matches('/');
    let ruim = || crate::tr!("invalid address: use the https:// address of the sharing (for example https://name.trycloudflare.com), without a path", "endereço inválido: use o endereço https:// do compartilhamento (por exemplo https://nome.trycloudflare.com), sem caminho");
    let host = match (e.strip_prefix("https://"), e.strip_prefix("http://")) {
        (Some(h), _) => h,
        (_, Some(h)) if http_local && h.split(':').next().is_some_and(|n| n == "127.0.0.1" || n == "localhost") => h,
        _ => return Err(ruim()),
    };
    let nome = host.rsplit_once(':').filter(|(_, p)| !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit())).map_or(host, |(n, _)| n);
    let valido = !nome.is_empty() && nome.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-');
    if valido { Ok(e.to_string()) } else { Err(ruim()) }
}

/// `bpk_` + 43 caracteres (a chave da tela Minha chave).
pub fn validar_chave(chave: &str) -> Result<(), String> {
    let ok = chave.strip_prefix(crate::acesso::chaves::PREFIXO).is_some_and(crate::acesso::segredo::forma_valida);
    if ok { Ok(()) } else { Err(crate::tr!("invalid key: copy it from the My key screen (it starts with bpk_)", "chave inválida: copie-a da tela Minha chave (ela começa com bpk_)")) }
}

fn json_bonito(v: &serde_json::Value) -> String {
    serde_json::to_string_pretty(v).expect("json") + "\n"
}

/// Hooks de comando (o app é remoto: hook `http` fixo não serve), em execução direta, sem shell.
fn hooks(exe: &str) -> serde_json::Value {
    let h = |evento: &str, prazo: u32| json!([{ "hooks": [{ "type": "command", "command": exe, "args": ["hook", evento, "--equipe"], "timeout": prazo }] }]);
    let mut sessao = h("sessao", 5);
    sessao[0]["matcher"] = json!("startup|resume|clear");
    let mut edicao = h("edicao", 3);
    edicao[0]["matcher"] = json!("Edit|Write");
    json!({ "hooks": {
        "SessionStart": sessao,
        "SessionEnd": h("fim", 5),
        "PostToolUse": edicao,
        "SubagentStart": h("subagente", 3),
        "SubagentStop": h("subagente", 3),
        "Stop": h("parada", 5),
    } })
}

/// Grava o marketplace com o plugin em `pasta`; nada aqui leva a chave nem o endereço.
pub fn gravar_plugin(pasta: &Path, exe: &Path) -> std::io::Result<()> {
    let exe = exe.display().to_string();
    let raiz = pasta.join("plugins").join(NOME);
    let arquivos = [
        (pasta.join(".claude-plugin/marketplace.json"), json!({ "name": MARKETPLACE, "description": "The Blueprint: a workspace shared with you (blueprint conectar)", "owner": { "name": "The Blueprint" }, "plugins": [{ "name": NOME, "source": format!("./plugins/{NOME}"), "description": "The MCP and the hooks of a workspace shared with you" }] })),
        (raiz.join(".claude-plugin/plugin.json"), json!({ "name": NOME, "version": crate::atualizador::VERSAO, "author": { "name": "The Blueprint" }, "description": "Connects Claude to a workspace shared with you in The Blueprint: the MCP and the hooks, with your key." })),
        (raiz.join(".mcp.json"), json!({ "mcpServers": { NOME: { "command": exe, "args": ["--mcp", "--equipe"] } } })),
        (raiz.join("hooks/hooks.json"), hooks(&exe)),
    ];
    for (caminho, v) in arquivos {
        std::fs::create_dir_all(caminho.parent().expect("pai"))?;
        std::fs::write(caminho, json_bonito(&v))?;
    }
    Ok(())
}

/// Grava a conexão e o plugin em `app` (a pasta do app) e registra nas CLIs que `achar` encontrar. Devolve o texto e o código de saída.
pub fn conectar_em(app: &Path, exe: &Path, url: &str, chave: &str, achar: impl Fn(&str) -> Option<PathBuf>) -> (String, i32) {
    let pasta = app.join("claude-plugin-equipe");
    if let Err(e) = equipe::gravar(&app.join("conexao-equipe.json"), url, chave).and_then(|()| gravar_plugin(&pasta, exe)) {
        return (crate::tr!("error: could not write to {}: {e}", "erro: não consegui gravar em {}: {e}", app.display()), 1);
    }
    let exe_txt = exe.display().to_string();
    let mcp_add = |escopo: &[&str]| [&["mcp", "add"][..], escopo, &[NOME, "--", &exe_txt, "--mcp", "--equipe"]].concat().iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let mut linhas = vec![crate::tr!("Connected to {url}.", "Conectado a {url}.")];
    let mut codigo = 0;
    match achar("claude") {
        None => linhas.push(crate::tr!("Claude Code: not found in PATH", "Claude Code: não encontrado no PATH")),
        Some(cli) => {
            let _ = rodar(&cli, &args(&["plugin", "marketplace", "remove", MARKETPLACE]));
            let plugin = rodar(&cli, &args(&["plugin", "marketplace", "add", &pasta.display().to_string()])).and_then(|()| rodar(&cli, &args(&["plugin", "install", PLUGIN, "--scope", "user"])));
            let _ = rodar(&cli, &args(&["mcp", "remove", "--scope", "user", NOME])); // o `claude mcp add blueprint-equipe` manual, se houver, daria o servidor duas vezes
            match plugin {
                Ok(()) => linhas.push(crate::tr!("Claude Code: plugin `{NOME}` installed (MCP and hooks)", "Claude Code: plugin `{NOME}` instalado (MCP e hooks)")),
                // sem o plugin a pessoa ainda fica com o MCP
                Err(e) => match rodar(&cli, &mcp_add(&["--scope", "user"])) {
                    Ok(()) => linhas.push(crate::tr!("Claude Code: plugin not installed ({e}); MCP `{NOME}` registered", "Claude Code: plugin não instalado ({e}); MCP `{NOME}` registrado")),
                    Err(e2) => {
                        codigo = 1;
                        linhas.push(crate::tr!("Claude Code: did not register ({e2})", "Claude Code: não registrou ({e2})"));
                    }
                },
            }
        }
    }
    match achar("codex") {
        None => linhas.push(crate::tr!("Codex: not found in PATH", "Codex: não encontrado no PATH")),
        Some(cli) => {
            let _ = rodar(&cli, &args(&["mcp", "remove", NOME]));
            match rodar(&cli, &mcp_add(&[])) {
                Ok(()) => linhas.push(crate::tr!("Codex: MCP `{NOME}` registered", "Codex: MCP `{NOME}` registrado")),
                Err(e) => {
                    codigo = 1;
                    linhas.push(crate::tr!("Codex: did not register ({e})", "Codex: não registrou ({e})"));
                }
            }
        }
    }
    linhas.push(crate::tr!("If the address changes, run blueprint conectar again with the new one. Open a new Claude session to use it.", "Se o endereço mudar, rode blueprint conectar de novo com o novo. Abra uma sessão nova do Claude para usar."));
    (linhas.join("\n"), codigo)
}

/// Tira o plugin, os MCPs e a conexão (a chave sai do disco).
pub fn remover_em(app: &Path, achar: impl Fn(&str) -> Option<PathBuf>) -> (String, i32) {
    if let Some(cli) = achar("claude") {
        let _ = rodar(&cli, &args(&["plugin", "uninstall", PLUGIN, "--scope", "user"]));
        let _ = rodar(&cli, &args(&["plugin", "marketplace", "remove", MARKETPLACE]));
        let _ = rodar(&cli, &args(&["mcp", "remove", "--scope", "user", NOME]));
    }
    if let Some(cli) = achar("codex") {
        let _ = rodar(&cli, &args(&["mcp", "remove", NOME]));
    }
    let _ = std::fs::remove_file(app.join("conexao-equipe.json"));
    let _ = std::fs::remove_dir_all(app.join("claude-plugin-equipe"));
    (crate::tr!("Disconnected: plugin, MCP and key removed from this computer.", "Desconectado: plugin, MCP e chave removidos deste computador."), 0)
}

pub fn executar(endereco: &str, chave: &str) -> (String, i32) {
    let http_local = std::env::var("BLUEPRINT_CONECTAR_HTTP_LOCAL").as_deref() == Ok("1"); // só os testes falam http com o túnel falso
    let url = match validar_endereco(endereco, http_local).and_then(|u| validar_chave(chave.trim()).map(|()| u)) {
        Ok(u) => u,
        Err(e) => return (crate::tr!("error: {e}", "erro: {e}"), 2),
    };
    let exe = match std::env::current_exe().map_err(|e| e.to_string()).and_then(super::exe_estavel) {
        Ok(e) => e,
        Err(e) => return (crate::tr!("error: {e}", "erro: {e}"), 1),
    };
    conectar_em(&crate::http::config::pasta_do_app(), &exe, &url, chave.trim(), procurar)
}

pub fn remover() -> (String, i32) {
    remover_em(&crate::http::config::pasta_do_app(), procurar)
}
