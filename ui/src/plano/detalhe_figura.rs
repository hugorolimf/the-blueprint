//! A figura grande da prévia, num quadro reto de fio fino: as marcas de canto já vêm na própria figura.

use super::super::acao_estilo::alfa;
use super::super::figura;
use super::super::linha::Pincel;
use super::super::modelo::Detalhe;
use super::Medidas;
use crate::tema::{BAIXO, CARTAO};
use egui::{Stroke, StrokeKind};

pub(super) fn figura_grande(c: &Pincel, servidor: &str, d: &Detalhe, m: &Medidas, x: f32, y: f32) {
    let r = c.caixa(x, y, m.w_fig, m.campo);
    c.p.rect_filled(r, 0, CARTAO);
    let area = c.caixa(x + 2.0, y + (m.campo - m.fig) / 2.0 + 1.0, m.w_fig - 4.0, m.fig - 2.0);
    let nome = if d.titulo.is_empty() { &d.id } else { &d.titulo };
    figura::desenhar(c.p, servidor, d.figura.as_ref(), nome, area);
    c.p.rect_stroke(r, 0, Stroke::new(1.0, alfa(BAIXO, 0.8)), StrokeKind::Inside);
}
