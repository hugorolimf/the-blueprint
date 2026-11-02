//! O vocabulário visual do painel da ação, do diff, da revisão e dos cartões de detalhe: raio, fios que somem, etiquetas, sombra e títulos de seção.
//! Traço fino sobre papel de planta; cor só onde informa. Os valores moram aqui para o resto não repetir número solto.

use super::linha::{Letra, Pincel};
use crate::tema::{rgba, BAIXO, CAMPO, MEDIO};
use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, Mesh, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

/// O canto dos cartões, etiquetas e botões: reto, como tudo na planta.
pub const RAIO: u8 = 0;
/// O texto corrido do painel: entre o médio e a tinta, para ler sem gritar.
pub const CORPO: Color32 = Color32::from_rgb(0xcf, 0xe4, 0xfb);
/// O fundo de um cartão dentro do painel e o do cartão com o mouse em cima.
pub const FUNDO_CARTAO: Color32 = crate::tema::PAPEL;
pub const FUNDO_CARTAO_SOBRE: Color32 = crate::tema::QUADRA;

pub fn raio() -> CornerRadius {
    CornerRadius::same(RAIO)
}

/// Mistura `a` e `b` (0 = `a`, 1 = `b`).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    a.lerp_to_gamma(b, t.clamp(0.0, 1.0))
}

/// A cor com transparência `a` de 0 a 1.
pub fn alfa(c: Color32, a: f32) -> Color32 {
    c.gamma_multiply(a.clamp(0.0, 1.0))
}

/// Retângulo de cantos suaves com fundo e traço de 1 px por dentro.
pub fn cartao(p: &Painter, r: Rect, fundo: Color32, traco: Color32) {
    p.rect_filled(r, raio(), fundo);
    p.rect_stroke(r, raio(), Stroke::new(1.0, traco), StrokeKind::Inside);
}

/// A sombra macia de baixo de um cartão que se ergue; `forca` de 0 a 1.
pub fn sombra(p: &Painter, r: Rect, forca: f32) {
    if forca <= 0.01 {
        return;
    }
    let s = Shadow { offset: [0, (4.0 + 6.0 * forca) as i8], blur: (10.0 + 14.0 * forca) as u8, spread: 0, color: rgba(0x02132e, (150.0 * forca) as u8) };
    p.add(s.as_shape(r, raio()));
}

/// Preenche `r` indo de `esq` a `dir`.
pub fn degrade_h(p: &Painter, r: Rect, esq: Color32, dir: Color32) {
    let mut m = Mesh::default();
    for (pos, cor) in [(r.left_top(), esq), (r.right_top(), dir), (r.right_bottom(), dir), (r.left_bottom(), esq)] {
        m.colored_vertex(pos, cor);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

/// Preenche `r` indo de cima (`topo`) para baixo (`fundo`).
pub fn degrade_v(p: &Painter, r: Rect, topo: Color32, fundo: Color32) {
    let mut m = Mesh::default();
    for (pos, cor) in [(r.left_top(), topo), (r.right_top(), topo), (r.right_bottom(), fundo), (r.left_bottom(), fundo)] {
        m.colored_vertex(pos, cor);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    p.add(Shape::mesh(m));
}

/// Um fio de 1 px que vai sumindo da esquerda para a direita.
pub fn fio_que_some(c: &Pincel, x0: f32, x1: f32, y: f32, cor: Color32, forca: f32) {
    if x1 <= x0 {
        return;
    }
    let r = Rect::from_min_max(c.pt(x0, y), c.pt(x1, y + 1.0));
    degrade_h(c.p, r, alfa(cor, forca), alfa(cor, forca * 0.08));
}

/// O título de uma seção, como as colunas da folha original: o nome em caixa alta e o fio embaixo. Ocupa 22 px abaixo de `y` (a linha de base).
pub fn rotulo_secao(c: &Pincel, x: f32, y: f32, t: &str, w: f32) {
    c.rotulo(t, x, y, Letra::nova(9.0, MEDIO));
    c.fio(x, y + 7.0, x + w, y + 7.0, BAIXO, 1.0);
}

/// A largura de uma etiqueta com este texto.
pub fn largura_etiqueta(c: &Pincel, texto: &str) -> f32 {
    c.largura(texto, &Letra::nova(9.5, MEDIO).trilha(0.04)) + 12.0
}

pub const ALTO_ETIQUETA: f32 = 16.0;

/// A etiqueta (área, dependência, data): cantos suaves, texto pequeno. Com `cor`, o fundo e o traço levam a cor do estado; sem, é só contorno.
/// `y` é o topo. Devolve a largura.
pub fn etiqueta(c: &Pincel, x: f32, y: f32, texto: &str, cor: Option<Color32>, forte: bool) -> f32 {
    let letra = Letra::nova(9.5, cor.unwrap_or(MEDIO)).trilha(0.04);
    let w = c.largura(texto, &letra) + 12.0;
    let r = c.caixa(x, y, w, ALTO_ETIQUETA);
    let (fundo, traco) = match cor {
        Some(k) if forte => (alfa(k, 0.16), alfa(k, 0.55)),
        Some(k) => (alfa(k, 0.08), alfa(k, 0.35)),
        None => (alfa(CAMPO, 0.5), alfa(MEDIO, 0.32)),
    };
    cartao(c.p, r, fundo, traco);
    c.texto(texto, x + 6.0, y + 11.5, letra);
    w
}

/// Um ponto de estado com o halo suave em volta.
pub fn ponto(p: &Painter, centro: Pos2, cor: Color32, raio: f32, halo: f32) {
    if halo > 0.0 {
        p.circle_filled(centro, raio + 2.5 + 1.5 * halo, alfa(cor, 0.10 + 0.10 * halo));
    }
    p.circle_filled(centro, raio, cor);
}

/// O contorno de um retângulo tracejado, de cantos retos.
pub fn tracejado(p: &Painter, r: Rect, cor: Color32, traco: f32) {
    let a = r.shrink(0.5);
    let pontos = [a.left_top(), a.right_top(), a.right_bottom(), a.left_bottom(), a.left_top()];
    p.extend(Shape::dashed_line(&pontos, Stroke::new(1.0, cor), traco, traco));
}

/// Corta `s` no meio com «…» para caber em `max`, sem perder o fim (caminhos de arquivo: o que importa é o último pedaço).
pub fn cortar_inicio(c: &Pincel, s: &str, max: f32, l: &Letra) -> (String, bool) {
    if c.largura(s, l) <= max {
        return (s.to_string(), false);
    }
    let letras: Vec<char> = s.chars().collect();
    let (mut a, mut b) = (0usize, letras.len());
    while b - a > 1 {
        let meio = (a + b) / 2;
        let t: String = std::iter::once('…').chain(letras[meio..].iter().copied()).collect();
        if c.largura(&t, l) <= max {
            b = meio;
        } else {
            a = meio;
        }
    }
    (std::iter::once('…').chain(letras[b..].iter().copied()).collect(), true)
}
