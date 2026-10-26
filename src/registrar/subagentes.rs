//! O `settings.json` do plugin `blueprint`: a única chave que ele leva é `subagentStatusLine` (o manual do plugin só aceita `agent` e `subagentStatusLine`, e `agent`
//! de propósito não: faria toda sessão usar o agente `blueprint`). O comando é este binário, com o caminho gravado pelo `registrar`, como nos hooks.
use serde_json::json;
use std::fs;
use std::io;
use std::path::Path;

/// O conteúdo do `settings.json`; as barras comuns e as aspas deixam o caminho com espaços inteiro para o shell do Claude Code.
pub(super) fn conteudo(exe: &Path) -> String {
    let comando = format!("\"{}\" hook subagentes", exe.display().to_string().replace('\\', "/"));
    serde_json::to_string_pretty(&json!({"subagentStatusLine": {"type": "command", "command": comando}})).expect("json") + "\n"
}

/// Grava `plugins/blueprint/settings.json` em `pasta` (o marketplace local); idempotente, só reescreve o que mudou.
pub(super) fn gravar(pasta: &Path, exe: &Path) -> io::Result<()> {
    let caminho = pasta.join("plugins/blueprint/settings.json");
    let novo = conteudo(exe);
    if fs::read_to_string(&caminho).is_ok_and(|atual| atual == novo) {
        return Ok(());
    }
    if let Some(pai) = caminho.parent() {
        fs::create_dir_all(pai)?;
    }
    fs::write(caminho, novo)
}
