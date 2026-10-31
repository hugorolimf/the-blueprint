//! O `.btn` da página: texto em caixa alta com espaçamento, borda fina, e as cores que se invertem ao passar o mouse.

use super::{borda, Texto};
use crate::fonte::Peso;
use crate::tema::{rgba, MEDIO, PAPEL, TINTA};
use egui::{vec2, Color32, Painter, Pos2, Rect, Response, Sense, Ui};

/// O `.btn` da página: texto em caixa alta com espaçamento, borda fina; ao passar o mouse inverte as cores. `cheio`: o botão principal.
pub struct Botao<'a> {
    pub texto: &'a str,
    pub tam: f32,
    pub cheio: bool,
    pub ativo: bool,
    pub altura: f32,
    pub pad_x: f32,
}

impl<'a> Botao<'a> {
    /// O `.btn` da entrada (46 px de altura, 18 px dos lados, 11 px de letra).
    pub fn entrada(texto: &'a str) -> Self {
        Botao { texto, tam: 11.0, cheio: false, ativo: true, altura: 46.0, pad_x: 18.0 }
    }
    /// O `.btn` das barras de ferramenta (34 px de altura, 15 px dos lados).
    pub fn barra(texto: &'a str) -> Self {
        Botao { texto, tam: 11.0, cheio: false, ativo: true, altura: 34.0, pad_x: 15.0 }
    }
    pub fn tam(mut self, t: f32) -> Self {
        self.tam = t;
        self
    }
    pub fn cheio(mut self, c: bool) -> Self {
        self.cheio = c;
        self
    }
    pub fn ativo(mut self, a: bool) -> Self {
        self.ativo = a;
        self
    }
    fn texto_pronto(&self) -> Texto<'static> {
        Texto::novo(self.texto.to_uppercase(), self.tam).trilha(0.1).peso(if self.cheio { Peso::Negrito } else { Peso::Normal })
    }
    pub fn largura(&self, p: &Painter) -> f32 {
        self.texto_pronto().largura(p) + self.pad_x * 2.0 + 2.0
    }
}

/// Desenha o botão com o canto de cima à esquerda em `pos` e devolve a resposta (clique, mouse em cima).
pub fn botao(ui: &mut Ui, pos: Pos2, b: &Botao) -> Response {
    let largura = b.largura(ui.painter());
    botao_com_largura(ui, pos, largura, b)
}

/// O mesmo botão com uma largura imposta (os que ocupam a linha toda).
pub fn botao_com_largura(ui: &mut Ui, pos: Pos2, largura: f32, b: &Botao) -> Response {
    let p = ui.painter().clone();
    let r = Rect::from_min_size(pos, vec2(largura, b.altura));
    let resposta = ui.interact(r, ui.scope_id().with(("btn", b.texto, pos.x as i32, pos.y as i32)), if b.ativo { Sense::click() } else { Sense::hover() });
    let invertido = b.ativo && (resposta.hovered() || resposta.has_focus());
    let (fundo, cor) = if b.cheio && invertido { (Color32::WHITE, PAPEL) } else if b.cheio || invertido { (TINTA, PAPEL) } else { (rgba(0x0c3a78, 0xcc), TINTA) };
    let opaco = |c: Color32| if b.ativo { c } else { c.gamma_multiply(0.45) };
    p.rect_filled(r, 0, opaco(fundo));
    borda(&p, r, opaco(MEDIO));
    let t = b.texto_pronto().cor(opaco(cor));
    let largura = t.largura(&p);
    t.na_linha(&p, r.min.x + (r.width() - largura) / 2.0, r.min.y, r.height());
    if b.ativo && resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta
}

