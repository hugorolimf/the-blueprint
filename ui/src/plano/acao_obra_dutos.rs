//! A dependência, desenhada como uma esteira de fábrica (a ideia vem do Factorio): ela sai de uma construção e entra na que espera por ela, sempre com a ponta da seta na que depende.
//! Enquanto a de origem não fica pronta, a esteira é um fantasma; o cadeado fica só na bandeira de quem espera (`acao_obra_rotulos.rs`), não no chão. Pronta, ela existe e leva o que foi feito: com a de destino em obra, as caixas andam; com ela parada, as caixas se acumulam na porta, esperando.

use super::super::modelo::Plano;
use super::figura::LADO;
use super::forma::indice;
use super::obra_figura::{chave_duto, Modulo, Obra};
use crate::anatomia::{Camada, Cena, Malha, Tinta, V3};
use crate::tema::{CAMPO, MEDIO, QUADRA, TINTA, VERDE};
use egui::{Color32, Stroke};

pub const FOLGA: f32 = 0.35;
const LARGURA: f32 = 0.74;
const GROSSURA: f32 = 0.2;
const PASSO_SETA: f32 = 0.8;
const TAM_SETA: f32 = 0.27;
const TAM_PONTA: f32 = 0.85;
const LADO_CAIXA: f32 = 0.36;
const PASSO_CAIXA: f32 = 1.6;
const CAIXAS_NA_PORTA: usize = 4;

/// Um duto: de quem, para quem, e quão aceso (0 a 1).
#[derive(Clone, Copy, Debug)]
pub struct Duto {
    pub de: usize,
    pub para: usize,
    pub n: usize,
}

pub fn todos(planos: &[&Plano]) -> Vec<Duto> {
    let mut v = Vec::new();
    for (para, q) in planos.iter().enumerate() {
        for de in q.depende_de.iter().filter_map(|d| indice(planos, d)) {
            v.push(Duto { de, para, n: v.len() });
        }
    }
    v
}

/// O caminho no plano, da borda de uma construção à borda da outra: reto quando elas se olham, em L quando estão em diagonal.
fn caminho(de: &Modulo, para: &Modulo) -> Vec<(f32, f32)> {
    let intervalo = |a: f32| (a - FOLGA, a + LADO + FOLGA);
    let ((ax0, ax1), (ay0, ay1)) = (intervalo(de.x), intervalo(de.y));
    let ((bx0, bx1), (by0, by1)) = (intervalo(para.x), intervalo(para.y));
    let (ox0, ox1) = (ax0.max(bx0), ax1.min(bx1));
    let (oy0, oy1) = (ay0.max(by0), ay1.min(by1));
    if ox1 - ox0 > LARGURA {
        let x = (ox0 + ox1) * 0.5;
        return if para.y > de.y { vec![(x, ay1), (x, by0)] } else { vec![(x, ay0), (x, by1)] };
    }
    if oy1 - oy0 > LARGURA {
        let y = (oy0 + oy1) * 0.5;
        return if para.x > de.x { vec![(ax1, y), (bx0, y)] } else { vec![(ax0, y), (bx1, y)] };
    }
    let (y, x) = (de.y + LADO * 0.5, para.x + LADO * 0.5);
    let saida = if para.x > de.x { ax1 } else { ax0 };
    let entrada = if para.y > de.y { by0 } else { by1 };
    vec![(saida, y), (x, y), (x, entrada)]
}

/// Sobe de um patamar ao outro ao longo do caminho.
fn com_altura(caminho: &[(f32, f32)], z0: f32, z1: f32) -> Vec<V3> {
    let total: f32 = caminho.windows(2).map(|p| (p[1].0 - p[0].0).hypot(p[1].1 - p[0].1)).sum::<f32>().max(1e-3);
    let mut andou = 0.0;
    let mut v = vec![[caminho[0].0, caminho[0].1, z0]];
    for p in caminho.windows(2) {
        andou += (p[1].0 - p[0].0).hypot(p[1].1 - p[0].1);
        v.push([p[1].0, p[1].1, z0 + (z1 - z0) * andou / total]);
    }
    v
}

