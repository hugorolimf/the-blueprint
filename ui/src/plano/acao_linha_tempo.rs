//! Os checkpoints da ação aberta, como na folha original (`itens` de `folha/acoes.rs`): o losango, cheio quando feito, e o nome.

use super::super::linha::Pincel;
use super::super::modelo::{Acao, Checkpoint};
use super::super::revisao::Alvo;
use super::{bloco, rotulo};
use crate::i18n::tr_com;
use crate::menu::{self, Alvo as AlvoMenu};
use crate::tema::{MEDIO, TINTA};
use crate::planta::Texto;
use egui::{Color32, Painter, Sense, Ui};

const ALTO_LINHA: f32 = 29.0;
const ALTO_ROTULO: f32 = 22.0;
const X_MARCA: f32 = 6.5;
const X_TEXTO: f32 = 24.0;
const RAIO_MARCA: f32 = 6.5;
const PASSO: f32 = super::super::acao_estilo::PASSO_LETRA;

fn texto_de<'a>(k: &'a Checkpoint, w: f32, cor: Color32) -> Texto<'a> {
    bloco(&k.nome, super::super::acao_estilo::LETRA, w - X_TEXTO, PASSO).cor(cor)
}

/// A linha de um checkpoint: cresce com o nome, que nunca é cortado.
fn alto_linha(p: &Painter, k: &Checkpoint, w: f32) -> f32 {
    (texto_de(k, w, TINTA).galeria(p).rows.len().max(1) as f32 * PASSO + 8.0).max(ALTO_LINHA)
}

/// Quanto os checkpoints ocupam. Zero sem checkpoints.
pub fn altura(p: &Painter, lista: &[&Checkpoint], w: f32) -> f32 {
    if lista.is_empty() {
        return 0.0;
    }
    ALTO_ROTULO + lista.iter().map(|k| alto_linha(p, k, w)).sum::<f32>()
}

/// Os checkpoints com o rótulo em `(x, y)`.
pub fn desenhar(ui: &Ui, c: &Pincel, alvo: Alvo, a: &Acao, lista: &[&Checkpoint], (x, y, w): (f32, f32, f32)) {
    if lista.is_empty() {
        return;
    }
    let feitos = lista.iter().filter(|k| k.feito).count();
    rotulo(c, x, y, &tr_com("Checkpoints · {feitos}/{total}", &[("feitos", &feitos.to_string()), ("total", &lista.len().to_string())]), w);
    let mut yy = y + ALTO_ROTULO - 7.0;
    for k in lista {
        linha(ui, c, alvo, a, k, (x, yy, w));
        yy += alto_linha(c.p, k, w);
    }
}

fn linha(ui: &Ui, c: &Pincel, alvo: Alvo, a: &Acao, k: &Checkpoint, (x, y, w): (f32, f32, f32)) {
    let area = c.caixa(x, y, w, alto_linha(c.p, k, w));
    menu::marcar(ui, area, || AlvoMenu::Item { ws: alvo.ws.into(), bp: alvo.bp.into(), acao: a.id.clone(), tipo: "checkpoints", id: k.id.clone(), texto: k.nome.clone(), feito: k.feito });
    let sobre = ui.interact(area, ui.scope_id().with(("plano-checkpoint", &k.id)), Sense::hover()).hovered();
    c.losango(x + X_MARCA, y + ALTO_LINHA / 2.0, RAIO_MARCA, k.feito);
    let cor = if sobre { Color32::WHITE } else if k.feito { MEDIO } else { TINTA };
    texto_de(k, w, cor).em(c.p, c.pt(x + X_TEXTO, y + 4.0));
}
