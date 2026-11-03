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
}

impl Tamanho {
    fn sufixo(self) -> &'static str {
        match self {
            Tamanho::Cartao => "cartao",
            Tamanho::Grande => "grande",
        }
    }
}

/// O retângulo no pixel inteiro, com o tamanho em pixels inteiros. `Image::paint_at` arredonda o canto e a borda oposta cada um por si,
/// e com rolagem ou escala fracionária o mesmo SVG pediria ±1 px conforme a posição; o carregador guarda por tamanho exato e, com dois
/// tamanhos do mesmo URI, apaga o que não foi usado no último quadro: rasterizaria e reenviaria a textura a cada troca.
/// Público para as outras imagens SVG que repintam a cada quadro (as figuras do Claude, o orquestrador).
pub fn no_pixel(ui: &Ui, r: Rect) -> Rect {
    let k = ui.pixels_per_point();
    Rect::from_min_size(((r.min.to_vec2() * k).round() / k).to_pos2(), (r.size() * k).round() / k)
}

/// O emblema «em construção» em 11 passos, do 0 ao 100%.
const PASSOS_CONSTRUCAO: [(&str, &[u8]); 11] = [
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

/// O emblema «em construção» dos cartões: as peças sólidas seguem o progresso (0 a 1), em passos de 10%.
pub fn pintar_construcao(ui: &Ui, r: Rect, progresso: f32) {
    let (uri, bytes) = PASSOS_CONSTRUCAO[(progresso.clamp(0.0, 1.0) * 10.0).round() as usize];
    let r = no_pixel(ui, r);
    Image::from_bytes(uri, bytes).fit_to_exact_size(r.size()).paint_at(ui, r);
}

/// O emblema pronto do cartão «novo blueprint». Os mesmos bytes do passo 100%, com URI próprio: ele tem outro tamanho que o dos cartões
/// prontos e, no mesmo URI, um apagaria o outro do carregador sempre que só um dos dois aparecesse na janela.
pub fn pintar_construcao_novo(ui: &Ui, r: Rect) {
    let r = no_pixel(ui, r);
    Image::from_bytes("bytes://construcao-novo-100.svg", PASSOS_CONSTRUCAO[10].1).fit_to_exact_size(r.size()).paint_at(ui, r);
}

/// A figura do Blueprint em 11 passos: da só no traço (0) à pronta (100).
const PASSOS_FIGURA: [(&str, &[u8]); 11] = [
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

/// A marca-d'água da lista: a marca do workspace «construída» conforme o progresso (só a padrão tem os passos; as outras vão inteiras), bem apagada.
pub fn pintar_fundo(ui: &Ui, r: Rect, icone: &str, progresso: f32) {
    let tom = egui::Color32::from_white_alpha(46);
    let r = no_pixel(ui, r);
    // `<icone>-previa`, comparado sem montar o nome
    let previa = EMBLEMAS.iter().find(|(n, _, _)| n.strip_suffix("-previa") == Some(icone));
    match previa {
        Some((_, uri, bytes)) if icone != "blueprint" => Image::from_bytes(*uri, *bytes).tint(tom).fit_to_exact_size(r.size()).paint_at(ui, r),
        _ => {
            let (uri, bytes) = PASSOS_FIGURA[(progresso.clamp(0.0, 1.0) * 10.0).round() as usize];
            Image::from_bytes(uri, bytes).tint(tom).fit_to_exact_size(r.size()).paint_at(ui, r);
        }
    }
}

/// A figura do Blueprint montada até `progresso` (0 a 1), inteira: o palco do «Novo workspace».
pub fn pintar_passo(ui: &Ui, r: Rect, progresso: f32) {
    let (uri, bytes) = PASSOS_FIGURA[(progresso.clamp(0.0, 1.0) * 10.0).round() as usize];
    let r = no_pixel(ui, r);
    Image::from_bytes(uri, bytes).fit_to_exact_size(r.size()).paint_at(ui, r);
}

/// Pinta o emblema da marca `icone` dentro de `r`, inteiro e centralizado. Marca desconhecida vira a padrão.
pub fn pintar(ui: &Ui, r: Rect, icone: &str, tamanho: Tamanho) {
    let sufixo = tamanho.sufixo();
    let achado = EMBLEMAS.iter().find(|(n, _, _)| e_de(n, icone, sufixo)).or_else(|| EMBLEMAS.iter().find(|(n, _, _)| e_de(n, "blueprint", sufixo)));
    if let Some((_, uri, bytes)) = achado {
        let r = no_pixel(ui, r);
        Image::from_bytes(*uri, *bytes).fit_to_exact_size(r.size()).paint_at(ui, r);
    }
}

/// O emblema `n` é `<marca>-<sufixo>` (comparado sem montar o nome a cada emblema).
fn e_de(n: &str, marca: &str, sufixo: &str) -> bool {
    n.strip_suffix(sufixo).and_then(|m| m.strip_suffix('-')) == Some(marca)
}
