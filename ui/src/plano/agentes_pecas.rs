//! As peças pequenas que as telas Agentes, «O que chegou» e a escrita ao vivo repetem: o cartão com o traço de cor do estado, a ficha, o ponto que pulsa,
//! a barra de andamento e os botões (o primário cheio, o de texto discreto). Só desenham; o que fazer com o clique é de quem chama.

use super::agentes_movimento as mov;
use super::linha::{Letra, Pincel};
use crate::tema::{BAIXO, CAMPO, MEDIO, TINTA};
use egui::{pos2, vec2, Color32, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui};

/// A altura de um botão pequeno, a mesma de `caixinha::botao_mini`.
pub const ALTO_BOTAO: f32 = 24.0;

/// O cartão: fundo, fio fino e, se houver, o traço de cor do estado ao longo do topo.
#[allow(clippy::too_many_arguments)]
pub fn cartao(c: &Pincel, x: f32, y: f32, w: f32, h: f32, fundo: Color32, borda: Color32, traco: Option<Color32>) {
    c.ret(x, y, w, h, Some(fundo), Some((borda, 1.0)));
    if let Some(cor) = traco {
        c.ret(x, y, w, 2.0, Some(cor), None);
    }
}

/// A ficha: texto pequeno num contorno fino (um contador, um ramo, uma área). Devolve a largura. `y` é o topo.
pub fn ficha(c: &Pincel, x: f32, y: f32, texto: &str, cor: Color32) -> f32 {
    let letra = Letra::nova(9.5, cor).trilha(0.04);
    let w = (c.largura(texto, &letra) + 12.0).round();
    c.ret(x, y, w, 16.0, Some(cor.gamma_multiply(0.1)), Some((cor.gamma_multiply(0.55), 1.0)));
    c.texto(texto, x + 6.0, y + 11.5, letra);
    w
}

/// O ponto de situação: cheio, e com dois anéis que se abrem e somem enquanto `vivo`.
pub fn ponto(c: &Pincel, x: f32, y: f32, cor: Color32, vivo: bool) {
    if vivo {
        let fase = mov::fase(c.p.ctx(), 1.8);
        for k in 0..2 {
            let f = (fase + k as f32 * 0.5) % 1.0;
            c.p.circle_stroke(c.pt(x, y), 3.5 + f * 8.0, Stroke::new(1.0, cor.gamma_multiply((1.0 - f).powi(2) * 0.85)));
        }
    }
    c.p.circle_filled(c.pt(x, y), 3.5, cor);
}

/// O ponto vazado: o que está desligado.
pub fn ponto_vazado(c: &Pincel, x: f32, y: f32, cor: Color32) {
    c.p.circle_stroke(c.pt(x, y), 3.5, Stroke::new(1.2, cor));
}

/// Como a barra de andamento se enche.
#[derive(Clone, Copy)]
pub enum Modo {
    /// quanto já foi feito (0 a 1)
    Fracao(f32),
    /// em curso, sem saber quanto falta: um trecho corre pela barra
    Corre,
    /// ainda não começou: só o trilho tracejado
    Espera,
}

/// A barra fina de andamento. `y` é o topo e `h` costuma ser 4.
pub fn barra(c: &Pincel, x: f32, y: f32, w: f32, h: f32, modo: Modo, cor: Color32) {
    let r = c.caixa(x, y, w, h);
    c.p.rect_filled(r, 0, CAMPO);
    match modo {
        Modo::Fracao(f) => {
            let parte = Rect::from_min_size(r.min, vec2(w * f.clamp(0.0, 1.0), h));
            c.p.rect_filled(parte, 0, cor);
        }
        Modo::Corre => {
            c.p.rect_filled(r, 0, cor.gamma_multiply(0.18));
            let corpo = (w * 0.3).max(24.0);
            let x0 = r.left() + (w + corpo) * mov::fase(c.p.ctx(), 1.7) - corpo;
            let trecho = Rect::from_min_max(pos2(x0.max(r.left()), r.top()), pos2((x0 + corpo).min(r.right()), r.bottom()));
            if trecho.width() > 0.0 {
                c.p.rect_filled(trecho, 0, cor);
            }
        }
        Modo::Espera => {
            let meio = [pos2(r.left() + 1.0, r.center().y), pos2(r.right() - 1.0, r.center().y)];
            c.p.extend(Shape::dashed_line(&meio, Stroke::new(1.0, cor.gamma_multiply(0.6)), 3.0, 3.0));
        }
    }
    c.p.rect_stroke(r, 0, Stroke::new(1.0, BAIXO.gamma_multiply(0.45)), StrokeKind::Inside);
}

/// A largura do botão de texto de `texto`.
pub fn largura_botao_texto(c: &Pincel, texto: &str) -> f32 {
    c.largura(&texto.to_uppercase(), &Letra::nova(10.0, MEDIO).trilha(0.1)) + 20.0
}

/// O botão discreto: só o contorno na cor da ação, que se enche de leve ao passar o mouse. O primário cheio é o `planta::botao` com `cheio: true`.
pub fn botao_texto(ui: &mut Ui, c: &Pincel, x: f32, y: f32, texto: &str, cor: Color32) -> Response {
    let letra = Letra::nova(10.0, cor).trilha(0.1);
    let rotulo = texto.to_uppercase();
    let (w, r) = (largura_botao_texto(c, texto), c.caixa(x, y, largura_botao_texto(c, texto), ALTO_BOTAO));
    let resposta = ui.interact(r, ui.scope_id().with(("btn-texto", texto, x as i32, y as i32)), Sense::click());
    let sobre = ui.ctx().animate_bool_with_time(resposta.id.with("sobre"), resposta.hovered(), 0.14);
    c.p.rect_filled(r, 0, cor.gamma_multiply(0.16 * sobre));
    c.p.rect_stroke(r, 0, Stroke::new(1.0, cor.gamma_multiply(0.4 + 0.5 * sobre)), StrokeKind::Inside);
    c.texto(&rotulo, x + (w - c.largura(&rotulo, &letra)) / 2.0 + 0.5, y + 16.0, Letra { cor: if sobre > 0.5 { TINTA } else { cor }, ..letra });
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta
}
