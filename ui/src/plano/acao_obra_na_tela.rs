//! O nível mais fundo do zoom num plano: dentro da tela. Vem depois da vista expandida, que abre no próprio palco (`acao_obra_explosao.rs`): a imagem nunca sai do componente que a comporta, e a câmera vai até ele, de frente, até ele encher o palco. Com as camadas abertas, ele está na bandeja das telas, e é de lá que a câmera parte e para onde volta.
//! A roda para a frente aproxima a imagem; para trás, abaixo do tamanho inteiro, sai da visualização e volta ao nível de onde veio: as camadas continuam abertas atrás dela. Esc e «Voltar» fazem o mesmo, e as setas passam às outras telas do plano. Embaixo, a barra com o que se decide sobre ela.

use super::super::acao_movimento::{com_alfa, mistura};
use super::super::caixinha::Lapis;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha};
use super::super::revisao::{self, Alvo, Meio, Peca as PecaRevisao};
use super::obra_figura::Modulo;
use super::obra_mergulho as mergulho;
use super::obra_pecas::{pintar_imagem, por_peca, Extra, PecaObra};
use super::obra_tela::Lugar;
use crate::anatomia::{Camera, Cena};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::{CAMPO, MEDIO, PAPEL, TINTA};
use egui::{vec2, Context, CursorIcon, Id, Key, Painter, PointerButton, Pos2, Rect, Response, Ui, Vec2};

const SEG: f32 = 0.55;
const MARGEM: f32 = 18.0;
const ALTO_BARRA: f32 = 46.0;
const LARGURA_NOTA: f32 = 460.0;
const ZOOM_MAX: f32 = 5.0;
/// Afastou até esta fração da tela inteira: saiu da visualização.
const ZOOM_SAI: f32 = 0.86;
const POR_PIXEL: f32 = 0.0024;
/// Soltou a roda antes de sair: volta sozinho ao tamanho inteiro.
const VOLTA: f32 = 0.18;
/// O que fica do resto da obra atrás da tela.
const VEU: f32 = 0.97;
/// De frente para a face da tela, que olha para +y.
const AZIMUTE_DE_FRENTE: f32 = 90.0;

#[derive(Clone, Debug)]
struct Visto {
    chave: String,
    zoom: f32,
    /// do meio da tela até o meio da vista, em x e z do mundo
    pan: Vec2,
    aberta: bool,
}

fn id(a: &Acao) -> Id {
    Id::unique(("plano-obra-na-tela", &a.referencia))
}

fn ler(ctx: &Context, a: &Acao) -> Option<Visto> {
    ctx.data(|d| d.get_temp::<Visto>(id(a)))
}

/// A tela em que se está.
pub fn aberta(ctx: &Context, a: &Acao) -> Option<String> {
    ler(ctx, a).filter(|v| v.aberta).map(|v| v.chave)
}

/// De 0 (na planta) a 1 (dentro da tela).
pub fn quanto(ctx: &Context, a: &Acao) -> f32 {
    mistura(ctx, ("plano-obra-na-tela", &a.referencia), aberta(ctx, a).is_some(), SEG)
}

/// A tela que a câmera está olhando, mesmo enquanto sai dela.
pub fn olhando(ctx: &Context, a: &Acao) -> Option<String> {
    ler(ctx, a).map(|v| v.chave)
}

fn abrir(ctx: &Context, a: &Acao, chave: &str) {
    ctx.data_mut(|d| d.insert_temp(id(a), Visto { chave: chave.to_string(), zoom: 1.0, pan: Vec2::ZERO, aberta: true }));
    crate::anima::mexer(ctx, 700.0);
}

/// Sai da tela. O nível de onde se entrou continua aberto atrás dela, a construção de perto ou as camadas: cada nível do zoom devolve ao de antes.
pub fn sair(ctx: &Context, a: &Acao) {
    if let Some(v) = ler(ctx, a) {
        ctx.data_mut(|d| d.insert_temp(id(a), Visto { aberta: false, ..v }));
    }
    crate::anima::mexer(ctx, 700.0);
}

/// Vai à tela de qualquer lugar: da ficha, da vista expandida, com o plano aberto ou não. As camadas de outro plano se fecham; as deste ficam abertas atrás da tela.
pub fn ver(ctx: &Context, a: &Acao, plano: &str, chave: &str) {
    if mergulho::expandida(ctx, a).is_some_and(|e| e != plano) {
        mergulho::recolher(ctx, a);
    }
    if mergulho::plano(ctx, a).as_deref() != Some(plano) {
        mergulho::entrar(ctx, a, plano);
    }
    mergulho::escolher(ctx, a, chave);
    abrir(ctx, a, chave);
}