/// A seta em `p`, deitada em cima do duto, apontando em `d`.
fn seta(p: V3, d: (f32, f32), tam: f32) -> Vec<V3> {
    let lado = (-d.1, d.0);
    let ponta = |u: f32, l: f32| [p[0] + d.0 * u + lado.0 * l, p[1] + d.1 * u + lado.1 * l, p[2]];
    vec![ponta(-tam, -tam * 1.1), ponta(0.0, 0.0), ponta(-tam, tam * 1.1)]
}

fn setas(cena: &mut Cena, trecho: &[V3], traco: Stroke, anda: f32) {
    for par in trecho.windows(2) {
        let (a, b) = (par[0], par[1]);
        let compr = (b[0] - a[0]).hypot(b[1] - a[1]);
        if compr < 0.1 {
            continue;
        }
        let d = ((b[0] - a[0]) / compr, (b[1] - a[1]) / compr);
        let mut u = (anda.fract() * PASSO_SETA) + TAM_SETA * 1.5;
        while u < compr - 0.05 {
            let t = u / compr;
            let p = [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t + GROSSURA + 0.01];
            cena.fio(Camada::DutosTopo, seta(p, d, TAM_SETA), traco, None);
            u += PASSO_SETA;
        }
    }
}

/// A ponta grande, na entrada de quem depende: de longe, é ela que diz o sentido.
fn ponta_grande(cena: &mut Cena, a: V3, b: V3, tinta: Tinta, chave: u32) {
    let compr = (b[0] - a[0]).hypot(b[1] - a[1]).max(1e-3);
    let d = ((b[0] - a[0]) / compr, (b[1] - a[1]) / compr);
    let lado = (-d.1, d.0);
    let tam = TAM_PONTA;
    let base = (b[0] - d.0 * tam, b[1] - d.1 * tam);
    let anel = [(b[0], b[1]), (base.0 + lado.0 * tam, base.1 + lado.1 * tam), (base.0 - lado.0 * tam, base.1 - lado.1 * tam)];
    cena.solido(Camada::DutosTopo, Malha::extrusao(&anel, b[2], GROSSURA + 0.05), tinta, chave);
}

/// O ponto a `s` do começo do caminho e o rumo, no plano, do trecho em que ele cai.
fn no_caminho_com_rumo(pontos: &[V3], s: f32) -> (V3, (f32, f32)) {
    let mut falta = s;
    let mut rumo = (1.0, 0.0);
    for par in pontos.windows(2) {
        let (dx, dy) = (par[1][0] - par[0][0], par[1][1] - par[0][1]);
        let compr = dx.hypot(dy).max(1e-3);
        if compr > 1e-3 {
            rumo = (dx / compr, dy / compr);
        }
        if falta <= compr {
            let t = falta / compr;
            return ([par[0][0] + dx * t, par[0][1] + dy * t, par[0][2] + (par[1][2] - par[0][2]) * t], rumo);
        }
        falta -= compr;
    }
    (pontos[pontos.len() - 1], rumo)
}

/// O ponto a `s` do começo do caminho.
fn no_caminho(pontos: &[V3], s: f32) -> V3 {
    no_caminho_com_rumo(pontos, s).0
}

fn compr_total(pontos: &[V3]) -> f32 {
    pontos.windows(2).map(|p| (p[1][0] - p[0][0]).hypot(p[1][1] - p[0][1])).sum()
}

/// As caixas em cima da esteira. Andando, vão uma atrás da outra; paradas, encostam na porta de quem ainda não começou.
fn caixas(cena: &mut Cena, pontos: &[V3], anda: Option<f32>, alfa: f32, chave: u32) {
    let total = compr_total(pontos);
    let lugares: Vec<f32> = match anda {
        Some(fase) => (0..((total / PASSO_CAIXA) as usize + 1)).map(|k| (k as f32 + fase) * PASSO_CAIXA).filter(|s| *s < total - TAM_PONTA).collect(),
        None => (0..CAIXAS_NA_PORTA).map(|k| total - TAM_PONTA - 0.3 - k as f32 * (LADO_CAIXA + 0.08)).filter(|s| *s > 0.0).collect(),
    };
    let meio = LADO_CAIXA * 0.5;
    let tinta = Tinta::estado(VERDE).com_alfa(alfa);
    for s in lugares {
        let p = no_caminho(pontos, s);
        cena.solido(Camada::DutosTopo, Malha::caixa([p[0] - meio, p[1] - meio, p[2] + GROSSURA], [LADO_CAIXA, LADO_CAIXA, LADO_CAIXA * 0.8]), tinta, chave);
    }
}

