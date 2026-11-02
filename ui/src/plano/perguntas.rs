//! As perguntas do agente sobre uma ação (`perguntas` de `tela-acao.mjs:33`): as opções lado a lado, a recomendada indicada, e o clique responde (`gesto/responder`).
//! «Outra…» abre um campo para a resposta que não está entre as opções; o rascunho fica na memória do egui. O que já foi respondido fica cheio, e pode ser trocado.

use super::acao::{bloco, rotulo};
use super::acao_estilo::{alfa, cartao, etiqueta};
use super::cartao_botao::{botao, icone, Botao, Icone};
use super::linha::{Letra, Pincel};
use super::modelo::{Acao, Folha, Pergunta};
use super::revisao::Alvo;
use crate::fonte::{familia, Peso, SUBIDA};
use crate::i18n::{tr, tr_com};
use crate::tema::{AZUL, BAIXO, CAMPO, MEDIO, TINTA, VERDE};
use egui::{vec2, Color32, FontId, Id, Key, Painter, Rect, Ui};
use serde_json::{json, Value};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Perguntas · {n}", "Questions · {n}"),
    ("recomendado", "recommended"),
    ("Outra…", "Other…"),
    ("Sua resposta…", "Your answer…"),
    ("Resposta: {r}", "Answer: {r}"),
];

const PASSO: f32 = 16.0;
/// Uma fileira de botões: os 24 do botão e a linha de «recomendado» embaixo.
const FILEIRA: f32 = 40.0;
const VAO_BOTOES: f32 = 6.0;
const X_TEXTO: f32 = 30.0;
const ALTO_RESPOSTA: f32 = 18.0;
const ALTO_CAMPO: f32 = 34.0;
const ALTO_ROTULO: f32 = 18.0;
const PAD_X: f32 = 14.0;
const PAD_TOPO: f32 = 24.0;
const PAD_BAIXO: f32 = 6.0;
const VAO_CARTAO: f32 = 8.0;

#[derive(Clone, Default)]
struct Outra {
    aberta: bool,
    focada: bool,
    texto: String,
}

fn id_outra(pergunta: &str) -> Id {
    Id::unique(("plano-pergunta-outra", pergunta))
}

fn outra(ctx: &egui::Context, pergunta: &str) -> Outra {
    ctx.data(|d| d.get_temp::<Outra>(id_outra(pergunta))).unwrap_or_default()
}

/// As perguntas que citam esta ação (`acoes` traz o `ref`).
pub fn da_acao<'a>(f: &'a Folha, a: &Acao) -> Vec<&'a Pergunta> {
    f.perguntas.iter().filter(|q| q.acoes.contains(&a.referencia)).collect()
}

/// O corpo de `gesto/responder`.
pub fn corpo_responder(ws: &str, bp: &str, pergunta: &str, resposta: &str) -> Value {
    json!({"workspaceId": ws, "blueprintId": bp, "pergunta": pergunta, "resposta": resposta})
}

/// Se esta opção é a que o agente recomenda.
pub fn recomendada(q: &Pergunta, opcao: &str) -> bool {
    !q.recomendada.is_empty() && q.recomendada == opcao
}

/// A resposta que não está entre as opções: as outras ficam cheias no próprio botão, esta precisa ser escrita.
fn resposta_livre(q: &Pergunta) -> Option<&str> {
    (!q.resposta.is_empty() && !q.opcoes.contains(&q.resposta)).then_some(q.resposta.as_str())
}

/// Onde cada botão fica, a partir da esquerda e da primeira fileira: quando não cabe, desce. Devolve (x, fileira) por botão.
pub fn fileiras(larguras: &[f32], max: f32) -> Vec<(f32, usize)> {
    let (mut x, mut fileira) = (0.0, 0);
    let mut lugares = Vec::with_capacity(larguras.len());
    for l in larguras {
        if x > 0.0 && x + l > max {
            x = 0.0;
            fileira += 1;
        }
        lugares.push((x, fileira));
        x += l + VAO_BOTOES;
    }
    lugares
}

/// A opção como cabe num botão: cortada para a fileira, mas é a inteira que vai na resposta.
fn opcao_curta(p: &Painter, o: &str, w: f32) -> (String, bool) {
    let c = Pincel { p, o: egui::Pos2::ZERO };
    c.cortado(o, (w - X_TEXTO - 20.0).max(40.0), &Letra::nova(10.0, TINTA).trilha(0.1))
}

