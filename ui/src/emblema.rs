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

/// O emblema «em construção» dos cartões: as peças sólidas seguem o progresso (0 a 1), em passos de 10%.
pub fn pintar_construcao(ui: &Ui, r: Rect, progresso: f32) {
    const PASSOS: [(&str, &[u8]); 11] = [
        ("bytes://construcao-000.svg", include_bytes!("../assets/emblemas/construcao-000.svg")),
        ("bytes://construcao-010.svg", include_bytes!("../assets/emblemas/construcao-010.svg")),
        ("bytes://construcao-020.svg", include_bytes!("../assets/emblemas/construcao-020.svg")),
        ("bytes://construcao-030.svg", include_bytes!("../assets/emblemas/construcao-030.svg")),
        ("bytes://construcao-040.svg", include_bytes!("../assets/emblemas/construcao-040.svg")),
        ("bytes://construcao-050.svg", include_bytes!("../assets/emblemas/construcao-050.svg")),
        ("bytes://construcao-060.svg", include_bytes!("../assets/emblemas/construcao-060.svg")),
        ("bytes://construcao-070.svg", include_bytes!("../assets/emblemas/construcao-070.svg")),
        ("bytes://construcao-080.svg", include_bytes!("../assets/emblemas/construcao-080.svg")),
        ("bytes://construcao-090.svg", include_bytes!("../assets/emblemas/construcao-090.svg")),
        ("bytes://construcao-100.svg", include_bytes!("../assets/emblemas/construcao-100.svg")),
    ];
    let (uri, bytes) = PASSOS[(progresso.clamp(0.0, 1.0) * 10.0).round() as usize];
    Image::from_bytes(uri, bytes).fit_to_exact_size(r.size()).paint_at(ui, r);
}

/// A marca-d'água da lista: a marca do workspace «construída» conforme o progresso (só a padrão tem os passos; as outras vão inteiras), bem apagada.
pub fn pintar_fundo(ui: &Ui, r: Rect, icone: &str, progresso: f32) {
    const PASSOS: [(&str, &[u8]); 11] = [
        ("bytes://fundo-000.svg", include_bytes!("../assets/emblemas/fundo-000.svg")),
        ("bytes://fundo-010.svg", include_bytes!("../assets/emblemas/fundo-010.svg")),
        ("bytes://fundo-020.svg", include_bytes!("../assets/emblemas/fundo-020.svg")),
        ("bytes://fundo-030.svg", include_bytes!("../assets/emblemas/fundo-030.svg")),
        ("bytes://fundo-040.svg", include_bytes!("../assets/emblemas/fundo-040.svg")),
        ("bytes://fundo-050.svg", include_bytes!("../assets/emblemas/fundo-050.svg")),
        ("bytes://fundo-060.svg", include_bytes!("../assets/emblemas/fundo-060.svg")),
        ("bytes://fundo-070.svg", include_bytes!("../assets/emblemas/fundo-070.svg")),
        ("bytes://fundo-080.svg", include_bytes!("../assets/emblemas/fundo-080.svg")),
        ("bytes://fundo-090.svg", include_bytes!("../assets/emblemas/fundo-090.svg")),
        ("bytes://fundo-100.svg", include_bytes!("../assets/emblemas/fundo-100.svg")),
    ];
    let tom = egui::Color32::from_white_alpha(46);
    if icone == "blueprint" || !EMBLEMAS.iter().any(|(n, _, _)| *n == format!("{icone}-previa")) {
        let (uri, bytes) = PASSOS[(progresso.clamp(0.0, 1.0) * 10.0).round() as usize];
        Image::from_bytes(uri, bytes).tint(tom).fit_to_exact_size(r.size()).paint_at(ui, r);
    } else if let Some((_, uri, bytes)) = EMBLEMAS.iter().find(|(n, _, _)| *n == format!("{icone}-previa")) {
        Image::from_bytes(*uri, *bytes).tint(tom).fit_to_exact_size(r.size()).paint_at(ui, r);
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
