//! As dependências que vêm de outra ação. Como nas pranchas, o fio que continua em outra folha termina numa placa com o número dela: aqui, uma esteira curta sai da placa e entra na construção. Clicar na placa leva até lá.

use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha, Plano};
use super::dutos::{esteira, FOLGA};
use super::figura::{feito, LADO};
use super::forma::indice;
use super::obra_cena::Quadro;
use super::obra_figura::{Modulo, Obra};
use super::obra_sumico::{CLICA, SUMIU};
use crate::anatomia::{Camera, Cena, V3};
use crate::i18n::tr_com;
use crate::tema::{CAMPO, MEDIO, QUADRA, TINTA, VERDE};
use egui::{epaint::PathShape, pos2, CursorIcon, Pos2, Rect, Sense, Stroke, Ui};

/// O comprimento da esteira que vem de fora, em unidades do mundo.
pub const ENTRADA: f32 = 2.6;
const ALTO_PLACA: f32 = 20.0;
const PONTA_PLACA: f32 = 7.0;
const VAO_PLACAS: f32 = 3.0;

/// O plano `r`, em qualquer ação da folha, e a ação dele.
pub fn achar<'f>(f: &'f Folha, r: &str) -> Option<(&'f Acao, &'f Plano)> {
    f.projetos.iter().flat_map(|p| &p.acoes).find_map(|a| a.planos.iter().find(|q| q.referencia == r).map(|q| (a, q)))
}

