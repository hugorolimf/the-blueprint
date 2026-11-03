//! As peças do plano numa lista que abre no lugar: decisão, pergunta e tela, cada uma com a sua forma, sem cartão e sem bloco solto embaixo.
//! A lista e o palco falam um com o outro: o mouse numa linha acende a peça no desenho, a peça sob o mouse acende a linha, e clicar nela lá abre aqui. Ao decidir, a próxima pendente abre sozinha.
//! A imagem de uma tela não aparece aqui: ela é vista no próprio componente, na planta (`obra_na_tela`).

use super::super::acao_estilo::CORPO;
use super::super::caixinha::Lapis;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Decisao, Detalhe, Folha, Pergunta, Plano};
use super::super::perguntas;
use super::super::revisao::{self, Alvo, Meio, Peca as PecaRevisao};
use super::super::vazio_iso::Iso;
use super::blocos::Bloco;
use super::decidir::{self, Item};
use super::figura::{self, Marca};
use super::obra_icones::{self as icones, Icone};
use super::obra_pecas::cor_da_marca;
use super::{bloco, na_tela, obra_mergulho as mergulho};
use crate::i18n::{tr, tr_com};
// `tr` também dá o rótulo «Peças», que quem chama passa
use crate::planta::{botao, Botao};
use crate::tema::{BAIXO, MEDIO, QUADRA, TINTA, VERDE};
use egui::{CursorIcon, Painter, Sense, Ui};

const ALTO_CABECA: f32 = 34.0;
const ALTO_LINHA: f32 = 46.0;
const ALTO_TRILHO: f32 = 5.0;
const VAO_TRILHO: f32 = 3.0;
/// Onde o texto começa, depois da figura da peça.
const X_TEXTO: f32 = 38.0;
const PAD_FORMA: f32 = 12.0;
const VAO_FORMA: f32 = 10.0;
const LETRA_TEXTO: f32 = 12.0;
const PASSO_TEXTO: f32 = 18.0;
const ALTO_VER: f32 = 30.0;
const ALTO_OPCAO: f32 = 30.0;
const PASSO_OPCAO: f32 = 16.0;
const LETRA_OPCAO: f32 = 11.5;
const LADO_MARCA: f32 = 14.0;
const X_OPCAO: f32 = 24.0;
const W_RECOMENDADO: f32 = 92.0;

fn largura_forma(w: f32) -> f32 {
    w - X_TEXTO - PAD_FORMA
}

fn alto_texto(p: &Painter, t: &str, w: f32) -> f32 {
    if t.trim().is_empty() { 0.0 } else { bloco(t, LETRA_TEXTO, w, PASSO_TEXTO).galeria(p).rows.len() as f32 * PASSO_TEXTO + VAO_FORMA }
}

fn pintar_texto(c: &Pincel, t: &str, x: f32, y: f32, w: f32) -> f32 {
    if t.trim().is_empty() {
        return y;
    }
    bloco(t, LETRA_TEXTO, w, PASSO_TEXTO).cor(CORPO).em(c.p, c.pt(x, y));
    y + alto_texto(c.p, t, w)
}

fn alto_verbos(p: &Painter, peca: &PecaRevisao, w: f32) -> f32 {
    let nota = revisao::altura_nota(p, peca, w);
    revisao::ALTO_VERBOS + if nota > 0.0 { nota + VAO_FORMA } else { 0.0 }
}

/// Os verbos e a observação. Devolve se a pessoa acabou de decidir.
#[allow(clippy::too_many_arguments)]
fn verbos(ui: &mut Ui, c: &Pincel, alvo: Alvo, peca: &PecaRevisao, meio: Meio, x: f32, y: f32, w: f32) -> bool {
    let lapis = Lapis { p: c.p, o: c.o };
    let decidiu = revisao::verbos(lapis, ui, alvo, x, x + w, y, peca, meio);
    revisao::nota(lapis, ui, alvo, x, y + revisao::ALTO_VERBOS + VAO_FORMA, w, peca);
    decidiu
}

fn alto_decisao(p: &Painter, f: &Folha, d: &Decisao, w: f32) -> f32 {
    alto_texto(p, &d.texto, w) + if d.pede_aprovacao { alto_verbos(p, &PecaRevisao::decisao(f, d), w) } else { 0.0 }
}

