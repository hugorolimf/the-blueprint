//! A barra da ação no cronograma (`barraAcao` de `folha.mjs`), com o andamento dentro dela: concluída em verde-água, em andamento em azul com brilho que respira,
//! aguardando plano em âmbar tracejado, planejada só em contorno, e a proposta tracejada em tinta. Os checkpoints são losangos.

use super::{situacao, Letra, Pincel};
use crate::documento::dia;
use crate::dominio::Situacao;
use crate::i18n::{tr, tr_com};
use crate::plano::escala::{curta, Escala};
use crate::plano::folha::estilo::{com_alfa, pulso, ANDANDO, ESPERA, FEITO, PLANEJADO};
use crate::plano::modelo::{Acao, EstadoEscrita};
use crate::tema::{CARTAO, MEDIO, PAPEL, TINTA};
use egui::Color32;

/// A altura da barra e o canto dela.
pub const ALTA: f32 = 18.0;
const RAIO: f32 = 5.0;

/// A barra da ação na data, cortada em `XR`, com o andamento, os checkpoints e as datas ao lado (ou antes, quando não cabe). `y` é o meio da linha.
pub fn barra(c: &Pincel, e: &Escala, a: &Acao, y: f32) {
    if a.escrita == EstadoEscrita::Aguardando {
        aguardando(c, e, y);
    } else if a.sem_data {
        c.texto_meio(tr("sem data"), e.x_sem_data(), y, Letra::nova(10.0, MEDIO));
    } else if let (Some(de), Some(ate)) = (dia(&a.de), dia(&a.ate)) {
        datada(c, e, a, y, de, ate);
    }
    marcos(c, e, a, y);
}

/// O lugar do que o agente ainda vai escrever: a caixa âmbar tracejada ao lado do hoje, respirando devagar.
fn aguardando(c: &Pincel, e: &Escala, y: f32) {
    let (x, p) = (e.x(e.hoje) + 14.0, pulso(c.p.ctx(), 2.6));
    c.suave(x, y - ALTA / 2.0, 96.0, ALTA, RAIO, Some(com_alfa(ESPERA, 0.05 + 0.06 * p)), None);
    c.suave_tracejado(x, y - ALTA / 2.0, 96.0, ALTA, RAIO, com_alfa(ESPERA, 0.6 + 0.4 * p), 4.0, 3.0);
    c.texto_meio(tr("Aguardando plano"), x + 108.0, y, Letra::nova(10.0, ESPERA));
}

fn datada(c: &Pincel, e: &Escala, a: &Acao, y: f32, de: i64, ate: i64) {
    let rotulo = format!("{} → {}", curta(&a.de), curta(&a.ate));
    let sit = situacao(a);
    let letra = Letra::nova(10.0, if sit == Situacao::Concluido { com_alfa(MEDIO, 0.8) } else { MEDIO });
    let Some((xa, xb)) = e.recorte(de, ate).map(|(xa, xb)| (xa, xb.max(xa + 3.0))) else {
        c.texto_meio(&rotulo, e.x0 + 8.0, y, letra);
        return;
    };
    let (topo, w) = (y - ALTA / 2.0, xb - xa);
    let raio = RAIO.min(w / 2.0);
    if a.nova {
        c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(TINTA, 0.06)), None);
        c.suave_tracejado(xa, topo, w, ALTA, raio, TINTA, 4.0, 3.0);
    } else {
        match sit {
            Situacao::Concluido => feita(c, a, xa, xb, topo, raio),
            Situacao::Andamento => andando(c, e, a, (xa, xb), topo, raio, (de, ate)),
            Situacao::Planejado => c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(CARTAO, 0.55)), Some((PLANEJADO, 1.2))),
        }
    }
    if a.agentes > 0 {
        agentes(c, a.agentes, xa - 6.0, y);
    }
    if xb + 10.0 + c.largura(&rotulo, &letra) < e.x1 {
        c.texto_meio(&rotulo, xb + 10.0, y, letra);
    } else {
        c.texto_meio(&rotulo, xa - 8.0, y, letra.fim());
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

fn andando(c: &Pincel, e: &Escala, a: &Acao, (xa, xb): (f32, f32), topo: f32, raio: f32, (de, ate): (i64, i64)) {
    let (w, y, p) = (xb - xa, topo + ALTA / 2.0, pulso(c.p.ctx(), 2.2));
    c.sombra(xa, topo + 1.0, w, ALTA, raio, com_alfa(Color32::BLACK, 0.28), 8, 2);
    c.suave(xa, topo, w, ALTA, raio, Some(com_alfa(ANDANDO, 0.13)), Some((com_alfa(ANDANDO, 0.42 + 0.3 * p), 1.1)));
    if a.numeros.planos == 0 || a.numeros.prontos == 0 {
        return;
    }
    // o andamento conta sobre a barra inteira: a parte recortada à esquerda também é feita
    let parte = (a.numeros.prontos as f32 / a.numeros.planos as f32).min(1.0);
    let (ini, fim) = (e.x(de.min(ate)), e.x(de.max(ate)));
    let xp = (ini + (fim - ini) * parte).min(xb);
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

/// Os losangos: na data do checkpoint; sem data, no fim da ação. Concluído vai cheio de verde-água.
pub(super) fn marcos(c: &Pincel, e: &Escala, a: &Acao, y: f32) {
    for k in &a.checkpoints {
        let x = e.x_iso(&k.data).or_else(|| if a.sem_data { None } else { e.x_iso(&a.ate) });
        if let Some(x) = x.filter(|x| *x >= e.x0 && *x <= e.x1) {
            c.marco(x, y, 5.5, k.feito.then_some(FEITO), PAPEL, TINTA);
        }
    }
}