/// O plano `r` já está pronto? Vale para qualquer ação da folha.
pub fn pronto(f: &Folha, r: &str) -> bool {
    achar(f, r).is_some_and(|(_, q)| feito(q))
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fora {
    pub para: usize,
    pub referencia: String,
    pub pronto: bool,
    pub nome_acao: String,
    /// entre os de fora deste plano: qual é este e quantos são
    pub lugar: (usize, usize),
}

pub fn todos(f: &Folha, planos: &[&Plano]) -> Vec<Fora> {
    let mut v = Vec::new();
    for (para, q) in planos.iter().enumerate() {
        let de_fora: Vec<(&Acao, &Plano)> = q.depende_de.iter().filter(|d| indice(planos, d).is_none()).filter_map(|d| achar(f, d)).collect();
        let total = de_fora.len();
        for (n, (a, o)) in de_fora.into_iter().enumerate() {
            v.push(Fora { para, referencia: o.referencia.clone(), pronto: feito(o), nome_acao: a.nome.clone(), lugar: (n, total) });
        }
    }
    v
}

/// De onde a esteira sai e onde entra na construção.
fn pontas(m: &Modulo, (n, total): (usize, usize)) -> (V3, V3) {
    let y = m.y + LADO * (n + 1) as f32 / (total + 1) as f32;
    let (x, z) = (m.x - FOLGA, m.base + 0.12);
    ([x - ENTRADA, y, z], [x, y, z])
}

/// Por onde a esteira passa.
pub fn trecho(o: &Obra, u: &Fora) -> [V3; 2] {
    let (a, b) = pontas(&o.modulos[u.para], u.lugar);
    [a, b]
}

/// Põe a esteira na cena. `subiu`: quanto o destino está de pé.
pub fn por(cena: &mut Cena, o: &Obra, u: &Fora, subiu: f32, alfa: f32, fase: f32, chave: u32) {
    esteira(cena, &trecho(o, u), u.pronto, subiu, 0.0, alfa, fase, chave);
}

/// A placa de largura `w`, com a ponta em `p`.
fn placa_em(p: Pos2, w: f32) -> Rect {
    Rect::from_min_max(pos2(p.x - w - PONTA_PLACA, p.y - ALTO_PLACA * 0.5), pos2(p.x, p.y + ALTO_PLACA * 0.5))
}

/// As placas, por cima da cena. Devolve a que foi clicada e de onde ela sai, na tela.
/// Onde cada placa fica: a ponta e a largura. De longe, as placas de um mesmo plano ficam juntas, uma em cima da outra.
fn posicoes(c: &Pincel, cam: &Camera, o: &Obra, lista: &[Fora]) -> Vec<(Pos2, Pos2, f32)> {
    let mut postas: Vec<Rect> = Vec::new();
    lista.iter().map(|u| {
        let inicio = cam.ponto(pontas(&o.modulos[u.para], u.lugar).0);
        let w = c.largura(&u.referencia, &Letra::nova(10.5, TINTA).forte()) + 16.0;
        let mut p = inicio;
        while postas.iter().any(|r| r.intersects(placa_em(p, w))) {
            p.y -= ALTO_PLACA + VAO_PLACAS;
        }
        postas.push(placa_em(p, w));
        (p, inicio, w)
    }).collect()
}

/// O que as placas ocupam na tela: os rótulos das construções desviam delas.
pub fn areas(c: &Pincel, cam: &Camera, o: &Obra, lista: &[Fora]) -> Vec<Rect> {
    posicoes(c, cam, o, lista).into_iter().map(|(p, _, w)| placa_em(p, w)).collect()
}

/// As placas, cada uma apagada pelo lugar dela na tela: a coluna de texto e a vista expandida levam a placa junto, e a que quase sumiu não pega o clique.
pub fn placas(ui: &mut Ui, c: &Pincel, q: &Quadro, alfa: f32) -> Option<(String, Pos2)> {
    let mut clicada = None;
    let medidas: Vec<(Pos2, f32, f32)> = posicoes(c, q.cam, q.o, q.de_fora).into_iter().zip(q.de_fora).map(|((p, inicio, w), u)| {
        let fica = q.fica_na_tela(u.para, placa_em(p, w));
        if p != inicio {
            c.p.line_segment([p, inicio], Stroke::new(1.0, MEDIO.gamma_multiply(alfa * fica)));
        }
        (p, w, fica)
    }).collect();
    for (u, (p, w, fica)) in q.de_fora.iter().zip(medidas) {
        if fica < SUMIU {
            continue;
        }
        let alfa = alfa * fica;
        let letra = Letra::nova(10.5, TINTA.gamma_multiply(alfa)).forte();
        let area = placa_em(p, w);
        let (x0, y0, x1, y1) = (area.left(), area.top(), p.x - PONTA_PLACA, area.bottom());
        let r = (fica >= CLICA).then(|| ui.interact(area, ui.scope_id().with(("plano-obra-placa", u.para, &u.referencia)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand));
        let fundo = if r.as_ref().is_some_and(|r| r.hovered()) { QUADRA } else { CAMPO };
        let contorno = if u.pronto { VERDE } else { MEDIO };
        let pontos = vec![pos2(x0, y0), pos2(x1, y0), pos2(p.x, p.y), pos2(x1, y1), pos2(x0, y1)];
        c.p.add(PathShape::convex_polygon(pontos, fundo.gamma_multiply(alfa), Stroke::new(1.2, contorno.gamma_multiply(alfa))));
        c.texto(&u.referencia, x0 + 8.0 - c.o.x, p.y + 4.0 - c.o.y, letra);
        if let Some(r) = r {
            if r.clicked() {
                clicada = Some((u.referencia.clone(), area.center()));
            }
            let _ = r.on_hover_text(tr_com("Depende de {r}, em outra ação: {nome}. Clique para ir até lá.", &[("r", &u.referencia), ("nome", &u.nome_acao)]));
        }
    }
    clicada
}

/// O espaço que a esteira de fora pede além da construção.
pub fn pede_espaco(q: &Plano, planos: &[&Plano]) -> bool {
    q.depende_de.iter().any(|d| indice(planos, d).is_none())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m() -> Modulo {
        Modulo { x: 10.0, y: 20.0, h: 2.0, base: 0.0, nivel: 0 }
    }

    #[test]
    fn a_esteira_de_fora_entra_pela_construcao_e_cada_uma_tem_o_seu_lugar() {
        let (a, b) = pontas(&m(), (0, 2));
        assert!(a[0] < b[0] && b[0] < m().x);
        assert_ne!(pontas(&m(), (1, 2)).1[1], b[1]);
        assert_eq!(a[1], b[1]);
    }
}
