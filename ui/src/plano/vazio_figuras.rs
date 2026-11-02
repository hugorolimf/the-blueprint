//! As figuras dos estados vazios do Planejar, desenhadas em traço isométrico (Anatomy) direto no painter, e o esqueleto de quem ainda carrega.
//! Agentes: a folha com a caixinha que se marca e o agente que aparece. «O que chegou»: a bandeja à espera da folha. Cada uma conta a sua história num laço.

use super::agentes_movimento as mov;
use super::linha::Pincel;
use super::vazio_iso::{Iso, Tons};
use crate::tema::{AZUL, BAIXO, CAMPO, CARTAO, MEDIO, QUADRA, TENUE, TINTA};
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke};

/// Quanto a figura de Agentes mede, em unidades de `k`: largura e altura.
pub const AGENTES_TAM: (f32, f32) = (12.2, 7.8);
/// Idem para a bandeja de «O que chegou».
pub const BANDEJA_TAM: (f32, f32) = (10.4, 6.6);

/// O instante da história, de 0 a 1. Sem movimento fica no fim, com tudo desenhado.
fn tempo(ctx: &egui::Context, periodo: f64) -> f32 {
    if mov::ligado() {
        mov::fase(ctx, periodo)
    } else {
        0.72
    }
}

/// 0 antes de `a`, 1 depois de `b`, suave no meio.
fn degrau(t: f32, a: f32, b: f32) -> f32 {
    let x = ((t - a) / (b - a)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

fn laje(iso: &Iso, lado: f32) {
    let tons = Tons { topo: QUADRA.gamma_multiply(0.32), esq: CARTAO, dir: CAMPO, traco: Stroke::new(1.0, BAIXO) };
    iso.grade(-1.0, lado + 1.0, 1.0, Stroke::new(0.7, BAIXO.gamma_multiply(0.22)));
    iso.caixa((0.0, 0.0, -0.35), (lado, lado, 0.35), tons);
}

/// O ponto da curva de três pontos de controle em `t` (0 a 1).
fn curva(a: (f32, f32, f32), b: (f32, f32, f32), c: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    let m = |p: f32, q: f32, r: f32| (1.0 - t).powi(2) * p + 2.0 * (1.0 - t) * t * q + t * t * r;
    (m(a.0, b.0, c.0), m(a.1, b.1, c.1), m(a.2, b.2, c.2))
}

/// A folha com a caixinha, o agente que ainda não existe e o fio de pontos entre eles. A caixinha se marca, o fio corre e o agente aparece, e tudo volta ao começo.
pub fn agentes(p: &Painter, centro: Pos2, k: f32) {
    let t = tempo(p.ctx(), 7.0);
    let iso = Iso { p, o: centro - vec2(0.0, 3.1 * k), k };
    laje(&iso, 7.0);
    let marca = degrau(t, 0.24, 0.36) * (1.0 - degrau(t, 0.88, 0.97));
    let agente = degrau(t, 0.44, 0.58) * (1.0 - degrau(t, 0.88, 0.97));
    // a folha, de pé sobre um pedestal, virada para o lado esquerdo da câmera
    let papel = Tons { topo: MEDIO.gamma_multiply(0.7), esq: CARTAO, dir: CAMPO, traco: Stroke::new(1.2, TINTA.gamma_multiply(0.9)) };
    iso.caixa((0.5, 3.8, 0.0), (3.2, 0.9, 0.18), Tons { traco: Stroke::new(1.0, BAIXO), ..papel });
    iso.caixa((0.6, 4.1, 0.18), (3.0, 0.1, 3.2), papel);
    let yf = 4.2;
    for (i, comprimento) in [1.5, 1.15, 1.35].into_iter().enumerate() {
        let z = 2.6 - i as f32 * 0.85;
        let cor = if i == 0 { TINTA.gamma_multiply(0.85) } else { MEDIO.gamma_multiply(0.5) };
        let caixinha = [(1.0, yf, z), (1.44, yf, z), (1.44, yf, z + 0.44), (1.0, yf, z + 0.44)];
        iso.face(&caixinha, CAMPO, Stroke::new(1.0, cor));
        iso.linha((1.8, yf, z + 0.22), (1.8 + comprimento, yf, z + 0.22), Stroke::new(1.0, cor));
        iso.linha((1.8, yf, z - 0.12), (1.8 + comprimento * 0.6, yf, z - 0.12), Stroke::new(1.0, cor.gamma_multiply(0.45)));
    }
    // o anel que chama a atenção para a caixinha, antes de ser marcada
    let anel = degrau(t, 0.04, 0.24) * (1.0 - degrau(t, 0.24, 0.3));
    if anel > 0.0 {
        let e = 0.05 + 0.3 * mov::onda(t * 12.0);
        let q = [(1.0 - e, yf, 2.6 - e), (1.44 + e, yf, 2.6 - e), (1.44 + e, yf, 3.04 + e), (1.0 - e, yf, 3.04 + e)];
        iso.face(&q, Color32::TRANSPARENT, Stroke::new(1.2, AZUL.gamma_multiply(0.8 * anel)));
    }
    if marca > 0.0 {
        let traco = Stroke::new(1.9, AZUL.gamma_multiply(marca));
        let (a, b, c) = ((1.08, yf, 2.88), (1.2, yf, 2.68), (1.38, yf, 3.02));
        let ate = degrau(t, 0.24, 0.36);
        let primeiro = (ate * 2.0).min(1.0);
        iso.linha(a, (a.0 + (b.0 - a.0) * primeiro, yf, a.2 + (b.2 - a.2) * primeiro), traco);
        if ate > 0.5 {
            let r = (ate - 0.5) * 2.0;
            iso.linha(b, (b.0 + (c.0 - b.0) * r, yf, b.2 + (c.2 - b.2) * r), traco);
        }
    }
    // o fio de pontos que leva da caixinha ao agente
    let fluxo = degrau(t, 0.34, 0.46) * (1.0 - degrau(t, 0.8, 0.9));
    if fluxo > 0.0 {
        let pontos: Vec<_> = (0..=24).map(|i| curva((1.22, yf, 2.82), (3.2, 3.0, 5.6), (5.4, 1.7, 3.1), i as f32 / 24.0)).collect();
        iso.tracejada(&pontos, Stroke::new(1.3, AZUL.gamma_multiply(0.9 * fluxo)), 3.0, 4.0, -(t * 70.0 % 7.0) * k * 0.4);
    }
    agente_em(&iso, agente, t);
}

/// O agente: tracejado enquanto não existe, sólido quando a caixinha foi marcada.
fn agente_em(iso: &Iso, a: f32, t: f32) {
    let (corpo, cabeca) = (((4.5, 0.8, 0.0), (1.8, 1.8, 1.4)), ((4.95, 1.25, 1.4), (0.9, 0.9, 0.62)));
    let fantasma = Stroke::new(1.1, MEDIO.gamma_multiply(0.7 * (1.0 - a) + 0.15));
    if a < 1.0 {
        iso.caixa_tracejada(corpo.0, corpo.1, fantasma, t * 30.0);
        iso.caixa_tracejada(cabeca.0, cabeca.1, fantasma, t * 30.0);
    }
    if a <= 0.01 {
        return;
    }
    let tons = Tons { topo: QUADRA, esq: TENUE.gamma_multiply(0.7), dir: CAMPO, traco: Stroke::new(1.2, TINTA) }.apagado(a);
    iso.caixa(corpo.0, corpo.1, tons);
    iso.caixa(cabeca.0, cabeca.1, tons);
    let olho = Stroke::new(1.0, AZUL.gamma_multiply(a));
    for x in [5.15, 5.6] {
        iso.face(&[(x, 2.15, 1.62), (x + 0.16, 2.15, 1.62), (x + 0.16, 2.15, 1.82), (x, 2.15, 1.82)], AZUL.gamma_multiply(a * 0.9), olho);
    }
    iso.linha((5.4, 1.7, 2.02), (5.4, 1.7, 2.55), Stroke::new(1.2, TINTA.gamma_multiply(a)));
    iso.circulo((5.4, 1.7), 2.62, 0.09, Stroke::new(1.4, AZUL.gamma_multiply(a)));
}

/// A bandeja vazia, com a folha que vai chegar pairando sobre ela e as ondas de quem espera.
pub fn bandeja(p: &Painter, centro: Pos2, k: f32) {
    let t = tempo(p.ctx(), 5.0);
    let iso = Iso { p, o: centro - vec2(0.0, 2.9 * k), k };
    laje(&iso, 6.0);
    let tons = Tons { topo: QUADRA.gamma_multiply(0.5), esq: CARTAO, dir: CAMPO, traco: Stroke::new(1.1, TINTA.gamma_multiply(0.85)) };
    iso.caixa((0.9, 0.9, 0.0), (4.2, 4.2, 0.15), tons);
    iso.caixa((0.9, 0.9, 0.15), (4.2, 0.12, 0.9), tons);
    iso.caixa((0.9, 0.9, 0.15), (0.12, 4.2, 0.9), tons);
    iso.caixa((0.9, 5.0, 0.15), (4.2, 0.1, 0.3), tons);
    iso.caixa((5.0, 0.9, 0.15), (0.1, 4.2, 0.3), tons);
    // as ondas sobre o fundo da bandeja
    for n in 0..2 {
        let f = (t * 2.0 + n as f32 * 0.5) % 1.0;
        iso.circulo((3.0, 3.0), 0.16, 0.3 + f * 1.5, Stroke::new(1.0, AZUL.gamma_multiply((1.0 - f).powi(2) * 0.7)));
    }
    // onde a folha vai pousar, e a folha pairando sobre a bandeja: sobe e desce devagar, com os pontos de queda embaixo
    let (x0, y0, w, d) = (1.5, 1.25, 2.9, 2.5);
    let z = 2.5 + 0.35 * mov::onda(t);
    let chao = [(x0, y0, 0.17), (x0 + w, y0, 0.17), (x0 + w, y0 + d, 0.17), (x0, y0 + d, 0.17), (x0, y0, 0.17)];
    iso.tracejada(&chao, Stroke::new(1.0, AZUL.gamma_multiply(0.45 + 0.35 * mov::onda(t * 2.0))), 3.0, 3.0, -t * 24.0);
    for (px, py) in [(x0, y0), (x0 + w, y0), (x0 + w, y0 + d), (x0, y0 + d)] {
        iso.tracejada(&[(px, py, z), (px, py, 0.17)], Stroke::new(1.0, MEDIO.gamma_multiply(0.4)), 2.0, 4.0, -t * 40.0);
    }
    let folha = [(x0, y0, z), (x0 + w, y0, z), (x0 + w, y0 + d, z), (x0, y0 + d, z)];
    iso.face(&folha, CARTAO.gamma_multiply(0.85), Stroke::new(1.3, TINTA.gamma_multiply(0.95)));
    for (i, comprimento) in [2.0, 1.55, 1.8, 1.2].into_iter().enumerate() {
        let y = y0 + 0.45 + i as f32 * 0.5;
        let cor = if i == 0 { TINTA.gamma_multiply(0.5 + 0.4 * mov::onda(t * 2.0)) } else { MEDIO.gamma_multiply(0.45) };
        iso.linha((x0 + 0.3, y, z), (x0 + 0.3 + comprimento, y, z), Stroke::new(1.0, cor));
    }
}

/// O esqueleto de uma lista que ainda carrega: `linhas` cartões vazios por onde passa o brilho.
pub fn esqueleto(c: &Pincel, x: f32, y: f32, w: f32, linhas: usize, alto: f32, vao: f32) {
    let ctx = c.p.ctx();
    let fase = mov::fase(ctx, 1.8);
    for i in 0..linhas {
        let topo = y + i as f32 * (alto + vao);
        c.ret(x, topo, w, alto, Some(CARTAO.gamma_multiply(0.55)), Some((BAIXO.gamma_multiply(0.3), 1.0)));
        let cinza = MEDIO.gamma_multiply(0.14);
        c.ret(x + 72.0, topo + alto * 0.3, (w * 0.28).max(80.0), 9.0, Some(cinza), None);
        c.ret(x + 72.0, topo + alto * 0.3 + 16.0, (w * 0.16).max(50.0), 6.0, Some(cinza.gamma_multiply(0.7)), None);
        c.ret(x + 12.0, topo + (alto - 30.0) / 2.0, 44.0, 30.0, Some(cinza), None);
        let r = Rect::from_min_size(pos2(c.o.x + x, c.o.y + topo), vec2(w, alto));
        mov::varredura(c.p, r, (fase + i as f32 * 0.1) % 1.0, TINTA.gamma_multiply(0.12));
    }
}
