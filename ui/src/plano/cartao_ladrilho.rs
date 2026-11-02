//! O cartão de uma figura de detalhe, na prancha da ação e na fita da prévia: a miniatura numa moldura com marcas de canto, o número, o título
//! e o selo do estado. Com o mouse em cima o cartão se ergue, a sombra cresce e a figura dá um zoom suave; sem a figura ainda, o esqueleto brilha.

use super::acao_estilo::{alfa, cartao, mix, sombra, CORPO};
use super::acao_movimento::{com_alfa, hover};
use super::caixinha::Lapis;
use super::figura;
use super::linha::{Letra, Pincel};
use super::modelo::Figura;
use super::revisao::{self, EstadoPeca};
use crate::tema::{BAIXO, CARTAO, MEDIO, PAPEL, QUADRA, TINTA};
use egui::{vec2, Rect, Response, Sense, Stroke, StrokeKind, Ui};

/// O pé do cartão: número e selo numa linha, o título na outra.
pub const RODAPE: f32 = 44.0;
const ZOOM: f32 = 0.07;
const SOBE: f32 = 3.0;

pub struct Ladrilho<'a> {
    /// `1.2-01`: a ação e o detalhe
    pub rotulo: String,
    pub titulo: &'a str,
    pub figura: Option<&'a Figura>,
    /// `None` se a figura não pede aprovação
    pub estado: Option<EstadoPeca>,
    /// o que está aberto na folha de detalhe (só a fita usa)
    pub atual: bool,
}

/// As marcas de canto da moldura (os cantos de uma prancha de planta), que aparecem com o mouse em cima.
fn marcas(c: &Pincel, r: Rect, forca: f32) {
    if forca < 0.02 {
        return;
    }
    let cor = alfa(TINTA, 0.8 * forca);
    for (canto, dx, dy) in [(r.left_top(), 1.0, 1.0), (r.right_top(), -1.0, 1.0), (r.left_bottom(), 1.0, -1.0), (r.right_bottom(), -1.0, -1.0)] {
        let a = canto + vec2(3.0 * dx, 3.0 * dy);
        c.p.line_segment([a, a + vec2(7.0 * dx, 0.0)], Stroke::new(1.0, cor));
        c.p.line_segment([a, a + vec2(0.0, 7.0 * dy)], Stroke::new(1.0, cor));
    }
}

/// Desenha o cartão em `(x, y, w, h)` (`h` é a altura da figura; o pé soma `RODAPE`). `entra` (0 a 1) é a chegada em cascata. Devolve a resposta do clique.
pub fn desenhar(ui: &Ui, c: &Pincel, servidor: &str, chave: &str, l: &Ladrilho, (x, y, w, h): (f32, f32, f32, f32), entra: f32) -> Response {
    let ctx = ui.ctx().clone();
    let resposta = ui.interact(c.caixa(x, y, w, h + RODAPE), ui.scope_id().with(("plano-ladrilho", chave)), Sense::click());
    let sobe = hover(&ctx, ("ladrilho", chave), resposta.hovered());
    let y = y - SOBE * sobe + (1.0 - entra) * 10.0;
    let pintor = com_alfa(c.p, entra);
    let c = Pincel { p: &pintor, o: c.o };
    let area = c.caixa(x, y, w, h + RODAPE);
    sombra(c.p, area, 0.9 * sobe);
    let traco = if l.atual { alfa(TINTA, 0.95) } else { mix(alfa(BAIXO, 0.5), alfa(TINTA, 0.9), sobe) };
    cartao(c.p, area, mix(PAPEL, QUADRA, sobe), traco);
    let moldura = c.caixa(x + 6.0, y + 6.0, w - 12.0, h - 6.0);
    c.p.rect_filled(moldura, 2.0, CARTAO);
    let dentro = c.p.with_clip_rect(moldura.intersect(c.p.clip_rect()));
    let zoom = 1.0 + ZOOM * sobe;
    let miolo = Rect::from_center_size(moldura.center(), (moldura.size() - vec2(8.0, 8.0)) * zoom);
    let nome = if l.titulo.is_empty() { l.rotulo.as_str() } else { l.titulo };
    figura::desenhar(&dentro, servidor, l.figura, nome, miolo);
    c.p.rect_stroke(moldura, 2.0, Stroke::new(1.0, alfa(BAIXO, 0.4)), StrokeKind::Inside);
    marcas(&c, moldura, sobe);
    if let Some(e) = l.estado {
        let lapis = Lapis { p: c.p, o: c.o };
        revisao::situacao(lapis, x + w - 8.0, y + h + 17.0, e, true, true);
    }
    let letra_id = Letra::nova(9.5, MEDIO);
    c.texto(&c.cortado(&l.rotulo, (w - 100.0).max(30.0), &letra_id).0, x + 10.0, y + h + 17.0, letra_id);
    let letra = Letra::nova(10.5, mix(CORPO, TINTA, sobe));
    let (titulo, cortou) = c.cortado(nome, w - 20.0, &letra);
    c.texto(&titulo, x + 10.0, y + h + 34.0, letra);
    if resposta.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if cortou {
        return resposta.on_hover_text(nome);
    }
    resposta
}
