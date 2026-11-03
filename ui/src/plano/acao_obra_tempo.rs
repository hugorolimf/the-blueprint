//! A quarta dimensão da planta: a ordem de construção. Cada casa é um nível (o que sobe junto), e arrastar a régua mostra a obra naquele instante, antes de acontecer.
//! Não há datas nem durações: o tempo aqui é só a ordem que as dependências impõem. Largar a régua em «Agora» volta à obra de verdade.

use super::super::acao_estilo::{alfa, CORPO};
use super::super::acao_movimento::{desligado, marcar_instante};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Plano};
use super::figura::feito;
use super::obra_figura::Obra;
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{AZUL, MEDIO, TINTA};
use egui::{Context, CursorIcon, Id, Sense, Ui};

pub const ALTO: f32 = 66.0;
const ALTO_TRILHO: f32 = 30.0;
/// Quantos níveis sobem por segundo quando se pede para ver o que vem.
const VELOCIDADE: f32 = 0.7;

fn id(a: &Acao, qual: &str) -> Id {
    Id::unique(("plano-obra-tempo", qual, &a.referencia))
}

/// Onde a régua está; `None` é agora, a obra de verdade.
pub fn instante(ctx: &Context, a: &Acao) -> Option<f32> {
    ctx.data(|d| d.get_temp::<Option<f32>>(id(a, "tau"))).flatten()
}

fn guardar(ctx: &Context, a: &Acao, tau: Option<f32>) {
    ctx.data_mut(|d| d.insert_temp(id(a, "tau"), tau));
    ctx.request_repaint();
}

/// O quanto um módulo deste nível está de pé no instante `tau`.
pub fn subida(tau: f32, nivel: usize) -> f32 {
    (tau - nivel as f32).clamp(0.0, 1.0)
}

/// Em que ponto da ordem a obra está agora: os níveis prontos, mais a fração do primeiro que falta.
pub fn agora(o: &Obra, planos: &[&Plano]) -> f32 {
    for v in 0..o.niveis {
        let no_nivel: Vec<&&Plano> = planos.iter().zip(&o.modulos).filter(|(_, m)| m.nivel == v).map(|(q, _)| q).collect();
        let prontos = no_nivel.iter().filter(|q| feito(q)).count();
        if prontos < no_nivel.len() {
            return v as f32 + prontos as f32 / no_nivel.len() as f32;
        }
    }
    o.niveis as f32
}

fn legenda(o: &Obra, planos: &[&Plano], falta: &[usize], tau: Option<f32>) -> String {
    let refs = |v: usize| planos.iter().zip(&o.modulos).filter(|(_, m)| m.nivel == v).map(|(q, _)| q.referencia.as_str()).collect::<Vec<_>>().join(", ");
    let total = planos.len().to_string();
    let Some(tau) = tau else {
        let feitos = planos.iter().filter(|q| feito(q)).count();
        let n = (agora(o, planos).floor() as usize).min(o.niveis.saturating_sub(1));
        if feitos == planos.len() {
            return tr_com("Agora: tudo de pé, {total} de {total}", &[("total", &total)]);
        }
        let proximos: Vec<&str> = planos.iter().zip(&o.modulos).filter(|(q, m)| m.nivel == n && !feito(q)).map(|(q, _)| q.referencia.as_str()).collect();
        return tr_com("Agora: {n} de {total} de pé. Próximo a subir: {lista}", &[("n", &feitos.to_string()), ("total", &total), ("lista", &proximos.join(", "))]);
    };
    let v = (tau.floor() as usize).min(o.niveis.saturating_sub(1));
    let decidir: usize = planos.iter().zip(&o.modulos).zip(falta).filter(|((_, m), _)| m.nivel == v).map(|(_, n)| *n).sum();
    let base = tr_com("Nível {n} de {total}: sobe {lista}", &[("n", &(v + 1).to_string()), ("total", &o.niveis.to_string()), ("lista", &refs(v))]);
    if decidir > 0 { format!("{base}. {}", tr_com("Antes disso, decidir {n}", &[("n", &decidir.to_string())])) } else { base }
}

