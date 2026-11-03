//! O que cada construção diz, em cima dela, com uma linha de chamada, como nas pranchas: o número, o agente que está nela, o sinal do estado e as peças que há nela, cada uma com a sua cor e o seu clique. O agente mora aqui, fora do desenho: a planta mostra a obra, e a barrinha, quem trabalha nela. O nome só aparece se pedido, como no modo Alt do Factorio, e sob o mouse.
//! Quem está livre mostra o «iniciar»; quem depende de um plano que nem começou mostra o cadeado, e o clique leva até esse plano. Um rótulo nunca cai em cima de outro: quem está mais atrás sobe.

use super::super::acao_estilo::alfa;
use super::super::acao_movimento::{com_alfa, mistura};
use super::super::caixinha;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Plano};
use super::super::revisao::Alvo;
use super::obra_bordo;
use super::obra_cena::{ligados, Quadro};
use super::obra_icones as icones;
use super::movimentos::{figura_viva, tem_figura};
use super::obra_mergulho as mergulho;
use super::obra_regra::{self as regra, Casa};
use super::obra_sumico::{CLICA, SUMIU};
use crate::anatomia::Camera;
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, CAMPO, DESTAQUE, TINTA};
use egui::{pos2, vec2, CursorIcon, Pos2, Rect, Sense, Ui};

const LARGURA_NOME: f32 = 300.0;
const ALTO: f32 = 26.0;
const ALTO_NOME: f32 = 17.0;
const CHAMADA: f32 = 20.0;
const VAO: f32 = 4.0;
const PAD_X: f32 = 8.0;
const BANDEIRA: f32 = 1.2;
/// O mastro desce até um pouco acima do teto: a bandeira fica presa à construção dela.
const PE_DO_MASTRO: f32 = 0.08;
const VAO_SINAL: f32 = 7.0;
const VAO_CASA: f32 = 7.0;
/// A figura do agente na barrinha, na proporção dela.
const ALTO_AGENTE: f32 = 22.0;
const W_AGENTE: f32 = ALTO_AGENTE / 0.77;

#[path = "acao_obra_sinal.rs"]
mod sinal;
pub use sinal::fala;
use sinal::{sinal_de, Sinal};

#[derive(Clone, Copy, Debug)]
struct Ancora {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    /// onde o mastro encosta no teto da construção
    pe: f32,
}

/// O rótulo pronto para pintar. `abre` vai de 0 (só o estado) a 1 (com o nome).
struct Montado {
    curto: String,
    sinal: Sinal,
    casas: Vec<Casa>,
    w_ref: f32,
    /// o agente que está no plano; zero se não há
    w_agente: f32,
    w_sinal: f32,
    abre: f32,
    w: f32,
    /// é o ponto de início ou o ponto em que a obra está
    aqui: bool,
}

/// Onde cada bandeira fica: sempre em cima da sua construção, presa a ela pelo mastro; se já houver outra ali, mais alta. As da frente escolhem primeiro.
fn dispor(ancoras: &[Ancora], ocupadas: &[Rect]) -> Vec<f32> {
    let mut ordem: Vec<usize> = (0..ancoras.len()).collect();
    ordem.sort_by(|a, b| ancoras[*b].y.total_cmp(&ancoras[*a].y));
    let mut ys = vec![0.0; ancoras.len()];
    let mut postos: Vec<Rect> = ocupadas.to_vec();
    for i in ordem {
        let u = ancoras[i];
        let mut y = u.y - CHAMADA - u.h;
        loop {
            let r = Rect::from_min_size(pos2(u.x, y), vec2(u.w, u.h));
            match postos.iter().find(|p| p.intersects(r)) {
                Some(p) => y = p.min.y - u.h - VAO,
                None => {
                    postos.push(r);
                    break;
                }
            }
        }
        ys[i] = y;
    }
    ys
}

/// O interruptor dos nomes, como o modo Alt do Factorio: por padrão a obra só diz o estado; os nomes aparecem com ele, com Alt apertado ou sob o mouse.
pub fn botao_nomes(ui: &mut Ui, c: &Pincel, x: f32, y: f32) {
    let ctx = ui.ctx().clone();
    let ligado = ctx.data(|d| d.get_temp::<bool>(id_nomes())).unwrap_or(false);
    let b = Botao { texto: tr("Nomes"), tam: 10.0, cheio: ligado, ativo: true, altura: 26.0, pad_x: 10.0 };
    let r = botao(ui, c.pt(x, y), &b).on_hover_text(tr("Mostra o nome de cada construção. Segurar Alt faz o mesmo."));
    if r.clicked() {
        ctx.data_mut(|d| d.insert_temp(id_nomes(), !ligado));
    }
}

fn id_nomes() -> egui::Id {
    egui::Id::unique("plano-obra-nomes")
}