fn decisao(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, d: &Decisao, x: f32, y: f32, w: f32) -> bool {
    let yy = pintar_texto(c, &d.texto, x, y, w);
    d.pede_aprovacao && verbos(ui, c, alvo, &PecaRevisao::decisao(f, d), Meio::Decisao, x, yy, w)
}

fn alto_tela(p: &Painter, f: &Folha, a: &Acao, d: &Detalhe, w: f32) -> f32 {
    alto_texto(p, &d.legenda, w) + if d.figura.is_some() { ALTO_VER + VAO_FORMA } else { 0.0 } + if d.pede_aprovacao { alto_verbos(p, &PecaRevisao::detalhe(f, &a.referencia, d), w) } else { 0.0 }
}

/// A tela não mostra a imagem aqui: leva até ela, no componente.
#[allow(clippy::too_many_arguments)]
fn tela(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, dono: Option<&Plano>, d: &Detalhe, x: f32, y: f32, w: f32) -> bool {
    let mut yy = pintar_texto(c, &d.legenda, x, y, w);
    if let Some(q) = dono.filter(|_| d.figura.is_some()) {
        let ver = Botao { texto: tr("Ver na tela"), tam: 10.5, cheio: !d.pede_aprovacao, ativo: true, altura: ALTO_VER, pad_x: 14.0 };
        if botao(ui, c.pt(x, yy), &ver).clicked() {
            na_tela::ver(ui.ctx(), a, &q.referencia, &format!("t:{}", d.id));
        }
        yy += ALTO_VER + VAO_FORMA;
    }
    d.pede_aprovacao && verbos(ui, c, alvo, &PecaRevisao::detalhe(f, &a.referencia, d), Meio::Tela, x, yy, w)
}

fn opcao_texto(o: &str, w: f32) -> crate::planta::Texto<'_> {
    bloco(o, LETRA_OPCAO, w - X_OPCAO - W_RECOMENDADO, PASSO_OPCAO)
}

fn alto_opcao(p: &Painter, o: &str, w: f32) -> f32 {
    (opcao_texto(o, w).galeria(p).rows.len() as f32 * PASSO_OPCAO + 12.0).max(ALTO_OPCAO)
}

fn alto_pergunta(p: &Painter, q: &Pergunta, w: f32) -> f32 {
    let livre = perguntas::resposta_livre(q).map_or(0.0, |r| alto_opcao(p, r, w));
    let campo = if perguntas::outra_aberta(p.ctx(), &q.id) { perguntas::ALTO_CAMPO } else { 0.0 };
    q.opcoes.iter().map(|o| alto_opcao(p, o, w)).sum::<f32>() + livre + ALTO_OPCAO + campo + VAO_FORMA
}

