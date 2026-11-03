//! O topo da ação aberta: o «− Recolher» à direita e os links à esquerda, numa linha fina em cima do palco. O nome e a miniatura já estão na linha da ação.

use super::super::linha::Pincel;
use super::super::modelo::Acao as AcaoPlano;
use super::{links, tem_links, Geometria, Saida};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use egui::{Painter, Ui};

const ALTO: f32 = 28.0;
const VAO: f32 = 8.0;

fn recolher<'a>() -> Botao<'a> {
    Botao { texto: tr("− Recolher"), tam: 10.0, cheio: false, ativo: true, altura: ALTO, pad_x: 12.0 }
}

pub fn altura(_p: &Painter, _a: &AcaoPlano, _g: &Geometria) -> f32 {
    ALTO + VAO
}

/// Os links e o botão. Devolve o que o clique em «Recolher» pede.
pub fn desenhar(ui: &mut Ui, c: &Pincel, a: &AcaoPlano, g: &Geometria, y: f32) -> Option<Saida> {
    let topo = y;
    let b = recolher();
    let x_botao = g.x + g.w - b.largura(c.p);
    if tem_links(&a.links) {
        links(ui, c, g.x, topo + ALTO * 0.5 + 4.0, x_botao - 12.0, &a.links, &a.id);
    }
    if botao(ui, c.pt(x_botao, topo), &b).clicked() {
        // as linhas de baixo sobem andando, como ao fechar pela linha da ação
        crate::anima::mexer(ui.ctx(), 350.0);
        return Some(Saida::Recolher);
    }
    None
}