/// As larguras dos botões da pergunta, com «Outra…» no fim.
fn larguras(p: &Painter, q: &Pergunta, w: f32) -> Vec<f32> {
    q.opcoes.iter().map(|o| Botao::novo(&opcao_curta(p, o, w).0).largura(p)).chain(std::iter::once(Botao::novo(tr("Outra…")).largura(p))).collect()
}

fn texto_da(q: &Pergunta, w: f32) -> crate::planta::Texto<'_> {
    bloco(&q.texto, 11.5, w - X_TEXTO, PASSO, 2)
}

/// O que cabe dentro do cartão da pergunta: o texto, os botões, a resposta livre e o campo.
fn miolo(p: &Painter, q: &Pergunta, w: f32) -> f32 {
    let linhas = texto_da(q, w).galeria(p).rows.len() as f32;
    let fileiras_n = fileiras(&larguras(p, q, w), w - X_TEXTO).last().map_or(1, |(_, f)| f + 1) as f32;
    let livre = if resposta_livre(q).is_some() { ALTO_RESPOSTA } else { 0.0 };
    let campo = if outra(p.ctx(), &q.id).aberta { ALTO_CAMPO } else { 0.0 };
    linhas * PASSO + 2.0 + fileiras_n * FILEIRA + 10.0 + livre + campo
}

/// O cartão inteiro: o respiro de cima e de baixo e o vão até o próximo.
fn altura_pergunta(p: &Painter, q: &Pergunta, w: f32) -> f32 {
    PAD_TOPO + miolo(p, q, w - 2.0 * PAD_X) + PAD_BAIXO + VAO_CARTAO
}

/// Quanto as perguntas da ação ocupam, para empilhar antes de desenhar. Zero sem perguntas.
pub fn altura(p: &Painter, f: &Folha, a: &Acao, w: f32) -> f32 {
    let lista = da_acao(f, a);
    if lista.is_empty() {
        return 0.0;
    }
    ALTO_ROTULO + lista.iter().map(|q| altura_pergunta(p, q, w)).sum::<f32>()
}

/// Desenha as perguntas com o rótulo em `(x, y)`, na largura `w`.
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, x: f32, y: f32, w: f32) {
    let lista = da_acao(f, a);
    if lista.is_empty() {
        return;
    }
    rotulo(c, x, y, &tr_com("Perguntas · {n}", &[("n", &lista.len().to_string())]), w);
    let mut yy = y + ALTO_ROTULO - 4.0;
    for q in lista {
        let h = altura_pergunta(c.p, q, w);
        pergunta(ui, c, alvo, q, x, yy + PAD_TOPO, w);
        yy += h;
    }
}

