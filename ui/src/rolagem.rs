//! A régua de rolagem das páginas, a mesma da interface web (`web/js/rolagem.js`): um trilho fino à direita, o polegar hachurado que se arrasta e a porcentagem do lado. A barra do egui fica escondida.
//! Clicar no trilho leva o meio do polegar até o ponto; arrastar o polegar rola em proporção. As setas, PageUp, PageDown, Home e End também andam.

use crate::fonte::{familia, Peso};
use crate::tema::{BAIXO, MEDIO, PAPEL, TINTA};
use egui::epaint::Shadow;
use egui::scroll_area::{ScrollAreaOutput, ScrollBarVisibility};
use egui::{pos2, vec2, Color32, Context, CursorIcon, FontId, Id, Key, Painter, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui};

const POLEGAR_MIN: f32 = 40.0;
const PASSO_SETA: f32 = 80.0;
const PASSO_PAGINA: f32 = 0.9;
/// A sobra que não conta como página maior que a janela.
const SOBRA: f32 = 2.0;

/// As medidas da régua: a da página e a compacta, de dentro de um painel.
struct Medidas {
    direita: f32,
    margem: f32,
    largura: f32,
    x_trilho: f32,
    x_polegar: f32,
    w_polegar: f32,
    /// a da página mostra a porcentagem, tem as pontas no trilho e responde ao teclado
    da_pagina: bool,
}

const DA_PAGINA: Medidas = Medidas { direita: 10.0, margem: 84.0, largura: 30.0, x_trilho: 14.0, x_polegar: 4.0, w_polegar: 22.0, da_pagina: true };
const COMPACTA: Medidas = Medidas { direita: 0.0, margem: 0.0, largura: 12.0, x_trilho: 5.0, x_polegar: 1.0, w_polegar: 10.0, da_pagina: false };

/// A área de rolagem de uma página, com a régua. Devolve o que o conteúdo devolveu.
pub fn mostrar<R>(ui: &mut Ui, area: ScrollArea, conteudo: impl FnOnce(&mut Ui) -> R) -> R {
    com_regua(ui, area, conteudo, &DA_PAGINA)
}

/// A mesma régua, fina, para o que rola dentro de um painel.
pub fn mostrar_compacta<R>(ui: &mut Ui, area: ScrollArea, conteudo: impl FnOnce(&mut Ui) -> R) -> R {
    com_regua(ui, area, conteudo, &COMPACTA)
}

/// O egui só deixa mudar a posição antes de mostrar a área: o que a régua pede num quadro vale no seguinte.
fn com_regua<R>(ui: &mut Ui, area: ScrollArea, conteudo: impl FnOnce(&mut Ui) -> R, m: &Medidas) -> R {
    let pedido = ui.scope_id().with("rolagem-pedido");
    let area = match ui.ctx().data_mut(|d| d.remove_temp::<f32>(pedido)) {
        Some(y) => area.vertical_scroll_offset(y),
        None => area,
    };
    let saida = area.scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden).show(ui, conteudo);
    regua(ui, saida, m, pedido)
}

/// As setas de cima e de baixo e as teclas de página, quando ninguém está digitando. O menu aberto já consumiu as dele.
fn teclado(ctx: &Context, offset: f32, janela: f32, curso: f32) -> Option<f32> {
    if ctx.egui_wants_keyboard_input() {
        return None;
    }
    ctx.input(|i| {
        if !i.modifiers.is_none() {
            return None;
        }
        let teclas = [(Key::ArrowDown, offset + PASSO_SETA), (Key::ArrowUp, offset - PASSO_SETA), (Key::PageDown, offset + janela * PASSO_PAGINA), (Key::PageUp, offset - janela * PASSO_PAGINA), (Key::Home, 0.0), (Key::End, curso)];
        teclas.iter().find(|(t, _)| i.key_pressed(*t)).map(|(_, y)| *y)
    })
}

fn hachura(p: &Painter, r: Rect, cor: Color32) {
    let recorte = p.with_clip_rect(r.intersect(p.clip_rect()));
    let mut d = -r.height();
    while d < r.width() {
        recorte.line_segment([pos2(r.left() + d, r.bottom()), pos2(r.left() + d + r.height(), r.top())], Stroke::new(1.4, cor));
        d += 6.0;
    }
}

fn pintar_trilho(p: &Painter, r: Rect, m: &Medidas) {
    let x = r.left() + m.x_trilho;
    p.rect_filled(Rect::from_min_max(pos2(x, r.top()), pos2(x + 2.0, r.bottom())), 0, BAIXO.gamma_multiply(0.8));
    if m.da_pagina {
        for y in [r.top() - 1.0, r.bottom() - 1.0] {
            p.rect_filled(Rect::from_min_size(pos2(x - 6.0, y), vec2(14.0, 2.0)), 0, TINTA);
        }
    }
}

