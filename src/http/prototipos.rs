//! /prototipos/<id>.html: HTML escrito por um agente. Roda isolado (origem opaca, sem rede, sem acesso ao app); nunca allow-same-origin.
use super::estado::Estado;
use super::pedido::{Corpo, Resposta};
use crate::mcp::acoes::id_prototipo_valido;

const CSP: &str = "sandbox allow-scripts; default-src 'none'; img-src data: blob:; media-src data: blob:; font-src data:; style-src 'unsafe-inline'; script-src 'unsafe-inline'";

/// None = o caminho não tem a forma de um protótipo (segue para as outras rotas, que respondem 404).
pub fn rota(estado: &Estado, caminho: &str) -> Option<Resposta> {
    let id = caminho.strip_prefix("/prototipos/")?.strip_suffix(".html")?;
    if !id_prototipo_valido(id) {
        return None;
    }
    let Ok(html) = std::fs::read(estado.mcp.prototipos.join(format!("{id}.html"))) else { return Some(Resposta::texto(404, "text/plain", "não encontrado")) };
    Some(Resposta {
        status: 200,
        cabecalhos: vec![
            ("content-type".into(), "text/html; charset=utf-8".into()),
            ("cache-control".into(), "no-store".into()),
            ("x-content-type-options".into(), "nosniff".into()),
            ("content-security-policy".into(), CSP.into()),
        ],
        corpo: Corpo::Bytes(html),
    })
}
