//! A observação da peça: só existe em Revisar. Enquanto se escreve é um campo que desliza para aparecer (e recolhe ao enviar ou cancelar);
//! depois é o recado preso à peça. O rascunho fica na memória do egui, por peça. `altura_nota` devolve a altura do quadro, já animada.

use crate::plano::acao_estilo::{alfa, cartao, CORPO};
use crate::plano::acao_movimento::mistura;
use crate::plano::caixinha::Lapis;
use crate::plano::cartao_botao::{botao, Botao, Icone};
use super::{enviar, Alvo, EstadoPeca, Peca};
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema::{CAMPO, MEDIO, TINTA};
use egui::{vec2, FontId, Id, Rect, Sense, Ui};

const PASSO_NOTA: f32 = 15.0;
const LINHAS_NOTA: usize = 3;
const NOTA_MARGEM: f32 = 10.0;
/// O campo de três linhas e a fileira de botões.
const ALTURA_COMPOR: f32 = LINHAS_NOTA as f32 * PASSO_NOTA + NOTA_MARGEM + 38.0;
const SEG: f32 = 0.3;

#[derive(Clone, Default)]
struct Rascunho {
    aberto: bool,
    focado: bool,
    texto: String,
}

fn id_rascunho(chave: &str) -> Id {
    Id::unique(("plano-revisao-rascunho", chave))
}

fn rascunho(ctx: &egui::Context, chave: &str) -> Rascunho {
    ctx.data(|d| d.get_temp::<Rascunho>(id_rascunho(chave))).unwrap_or_default()
}

/// Se a observação desta peça está sendo escrita agora.
pub fn compondo(ctx: &egui::Context, chave: &str) -> bool {
    rascunho(ctx, chave).aberto
}

/// A observação só existe em Revisar: escrevendo, ou já mandada.
pub fn mostra_nota(estado: EstadoPeca, compondo: bool) -> bool {
    compondo || estado == EstadoPeca::Revisar
}

/// Abre ou fecha o campo da observação desta peça.
pub(super) fn alternar(ctx: &egui::Context, chave: &str, aberto: bool) {
    let mut r = rascunho(ctx, chave);
    r.aberto = aberto;
    r.focado = false;
    ctx.data_mut(|d| d.insert_temp(id_rascunho(chave), r));
}

fn texto_nota(obs: &str, w: f32) -> Texto<'_> {
    Texto::novo(obs, 10.5).caixa(w - 26.0).lh(PASSO_NOTA).linhas(LINHAS_NOTA)
}

/// A altura do recado já mandado; zero se a peça não está em Revisar.
fn altura_fixa(p: &egui::Painter, peca: &Peca, w: f32) -> f32 {
    if peca.estado() != EstadoPeca::Revisar {
        return 0.0;
    }
    let obs = peca.revisao.map_or("", |r| r.observacao.as_str());
    let linhas = if obs.is_empty() { 1 } else { texto_nota(obs, w).galeria(p).rows.len() };
    linhas as f32 * PASSO_NOTA + NOTA_MARGEM + 2.0
}

/// O quanto (0 a 1) do campo de escrever já está aberto.
fn abertura(ctx: &egui::Context, chave: &str) -> f32 {
    mistura(ctx, ("nota", chave), compondo(ctx, chave), SEG)
}

/// Quanto a observação ocupa, para empilhar antes de desenhar: o recado, ou o campo que desliza para abrir e para fechar.
pub fn altura_nota(p: &egui::Painter, peca: &Peca, w: f32) -> f32 {
    let (fixa, k) = (altura_fixa(p, peca, w), abertura(p.ctx(), &peca.chave));
    fixa + (ALTURA_COMPOR - fixa) * k
}

/// A observação da peça em `(x, y)`: o campo enquanto se escreve, depois o recado preso à peça. Devolve a altura ocupada, a mesma de `altura_nota`.
pub fn nota(l: Lapis, ui: &mut Ui, alvo: Alvo, x: f32, y: f32, w: f32, peca: &Peca) -> f32 {
    let h = altura_nota(l.p, peca, w);
    if h <= 0.5 {
        return 0.0;
    }
    let k = abertura(ui.ctx(), &peca.chave);
    if k > 0.01 {
        compor(l, ui, alvo, (x, y, w, h), k, peca);
        return h;
    }
    recado(l, ui, (x, y, w, h), peca);
    h
}

