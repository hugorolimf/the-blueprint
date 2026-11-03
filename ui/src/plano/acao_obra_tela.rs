//! O componente Tela: o único jeito de uma tela aparecer na planta. A figura de verdade numa moldura, de pé, com a cor dizendo como está a aprovação; o clique leva até ela e abre a ficha.
//! Ela compõe a construção: um plano com tela é desenhado com ela, de longe e de perto, sempre no mesmo lugar. Os moldes ficam em `CATALOGO`: variante nova é uma linha lá, e o plano a chama pelo nome em `variante`. Sem nome, a proporção da figura escolhe.

use super::super::modelo::{Detalhe, Plano};
use super::figura::LADO;
use super::obra_figura::ALTO;
use super::obra_formas::{Forma, Papel, Partes};
use super::obra_pecas::Extra;
use crate::anatomia::{Camada, Cena, Malha, Tinta, V3};
use crate::tema::TINTA;
use egui::Color32;

/// Quantas telas de pé cabem numa construção; a ficha lista todas.
pub const MAX: usize = 3;
const VAO: f32 = 0.12;
/// A parede atrás da tela: fina, para a tela ficar na frente dela.
const RECUO_PAREDE: f32 = 0.12;
const FUNDO_PAREDE: f32 = 0.18;
const ALTO_PISO: f32 = 0.12;
const V_NA_PAREDE: f32 = 0.42;
const LARGURA_NA_PAREDE: f32 = LADO - 0.24;
const ALTO_NA_PAREDE: f32 = 2.05;
const PE_MINIMO: f32 = 0.34;
/// O painel em cima de uma construção que é outra coisa.
const FRACAO_SOB_PAINEL: f32 = 0.5;
const LARGURA_NO_ALTO: f32 = 3.0;
const ALTO_NO_ALTO: f32 = 1.3;
const PE_NO_ALTO: f32 = 0.14;

/// O que segura a tela.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pe {
    /// a haste e o pé de um monitor
    Monitor,
    /// dois postes, como uma placa de estrada
    Postes,
    /// uma haste fina e uma base pequena
    Apoio,
}

/// Um molde do catálogo. `escala` é o quanto ele ocupa do lugar que tem.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Molde {
    pub nome: &'static str,
    pub pe: Pe,
    pub moldura: f32,
    pub espessura: f32,
    pub escala: f32,
}

/// Os moldes que a planta conhece. A mesma lista está em `VARIANTES`, de `src/plano/validar.rs`, e no guia.
pub const CATALOGO: [Molde; 5] = [
    Molde { nome: "janela", pe: Pe::Monitor, moldura: 0.05, espessura: 0.1, escala: 1.0 },
    Molde { nome: "celular", pe: Pe::Apoio, moldura: 0.06, espessura: 0.08, escala: 0.95 },
    Molde { nome: "dialogo", pe: Pe::Apoio, moldura: 0.05, espessura: 0.08, escala: 0.72 },
    Molde { nome: "faixa", pe: Pe::Postes, moldura: 0.04, espessura: 0.08, escala: 1.0 },
    Molde { nome: "quadro", pe: Pe::Postes, moldura: 0.07, espessura: 0.06, escala: 1.0 },
];

const PROPORCAO_PADRAO: f32 = 4.0 / 3.0;
const RETRATO_ATE: f32 = 0.8;
const FAIXA_DESDE: f32 = 2.6;

fn proporcao(d: &Detalhe) -> f32 {
    d.figura.as_ref().and_then(|f| Some(f.largura? as f32 / f.altura.filter(|a| *a > 0)? as f32)).filter(|p| *p > 0.0).unwrap_or(PROPORCAO_PADRAO)
}

fn molde(nome: &str) -> &'static Molde {
    CATALOGO.iter().find(|m| m.nome == nome).unwrap_or(&CATALOGO[0])
}

/// O molde que o plano pediu; sem pedido, ou com um nome que a planta não conhece, a proporção da figura escolhe.
pub fn molde_de(d: &Detalhe) -> &'static Molde {
    if let Some(m) = CATALOGO.iter().find(|m| m.nome == d.variante) {
        return m;
    }
    match proporcao(d) {
        p if p <= RETRATO_ATE => molde("celular"),
        p if p >= FAIXA_DESDE => molde("faixa"),
        _ => molde("janela"),
    }
}