/// Onde a tela cabe no palco: tudo menos a barra.
fn caixa(area: Rect) -> Rect {
    Rect::from_min_max(area.min + vec2(MARGEM, MARGEM), area.max - vec2(MARGEM, MARGEM + ALTO_BARRA))
}

fn escala(l: &Lugar, area: Rect) -> f32 {
    let c = caixa(area);
    (c.width() / l.w).min(c.height() / l.h)
}

/// A câmera lá dentro.
pub fn camera(ctx: &Context, a: &Acao, p: &PecaObra, m: &Modulo, area: Rect) -> Option<Camera> {
    let (l, v) = (p.lugar?, ler(ctx, a)?);
    let [x, y, z] = p.centro(m);
    Some(Camera::nova(AZIMUTE_DE_FRENTE, 0.0, escala(&l, area) * v.zoom, [x + v.pan.x, y, z + v.pan.y], caixa(area).center(), 0.0))
}

/// O olhar anda só até a imagem encostar na borda da vista.
fn prende(pan: Vec2, l: &Lugar, area: Rect, zoom: f32) -> Vec2 {
    let (c, k) = (caixa(area), escala(l, area) * zoom);
    let folga = vec2((l.w - c.width() / k).max(0.0), (l.h - c.height() / k).max(0.0)) * 0.5;
    vec2(pan.x.clamp(-folga.x, folga.x), pan.y.clamp(-folga.y, folga.y))
}

/// A roda aproxima em torno do ponteiro; arrastar anda pela imagem. Abaixo do tamanho inteiro, sai.
pub fn gesto(r: &Response, a: &Acao, l: &Lugar, area: Rect) {
    let ctx = &r.ctx;
    let Some(mut v) = ler(ctx, a) else { return };
    let k0 = escala(l, area);
    let mut rodou = false;
    if r.contains_pointer() {
        let (roda, pinca) = ctx.input(|i| (i.smooth_scroll_delta(), i.zoom_delta()));
        ctx.input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
        let fator = pinca * (roda.y * POR_PIXEL).exp();
        if fator != 1.0 {
            rodou = true;
            let novo = (v.zoom * fator).min(ZOOM_MAX);
            if novo < ZOOM_SAI {
                sair(ctx, a);
                return;
            }
            let meio = caixa(area).center();
            let m = ctx.pointer_hover_pos().unwrap_or(meio) - meio;
            v.pan += vec2(m.x, -m.y) * (1.0 / (k0 * v.zoom) - 1.0 / (k0 * novo));
            v.zoom = novo;
        }
    }
    if !rodou && v.zoom < 1.0 {
        v.zoom = (v.zoom + (1.0 - v.zoom) * VOLTA + 0.002).min(1.0);
        ctx.request_repaint();
    }
    if [PointerButton::Primary, PointerButton::Secondary, PointerButton::Middle].iter().any(|b| r.dragged_by(*b)) {
        let d = r.drag_delta() / (k0 * v.zoom);
        v.pan += vec2(-d.x, d.y);
        ctx.request_repaint();
    }
    v.pan = prende(v.pan, l, area, v.zoom);
    if r.hovered() && v.zoom > 1.0 {
        ctx.set_cursor_icon(if r.dragged() { CursorIcon::Grabbing } else { CursorIcon::Grab });
    }
    ctx.data_mut(|d| d.insert_temp(id(a), v));
}

/// O véu sobre o resto da obra e, em cima dele, só o componente com a imagem.
#[allow(clippy::too_many_arguments)]
pub fn pintar(ctx: &Context, servidor: &str, cam: &Camera, p: &Painter, peca: &PecaObra, m: &Modulo, area: Rect, t: f32) {
    p.rect_filled(area, 0, PAPEL.gamma_multiply(t * VEU));
    let (mut cena, mut extras) = (Cena::default(), Vec::new());
    let chao = peca.lugar.map_or(0.0, |l| l.chao);
    por_peca(&mut cena, &mut extras, peca, (m.x + peca.u, m.y + peca.v, m.base + 0.12 + chao), 1.0, false, 0);
    cena.desenhar(cam, p, |n| {
        let Extra::Imagem(d, cantos, alfa) = &extras[n];
        pintar_imagem(ctx, servidor, d, cam, p, cantos, *alfa);
    });
    if peca.tela.is_some_and(|d| d.figura.is_none()) {
        let meio = caixa(area).center();
        Pincel { p, o: Pos2::ZERO }.texto(tr("Esta tela ainda não tem imagem."), meio.x, meio.y, Letra::nova(13.0, MEDIO.gamma_multiply(t)).meio());
    }
}

