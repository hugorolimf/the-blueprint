//! As decisões da ação aberta, em linhas separadas por um fio: à esquerda, o título e o texto inteiro; à direita, numa coluna só,
//! os botões de revisão ou o resultado. Embaixo, a observação, que desliza para abrir. Os botões à vista já dizem que espera você.

use super::super::acao_estilo::CORPO;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Decisao, Folha};
use super::super::revisao::{self, Alvo, EstadoPeca, Meio, Peca};
use super::{bloco, rotulo};
use crate::i18n::tr_com;
use crate::planta::Texto;
use crate::tema::{MEDIO, TINTA};
use egui::{Painter, Ui};

const PASSO: f32 = super::super::acao_estilo::PASSO_LETRA;
const X_CORPO: f32 = 0.0;
const VAO: f32 = 16.0;
const ALTO_ROTULO: f32 = 28.0;
/// A linha do título (com o controle) antes do texto.
const CABECA: f32 = 30.0;

/// As decisões que citam esta ação (`acoes` traz o `ref`).
pub fn da_acao<'a>(f: &'a Folha, a: &Acao) -> Vec<&'a Decisao> {
    f.decisoes.iter().filter(|d| d.acoes.contains(&a.referencia)).collect()
}

fn corpo<'a>(d: &'a Decisao, w: f32, cor: egui::Color32) -> Texto<'a> {
    bloco(&d.texto, super::super::acao_estilo::LETRA, w.min(super::super::acao_estilo::MEDIDA_TEXTO), PASSO).cor(cor)
}

/// Quanto uma decisão ocupa: o título, o texto, a observação quando há, e o vão até a próxima.
fn altura_decisao(p: &Painter, f: &Folha, d: &Decisao, w: f32) -> f32 {
    let linhas = corpo(d, w, TINTA).galeria(p).rows.len().max(1) as f32;
    let nota = if d.pede_aprovacao { revisao::altura_nota(p, &Peca::decisao(f, d), w - 2.0 * X_CORPO) } else { 0.0 };
    CABECA + linhas * PASSO + 10.0 + if nota > 0.0 { nota + 10.0 } else { 0.0 } + VAO
}

/// Quanto estas decisões ocupam, para empilhar antes de desenhar. Zero sem decisões.
pub fn altura(p: &Painter, f: &Folha, lista: &[&Decisao], w: f32) -> f32 {
    if lista.is_empty() {
        return 0.0;
    }
    ALTO_ROTULO + lista.iter().map(|d| altura_decisao(p, f, d, w)).sum::<f32>() - VAO
}

/// Desenha as decisões com o rótulo em `(x, y)`, na largura `w`.
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, lista: &[&Decisao], x: f32, y: f32, w: f32) {
    if lista.is_empty() {
        return;
    }
    rotulo(c, x, y, &tr_com("Decisões · {n}", &[("n", &lista.len().to_string())]), w);
    let mut yy = y + ALTO_ROTULO - 8.0;
    for (n, d) in lista.iter().enumerate() {
        if n > 0 {
            c.fio(x, yy - VAO / 2.0, x + w, yy - VAO / 2.0, crate::tema::BAIXO, 1.0);
        }
        decisao(ui, c, alvo, f, d, x, yy, w);
        yy += altura_decisao(c.p, f, d, w);
    }
}

fn decisao(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, d: &Decisao, x: f32, y: f32, w: f32) {
    let lapis = super::lapis(c);
    let peca = Peca::decisao(f, d);
    let estado = if d.pede_aprovacao { peca.estado() } else { EstadoPeca::Pendente };
    let fora = estado == EstadoPeca::Reprovada;
    let forte = Letra::nova(super::super::acao_estilo::LETRA_TITULO, if fora { MEDIO } else { TINTA }).forte();
    let reserva = if d.pede_aprovacao { revisao::medir_partes(c.p, Meio::Decisao).1 + 16.0 } else { 0.0 };
    let (rot, _) = c.cortado(&d.rotulo, (w - reserva).max(60.0), &forte);
    c.texto(&rot, x, y + 19.0, forte);
    if d.pede_aprovacao {
        revisao::canto(lapis, ui, alvo, x + w, y + 19.0, &peca, Meio::Decisao);
    }
    let cor = if matches!(estado, EstadoPeca::Aprovada | EstadoPeca::Reprovada) { MEDIO } else { CORPO };
    let texto = corpo(d, w, cor);
    let alto_texto = texto.galeria(c.p).rows.len().max(1) as f32 * PASSO;
    texto.em(c.p, c.pt(x + X_CORPO, y + CABECA - 2.0));
    if fora {
        revisao::risco(lapis, x + X_CORPO, x + w - X_CORPO, y + CABECA + 7.0);
    }
    if d.pede_aprovacao {
        revisao::nota(lapis, ui, alvo, x + X_CORPO, y + CABECA + alto_texto + 4.0, w - 2.0 * X_CORPO, &peca);
    }
}
