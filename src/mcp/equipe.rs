//! O Claude do convidado (`blueprint conectar`): o `--mcp --equipe` e o `blueprint hook <evento> --equipe` do plugin `blueprint-equipe` leem
//! o endereço e a chave de `conexao-equipe.json`, na pasta do app (`%APPDATA%\Blueprint`), a cada pedido. Assim a chave não fica nos argumentos
//! de nenhum processo nem na cópia do plugin que o Claude Code guarda, e trocar o endereço (o túnel muda a cada vez) vale na hora, até para
//! sessões abertas. Sem `--equipe`, nada daqui roda: o MCP de sempre fala com o app local (ou BLUEPRINT_URL + BLUEPRINT_CHAVE).
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

static LIGADO: AtomicBool = AtomicBool::new(false);

/// Este processo é do plugin do convidado (`--equipe` na linha de comando).
pub fn ligar() {
    LIGADO.store(true, Ordering::Relaxed);
}

pub fn ligado() -> bool {
    LIGADO.load(Ordering::Relaxed)
}

pub struct Conexao {
    pub url: String,
    pub chave: String,
}

pub fn arquivo() -> PathBuf {
    crate::http::config::pasta_do_app().join("conexao-equipe.json")
}

pub fn ler_de(caminho: &Path) -> Option<Conexao> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(caminho).ok()?).ok()?;
    let texto = |k: &str| v.get(k).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from);
    Some(Conexao { url: texto("url")?.trim_end_matches('/').to_string(), chave: texto("chave")? })
}

/// Com `--equipe`, a conexão gravada pelo `blueprint conectar` (None = não conectado). Sem `--equipe`, None.
pub fn conexao() -> Option<Conexao> {
    ligado().then(|| ler_de(&arquivo())).flatten()
}

/// O MCP fala com um servidor remoto (convidado ou equipe)? Então nada vai para os pendentes locais: o app desta máquina não é o dono dos dados.
pub fn remoto() -> bool {
    ligado() || std::env::var("BLUEPRINT_CHAVE").is_ok_and(|c| !c.trim().is_empty())
}

pub fn gravar(caminho: &Path, url: &str, chave: &str) -> std::io::Result<()> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai)?;
    }
    std::fs::write(caminho, json!({"url": url, "chave": chave}).to_string() + "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grava_e_le_a_conexao_e_ignora_arquivo_torto() {
        let pasta = std::env::temp_dir().join(format!("blueprint-equipe-{}", std::process::id()));
        let arq = pasta.join("conexao-equipe.json");
        gravar(&arq, "https://a.trycloudflare.com", "bpk_x").unwrap();
        let c = ler_de(&arq).unwrap();
        assert_eq!((c.url.as_str(), c.chave.as_str()), ("https://a.trycloudflare.com", "bpk_x"));
        std::fs::write(&arq, "{\"url\": \"https://a\"}").unwrap();
        assert!(ler_de(&arq).is_none(), "sem chave não é conexão");
        std::fs::write(&arq, "lixo").unwrap();
        assert!(ler_de(&arq).is_none());
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
