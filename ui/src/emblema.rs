//! Os emblemas isométricos dos cartões e da tela de nome: os SVG exportados do kit por `node scripts/exportar-emblemas.mjs`
//! (as mesmas peças que a página desenha), embutidos no programa.

use egui::{Image, Rect, Ui};

macro_rules! emblemas {
    ($($nome:literal),* $(,)?) => {
        const EMBLEMAS: &[(&str, &str, &[u8])] = &[
            $(($nome, concat!("bytes://", $nome, ".svg"), include_bytes!(concat!("../assets/emblemas/", $nome, ".svg")))),*
        ];
    };
}

emblemas!(
    "blueprint-cartao", "blueprint-grande", "blueprint-previa", "moedas-cartao", "moedas-grande", "moedas-previa",
    "livros-cartao", "livros-grande", "livros-previa", "chip-cartao", "chip-grande", "chip-previa",
    "caminhao-cartao", "caminhao-grande", "caminhao-previa",
);

#[derive(Clone, Copy)]
pub enum Tamanho {
    /// 280x120, nos cartões
    Cartao,
    /// 390x280, na folha da tela de nome
    Grande,
    /// 520x380, na prévia do novo workspace
    Previa,
}

impl Tamanho {
    fn sufixo(self) -> &'static str {
        match self {
            Tamanho::Cartao => "cartao",
            Tamanho::Grande => "grande",
            Tamanho::Previa => "previa",
        }
    }
}

/// Pinta o emblema da marca `icone` dentro de `r`, inteiro e centralizado. Marca desconhecida vira a padrão.
pub fn pintar(ui: &Ui, r: Rect, icone: &str, tamanho: Tamanho) {
    let nome = |marca: &str| format!("{marca}-{}", tamanho.sufixo());
    let achado = EMBLEMAS.iter().find(|(n, _, _)| *n == nome(icone)).or_else(|| EMBLEMAS.iter().find(|(n, _, _)| *n == nome("blueprint")));
    if let Some((_, uri, bytes)) = achado {
        Image::from_bytes(*uri, *bytes).fit_to_exact_size(r.size()).paint_at(ui, r);
    }
}