/// Como a construção é composta com as telas dela.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Composicao {
    /// sem tela: a forma do plano, inteira
    Sem,
    /// o plano é a tela: o piso, a parede que sobe com a obra e as telas de pé na frente dela
    Parede,
    /// o plano é outra coisa (dados, serviço): a construção mais baixa, com o painel em cima
    NoAlto,
}

pub fn composicao(q: &Plano, tem_tela: bool) -> Composicao {
    if !tem_tela {
        Composicao::Sem
    } else if matches!(Forma::do_plano(q), Forma::Tela | Forma::Padrao) {
        Composicao::Parede
    } else {
        Composicao::NoAlto
    }
}

/// O corpo da construção nesta composição, com a altura `h`.
pub fn partes(comp: Composicao, q: &Plano, x: f32, y: f32, z: f32, h: f32) -> Partes {
    match comp {
        Composicao::Sem => Forma::do_plano(q).partes(x, y, z, h),
        Composicao::NoAlto => Forma::do_plano(q).partes(x, y, z, h * FRACAO_SOB_PAINEL),
        Composicao::Parede => vec![
            (Malha::laje([x + RECUO_PAREDE, y + RECUO_PAREDE, z], [LADO - RECUO_PAREDE * 2.0, LADO - RECUO_PAREDE * 2.0, ALTO_PISO], 0.1), Papel::Corpo),
            (Malha::caixa([x + RECUO_PAREDE, y + RECUO_PAREDE, z], [LADO - RECUO_PAREDE * 2.0, FUNDO_PAREDE, h.max(ALTO_PISO)]), Papel::Corpo),
        ],
    }
}

/// O lugar de uma tela, relativo ao lote: o canto em `(u, v)`, de pé em `chao`, com o pé `pe`, o tamanho `w` × `h`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Lugar {
    pub u: f32,
    pub v: f32,
    pub chao: f32,
    pub pe: f32,
    pub w: f32,
    pub h: f32,
}

fn caber(p: f32, w: f32, h: f32) -> (f32, f32) {
    if w / p <= h { (w, w / p) } else { (h * p, h) }
}

/// Onde as telas ficam, na ordem. Cabem até `MAX`, lado a lado, cada uma na sua proporção.
pub fn fileira(telas: &[&Detalhe], comp: Composicao) -> Vec<Lugar> {
    let n = telas.len().min(MAX);
    if n == 0 {
        return Vec::new();
    }
    let no_alto = comp == Composicao::NoAlto;
    let (util, teto) = if no_alto { (LARGURA_NO_ALTO, ALTO_NO_ALTO) } else { (LARGURA_NA_PAREDE, ALTO_NA_PAREDE) };
    let celula = (util - VAO * (n as f32 - 1.0)) / n as f32;
    let inicio = (LADO - util) * 0.5;
    telas.iter().take(n).enumerate().map(|(i, d)| {
        let e = molde_de(d).escala;
        let (w, h) = caber(proporcao(d), celula * e, teto * e);
        let u = inicio + i as f32 * (celula + VAO) + (celula - w) * 0.5;
        if no_alto {
            Lugar { u, v: LADO * 0.5 - 0.05, chao: ALTO * FRACAO_SOB_PAINEL, pe: PE_NO_ALTO, w, h }
        } else {
            Lugar { u, v: V_NA_PAREDE, chao: ALTO_PISO, pe: ((ALTO - ALTO_PISO - h) * 0.5).max(PE_MINIMO), w, h }
        }
    }).collect()
}

/// O que segura a tela, de `z` até `alto`.
fn por_pe(cena: &mut Cena, m: &Molde, (x, y, z): (f32, f32, f32), w: f32, alto: f32, tinta: Tinta, chave: u32) {
    let e = m.espessura;
    match m.pe {
        Pe::Monitor => {
            let base = (w * 0.46).min(1.3);
            cena.solido(Camada::Pecas, Malha::laje([x + (w - base) * 0.5, y - 0.2, z], [base, 0.4 + e, 0.05], 0.06), tinta, chave);
            cena.solido(Camada::Pecas, Malha::caixa([x + w * 0.5 - 0.1, y + e * 0.2, z], [0.2, e * 0.6, alto + 0.04]), tinta, chave);
        }
        Pe::Postes => {
            for fracao in [0.12, 0.88] {
                cena.solido(Camada::Pecas, Malha::cilindro((x + w * fracao, y + e * 0.5), 0.035, z, alto + 0.04), tinta, chave);
            }
        }
        Pe::Apoio => {
            cena.solido(Camada::Pecas, Malha::laje([x + w * 0.2, y - 0.12, z], [w * 0.6, 0.24 + e, 0.05], 0.05), tinta, chave);
            cena.solido(Camada::Pecas, Malha::caixa([x + w * 0.5 - 0.05, y + e * 0.3, z], [0.1, e * 0.4, alto + 0.04]), tinta, chave);
        }
    }
}

