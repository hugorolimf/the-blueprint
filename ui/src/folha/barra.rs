//! A barra de uma ação e como ela se desenha ao abrir a folha (`desenharBarras` de web/js/barras.js): o traço da caixa corre em volta
//! (500 ms; a caixa a fazer só aparece), o miolo cresce da esquerda (600 ms, depois de 250), cada losango surge quando o miolo passa
//! por ele, a cota se abre da esquerda (depois de 450) e o selo vem por último (800). As de cima começam antes (até 450 ms).

use super::cena::{Cena, Fio};
use crate::anima::{Abertura, BARRA};
use crate::dominio::Situacao;
use crate::tema::{MEDIO, PAPEL, TINTA};
use egui::{pos2, Painter, Rect, Stroke};

/// A vez desta barra na abertura em curso; sem abertura (ou fora da janela) tudo está pronto.
#[derive(Clone, Copy)]
pub struct Vez(Option<(Abertura, f32)>);

impl Vez {
    /// A vez da ação que ocupa de `topo` a `fundo` (coordenadas da folha).
    pub fn da(c: &Cena, topo: f32, fundo: f32) -> Vez {
        Vez(crate::anima::atual(c.p.ctx()).and_then(|a| a.vez_da_barra(c.pt(0.0, topo).y, c.pt(0.0, fundo).y).map(|v| (a, v))))
    }

    /// O quanto (0 a 1) da peça que começa `atraso` ms depois da vez da barra e dura `dur`.
    pub fn passo(&self, atraso: f32, dur: f32) -> f32 {
        self.0.map_or(1.0, |(a, v)| a.passo(v + atraso, dur, BARRA))
    }
}

/// O pincel com a opacidade `a` (0 a 1) por cima da que ele já tinha.
pub fn opaco(p: &Painter, a: f32) -> Painter {
    let mut q = p.clone();
    q.set_opacity(p.opacity() * a.clamp(0.0, 1.0));
    q
}

/// A caixa da barra: a fazer, só o contorno médio; em andamento e concluída, o papel e o traço de tinta.
pub fn caixa(c: &Cena, x: f32, y: f32, w: f32, h: f32, status: Situacao, vez: Vez) {
    let k = vez.passo(0.0, 500.0);
    match status {
        Situacao::Planejado => Cena { p: &opaco(c.p, k), o: c.o }.moldura(x, y, w, h, MEDIO, 1.2, false),
        _ => {
            c.retangulo(x, y, w, h, Some(PAPEL), None);
            contorno(c, x, y, w, h, Stroke::new(if status == Situacao::Andamento { 1.8 } else { 1.2 }, TINTA), k);
        }
    }
}

/// O traço do retângulo até a fração `k` do perímetro, a partir do canto de cima à esquerda (o `stroke-dashoffset` com `pathLength=1`).
fn contorno(c: &Cena, x: f32, y: f32, w: f32, h: f32, traco: Stroke, k: f32) {
    if k >= 1.0 {
        return c.retangulo(x, y, w, h, None, Some((traco.color, traco.width)));
    }
    let cantos = [(x, y), (x + w, y), (x + w, y + h), (x, y + h), (x, y)];
    let mut resta = 2.0 * (w + h) * k;
    let mut pts = vec![c.pt(x, y)];
    for par in cantos.windows(2) {
        let (a, b) = (par[0], par[1]);
        let lado = (b.0 - a.0).abs() + (b.1 - a.1).abs();
        if resta <= 0.0 || lado <= 0.0 {
            break;
        }
        let t = (resta / lado).min(1.0);
        pts.push(c.pt(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        resta -= lado;
    }
    if pts.len() > 1 {
        c.p.add(egui::Shape::line(pts, traco));
    }
}

/// O miolo: o hachurado da concluída ou o andamento, crescendo da esquerda.
pub fn miolo(c: &Cena, x: f32, y: f32, w: f32, h: f32, status: Situacao, pct: f32, vez: Vez) {
    let k = vez.passo(250.0, 600.0);
    match status {
        Situacao::Concluido => c.hachura(x, y, w * k, h),
        Situacao::Andamento => c.retangulo(x, y, (w * pct).max(0.0) * k, h, Some(TINTA.gamma_multiply(0.9)), None),
        Situacao::Planejado => {}
    }
}

/// O losango de checkpoint em `mx` da barra `x0..x1`: surge crescendo do centro quando o miolo chega nele.
#[allow(clippy::too_many_arguments)]
pub fn losango(c: &Cena, x0: f32, x1: f32, mx: f32, yc: f32, r: f32, feito: bool, vez: Vez) {
    let f = if x1 > x0 { ((mx - x0) / (x1 - x0)).clamp(0.0, 1.0) } else { 0.5 };
    let k = vez.passo(250.0 + 600.0 * f, 260.0);
    if k > 0.0 {
        Cena { p: &opaco(c.p, k), o: c.o }.losango_r(mx, yc, r * (0.2 + 0.8 * k), feito, 1.4);
    }
}

/// O selo ✓ da concluída, por último.
pub fn selo(c: &Cena, cx: f32, cy: f32, vez: Vez) {
    let k = vez.passo(800.0, 300.0);
    if k <= 0.0 {
        return;
    }
    let s = 0.2 + 0.8 * k;
    let ce = Cena { p: &opaco(c.p, k), o: c.o };
    ce.p.circle(ce.pt(cx, cy), 8.0 * s, PAPEL, Stroke::new(1.0, TINTA));
    ce.fios(Fio::Ln, &[(cx - 3.6 * s, cy, cx - 1.0 * s, cy + 3.0 * s), (cx - 1.0 * s, cy + 3.0 * s, cx + 3.6 * s, cy - 3.0 * s)]);
}

/// O pincel que mostra só a parte da cota (`x0..x1`, com folga para as datas) já aberta da esquerda.
pub fn cota(c: &Cena, x0: f32, x1: f32, topo: f32, fundo: f32, vez: Vez) -> Painter {
    let k = vez.passo(450.0, 500.0);
    let (de, ate) = (x0 - 60.0, x1 + 60.0);
    let r = Rect::from_min_max(c.pt(de, topo), pos2(c.pt(de + (ate - de) * k, 0.0).x, c.pt(0.0, fundo).y));
    c.p.with_clip_rect(r.intersect(c.p.clip_rect()))
}

/// O x do checkpoint `i` de `n` na barra `x0..x1`: juntos no fim dela, separados por `passo`, sem sair pela esquerda.
pub fn x_marco(x0: f32, x1: f32, i: usize, n: usize, passo: f32) -> f32 {
    (x1 - 4.0 - n.saturating_sub(i + 1) as f32 * passo).max(x0 + 4.0)
}