fn pintar_polegar(p: &Painter, polegar: Rect, m: &Medidas, aceso: bool) {
    if m.da_pagina {
        p.add(Shadow { offset: [0, 4], blur: 14, spread: 0, color: Color32::from_rgba_unmultiplied(0x02, 0x13, 0x2e, 0x99) }.as_shape(polegar, 0));
    }
    p.rect_filled(polegar, 0, if aceso { TINTA } else { PAPEL });
    hachura(p, polegar, if aceso { PAPEL.gamma_multiply(0.6) } else { MEDIO.gamma_multiply(0.33) });
    p.rect_stroke(polegar, 0, Stroke::new(1.5, TINTA), StrokeKind::Inside);
}

/// A porcentagem, à esquerda da régua, acompanhando o polegar.
fn pintar_rotulo(p: &Painter, r: Rect, y: f32, frac: f32) {
    let texto = format!("{}%", (frac * 100.0).round() as u32);
    let galeria = p.layout_no_wrap(texto, FontId::new(10.0, familia(Peso::Normal)), TINTA);
    let caixa = Rect::from_min_size(pos2(r.right() - 34.0 - galeria.size().x - 14.0, y), galeria.size() + vec2(14.0, 6.0));
    p.rect_filled(caixa, 0, PAPEL);
    p.rect_stroke(caixa, 0, Stroke::new(1.0, MEDIO), StrokeKind::Inside);
    p.galley(caixa.min + vec2(7.0, 3.0), galeria, TINTA);
}

/// Desenha a régua em cima da área e aplica o que a mão e o teclado pediram. Página que cabe na janela não tem régua.
fn regua<R>(ui: &mut Ui, s: ScrollAreaOutput<R>, m: &Medidas, pedido: Id) -> R {
    let ScrollAreaOutput { inner, id, state, content_size, inner_rect, .. } = s;
    let offset = state.clamped_offset().y;
    let (janela, total) = (inner_rect.height(), content_size.y);
    if total <= janela + SOBRA {
        return inner;
    }
    let ctx = ui.ctx().clone();
    let area = Rect::from_min_max(pos2(inner_rect.right() - m.direita - m.largura, inner_rect.top() + m.margem), pos2(inner_rect.right() - m.direita, inner_rect.bottom() - m.margem));
    let (trilho, curso) = (area.height().max(1.0), total - janela);
    let alto = POLEGAR_MIN.max(trilho * janela / total).min(trilho);
    let livre = (trilho - alto).max(1.0);
    let frac = (offset / curso).clamp(0.0, 1.0);
    let polegar = Rect::from_min_size(pos2(area.left() + m.x_polegar, area.top() + livre * frac), vec2(m.w_polegar, alto));
    let r = ui.interact(area, id.with("regua"), Sense::click_and_drag());
    let ao_mexer = id.with("regua-polegar");
    let mut novo = if m.da_pagina { teclado(&ctx, offset, janela, curso) } else { None };
    if r.is_pointer_button_down_on() {
        if ctx.input(|i| i.pointer.primary_pressed()) {
            let pegou = r.interact_pointer_pos().is_some_and(|q| polegar.expand(2.0).contains(q));
            ctx.data_mut(|d| d.insert_temp(ao_mexer, pegou));
        }
        if ctx.data(|d| d.get_temp::<bool>(ao_mexer)).unwrap_or(false) {
            novo = Some(offset + ctx.input(|i| i.pointer.delta().y) * curso / livre);
        } else if let Some(q) = r.interact_pointer_pos() {
            novo = Some((q.y - area.top() - alto * 0.5) / livre * curso);
        }
    }
    let segura = r.is_pointer_button_down_on();
    let sobre_polegar = r.hover_pos().is_some_and(|q| polegar.contains(q));
    if segura || r.hovered() {
        ctx.set_cursor_icon(if segura { CursorIcon::Grabbing } else if sobre_polegar { CursorIcon::Grab } else { CursorIcon::Default });
    }
    // em cima de tudo que a página desenhou, mas sem sair da área
    let p = ui.painter_at(inner_rect);
    pintar_trilho(&p, area, m);
    pintar_polegar(&p, polegar, m, segura || sobre_polegar);
    if m.da_pagina {
        pintar_rotulo(&p, area, (polegar.center().y - 12.0).max(area.top()), frac);
    }
    if let Some(y) = novo {
        ctx.data_mut(|d| d.insert_temp(pedido, y.clamp(0.0, curso)));
        ctx.request_repaint();
    }
    inner
}