/// O sinal, com o clique dele: quem está livre inicia, ou vai ao que falta decidir; quem espera leva a quem falta. Devolve o plano a visitar.
#[allow(clippy::too_many_arguments)]
fn sinal(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &Acao, plano: &Plano, t: &Montado, casas: &[Casa], pede: usize, x: f32, y: f32) -> Option<(String, Pos2)> {
    t.sinal.icone()?;
    let ctx = ui.ctx().clone();
    let area = c.caixa(x - 2.0, y - 11.0, t.w_sinal + 4.0, 22.0);
    let responde = matches!(t.sinal, Sinal::Livre | Sinal::Travado(_) | Sinal::NaFila(_));
    let r = ui.interact(area, ui.scope_id().with(("plano-obra-sinal", &a.referencia, &plano.referencia)), if responde { Sense::click() } else { Sense::hover() });
    if responde && r.hovered() {
        ctx.set_cursor_icon(CursorIcon::PointingHand);
        c.p.rect_filled(area, 0, alfa(TINTA, 0.14));
    }
    pintar_sinal(c, t, x, y);
    let clicou = r.clicked();
    let _ = r.on_hover_text(t.sinal.dica(plano, pede));
    if !clicou {
        return None;
    }
    match &t.sinal {
        Sinal::Livre if pede > 0 => {
            if let Some(casa) = casas.iter().find(|k| k.pede > 0 && k.tipo != super::figura::Peca::Checkpoint) {
                obra_bordo::ir(&ctx, a, casa);
            }
        }
        Sinal::Livre => caixinha::marcar(&ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &[plano.id.as_str()]),
        Sinal::Travado(l) | Sinal::NaFila(l) => return l.first().map(|r| (r.clone(), area.center())),
        _ => {}
    }
    None
}

/// O sinal sem clique: o que vai sumindo embaixo da coluna só se nota.
fn pintar_sinal(c: &Pincel, t: &Montado, x: f32, y: f32) {
    let Some((icone, cor)) = t.sinal.icone() else { return };
    let cor = if t.aqui && matches!(t.sinal, Sinal::Livre) { DESTAQUE } else { cor };
    icones::desenhar(c.p, c.pt(x + icones::LADO * 0.5, y), icones::LADO, icone, cor);
    let quem = t.sinal.quem();
    if !quem.is_empty() {
        c.texto(&quem, x + icones::LADO + 3.0, y + 4.0, Letra::nova(10.5, cor));
    }
}

