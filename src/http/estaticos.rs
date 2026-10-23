//! A interface e o kit do anatomy como arquivos estáticos: vêm embutidos no binário (`build.rs`) ou, em desenvolvimento, de uma pasta.
//! Só sai o que a lista permite (a página, `emblemas.mjs`, `web/*.css|js` e `kit/**/*.mjs`); nada mais fica exposto. Com ETag e gzip.
use super::pedido::{Pedido, Resposta};
use flate2::{write::GzEncoder, Compression};
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Abaixo disto o gzip não compensa.
const MIN_GZIP: usize = 1024;

struct Pronto {
    etag: String,
    gzip: Option<Arc<Vec<u8>>>,
}

pub struct Assets {
    pasta: Option<PathBuf>,
    prontos: Mutex<HashMap<String, Arc<Pronto>>>,
}

/// `[\w-]+` do JavaScript
fn segmento_valido(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
}

/// A lista do servidor Node: /blueprint.html, /emblemas.mjs, /web/**/*.css|js e /kit/**/*.mjs, sem ".." possível.
pub fn permitido(caminho: &str) -> bool {
    if matches!(caminho, "/blueprint.html" | "/emblemas.mjs" | "/agentes-figura.mjs" | "/agentes-claude.mjs") {
        return true;
    }
    let (prefixo, extensoes): (&str, &[&str]) = if caminho.starts_with("/web/") { ("/web/", &["css", "js"]) } else if caminho.starts_with("/kit/") { ("/kit/", &["mjs"]) } else { return false };
    let resto = &caminho[prefixo.len()..];
    let Some((sem_extensao, extensao)) = resto.rsplit_once('.') else { return false };
    extensoes.contains(&extensao) && sem_extensao.split('/').all(segmento_valido)
}

fn tipo_de(caminho: &str) -> &'static str {
    match caminho.rsplit('.').next() {
        Some("html") => "text/html",
        Some("css") => "text/css",
        _ => "text/javascript",
    }
}

/// FNV-1a de 64 bits: barato e suficiente para validar cache (não é segurança).
fn etag_de(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h = (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
    }
    format!("\"{h:016x}\"")
}

fn comprimir(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(bytes).ok()?;
    e.finish().ok()
}

impl Assets {
    pub fn novo(pasta: Option<PathBuf>) -> Assets {
        Assets { pasta, prontos: Mutex::new(HashMap::new()) }
    }

    fn ler(&self, caminho: &str) -> Option<Cow<'static, [u8]>> {
        if !permitido(caminho) {
            return None;
        }
        match &self.pasta {
            Some(p) => std::fs::read(p.join(caminho.trim_start_matches('/'))).ok().map(Cow::Owned),
            None => ASSETS.iter().find(|(c, _)| *c == caminho).map(|(_, b)| Cow::Borrowed(*b)),
        }
    }

    /// ETag e gzip de cada arquivo embutido são feitos uma vez; da pasta (desenvolvimento), a cada pedido.
    fn preparar(&self, caminho: &str, bytes: &[u8]) -> Arc<Pronto> {
        let calcular = || Arc::new(Pronto { etag: etag_de(bytes), gzip: (bytes.len() >= MIN_GZIP).then(|| comprimir(bytes)).flatten().map(Arc::new) });
        if self.pasta.is_some() {
            return calcular();
        }
        let mut prontos = self.prontos.lock().unwrap_or_else(|e| e.into_inner());
        prontos.entry(caminho.to_string()).or_insert_with(calcular).clone()
    }

    /// None = não é um arquivo estático (ou não existe): quem chama responde 404.
    pub fn responder(&self, caminho: &str, pedido: &Pedido) -> Option<Resposta> {
        let bytes = self.ler(caminho)?;
        let pronto = self.preparar(caminho, &bytes);
        let base = |status: u16, corpo: Vec<u8>| Resposta::texto(status, tipo_de(caminho), corpo).com("etag", &pronto.etag).com("cache-control", "no-cache").com("vary", "accept-encoding");
        if pedido.cabecalho("if-none-match").is_some_and(|v| v.split(',').any(|e| e.trim() == pronto.etag || e.trim() == "*")) {
            return Some(Resposta { cabecalhos: vec![("etag".into(), pronto.etag.clone()), ("cache-control".into(), "no-cache".into())], ..base(304, Vec::new()) });
        }
        let aceita_gzip = pedido.cabecalho("accept-encoding").is_some_and(|v| v.split(',').any(|e| e.trim().starts_with("gzip")));
        Some(match (&pronto.gzip, aceita_gzip) {
            (Some(gz), true) => base(200, gz.as_ref().clone()).com("content-encoding", "gzip"),
            _ => base(200, bytes.into_owned()),
        })
    }
}

#[cfg(test)]
mod tests;