/// Por onde a esteira `u` passa.
pub fn pontos(o: &Obra, u: &Duto) -> Vec<V3> {
    let (m_de, m_para) = (&o.modulos[u.de], &o.modulos[u.para]);
    com_altura(&caminho(m_de, m_para), m_de.base + 0.12, m_para.base + 0.12)
}

/// Põe na cena a esteira `u`. `aceso` vai de 0 a 1 quando uma das pontas está sob o mouse; `alfa` apaga o que não importa agora; `de` e `para` dizem quanto cada ponta está de pé neste instante.
#[allow(clippy::too_many_arguments)]
pub fn por(cena: &mut Cena, o: &Obra, u: &Duto, aceso: f32, alfa: f32, de: f32, para: f32, fase: f32) {
    esteira(cena, &pontos(o, u), de >= 1.0, para, aceso, alfa, fase, chave_duto(u.n));
}

/// A esteira por `pontos`, até quem depende. `pronta`: o que vem por ela já existe; `para`: quanto o destino está de pé.
#[allow(clippy::too_many_arguments)]
pub fn esteira(cena: &mut Cena, pontos: &[V3], pronta: bool, para: f32, aceso: f32, alfa: f32, fase: f32, chave: u32) {
    let cor: Color32 = if aceso > 0.5 || pronta { TINTA } else { MEDIO };
    let largura = LARGURA + 0.14 * aceso;
    let tinta = if pronta { Tinta::de(CAMPO, QUADRA, cor).com_traco(cor, 1.1 + 0.6 * aceso) } else { Tinta::fantasma(cor) }.com_alfa(alfa);
    for par in pontos.windows(2) {
        cena.solido(Camada::Dutos, Malha::viga(par[0], par[1], largura, GROSSURA), tinta, chave);
    }
    for dobra in &pontos[1..pontos.len() - 1] {
        cena.solido(Camada::Dutos, Malha::cilindro((dobra[0], dobra[1]), largura * 0.5, dobra[2], GROSSURA), tinta, chave);
    }
    let correndo = pronta && para > 0.0 && para < 1.0;
    setas(cena, pontos, Stroke::new(1.6 + 0.4 * aceso, cor.gamma_multiply(alfa)), if correndo { fase } else { 0.0 });
    let (a, b) = (pontos[pontos.len() - 2], pontos[pontos.len() - 1]);
    ponta_grande(cena, a, b, Tinta::estado(cor).com_traco(cor, 1.2).com_alfa(alfa), chave);
    if pronta && para < 1.0 {
        caixas(cena, pontos, correndo.then_some(fase), alfa, chave);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(x: f32, y: f32) -> Modulo {
        Modulo { x, y, h: 2.0, base: 0.0, nivel: 0 }
    }

    #[test]
    fn de_frente_e_reto_e_em_diagonal_e_em_l() {
        assert_eq!(caminho(&m(0.0, 0.0), &m(1.0, 8.0)).len(), 2);
        assert_eq!(caminho(&m(0.0, 0.0), &m(9.0, 1.0)).len(), 2);
        assert_eq!(caminho(&m(0.0, 0.0), &m(8.0, 8.0)).len(), 3);
    }

    #[test]
    fn o_ponto_no_caminho_anda_pelas_dobras() {
        let p = com_altura(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0)], 0.0, 0.0);
        assert_eq!(no_caminho(&p, 2.0), [2.0, 0.0, 0.0]);
        assert_eq!(no_caminho(&p, 6.0), [4.0, 2.0, 0.0]);
        assert_eq!(no_caminho(&p, 99.0), [4.0, 4.0, 0.0]);
    }

    #[test]
    fn o_duto_sobe_ate_o_patamar_de_quem_depende() {
        let p = com_altura(&[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0)], 0.0, 1.0);
        assert_eq!(p[0][2], 0.0);
        assert!((p[1][2] - 0.5).abs() < 1e-5);
        assert_eq!(p[2][2], 1.0);
    }
}
