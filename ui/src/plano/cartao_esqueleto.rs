//! O esqueleto de uma figura que ainda está sendo desenhada: o desenho de um gráfico apagado (título, legenda, barras) e um brilho que varre o cartão.
//! Só repinta enquanto aparece; quando a figura chega, o quadro volta a dormir.

use super::acao_estilo::alfa;
use super::acao_movimento::{fase, pulso};
use super::linha::{Letra, Pincel};
use crate::tema::{AZUL, CARTAO, MEDIO};
use egui::{pos2, vec2, Mesh, Painter, Rect, Shape};

/// As barras do gráfico apagado, em fração da largura útil.
const BARRAS: [f32; 4] = [0.82, 0.62, 0.46, 0.3];

/// O brilho: uma faixa inclinada, transparente nas pontas e clara no meio, que cruza `area` de um lado ao outro.
fn brilho(p: &Painter, area: Rect, t: f32) {
    let faixa = (area.width() * 0.4).max(70.0);
    let xc = area.min.x - faixa + (area.width() + 2.0 * faixa) * t;
    let inclina = area.height() * 0.25;
    let (zero, claro) = (alfa(AZUL, 0.0), alfa(AZUL, 0.16));
    let mut m = Mesh::default();
    let (topo, base) = (area.min.y, area.max.y);
    for (x, cor) in [(xc - faixa / 2.0, zero), (xc, claro), (xc + faixa / 2.0, zero)] {
        m.colored_vertex(pos2(x + inclina, topo), cor);
        m.colored_vertex(pos2(x - inclina, base), cor);
    }
    m.add_triangle(0, 1, 2);
    m.add_triangle(1, 2, 3);
    m.add_triangle(2, 3, 4);
    m.add_triangle(3, 4, 5);
    p.with_clip_rect(area.intersect(p.clip_rect())).add(Shape::mesh(m));
}

/// O esqueleto em `area`, com `texto` («desenhando…») embaixo.
pub fn desenhar(p: &Painter, area: Rect, texto: &str) {
    p.rect_filled(area, 0, CARTAO);
    let (w, h) = (area.width(), area.height());
    let barra = |x: f32, y: f32, largura: f32, alto: f32, forca: f32| {
        p.rect_filled(Rect::from_min_size(area.min + vec2(x, y), vec2(largura.max(1.0), alto)), 2.0, alfa(MEDIO, forca));
    };
    let alto = (h * 0.055).clamp(2.0, 9.0);
    let util = w * 0.76;
    let x0 = w * 0.12;
    barra(x0, h * 0.13, w * 0.3, alto * 1.3, 0.2);
    barra(x0, h * 0.13 + alto * 2.4, w * 0.18, alto * 0.8, 0.12);
    let passo = (h * 0.14).max(alto * 2.2);
    for (i, f) in BARRAS.iter().enumerate() {
        let y = h * 0.36 + passo * i as f32;
        if y + alto > h * 0.84 {
            break;
        }
        barra(x0, y, w * 0.07, alto * 0.8, 0.12);
        barra(x0 + w * 0.1, y, (util - w * 0.1) * f, alto, 0.17);
    }
    brilho(p, area, fase(p.ctx(), 1.7));
    if h >= 54.0 {
        let c = Pincel { p, o: area.min };
        let respira = 0.55 + 0.45 * pulso(p.ctx(), 1.8);
        c.texto(texto, w / 2.0, h - 8.0, Letra { cor: alfa(MEDIO, respira), ..Letra::nova(9.5, MEDIO).meio() });
    }
}
