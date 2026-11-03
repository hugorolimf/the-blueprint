//! Quem está fazendo o quê agora, na mesa da ação. Cada agente é um personagem vivo, com o plano em que está e o motivo de ter parado, se parou; embaixo, os recados que eles deixaram.
//! O clique na linha voa até a construção; «Abrir Agentes» leva à tela de todos os agentes do blueprint. Sem ninguém na obra, a seção é uma linha curta, sem o vão das fileiras.

use super::super::acao_estilo::{alfa, CORPO};
use super::super::acao_movimento::{cascata, desde_que_abriu, hover};
use super::super::agentes::{figuras, pose};
use super::super::agentes_movimento as mov;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Plano, Situacao};
use super::super::orquestrador::figura_svg;
use super::estado::{pose_de, situacao_de};
use super::{bloco, obra_mergulho as mergulho, rotulo, Saida};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao, Texto};
use crate::tema::{MEDIO, TINTA};
use egui::{vec2, CursorIcon, Painter, Rect, Sense, Ui};

const ALTO_ROTULO: f32 = 28.0;
const ALTO_LINHA: f32 = 52.0;
const ALTO_BOTAO: f32 = 30.0;
const VAO: f32 = 26.0;
const LETRA_RECADO: f32 = 12.5;
const PASSO_RECADO: f32 = 18.0;
const VAO_RECADO: f32 = 12.0;
const LETRA_VAZIO: f32 = 12.0;
const PASSO_VAZIO: f32 = 18.0;

/// O plano tem um agente para mostrar?
pub fn tem_figura(q: &Plano) -> bool {
    pose_de(q).is_some()
}

/// A figura do agente, nos quadros da pose dele.
pub fn figura_viva(ui: &Ui, q: &Plano, r: Rect) {
    if pose_de(q).is_none() {
        return;
    }
    let ctx = ui.ctx();
    let ms = if mov::ligado() { (mov::agora(ctx) * 1000.0) as u64 } else { 0 };
    let ((nome, gz), proximo) = figuras::do_agente(pose(q.situacao), ms);
    if let Some(espera) = proximo {
        ctx.request_repaint_after(espera);
    }
    figura_svg(ui, r, nome, gz);
}

/// Quem tem o que mostrar: com agente, ou esperando a vez.
fn em_movimento(a: &Acao) -> Vec<&Plano> {
    a.planos.iter().filter(|q| pose_de(q).is_some()).collect()
}

fn alto_recados(p: &Painter, a: &Acao, w: f32) -> f32 {
    if a.recados.is_empty() {
        return 0.0;
    }
    let soma: f32 = a.recados.iter().map(|r| bloco(&r.texto, LETRA_RECADO, w, PASSO_RECADO).galeria(p).rows.len() as f32 * PASSO_RECADO + 18.0 + VAO_RECADO).sum();
    VAO + ALTO_ROTULO + soma
}

/// A frase de quando ninguém está na obra: quebra só se a coluna for estreita.
fn vazio(w: f32) -> Texto<'static> {
    bloco(tr("Nenhum movimento agora. Libere um plano para começar."), LETRA_VAZIO, w, PASSO_VAZIO).cor(MEDIO)
}

/// As fileiras dos agentes; sem nenhum, só a frase curta.
fn alto_lista(p: &Painter, a: &Acao, w: f32) -> f32 {
    match em_movimento(a).len() {
        0 => vazio(w).galeria(p).rows.len().max(1) as f32 * PASSO_VAZIO,
        n => n as f32 * ALTO_LINHA,
    }
}

pub fn altura(p: &Painter, a: &Acao, w: f32) -> f32 {
    // sem agentes, a frase já fecha a seção: nada de vão embaixo
    let fim = if em_movimento(a).is_empty() { 0.0 } else { VAO };
    ALTO_ROTULO + alto_lista(p, a, w) + alto_recados(p, a, w) + fim
}

