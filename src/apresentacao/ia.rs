//! Os arquivos fixos da biblioteca (`apresentacao-ia/`), embutidos pelo `build.rs`: o que não existia na compilação não está na tabela.
include!(concat!(env!("OUT_DIR"), "/ia_assets.rs"));

/// O texto de um deles: `modelo.html`, `dist/biblioteca.js`, `dist/estilo.css` ou `termos-tecnicos.json`.
pub fn arquivo(nome: &str) -> Option<&'static str> {
    IA.iter().find(|(n, _)| *n == nome).and_then(|(_, b)| std::str::from_utf8(b).ok())
}