/// A régua com o topo em `y`. `falta` é o que falta decidir em cada plano, na mesma ordem de `planos`.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, a: &Acao, o: &Obra, planos: &[&Plano], falta: &[usize], x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let id_toca = id(a, "tocando");
    let mut tocando = ctx.data(|d| d.get_temp::<bool>(id_toca)).unwrap_or(false);
    let mut tau = instante(&ctx, a);
    let n = o.niveis as f32;
    let hoje = agora(o, planos);
    if tocando {
        let dt = ctx.input(|i| i.stable_dt).min(0.1);
        let t = (tau.unwrap_or(hoje) + dt * VELOCIDADE).min(n);
        tau = Some(t);
        tocando = t < n && !desligado();
        ctx.request_repaint();
    }
    let ver = if tocando { tr("Pausar") } else { tr("Ver o que vem") };
    let b = Botao { texto: ver, tam: 10.5, cheio: !tocando && tau.is_none(), ativo: true, altura: ALTO_TRILHO, pad_x: 14.0 };
    let mut xt = x;
    if botao(ui, c.pt(xt, y), &b).clicked() {
        tocando = !tocando;
        if tocando && tau.is_none_or(|t| t >= n) {
            tau = Some(if hoje >= n { 0.0 } else { hoje });
        }
    }
    xt += b.largura(c.p) + 10.0;
    let volta = Botao { texto: tr("Agora"), tam: 10.5, cheio: false, ativo: tau.is_some(), altura: ALTO_TRILHO, pad_x: 12.0 };
    if botao(ui, c.pt(xt, y), &volta).clicked() {
        tau = None;
        tocando = false;
    }
    xt += volta.largura(c.p) + 14.0;
    let wt = (x + w - xt).max(40.0);
    let cw = wt / n.max(1.0);
    for v in 0..o.niveis {
        let xx = xt + v as f32 * cw;
        let (no, prontos) = planos.iter().zip(&o.modulos).filter(|(_, m)| m.nivel == v).fold((Vec::new(), 0), |(mut l, p), (q, _)| {
            l.push(q.referencia.as_str());
            (l, p + usize::from(feito(q)))
        });
        c.ret(xx, y, cw * prontos as f32 / no.len().max(1) as f32, ALTO_TRILHO, Some(alfa(TINTA, 0.22)), None);
        c.ret(xx, y, cw, ALTO_TRILHO, None, Some((alfa(MEDIO, 0.8), 1.0)));
        let l = Letra::nova(11.0, TINTA);
        let texto = c.cortado(&no.join("  "), cw - 12.0, &l).0;
        c.texto(&texto, xx + 8.0, y + 20.0, l);
    }
    let xa = xt + hoje * cw;
    c.fio(xa, y - 5.0, xa, y + ALTO_TRILHO + 5.0, MEDIO, 1.4);
    c.losango(xa, y - 6.0, 4.0, true);
    if let Some(t) = tau {
        let xp = xt + t * cw;
        c.p.rect_filled(c.caixa(xt, y, t * cw, ALTO_TRILHO), 0, alfa(AZUL, 0.18));
        c.fio(xp, y - 6.0, xp, y + ALTO_TRILHO + 6.0, TINTA, 2.2);
    }
    let r = ui.interact(c.caixa(xt, y - 8.0, wt, ALTO_TRILHO + 16.0), ui.scope_id().with(("plano-obra-regua", &a.referencia)), Sense::click_and_drag()).on_hover_cursor(CursorIcon::ResizeHorizontal);
    if r.dragged() || r.clicked() {
        if let Some(pos) = r.interact_pointer_pos() {
            tau = Some(((pos.x - c.pt(xt, 0.0).x) / cw).clamp(0.0, n));
            tocando = false;
        }
    }
    let l = Letra::nova(11.5, if tau.is_some() { AZUL } else { CORPO });
    c.texto(&c.cortado(&legenda(o, planos, falta, tau), wt, &l).0, xt, y + ALTO_TRILHO + 21.0, l);
    ctx.data_mut(|d| d.insert_temp(id_toca, tocando));
    if tau != instante(&ctx, a) {
        marcar_instante(&ctx, ("plano-obra-tempo-mexeu", &a.referencia));
        guardar(&ctx, a, tau);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_nivel_sobe_na_sua_vez() {
        assert_eq!(subida(0.0, 0), 0.0);
        assert_eq!(subida(0.5, 0), 0.5);
        assert_eq!(subida(0.5, 1), 0.0);
        assert_eq!(subida(1.5, 0), 1.0);
        assert_eq!(subida(1.5, 1), 0.5);
    }
}