fn linha(ui: &mut Ui, c: &Pincel, a: &Acao, q: &Plano, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let area = c.caixa(x, y, w, ALTO_LINHA);
    let r = ui.interact(area, ui.scope_id().with(("plano-movimento", &q.referencia)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    let h = hover(&ctx, ("plano-movimento-sobre", &q.referencia), r.hovered() || r.has_focus());
    if h > 0.0 {
        c.p.rect_filled(area, 0, alfa(TINTA, 0.08 * h));
    }
    figura_viva(ui, q, c.caixa(x + 6.0, y + 8.0, 44.0, 34.0));
    let num = c.texto(&q.referencia, x + 62.0, y + 21.0, Letra::nova(12.0, TINTA).forte());
    let l = Letra::nova(12.0, CORPO);
    let xt = x + 62.0 + num + 8.0;
    c.texto(&c.cortado(&q.texto, x + w - xt - 8.0, &l).0, xt, y + 21.0, l);
    let (estado, cor) = situacao_de(q).unwrap_or(("", MEDIO));
    let mut frase = if q.por.is_empty() { estado.to_string() } else { format!("{estado} · {}", q.por) };
    if !q.motivo.is_empty() {
        frase = format!("{frase} · {}", q.motivo);
    }
    let ls = Letra::nova(11.0, cor);
    c.texto(&c.cortado(&frase, w - 70.0, &ls).0, x + 62.0, y + 40.0, ls);
    c.fio(x, y + ALTO_LINHA, x + w, y + ALTO_LINHA, alfa(MEDIO, 0.3), 1.0);
    let clicou = r.clicked();
    let _ = r.on_hover_text(if q.motivo.is_empty() { q.texto.clone() } else { format!("{}\n\n{}", q.texto, q.motivo) });
    if clicou {
        mergulho::entrar(&ctx, a, &q.referencia);
    }
}

pub fn desenhar(ui: &mut Ui, c: &Pincel, a: &Acao, x: f32, y: f32, w: f32) -> Option<Saida> {
    let ctx = ui.ctx().clone();
    let lista = em_movimento(a);
    let trabalhando = lista.iter().filter(|q| q.situacao == Situacao::Trabalhando).count();
    rotulo(c, x, y + 10.0, &tr_com("Agentes · {n} trabalhando", &[("n", &trabalhando.to_string())]), w);
    let mut saida = None;
    let b = Botao { texto: tr("Abrir Agentes"), tam: 10.0, cheio: false, ativo: true, altura: ALTO_BOTAO - 6.0, pad_x: 10.0 };
    if botao(ui, c.pt(x + w - b.largura(c.p), y - 8.0), &b).clicked() {
        saida = Some(Saida::Agentes);
    }
    let ms = desde_que_abriu(&ctx, ("acao-movimentos", &a.referencia));
    let topo = y + ALTO_ROTULO;
    if lista.is_empty() {
        vazio(w).em(c.p, c.pt(x, topo));
    }
    for (i, q) in lista.iter().enumerate() {
        let sobe = (1.0 - cascata(&ctx, ms, i, 0.0)) * 12.0;
        linha(ui, &Pincel { p: c.p, o: c.o + vec2(0.0, sobe) }, a, q, x, topo + i as f32 * ALTO_LINHA, w);
    }
    if a.recados.is_empty() {
        return saida;
    }
    let mut yy = topo + alto_lista(c.p, a, w) + VAO;
    rotulo(c, x, yy + 10.0, &tr_com("Recados · {n}", &[("n", &a.recados.len().to_string())]), w);
    yy += ALTO_ROTULO;
    for r in &a.recados {
        c.texto(&r.de, x, yy + 12.0, Letra::nova(11.0, MEDIO));
        let t = bloco(&r.texto, LETRA_RECADO, w, PASSO_RECADO).cor(CORPO);
        let linhas = t.galeria(c.p).rows.len() as f32;
        t.em(c.p, c.pt(x, yy + 18.0));
        yy += linhas * PASSO_RECADO + 18.0 + VAO_RECADO;
    }
    saida
}