fn recado(l: Lapis, ui: &mut Ui, (x, y, w, h): (f32, f32, f32, f32), peca: &Peca) {
    let obs = peca.revisao.map_or("", |r| r.observacao.as_str());
    let area = Rect::from_min_size(l.pt(x, y), vec2(w, h - 2.0));
    cartao(l.p, area, CAMPO, alfa(MEDIO, 0.5));
    if obs.is_empty() {
        Texto::novo(tr("Diga o que mudar…"), 10.5).lh(PASSO_NOTA).cor(MEDIO).em(l.p, l.pt(x + 13.0, y + 5.0));
        return;
    }
    let t = texto_nota(obs, w).cor(CORPO);
    t.em(l.p, l.pt(x + 13.0, y + 5.0));
    // cortada em três linhas: o resto aparece ao passar o mouse
    if t.galeria(l.p).elided {
        ui.interact(area, ui.scope_id().with(("plano-revisao-nota", &peca.chave)), Sense::hover()).on_hover_text(obs);
    }
}

fn compor(l: Lapis, ui: &mut Ui, alvo: Alvo, (x, y, w, h): (f32, f32, f32, f32), k: f32, peca: &Peca) {
    let ctx = ui.ctx().clone();
    let moldura = Rect::from_min_size(l.pt(x, y), vec2(w, h.max(1.0) - 2.0));
    // o campo cresce de cima para baixo: o que passa da altura de agora fica cortado, e o conteúdo sobe suave
    let clip = moldura.intersect(ui.clip_rect());
    let recorte = l.p.with_clip_rect(clip);
    cartao(&recorte, moldura, CAMPO, alfa(MEDIO, 0.5));
    if !compondo(&ctx, &peca.chave) {
        return;
    }
    ui.scope(|ui| {
        ui.set_clip_rect(clip);
        let p = ui.painter().clone();
        campo_e_botoes(Lapis { p: &p, o: l.o }, ui, alvo, (x, y, w), (1.0 - k) * -10.0, peca);
    });
}

fn campo_e_botoes(q: Lapis, ui: &mut Ui, alvo: Alvo, (x, y, w): (f32, f32, f32), desliza: f32, peca: &Peca) {
    let ctx = ui.ctx().clone();
    let mut r = rascunho(&ctx, &peca.chave);
    let area = Rect::from_min_size(q.pt(x + 13.0, y + 7.0 + desliza), vec2(w - 26.0, LINHAS_NOTA as f32 * PASSO_NOTA));
    let dica = egui::RichText::new(tr("Diga o que mudar…")).size(10.5).color(MEDIO);
    let editor = egui::TextEdit::multiline(&mut r.texto).id(id_rascunho(&peca.chave).with("campo")).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(10.5, familia(Peso::Normal))).text_color(TINTA).hint_text(dica).desired_rows(LINHAS_NOTA).desired_width(area.width());
    let campo = ui.put(area, editor);
    if !r.focado {
        campo.request_focus();
        r.focado = true;
    }
    let linha = y + LINHAS_NOTA as f32 * PASSO_NOTA + NOTA_MARGEM + 6.0 + desliza;
    let cheio = !r.texto.trim().is_empty();
    let b_enviar = Botao::novo(tr("Enviar")).icone(Icone::Enviar).cheio(true).ativo(cheio);
    let largura_enviar = b_enviar.largura(q.p);
    let chave = peca.chave.as_str();
    if botao(ui, ("enviar", chave), q.pt(x + 13.0, linha), &b_enviar).clicked() && cheio {
        enviar(&ctx, alvo, peca, "revisar", &r.texto);
        ctx.data_mut(|d| d.remove::<Rascunho>(id_rascunho(&peca.chave)));
        return;
    }
    if botao(ui, ("cancelar", chave), q.pt(x + 13.0 + largura_enviar + 8.0, linha), &Botao::novo(tr("Cancelar"))).clicked() {
        ctx.data_mut(|d| d.remove::<Rascunho>(id_rascunho(&peca.chave)));
        return;
    }
    ctx.data_mut(|d| d.insert_temp(id_rascunho(&peca.chave), r));
}
