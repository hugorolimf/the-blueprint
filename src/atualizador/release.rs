//! Lê a lista de releases do GitHub (`GET /repos/<dono>/<repo>/releases`) e escolhe a que o app deve baixar. Usa a lista, e não `/releases/latest`,
//! porque o `latest` ignora pré-lançamentos e o The Blueprint é beta: a v0.1.0 sai como pré-lançamento.
use super::versao::{mais_nova, Versao};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// sem o `v` da tag (`0.1.1`)
    pub versao: String,
    pub notas: String,
    pub instalador: Asset,
    pub sha256sums: Asset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub nome: String,
    pub url: String,
    pub tamanho: u64,
}

fn asset(a: &Value) -> Option<Asset> {
    Some(Asset { nome: a["name"].as_str()?.to_string(), url: a["browser_download_url"].as_str()?.to_string(), tamanho: a["size"].as_u64().unwrap_or(0) })
}

/// O nome do instalador que o `.iss` gera: `Blueprint-<versão>-instalador.exe`.
fn eh_instalador(nome: &str) -> bool {
    nome.starts_with("Blueprint-") && nome.ends_with("-instalador.exe")
}

/// A release mais nova que `instalada` e com instalador e `SHA256SUMS` (rascunho e release incompleta não contam). Com `so_estaveis`,
/// as marcadas como pré-lançamento no GitHub ficam de fora. `Err` só quando o JSON não é uma lista de releases.
pub fn escolher(json: &str, instalada: &str, so_estaveis: bool) -> Result<Option<Release>, String> {
    let lista: Value = serde_json::from_str(json).map_err(|e| format!("resposta do GitHub inválida: {e}"))?;
    let lista = lista.as_array().ok_or("resposta do GitHub inesperada: não é uma lista de releases")?;
    let mut melhor: Option<(Versao, Release)> = None;
    for r in lista {
        if r["draft"].as_bool() == Some(true) || (so_estaveis && r["prerelease"].as_bool() == Some(true)) {
            continue;
        }
        let tag = r["tag_name"].as_str().unwrap_or("");
        if !mais_nova(instalada, tag) {
            continue;
        }
        let assets: Vec<Asset> = r["assets"].as_array().into_iter().flatten().filter_map(asset).collect();
        let (Some(instalador), Some(sums)) = (assets.iter().find(|a| eh_instalador(&a.nome)), assets.iter().find(|a| a.nome == "SHA256SUMS")) else { continue };
        let versao = Versao::ler(tag).expect("mais_nova só aceita versão válida");
        if melhor.as_ref().is_none_or(|(v, _)| versao > *v) {
            let release = Release { versao: tag.trim_start_matches(['v', 'V']).to_string(), notas: r["body"].as_str().unwrap_or("").chars().take(4000).collect(), instalador: instalador.clone(), sha256sums: sums.clone() };
            melhor = Some((versao, release));
        }
    }
    Ok(melhor.map(|(_, r)| r))
}
