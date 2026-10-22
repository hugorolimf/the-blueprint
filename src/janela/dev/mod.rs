//! Modo de desenvolvimento (`blueprint dev`): o próprio app (banco real, porta 3000, instância única), mas com a interface lida direto
//! da pasta do projeto, F12 nas janelas e recarga da janela ao salvar `web/`, `kit/` e a página. Só o Rust (`src/`) pede recompilar.
//! shortcut: o vigia compara as datas dos arquivos a cada 300 ms (só neste modo; o app normal não faz isso). Se pesar, trocar por
//! ReadDirectoryChangesW.
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Arquivos soltos na raiz que a interface serve; o resto vem de `web/` e `kit/`.
const NA_RAIZ: [&str; 3] = ["blueprint.html", "emblemas.mjs", "agentes-figura.mjs"];
const EXTENSOES: [&str; 4] = ["css", "js", "mjs", "html"];

/// A pasta do projeto onde este binário foi compilado (`cargo run -- dev`).
pub fn raiz_do_projeto() -> Option<PathBuf> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    raiz.join("blueprint.html").is_file().then_some(raiz)
}

pub struct Vigia {
    raiz: PathBuf,
    vistos: HashMap<PathBuf, SystemTime>,
}

fn coletar(caminho: &Path, saida: &mut HashMap<PathBuf, SystemTime>) {
    let Ok(meta) = std::fs::metadata(caminho) else { return };
    if meta.is_dir() {
        for entrada in std::fs::read_dir(caminho).into_iter().flatten().flatten() {
            coletar(&entrada.path(), saida);
        }
    } else if caminho.extension().and_then(|e| e.to_str()).is_some_and(|e| EXTENSOES.contains(&e)) {
        if let Ok(data) = meta.modified() {
            saida.insert(caminho.to_path_buf(), data);
        }
    }
}

impl Vigia {
    pub fn novo(raiz: PathBuf) -> Vigia {
        let mut v = Vigia { raiz, vistos: HashMap::new() };
        v.vistos = v.ler();
        v
    }

    fn ler(&self) -> HashMap<PathBuf, SystemTime> {
        let mut m = HashMap::new();
        for nome in NA_RAIZ.iter().chain(["web", "kit"].iter()) {
            coletar(&self.raiz.join(nome), &mut m);
        }
        m
    }

    /// None = nada mudou; Some(true) = só mudou CSS (basta trocar o estilo); Some(false) = recarregar a página.
    pub fn mudou(&mut self) -> Option<bool> {
        let agora = self.ler();
        let mut alterados: Vec<PathBuf> = agora.iter().filter(|(p, t)| self.vistos.get(*p) != Some(*t)).map(|(p, _)| p.clone()).collect();
        alterados.extend(self.vistos.keys().filter(|p| !agora.contains_key(*p)).cloned());
        self.vistos = agora;
        (!alterados.is_empty()).then(|| alterados.iter().all(|p| p.extension().is_some_and(|e| e == "css")))
    }
}

/// Troca as folhas de estilo sem recarregar a página (a tela em que você está continua).
pub const TROCA_CSS: &str = r#"document.querySelectorAll('link[rel="stylesheet"]').forEach((l) => { l.href = l.href.split("?")[0] + "?v=" + Date.now(); });"#;