/// Uma escolha: a placa, que ganha o visto quando é a resposta, e o texto inteiro. Devolve se clicaram.
#[allow(clippy::too_many_arguments)]
fn opcao(ui: &mut Ui, c: &Pincel, chave: (&str, usize), texto: &str, medido: &str, escolhida: bool, recomendada: bool, x: f32, y: f32, w: f32) -> bool {
    let alto = alto_opcao(c.p, medido, w);
    let r = ui.interact(c.caixa(x, y, w, alto), ui.scope_id().with(("plano-lista-opcao", chave)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    if r.hovered() {
        c.ret(x, y, w, alto, Some(TINTA.gamma_multiply(0.1)), None);
    }
    let ym = y + ALTO_OPCAO * 0.5;
    icones::desenhar(c.p, c.pt(x + LADO_MARCA * 0.5 + 2.0, ym), LADO_MARCA, if escolhida { Icone::Visto } else { Icone::Opcao }, if escolhida { VERDE } else { MEDIO });
    opcao_texto(medido, w).cor(if escolhida { TINTA } else { CORPO }).em(c.p, c.pt(x + X_OPCAO, y + 7.0));
    let _ = texto;
    if recomendada {
        c.texto(tr("recomendado"), x + w - 6.0, ym + 4.0, Letra::nova(9.5, MEDIO).fim());
    }
    r.clicked()
}

/// Cada opção numa linha: um clique responde. Devolve se a pessoa acabou de responder.
fn pergunta(ui: &mut Ui, c: &Pincel, alvo: Alvo, q: &Pergunta, x: f32, y: f32, w: f32) -> bool {
    let ctx = ui.ctx().clone();
    let (mut yy, mut respondeu) = (y, false);
    for (n, o) in q.opcoes.iter().enumerate() {
        if opcao(ui, c, (&q.id, n), o, o, q.resposta == *o, perguntas::recomendada(q, o), x, yy, w) {
            alvo.estado.gesto(&ctx, alvo.servidor, "responder", &perguntas::corpo_responder(alvo.ws, alvo.bp, &q.id, o));
            respondeu = q.resposta != *o;
        }
        yy += alto_opcao(c.p, o, w);
    }
    if let Some(livre) = perguntas::resposta_livre(q) {
        let _ = opcao(ui, c, (&q.id, q.opcoes.len()), livre, livre, true, false, x, yy, w);
        yy += alto_opcao(c.p, livre, w);
    }
    if opcao(ui, c, (&q.id, q.opcoes.len() + 1), tr("Outra…"), tr("Outra…"), false, false, x, yy, w) {
        perguntas::alternar_outra(&ctx, &q.id);
    }
    yy += ALTO_OPCAO;
    if perguntas::outra_aberta(&ctx, &q.id) {
        perguntas::campo_outra(ui, c, alvo, q, x, yy, w);
    }
    respondeu
}

fn alto_forma(p: &Painter, f: &Folha, a: &Acao, i: &Item, w: f32) -> f32 {
    let lw = largura_forma(w);
    PAD_FORMA + match i {
        Item::Decisao(d) => alto_decisao(p, f, d, lw),
        Item::Pergunta(q) => alto_pergunta(p, q, lw),
        Item::Tela(d) => alto_tela(p, f, a, d, lw),
    }
}

pub fn altura(p: &Painter, f: &Folha, a: &Acao, b: &Bloco, w: f32) -> f32 {
    let todas = decidir::todas(b);
    if todas.is_empty() {
        return 0.0;
    }
    let aberta = mergulho::peca(p.ctx(), a, b).map_or(0.0, |i| alto_forma(p, f, a, &i, w));
    ALTO_CABECA + todas.len() as f32 * ALTO_LINHA + aberta
}

/// O nome, a contagem e o trilho com um segmento por peça, na cor dela: de relance, o que falta. O clique num segmento abre a peça.
#[allow(clippy::too_many_arguments)]
fn cabeca(ui: &mut Ui, c: &Pincel, a: &Acao, titulo: &str, todas: &[Item], aberta: Option<&str>, x: f32, y: f32, w: f32) {
    c.rotulo(titulo, x, y + 12.0, Letra::nova(10.0, MEDIO));
    // o que é só para saber não entra na conta: não há o que decidir
    let total = todas.iter().filter(|i| i.marca() != Marca::Neutra).count();
    if total > 0 {
        let feitas = todas.iter().filter(|i| i.marca() == Marca::Ok).count();
        let conta = tr_com("{n} de {total} decididas", &[("n", &feitas.to_string()), ("total", &total.to_string())]);
        c.texto(&conta, x + w, y + 12.0, Letra::nova(10.0, if feitas == total { VERDE } else { TINTA }).fim());
    }
    let passo = (w + VAO_TRILHO) / todas.len() as f32;
    for (n, i) in todas.iter().enumerate() {
        let xs = x + n as f32 * passo;
        let sel = aberta == Some(i.chave().as_str());
        let area = c.caixa(xs, y + 16.0, passo - VAO_TRILHO, 12.0);
        let r = ui.interact(area, ui.scope_id().with(("plano-lista-trilho", &a.referencia, n)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        let alto = if sel || r.hovered() { ALTO_TRILHO + 2.0 } else { ALTO_TRILHO };
        let cor = cor_da_marca(i.marca());
        c.ret(xs, y + 22.0 - alto * 0.5, passo - VAO_TRILHO, alto, Some(if sel { cor } else { cor.gamma_multiply(0.62) }), None);
        if r.clicked() {
            mergulho::escolher(ui.ctx(), a, &i.chave());
        }
        let _ = r.on_hover_text(i.titulo());
    }
}

/// A linha de uma peça. Devolve o clique e se o mouse está nela.
#[allow(clippy::too_many_arguments)]
fn linha(ui: &mut Ui, c: &Pincel, a: &Acao, i: &Item, numero: &str, aberta: bool, acesa: bool, x: f32, y: f32, w: f32) -> (bool, bool) {
    let area = c.caixa(x, y, w, ALTO_LINHA);
    let r = ui.interact(area, ui.scope_id().with(("plano-lista-linha", &a.referencia, i.chave())), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    if !aberta && (r.hovered() || acesa) {
        c.ret(x, y, w, ALTO_LINHA, Some(QUADRA.gamma_multiply(0.75)), None);
    }
    figura::peca(&Iso { p: c.p, o: c.pt(x + 12.0, y + 23.0), k: 15.0 }, i.peca(), 0.0, 0.0, i.marca());
    let (estado, cor) = i.estado();
    c.texto(estado, x + w - 10.0, y + 18.0, Letra::nova(10.0, cor).fim());
    let w_num = c.texto(numero, x + X_TEXTO, y + 18.0, Letra::nova(10.5, TINTA).forte());
    c.texto(i.tipo(), x + X_TEXTO + w_num + 8.0, y + 18.0, Letra::nova(10.0, MEDIO));
    let t = Letra::nova(11.5, TINTA);
    c.texto(&c.cortado(i.titulo(), w - X_TEXTO - 10.0, &t).0, x + X_TEXTO, y + 35.0, t);
    (r.clicked(), r.hovered())
}

#[allow(clippy::too_many_arguments)]
/// A lista com o topo em `y`. `dono` é o plano; sem ele, as peças são da ação inteira, e a numeração segue a ação.
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, dono: Option<&Plano>, titulo: &str, b: &Bloco, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let todas = decidir::todas(b);
    if todas.is_empty() {
        return;
    }
    let aberta = mergulho::peca(&ctx, a, b).map(|i| i.chave());
    let acesa = mergulho::acesa(&ctx, a);
    cabeca(ui, c, a, titulo, &todas, aberta.as_deref(), x, y, w);
    // as peças da ação inteira não seguem um plano: «2 · 1» não se confunde com o plano 2.1
    let numero = |n: usize| dono.map_or_else(|| format!("{} · {n}", a.referencia), |q| format!("{}.{n}", q.referencia));
    let mut yy = y + ALTO_CABECA;
    let mut decidiu = None;
    for (n, i) in todas.iter().enumerate() {
        let chave = i.chave();
        let esta = aberta.as_deref() == Some(chave.as_str());
        let alto = ALTO_LINHA + if esta { alto_forma(c.p, f, a, i, w) } else { 0.0 };
        if esta {
            c.ret(x, yy, w, alto, Some(QUADRA.gamma_multiply(0.75)), None);
            c.fio(x + 1.0, yy, x + 1.0, yy + alto, TINTA, 2.4);
            if mergulho::rolar_pedido(&ctx, a) {
                ui.scroll_to_rect(c.caixa(x, yy, w, alto), None);
            }
        }
        c.fio(x, yy + alto, x + w, yy + alto, BAIXO.gamma_multiply(0.55), 1.0);
        let (clicou, sobre) = linha(ui, c, a, i, &numero(n + 1), esta, acesa.as_deref() == Some(chave.as_str()), x, yy, w);
        if clicou {
            if esta { mergulho::fechar_peca(&ctx, a) } else { mergulho::escolher(&ctx, a, &chave) }
        }
        if sobre {
            mergulho::acender(&ctx, a, Some(chave.clone()));
        }
        if esta {
            let (xf, yf, wf) = (x + X_TEXTO, yy + ALTO_LINHA, largura_forma(w));
            let pronta = match i {
                Item::Decisao(d) => decisao(ui, c, alvo, f, d, xf, yf, wf),
                Item::Pergunta(p) => pergunta(ui, c, alvo, p, xf, yf, wf),
                Item::Tela(d) => tela(ui, c, alvo, f, a, dono, d, xf, yf, wf),
            };
            if pronta {
                decidiu = Some(chave);
            }
        }
        yy += alto;
    }
    if let Some(feita) = decidiu {
        if let Some(seguinte) = todas.iter().find(|o| o.chave() != feita && o.marca() == Marca::Espera) {
            mergulho::escolher(&ctx, a, &seguinte.chave());
        }
    }
}
