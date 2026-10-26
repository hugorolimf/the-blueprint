//! A escolha do endereço do compartilhamento (docs/compartilhar.md, "A fixed address"): `temporario` (quick tunnel da Cloudflare, o endereço muda a cada vez) ou `fixo`
//! (Tailscale Funnel, o mesmo endereço sempre). Guardada em `%APPDATA%\Blueprint\compartilhar.json` (`{"provedor":"fixo"}`), sem segredo nenhum.
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Provedor {
    #[default]
    Temporario,
    Fixo,
}

impl Provedor {
    pub fn texto(self) -> &'static str {
        match self {
            Provedor::Temporario => "temporario",
            Provedor::Fixo => "fixo",
        }
    }

    pub fn de_texto(t: &str) -> Option<Provedor> {
        match t {
            "temporario" => Some(Provedor::Temporario),
            "fixo" => Some(Provedor::Fixo),
            _ => None,
        }
    }
}

pub fn arquivo() -> PathBuf {
    crate::http::config::pasta_do_app().join("compartilhar.json")
}

/// Sem arquivo, ilegível ou com valor desconhecido: o temporário (o de sempre).
pub fn ler_de(caminho: &Path) -> Provedor {
    std::fs::read_to_string(caminho).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()).and_then(|v| Provedor::de_texto(v["provedor"].as_str()?)).unwrap_or_default()
}

pub fn gravar_em(caminho: &Path, p: Provedor) -> std::io::Result<()> {
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai)?;
    }
    std::fs::write(caminho, serde_json::json!({"provedor": p.texto()}).to_string() + "\n")
}
