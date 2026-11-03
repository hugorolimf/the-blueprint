//! A barra da ação no cronograma (`barraAcao` de `folha.mjs`), com o andamento dentro dela: concluída em verde-água, em andamento em azul com brilho que respira,
//! aguardando plano em âmbar tracejado, planejada só em contorno, e a proposta tracejada em tinta. Os checkpoints são losangos.

use super::{situacao, Letra, Pincel};
use crate::dominio::Situacao;
use crate::i18n::{tr, tr_com};
use crate::folha::barra::x_marco;
use crate::plano::escala::Escala;
use crate::plano::folha::estilo::{com_alfa, pulso, ANDANDO, ESPERA, FEITO, PLANEJADO};
use crate::plano::modelo::{Acao, EstadoEscrita};
use crate::tema::{CARTAO, PAPEL, TINTA};
use egui::Color32;

/// A altura da barra e o canto dela.
pub const ALTA: f32 = 18.0;
const RAIO: f32 = 5.0;

/// A barra da ação no degrau dela, com o andamento e os checkpoints. `y` é o meio da linha.
pub fn barra(c: &Pincel, e: &Escala, a: &Acao, y: f32) {
    let Some((xa, xb)) = e.trecho(a) else { return };
    if a.escrita == EstadoEscrita::Aguardando {
        aguardando(c, xa, xb - xa, y);
    } else {
        cheia(c, a, y, xa, xb);
    }
    marcos(c, e, a, y);
}

/// O lugar do que o agente ainda vai escrever: a caixa âmbar tracejada no degrau da ação, respirando devagar.
fn aguardando(c: &Pincel, x: f32, w: f32, y: f32) {
    let p = pulso(c.p.ctx(), 2.6);
    c.suave(x, y - ALTA / 2.0, w, ALTA, RAIO.min(w / 2.0), Some(com_alfa(ESPERA, 0.05 + 0.06 * p)), None);
    c.suave_tracejado(x, y - ALTA / 2.0, w, ALTA, RAIO.min(w / 2.0), com_alfa(ESPERA, 0.6 + 0.4 * p), 4.0, 3.0);
    c.texto_meio(tr("Aguardando plano"), x + w + 12.0, y, Letra::nova(10.0, ESPERA));
}

fn cheia(c: &Pincel, a: &Acao, y: f32, xa: f32, xb: f32) {
    let xb = xb.max(xa + 3.0);
    let sit = situacao(a);
    let (topo, w) = (y - ALTA / 2.0, xb - xa);
    let raio = RAIO.min(w / 2.0);
    if a.nova {
        c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(TINTA, 0.06)), None);
        c.suave_tracejado(xa, topo, w, ALTA, raio, TINTA, 4.0, 3.0);
    } else {
        match sit {
            Situacao::Concluido => feita(c, a, xa, xb, topo, raio),
            Situacao::Andamento => andando(c, a, (xa, xb), topo, raio),
            Situacao::Planejado => c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(CARTAO, 0.55)), Some((PLANEJADO, 1.2))),
        }
    }
    if a.agentes > 0 {
        agentes(c, a.agentes, xa - 6.0, y);
    }
}

/// «3/5» dentro da barra, em tinta escura sobre a cor cheia.
fn contagem(c: &Pincel, a: &Acao, x: f32, y: f32, largura: f32) {
    if a.numeros.planos > 0 && largura >= 46.0 {
        c.texto_meio(&format!("{}/{}", a.numeros.prontos, a.numeros.planos), x + 8.0, y, Letra::nova(9.5, PAPEL).forte());
    }
}

fn feita(c: &Pincel, a: &Acao, xa: f32, xb: f32, topo: f32, raio: f32) {
    let w = xb - xa;
    c.sombra(xa, topo + 1.0, w, ALTA, raio, com_alfa(Color32::BLACK, 0.32), 8, 2);
    c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(FEITO, 0.82)), Some((FEITO, 1.0)));
    c.reflexo(xa, topo, w, raio, com_alfa(Color32::WHITE, 0.35));
    contagem(c, a, xa, topo + ALTA / 2.0, w);
    if w >= 26.0 {
        c.visto(xb - 11.0, topo + ALTA / 2.0, 8.0, PAPEL, 1.7);
    }
}

fn andando(c: &Pincel, a: &Acao, (xa, xb): (f32, f32), topo: f32, raio: f32) {
    let (w, y, p) = (xb - xa, topo + ALTA / 2.0, pulso(c.p.ctx(), 2.2));
    c.sombra(xa, topo + 1.0, w, ALTA, raio, com_alfa(Color32::BLACK, 0.28), 8, 2);
    c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(ANDANDO, 0.13)), Some((com_alfa(ANDANDO, 0.42 + 0.3 * p), 1.1)));
    if a.numeros.planos == 0 || a.numeros.prontos == 0 {
        return;
    }
    let parte = (a.numeros.prontos as f32 / a.numeros.planos as f32).min(1.0);
    let xp = (xa + w * parte).min(xb);
    let feito = xp - xa;
    if feito <= 3.0 {
        return;
    }
    c.brilho(xa, topo, feito, ALTA, raio, com_alfa(ANDANDO, 0.12 + 0.2 * p), 12);
    c.suave(xa, topo, feito, ALTA, raio.min(feito / 2.0), Some(com_alfa(ANDANDO, 0.9)), None);
    c.reflexo(xa, topo, feito, raio, com_alfa(Color32::WHITE, 0.4));
    if xp < xb - 4.0 {
        c.fio(xp - 1.5, topo + 4.0, xp - 1.5, topo + ALTA - 4.0, com_alfa(Color32::WHITE, 0.5 + 0.4 * p), 1.4);
    }
    contagem(c, a, xa, y, feito);
}

/// «×2» com o ponto azul que pulsa: quantos agentes estão nesta ação.
fn agentes(c: &Pincel, n: u32, x: f32, y: f32) {
    let p = pulso(c.p.ctx(), 1.8);
    let w = c.texto_meio(&tr_com("×{n}", &[("n", &n.to_string())]), x - 12.0, y, Letra::nova(9.5, ANDANDO).forte().fim());
    c.ponto(x - 12.0 - w - 7.0, y, 2.6, ANDANDO, p);
}

/// Os losangos, juntos no fim da barra. Concluído vai cheio de verde-água.
pub(super) fn marcos(c: &Pincel, e: &Escala, a: &Acao, y: f32) {
    let Some((xa, xb)) = e.trecho(a) else { return };
    for (i, k) in a.checkpoints.iter().enumerate() {
        c.marco(x_marco(xa, xb, i, a.checkpoints.len(), 14.0), y, 5.5, k.feito.then_some(FEITO), PAPEL, TINTA);
    }
}
