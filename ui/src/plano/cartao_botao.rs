//! Os botões e os ícones desenhados do painel: traço fino, cantos suaves, cor que entra no hover e um clarão no clique.
//! O que sai daqui vale para a ação aberta, o diff, a revisão e a folha de detalhe: um botão só, com o mesmo jeito em todo lugar.

use super::acao_estilo::{alfa, mix, raio};
use super::acao_movimento::{clarao, hover, marcar_instante};
use super::linha::{Letra, Pincel};
use crate::tema::{MEDIO, PAPEL, TINTA};
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui};
use std::hash::Hash;

pub const ALTO: f32 = 24.0;
const PAD: f32 = 10.0;
const LARGURA_ICONE: f32 = 16.0;

/// Os ícones, desenhados em traço (sem fonte de ícones, sem emoji).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icone {
    Check,
    Lapis,
    Xis,
    Recolher,
    Esquerda,
    Direita,
    Enviar,
}

/// Desenha `i` com o centro em `c`, de uns 9 px de lado.
pub fn icone(p: &Painter, c: Pos2, i: Icone, cor: Color32) {
    let traco = Stroke::new(1.35, cor);
    let linha = |pts: &[(f32, f32)]| {
        p.add(Shape::line(pts.iter().map(|(x, y)| pos2(c.x + x, c.y + y)).collect(), traco));
    };
    match i {
        Icone::Check => linha(&[(-4.2, 0.4), (-1.4, 3.2), (4.2, -3.4)]),
        Icone::Lapis => {
            linha(&[(-4.2, 4.2), (-3.4, 1.0), (2.6, -5.0), (5.0, -2.6), (-1.0, 3.4), (-4.2, 4.2)]);
            linha(&[(0.6, -2.0), (3.0, 0.4)]);
        }
        Icone::Xis => {
            linha(&[(-3.4, -3.4), (3.4, 3.4)]);
            linha(&[(3.4, -3.4), (-3.4, 3.4)]);
        }
        Icone::Recolher => linha(&[(-4.0, 2.0), (0.0, -2.0), (4.0, 2.0)]),
        Icone::Esquerda => linha(&[(2.0, -4.0), (-2.0, 0.0), (2.0, 4.0)]),
        Icone::Direita => linha(&[(-2.0, -4.0), (2.0, 0.0), (-2.0, 4.0)]),
        Icone::Enviar => {
            linha(&[(-4.5, 0.0), (4.0, 0.0)]);
            linha(&[(0.5, -3.8), (4.2, 0.0), (0.5, 3.8)]);
        }
    }
}

/// Um botão: texto em caixa alta, ícone opcional e a cor que ele toma no hover.
#[derive(Clone, Copy)]
pub struct Botao<'a> {
    pub texto: &'a str,
    pub icone: Option<Icone>,
    pub cor: Color32,
    pub cheio: bool,
    pub ativo: bool,
    /// o ícone depois do texto (setas de «Próximo»)
    pub icone_fim: bool,
}

impl<'a> Botao<'a> {
    pub fn novo(texto: &'a str) -> Self {
        Botao { texto, icone: None, cor: TINTA, cheio: false, ativo: true, icone_fim: false }
    }
    pub fn icone(self, i: Icone) -> Self {
        Botao { icone: Some(i), ..self }
    }
    pub fn cor(self, c: Color32) -> Self {
        Botao { cor: c, ..self }
    }
    pub fn no_fim(self) -> Self {
        Botao { icone_fim: true, ..self }
    }
    pub fn cheio(self, c: bool) -> Self {
        Botao { cheio: c, ..self }
    }
    pub fn ativo(self, a: bool) -> Self {
        Botao { ativo: a, ..self }
    }
    fn letra(&self, cor: Color32) -> Letra {
        let l = Letra::nova(10.0, cor).trilha(0.1);
        if self.cheio {
            l.forte()
        } else {
            l
        }
    }
    pub fn largura(&self, p: &Painter) -> f32 {
        let c = Pincel { p, o: Pos2::ZERO };
        c.largura(&self.texto.to_uppercase(), &self.letra(TINTA)) + 2.0 * PAD + if self.icone.is_some() { LARGURA_ICONE } else { 0.0 }
    }
}

/// Desenha o botão com o canto de cima à esquerda em `pos` (coordenadas da tela). `id` é estável entre quadros: o clique precisa dele.
pub fn botao(ui: &mut Ui, id: impl Hash + std::fmt::Debug + Copy, pos: Pos2, b: &Botao) -> Response {
    let r = Rect::from_min_size(pos, vec2(b.largura(ui.painter()), ALTO));
    let resposta = ui.interact(r, ui.scope_id().with(("cartao-botao", id)), if b.ativo { Sense::click() } else { Sense::hover() });
    let ctx = ui.ctx().clone();
    let sobre = b.ativo && resposta.hovered();
    let h = hover(&ctx, ("botao", id), sobre);
    let baixo = b.ativo && resposta.is_pointer_button_down_on();
    if resposta.clicked() && b.ativo {
        marcar_instante(&ctx, ("botao", id));
    }
    let flash = clarao(&ctx, ("botao", id), 0.38);
    let p = ui.painter().clone();
    let r = if baixo { r.translate(vec2(0.0, 0.5)) } else { r };
    let (fundo, traco, cor_texto) = if b.cheio {
        (mix(alfa(b.cor, 0.92), Color32::WHITE, h * 0.35), alfa(b.cor, 0.9), PAPEL)
    } else {
        (mix(PAPEL, mix(PAPEL, b.cor, if baixo { 0.34 } else { 0.2 }), h), mix(alfa(MEDIO, 0.55), alfa(b.cor, 0.95), h), mix(alfa(TINTA, 0.92), b.cor, h * 0.35))
    };
    let k = if b.ativo { 1.0 } else { 0.45 };
    p.rect_filled(r, raio(), alfa(fundo, k));
    if flash > 0.0 {
        p.rect_filled(r, raio(), alfa(b.cor, 0.5 * flash));
    }
    p.rect_stroke(r, raio(), Stroke::new(1.0, alfa(traco, k)), StrokeKind::Inside);
    let c = Pincel { p: &p, o: Pos2::ZERO };
    let letra = b.letra(alfa(cor_texto, k));
    let texto = b.texto.to_uppercase();
    let mut x = r.min.x + PAD;
    if let (Some(i), false) = (b.icone, b.icone_fim) {
        icone(&p, pos2(x + 5.0, r.center().y), i, alfa(cor_texto, k));
        x += LARGURA_ICONE;
    }
    let w_texto = c.texto(&texto, x, r.center().y + 3.6, letra);
    if let (Some(i), true) = (b.icone, b.icone_fim) {
        icone(&p, pos2(x + w_texto + 9.0, r.center().y), i, alfa(cor_texto, k));
    }
    if sobre {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta
}