/// Passa à tela vizinha no plano.
fn vizinha(ctx: &Context, a: &Acao, irmas: &[&PecaObra], p: &PecaObra, passo: isize) {
    let aqui = irmas.iter().position(|o| o.chave == p.chave).unwrap_or(0) as isize;
    if let Some(o) = usize::try_from(aqui + passo).ok().and_then(|i| irmas.get(i)) {
        mergulho::escolher(ctx, a, &o.chave);
        abrir(ctx, a, &o.chave);
    }
}

/// A barra embaixo do palco: voltar, as outras telas do plano, o nome e o que se decide.
#[allow(clippy::too_many_arguments)]
pub fn barra(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, irmas: &[&PecaObra], p: &PecaObra, area: Rect, t: f32) {
    let k = ((t - 0.6) / 0.4).clamp(0.0, 1.0);
    if k <= 0.0 {
        return;
    }
    let ctx = ui.ctx().clone();
    let (x, y, w) = (area.left() - c.o.x, area.bottom() - ALTO_BARRA - c.o.y, area.width());
    let opaco = ui.opacity();
    ui.set_opacity(opaco * k);
    let pintor = com_alfa(c.p, k);
    let pc = Pincel { p: &pintor, o: c.o };
    pc.ret(x, y, w, ALTO_BARRA, Some(CAMPO), None);
    pc.fio(x, y, x + w, y, TINTA, 1.2);
    let ym = y + ALTO_BARRA * 0.5;
    let mut xx = x + 10.0;
    let voltar = Botao { texto: tr("← Voltar"), tam: 10.0, cheio: false, ativo: true, altura: 28.0, pad_x: 10.0 };
    if botao(ui, c.pt(xx, ym - 14.0), &voltar).clicked() {
        sair(&ctx, a);
    }
    xx += voltar.largura(c.p) + 8.0;
    if irmas.len() > 1 {
        for (texto, passo) in [("‹", -1), ("›", 1)] {
            let b = Botao { texto, tam: 11.0, cheio: false, ativo: true, altura: 28.0, pad_x: 9.0 };
            if botao(ui, c.pt(xx, ym - 14.0), &b).clicked() {
                vizinha(&ctx, a, irmas, p, passo);
            }
            xx += b.largura(c.p) + 4.0;
        }
        if !ctx.egui_wants_keyboard_input() {
            if ctx.input(|i| i.key_pressed(Key::ArrowLeft)) {
                vizinha(&ctx, a, irmas, p, -1);
            }
            if ctx.input(|i| i.key_pressed(Key::ArrowRight)) {
                vizinha(&ctx, a, irmas, p, 1);
            }
        }
        xx += 4.0;
    }
    let mut x_fim = x + w - 10.0;
    if let Some(d) = p.tela.filter(|d| d.pede_aprovacao) {
        let peca = PecaRevisao::detalhe(f, &a.referencia, d);
        let lapis = Lapis { p: pc.p, o: pc.o };
        x_fim -= revisao::medir_partes(c.p, Meio::Tela).1 + 12.0;
        revisao::canto(lapis, ui, alvo, x + w - 10.0, ym + 4.0, &peca, Meio::Tela);
        let alto = revisao::altura_nota(c.p, &peca, LARGURA_NOTA);
        if alto > 0.0 {
            let xn = x + w - LARGURA_NOTA - 20.0;
            pc.ret(xn, y - alto - 14.0, LARGURA_NOTA + 20.0, alto + 14.0, Some(CAMPO), None);
            revisao::nota(lapis, ui, alvo, xn + 10.0, y - alto - 7.0, LARGURA_NOTA, &peca);
        }
    }
    let w_num = pc.texto(&p.numero, xx, ym + 4.0, Letra::nova(10.5, MEDIO));
    let titulo = Letra::nova(12.0, TINTA).forte();
    let x_titulo = xx + w_num + 8.0;
    pc.texto(&c.cortado(&p.nome, (x_fim - x_titulo).max(40.0), &titulo).0, x_titulo, ym + 4.0, titulo);
    ui.set_opacity(opaco);
}
