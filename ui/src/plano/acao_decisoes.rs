//! As decisões da ação aberta, em linhas separadas por um fio: à esquerda, o título e duas linhas de texto (o resto em «ver tudo»); à direita, numa coluna só,
//! os botões de revisão ou o resultado. Embaixo, a observação, que desliza para abrir. Os botões à vista já dizem que espera você.

use super::super::acao_estilo::CORPO;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Decisao, Folha};
use super::super::revisao::{self, Alvo, EstadoPeca, Meio, Peca};
use super::ver::{alternado, chave_ver, ver_tudo};
use super::{bloco, rotulo};
use crate::i18n::tr_com;
use crate::planta::Texto;
use crate::tema::{MEDIO, TINTA};
use egui::{Context, Painter, Ui};

const PASSO: f32 = 16.0;
const X_CORPO: f32 = 0.0;
/// O lugar de «ver tudo» na ponta da última linha.
const RESERVA: f32 = 64.0;
/// A coluna dos botões, à direita do texto.
const COLUNA_BOTOES: f32 = 300.0;
const VAO: f32 = 14.0;
const ALTO_ROTULO: f32 = 28.0;
/// A linha do título (com o controle) antes do texto.
const CABECA: f32 = 24.0;

/// As decisões que citam esta ação (`acoes` traz o `ref`).
pub fn da_acao<'a>(f: &'a Folha, a: &Acao) -> Vec<&'a Decisao> {
    f.decisoes.iter().filter(|d| d.acoes.contains(&a.referencia)).collect()
}

fn aberta(ctx: &Context, d: &Decisao) -> bool {
    alternado(ctx, &chave_ver("d", &d.id))
}

fn corpo<'a>(ctx: &Context, d: &'a Decisao, w: f32, cor: egui::Color32) -> Texto<'a> {
    let linhas = if aberta(ctx, d) { usize::MAX } else { 2 };
    bloco(&d.texto, 11.5, (w - COLUNA_BOTOES - RESERVA).max(120.0), PASSO, linhas).cor(cor)
}

/// Quanto uma decisão ocupa: o título, o texto, a observação quando há, e o vão até a próxima.
fn altura_decisao(p: &Painter, f: &Folha, d: &Decisao, w: f32) -> f32 {
    let linhas = corpo(p.ctx(), d, w, TINTA).galeria(p).rows.len().max(1) as f32;
    let nota = if d.pede_aprovacao { revisao::altura_nota(p, &Peca::decisao(f, d), w - 2.0 * X_CORPO) } else { 0.0 };
    CABECA + linhas * PASSO + 10.0 + if nota > 0.0 { nota + 10.0 } else { 0.0 } + VAO
}

/// Quanto as decisões da ação ocupam, para empilhar antes de desenhar. Zero sem decisões.
pub fn altura(p: &Painter, f: &Folha, a: &Acao, w: f32) -> f32 {
    let lista = da_acao(f, a);
    if lista.is_empty() {
        return 0.0;
    }
    ALTO_ROTULO + lista.iter().map(|d| altura_decisao(p, f, d, w)).sum::<f32>()
}

/// Desenha as decisões com o rótulo em `(x, y)`, na largura `w`.
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, x: f32, y: f32, w: f32) {
    let lista = da_acao(f, a);
    if lista.is_empty() {
        return;
    }
    rotulo(c, x, y, &tr_com("Decisões · {n}", &[("n", &lista.len().to_string())]), w);
    let mut yy = y + ALTO_ROTULO - 8.0;
    for (n, d) in lista.into_iter().enumerate() {
        if n > 0 {
            c.fio(x, yy - VAO / 2.0, x + w, yy - VAO / 2.0, crate::tema::BAIXO, 1.0);
        }
        decisao(ui, c, alvo, f, d, x, yy, w);
        yy += altura_decisao(c.p, f, d, w);
    }
}

fn decisao(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, d: &Decisao, x: f32, y: f32, w: f32) {
    let lapis = super::lapis(c);
    let ctx = ui.ctx().clone();
    let peca = Peca::decisao(f, d);
    // sem pedido de aprovação não há estado a mostrar: sai do primeiro plano, como uma aprovada, mas sem carimbo
    let estado = if d.pede_aprovacao { peca.estado() } else { EstadoPeca::Aprovada };
    let fora = estado == EstadoPeca::Reprovada;
    let forte = Letra::nova(12.0, if fora { MEDIO } else { TINTA }).forte();
    let reserva = if d.pede_aprovacao { revisao::medir_partes(c.p, Meio::Revisar).1 + 16.0 } else { 0.0 };
    let (rot, _) = c.cortado(&d.rotulo, w - reserva.max(COLUNA_BOTOES), &forte);
    c.texto(&rot, x, y + 16.0, forte);
    if d.pede_aprovacao {
        revisao::canto(lapis, ui, alvo, x + w, y + 16.0, &peca, Meio::Revisar);
    }
    let cor = if matches!(estado, EstadoPeca::Aprovada | EstadoPeca::Reprovada) { MEDIO } else { CORPO };
    let texto = corpo(&ctx, d, w, cor);
    let g = texto.galeria(c.p);
    texto.em(c.p, c.pt(x + X_CORPO, y + CABECA - 2.0));
    let alto_texto = g.rows.len().max(1) as f32 * PASSO;
    ver_tudo(ui, c, &chave_ver("d", &d.id), x + w - COLUNA_BOTOES, y + CABECA + 11.0 + alto_texto - PASSO, g.elided);
    if fora {
        revisao::risco(lapis, x + X_CORPO, x + w - X_CORPO, y + CABECA + 4.0);
    }
    if d.pede_aprovacao {
        revisao::nota(lapis, ui, alvo, x + X_CORPO, y + CABECA + alto_texto + 4.0, w - 2.0 * X_CORPO, &peca);
    }
}