fn pergunta(ui: &mut Ui, c: &Pincel, alvo: Alvo, q: &Pergunta, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let id_curto = c.cortado(&q.id, X_TEXTO - 12.0, &Letra::nova(9.5, MEDIO)).0;
    etiqueta(c, x, y - 12.0, &id_curto, if q.resposta.is_empty() { Some(AZUL) } else { None }, false);
    if !q.resposta.is_empty() {
        icone(c.p, c.pt(x + 36.0, y - 4.0), Icone::Check, alfa(VERDE, 0.9));
    }
    let t = texto_da(q, w);
    let g = t.galeria(c.p);
    t.em(c.p, c.pt(x + X_TEXTO, y - SUBIDA * 11.5));
    if g.elided {
        let _ = ui.interact(c.caixa(x + X_TEXTO, y - 12.0, w - X_TEXTO, g.rows.len() as f32 * PASSO), ui.scope_id().with(("plano-pergunta-dica", &q.id)), egui::Sense::hover()).on_hover_text(q.texto.as_str());
    }
    let topo = y + g.rows.len() as f32 * PASSO + 2.0 - 4.0;
    let lugares = fileiras(&larguras(c.p, q, w), w - X_TEXTO);
    let mut rascunho = outra(&ctx, &q.id);
    for (i, o) in q.opcoes.iter().enumerate() {
        let (lx, fileira) = lugares[i];
        let (bx, by) = (x + X_TEXTO + lx, topo + fileira as f32 * FILEIRA);
        let (curta, cortou) = opcao_curta(c.p, o, w);
        let b = Botao::novo(&curta).cheio(q.resposta == *o).cor(AZUL);
        let largura = b.largura(c.p);
        let resposta = botao(ui, ("pergunta-opcao", &q.id, i), c.pt(bx, by), &b);
        if resposta.clicked() {
            alvo.estado.gesto(&ctx, alvo.servidor, "responder", &corpo_responder(alvo.ws, alvo.bp, &q.id, o));
        }
        if cortou {
            let _ = resposta.on_hover_text(o.as_str());
        }
        if recomendada(q, o) {
            c.texto(tr("recomendado"), bx + largura / 2.0 + 4.0, by + 35.0, Letra::nova(9.0, MEDIO).meio());
            c.losango(bx + largura / 2.0 - c.largura(tr("recomendado"), &Letra::nova(9.0, MEDIO)) / 2.0 - 2.0, by + 32.0, 2.6, true);
        }
    }
    let (lx, fileira) = lugares[q.opcoes.len()];
    let botao_outra = Botao::novo(tr("Outra…")).cheio(resposta_livre(q).is_some()).cor(AZUL);
    if botao(ui, ("pergunta-outra", &q.id), c.pt(x + X_TEXTO + lx, topo + fileira as f32 * FILEIRA), &botao_outra).clicked() {
        rascunho.aberta = !rascunho.aberta;
        rascunho.focada = false;
    }
    let mut abaixo = topo + (fileira + 1) as f32 * FILEIRA + 10.0;
    if let Some(livre) = resposta_livre(q) {
        let letra = Letra::nova(10.5, TINTA);
        let frase = tr_com("Resposta: {r}", &[("r", livre)]);
        let (corte, cortou) = c.cortado(&frase, w - X_TEXTO, &letra);
        c.texto(&corte, x + X_TEXTO, abaixo + 8.0, letra);
        if cortou {
            let _ = ui.interact(c.caixa(x + X_TEXTO, abaixo - 4.0, w - X_TEXTO, 16.0), ui.scope_id().with(("plano-pergunta-livre", &q.id)), egui::Sense::hover()).on_hover_text(livre);
        }
        abaixo += ALTO_RESPOSTA;
    }
    if rascunho.aberta {
        campo(ui, c, alvo, q, &mut rascunho, x + X_TEXTO, abaixo, w - X_TEXTO);
    }
    ctx.data_mut(|d| d.insert_temp(id_outra(&q.id), rascunho));
}

/// O campo de «Outra…», com enviar (ou Enter) e cancelar.
fn campo(ui: &mut Ui, c: &Pincel, alvo: Alvo, q: &Pergunta, r: &mut Outra, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let cheio = !r.texto.trim().is_empty();
    let b_enviar = Botao::novo(tr("Enviar")).icone(Icone::Enviar).cor(AZUL).cheio(true).ativo(cheio);
    let b_cancelar = Botao::novo(tr("Cancelar"));
    let (le, lc) = (b_enviar.largura(c.p), b_cancelar.largura(c.p));
    let largura_campo = (w - le - lc - 16.0).max(60.0);
    let area = Rect::from_min_size(c.pt(x, y), vec2(largura_campo, 24.0));
    cartao(c.p, area, CAMPO, alfa(BAIXO, 0.8));
    let dica = egui::RichText::new(tr("Sua resposta…")).size(10.5).color(MEDIO);
    let editor = egui::TextEdit::singleline(&mut r.texto).id(id_outra(&q.id).with("campo")).frame(egui::Frame::NONE).margin(egui::Margin::symmetric(8, 5)).font(FontId::new(10.5, familia(Peso::Normal))).text_color(TINTA).background_color(Color32::TRANSPARENT).hint_text(dica).desired_width(largura_campo);
    let resposta = ui.put(area, editor);
    if !r.focada {
        resposta.request_focus();
        r.focada = true;
    }
    let enter = resposta.lost_focus() && ctx.input(|i| i.key_pressed(Key::Enter));
    let enviar = botao(ui, ("pergunta-enviar", &q.id), c.pt(x + largura_campo + 8.0, y), &b_enviar).clicked();
    if (enviar || enter) && cheio {
        alvo.estado.gesto(&ctx, alvo.servidor, "responder", &corpo_responder(alvo.ws, alvo.bp, &q.id, r.texto.trim()));
        *r = Outra::default();
        return;
    }
    if botao(ui, ("pergunta-cancelar", &q.id), c.pt(x + largura_campo + 8.0 + le + 8.0, y), &b_cancelar).clicked() {
        *r = Outra::default();
    }
}