/// Desenha os rótulos. No mergulho, somem. Devolve o plano que o clique pediu para visitar.
/// `ocupadas` é o que já está na tela (as placas de fora), em coordenadas do pincel: os rótulos desviam. A coluna de texto e a vista expandida apagam cada bandeira pelo lugar dela na tela, e a que quase sumiu não pega o clique.
pub fn desenhar(ui: &mut Ui, c: &Pincel, cam: &Camera, alvo: Alvo, a: &Acao, q: &Quadro, ocupadas: &[Rect]) -> Option<(String, Pos2)> {
    let ctx = ui.ctx().clone();
    let aparece = 1.0 - q.t_dentro;
    if aparece <= 0.02 {
        return None;
    }
    let forte = Letra::nova(11.5, TINTA).forte();
    let leve = Letra::nova(11.0, TINTA);
    let todos = ctx.data(|d| d.get_temp::<bool>(id_nomes())).unwrap_or(false) || ctx.input(|i| i.modifiers.alt);
    let montados: Vec<Montado> = q.lista.iter().enumerate().map(|(i, (plano, b))| {
        let nome = if plano.entrega.is_empty() { &plano.texto } else { &plano.entrega };
        let curto = c.cortado(nome, LARGURA_NOME, &leve).0;
        let sinal = sinal_de(q.f, plano);
        let casas = regra::casas(std::iter::once(*b));
        let w_ref = c.largura(&plano.referencia, &forte);
        let quem = sinal.quem();
        let w_sinal = if sinal.icone().is_none() { 0.0 } else { icones::LADO + if quem.is_empty() { 0.0 } else { 3.0 + c.largura(&quem, &Letra::nova(10.5, AMBAR)) } };
        let w_casas: f32 = casas.iter().map(|k| icones::largura_casa(c, k) + VAO_CASA).sum();
        let w_agente = if tem_figura(plano) { W_AGENTE } else { 0.0 };
        let linha = w_ref + if w_agente > 0.0 { VAO_SINAL + w_agente } else { 0.0 } + if w_sinal > 0.0 { VAO_SINAL + w_sinal } else { 0.0 } + if w_casas > 0.0 { VAO_SINAL + w_casas - VAO_CASA } else { 0.0 };
        let abre = mistura(&ctx, ("plano-obra-nome", &a.referencia, &plano.referencia), todos || q.aceso == Some(i), 0.14);
        let w = PAD_X * 2.0 + linha.max(abre * c.largura(&curto, &leve));
        Montado { curto, sinal, casas, w_ref, w_agente, w_sinal, abre, w, aqui: q.em_destaque(i) }
    }).collect();
    let ancoras: Vec<Ancora> = (0..q.lista.len()).map(|i| {
        let m = &q.o.modulos[i];
        let (alto, teto) = (cam.ponto(m.meio(m.h + BANDEIRA)), cam.ponto(m.meio(m.h + PE_DO_MASTRO)));
        Ancora { x: alto.x - c.o.x, y: alto.y - c.o.y, w: montados[i].w, h: ALTO + ALTO_NOME * montados[i].abre, pe: teto.y - c.o.y }
    }).collect();
    let ys = dispor(&ancoras, ocupadas);
    let mut pedido = None;
    for (i, (plano, b)) in q.lista.iter().enumerate() {
        let (u, y, t) = (ancoras[i], ys[i], &montados[i]);
        let fica = q.fica_na_tela(i, c.caixa(u.x, y, u.w, u.h));
        if fica < SUMIU {
            continue;
        }
        let clica = fica >= CLICA;
        let apaga = q.aceso.is_some_and(|d| !ligados(q.planos, d)[i]);
        let k = aparece * fica * if apaga { 0.35 } else { 1.0 };
        let cor_fio = if t.aqui { DESTAQUE } else { TINTA };
        c.fio(u.x, y + u.h, u.x, u.pe, alfa(cor_fio, 0.8 * k), 1.0);
        c.p.circle_filled(c.pt(u.x, u.pe), 2.2, alfa(cor_fio, 0.9 * k));
        c.ret(u.x, y, u.w, u.h, Some(alfa(CAMPO, 0.82 * k)), None);
        c.fio(u.x, y, u.x, y + u.h, cor_fio.gamma_multiply(k), 2.0);
        if t.aqui {
            // o número em cima da cor: dá para achar de longe
            c.ret(u.x, y, PAD_X * 2.0 + t.w_ref, ALTO, Some(alfa(DESTAQUE, k)), None);
        }
        if clica {
            let r = ui.interact(c.caixa(u.x, y, u.w, u.h), ui.scope_id().with(("plano-obra-rotulo", &a.referencia, &plano.referencia)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
            if r.clicked() {
                mergulho::entrar(&ctx, a, &plano.referencia);
            }
            let _ = r.on_hover_text(plano.texto.clone());
        }
        let pintor = com_alfa(c.p, k);
        let pc = Pincel { p: &pintor, o: c.o };
        let ym = y + ALTO * 0.5;
        pc.texto(&plano.referencia, u.x + PAD_X, ym + 4.0, Letra::nova(11.5, if t.aqui { CAMPO } else { TINTA }).forte());
        let mut xx = u.x + PAD_X + t.w_ref + VAO_SINAL;
        if t.w_agente > 0.0 {
            let opaco = ui.opacity();
            ui.set_opacity(opaco * fica);
            figura_viva(ui, plano, Rect::from_min_size(c.pt(xx, ym - ALTO_AGENTE * 0.5), vec2(W_AGENTE, ALTO_AGENTE)));
            ui.set_opacity(opaco);
            xx += W_AGENTE + VAO_SINAL;
        }
        if clica {
            pedido = sinal(ui, &pc, alvo, a, plano, t, &t.casas, b.a_decidir().0, xx, ym).or(pedido);
        } else {
            pintar_sinal(&pc, t, xx, ym);
        }
        if t.w_sinal > 0.0 {
            xx += t.w_sinal + VAO_SINAL;
        }
        for (j, casa) in t.casas.iter().enumerate() {
            if !clica {
                icones::pintar_casa(&pc, casa, xx, ym);
            } else if icones::casa(ui, &pc, (&plano.referencia, "rotulo", j), casa, xx, ym) {
                obra_bordo::ir(&ctx, a, casa);
            }
            xx += icones::largura_casa(c, casa) + VAO_CASA;
        }
        if t.abre > 0.02 {
            c.texto(&t.curto, u.x + PAD_X, y + ALTO - 4.0 + ALTO_NOME, Letra::nova(11.0, TINTA.gamma_multiply(k * t.abre)));
        }
    }
    pedido
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tema::MEDIO;

    fn plano(r: &str, dep: &[&str]) -> Plano {
        Plano { referencia: r.into(), depende_de: dep.iter().map(|d| d.to_string()).collect(), ..Plano::default() }
    }

    #[test]
    fn a_frase_diz_de_quem_a_construcao_espera() {
        let p = [plano("1", &[]), plano("2", &["1"])];
        let pronto = |_: &str| false;
        assert!(fala(&p[1], pronto).0.contains('1'));
        assert_eq!(fala(&p[0], pronto).1, MEDIO);
    }

    #[test]
    fn dois_rotulos_nunca_caem_um_em_cima_do_outro() {
        let a = |x: f32, y: f32| Ancora { x, y, w: 200.0, h: ALTO, pe: y };
        let ys = dispor(&[a(0.0, 100.0), a(50.0, 110.0), a(500.0, 100.0)], &[]);
        assert!((ys[0] - ys[1]).abs() >= ALTO);
        assert_eq!(ys[2], 100.0 - CHAMADA - ALTO);
    }
}