/// Põe a tela na cena, com o pé em `(x, y, z)`. A escolhida leva o traço forte.
#[allow(clippy::too_many_arguments)]
pub fn por<'a>(cena: &mut Cena, extras: &mut Vec<Extra<'a>>, d: &'a Detalhe, l: &Lugar, (x, y, z): (f32, f32, f32), cor: Color32, sel: bool, alfa: f32, chave: u32) {
    let m = molde_de(d);
    let tinta = Tinta::estado(cor).com_traco(if sel { TINTA } else { cor }, if sel { 2.0 } else { 1.1 }).com_alfa(alfa);
    por_pe(cena, m, (x, y, z), l.w, l.pe, Tinta::estado(cor).com_traco(cor, 1.0).com_alfa(alfa), chave);
    let z0 = z + l.pe;
    cena.solido(Camada::Pecas, Malha::caixa([x, y, z0], [l.w, m.espessura, l.h]), tinta, chave);
    let (f, b) = (y + m.espessura, m.moldura);
    let cantos: [V3; 4] = [[x + b, f, z0 + l.h - b], [x + l.w - b, f, z0 + l.h - b], [x + l.w - b, f, z0 + b], [x + b, f, z0 + b]];
    extras.push(Extra::Imagem(d, cantos, alfa));
    cena.extra(Camada::Pecas, [x + l.w * 0.5, f + 0.01, z0 + l.h * 0.5], extras.len() - 1, chave, 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plano::modelo::Figura;

    fn tela(l: u32, a: u32, variante: &str) -> Detalhe {
        Detalhe { variante: variante.into(), figura: Some(Figura { largura: Some(l), altura: Some(a), ..Figura::default() }), ..Detalhe::default() }
    }

    #[test]
    fn a_proporcao_escolhe_quando_o_plano_nao_pede() {
        assert_eq!(molde_de(&tela(960, 490, "")).nome, "janela");
        assert_eq!(molde_de(&tela(390, 844, "")).nome, "celular");
        assert_eq!(molde_de(&tela(1200, 232, "")).nome, "faixa");
        assert_eq!(molde_de(&tela(1200, 232, "quadro")).nome, "quadro");
        assert_eq!(molde_de(&tela(960, 490, "nao-existe")).nome, "janela");
    }

    #[test]
    fn a_tela_cabe_no_lote_e_guarda_a_proporcao() {
        let (a, b, c, d) = (tela(960, 490, ""), tela(390, 844, ""), tela(1200, 232, ""), tela(960, 640, ""));
        for telas in [vec![&a], vec![&a, &b], vec![&a, &b, &c], vec![&a, &b, &c, &d]] {
            for comp in [Composicao::Parede, Composicao::NoAlto] {
                let v = fileira(&telas, comp);
                assert_eq!(v.len(), telas.len().min(MAX));
                for (l, t) in v.iter().zip(&telas) {
                    assert!(l.u >= 0.0 && l.u + l.w <= LADO + 1e-3, "{l:?}");
                    assert!((l.w / l.h - proporcao(t)).abs() < 1e-3, "{l:?}");
                }
            }
        }
    }

    #[test]
    fn o_plano_que_e_tela_vira_parede_e_o_outro_leva_o_painel() {
        let q = |forma: &str| Plano { forma: forma.into(), ..Plano::default() };
        assert_eq!(composicao(&q("tela"), true), Composicao::Parede);
        assert_eq!(composicao(&q("dados"), true), Composicao::NoAlto);
        assert_eq!(composicao(&q("dados"), false), Composicao::Sem);
    }
}
