//! A prévia de um detalhe, na própria folha do blueprint: um clique numa figura da ação aberta troca a grade por ela, sem sair da folha.
//! A figura manda; ao lado ficam a decisão e a legenda, e embaixo a fita com as vizinhas. O que está aberto mora na memória do egui, porque a folha é refeita
//! a cada resposta do servidor, e a altura sai da mesma conta que `empilhar` soma.

use super::acao_estilo::CORPO;
use super::acao_movimento::{cascata, com_alfa, desde_que_abriu};
use super::cartao_ladrilho::RODAPE;
use super::figura;
use super::linha::Pincel;
use super::modelo::{Acao as AcaoPlano, Detalhe, Figura, Folha};
use super::revisao::{self, Alvo, Peca};
use crate::planta::Texto;
use egui::{Align, Context, Id, Painter, Ui};
use std::ops::Range;

#[path = "detalhe_blocos.rs"]
mod blocos;
#[path = "detalhe_figura.rs"]
mod grande;
#[path = "detalhe_nav.rs"]
mod nav;

pub const TEXTOS: &[(&str, &str)] = &[("‹ Anterior", "‹ Previous"), ("Próximo ›", "Next ›")];

const ALTO_CABECA: f32 = 27.0;
const VAO_CABECA: f32 = 14.0;
/// Baixa o bastante para a figura, a decisão e a fita caberem juntas numa janela de notebook.
const FIGURA_MAX: f32 = 460.0;
const CAMPO_MIN: f32 = 240.0;
const LADO_MIN: f32 = 260.0;
const LADO_MAX: f32 = 360.0;
const VAO_LADO: f32 = 28.0;
const ALTO_CONTROLE: f32 = 24.0;
const VAO_CONTROLE: f32 = 18.0;
const PASSO_LEGENDA: f32 = 18.0;
const LINHAS_LEGENDA: usize = 16;
const VAO_FITA: f32 = 20.0;
const ALTO_MINIATURA: f32 = 76.0;

fn id_aberto(acao: &str) -> Id {
    Id::unique(("plano-detalhe-aberto", acao))
}

fn id_rolar(acao: &str) -> Id {
    Id::unique(("plano-detalhe-rolar", acao))
}

/// Mostra este detalhe na folha e leva a folha até ele.
pub fn abrir(ctx: &Context, acao: &str, det: &str) {
    ctx.data_mut(|d| {
        d.insert_temp(id_aberto(acao), Some(det.to_string()));
        d.insert_temp(id_rolar(acao), true);
    });
    ctx.request_repaint();
}

/// Volta à grade de figuras.
pub fn fechar(ctx: &Context, acao: &str) {
    ctx.data_mut(|d| d.insert_temp(id_aberto(acao), None::<String>));
    ctx.request_repaint();
}

/// (altura da figura, altura do campo). A figura vem das medidas do servidor, com teto; figura baixa fica num campo de altura mínima. Sem figura, 16:9.
pub fn medidas_figura(f: Option<&Figura>, largura: f32) -> (f32, f32) {
    let alta = f.map_or(largura * 9.0 / 16.0, |f| figura::altura_para(Some(f), largura)).round();
    let fig = alta.min(FIGURA_MAX);
    (fig, fig.max(CAMPO_MIN))
}

/// O anterior e o próximo de `i` numa lista de `n`.
pub fn vizinhos(i: usize, n: usize) -> (Option<usize>, Option<usize>) {
    (i.checked_sub(1), Some(i + 1).filter(|k| *k < n))
}

/// Quais detalhes a fita mostra: dois antes e quatro depois do atual.
pub fn trecho_fita(i: usize, n: usize) -> Range<usize> {
    i.saturating_sub(2)..(i + 5).min(n)
}

/// Os detalhes da ação: os da árvore e os que chegam na versão pendente.
fn detalhes_da(a: &AcaoPlano) -> Vec<&Detalhe> {
    let novos = a.diff.iter().flat_map(|d| &d.detalhes_novos).filter(|n| !a.detalhes.iter().any(|d| d.id == n.id));
    a.detalhes.iter().chain(novos).collect()
}

/// A posição do detalhe aberto; `None` também quando ele saiu do plano.
fn indice(ctx: &Context, a: &AcaoPlano, lista: &[&Detalhe]) -> Option<usize> {
    let det = ctx.data(|d| d.get_temp::<Option<String>>(id_aberto(&a.referencia))).flatten()?;
    lista.iter().position(|d| d.id == det)
}

/// As medidas da figura e da coluna ao lado dela.
struct Medidas {
    w_fig: f32,
    w_lado: f32,
    x_lado: f32,
    fig: f32,
    campo: f32,
    alto: f32,
}

fn texto_legenda(d: &Detalhe, w: f32) -> Texto<'_> {
    Texto::novo(&d.legenda, 11.5).trilha(0.03).caixa(w).lh(PASSO_LEGENDA).linhas(LINHAS_LEGENDA).cor(CORPO)
}

/// Onde a legenda começa, abaixo da decisão.
fn y_legenda(p: &Painter, d: &Detalhe, peca: &Peca, w_lado: f32) -> f32 {
    if !d.pede_aprovacao {
        return 0.0;
    }
    let nota = revisao::altura_nota(p, peca, w_lado);
    ALTO_CONTROLE + if nota > 0.0 { VAO_CONTROLE + nota } else { 0.0 } + VAO_CONTROLE
}

fn medir(p: &Painter, d: &Detalhe, peca: &Peca, x: f32, w: f32) -> Medidas {
    let w_lado = (w * 0.26).clamp(LADO_MIN, LADO_MAX);
    let w_fig = w - w_lado - VAO_LADO;
    let (fig, campo) = medidas_figura(d.figura.as_ref(), w_fig);
    let legenda = if d.legenda.is_empty() { 0.0 } else { texto_legenda(d, w_lado).galeria(p).rows.len() as f32 * PASSO_LEGENDA };
    let alto = campo.max(y_legenda(p, d, peca, w_lado) + legenda);
    Medidas { w_fig, w_lado, x_lado: x + w_fig + VAO_LADO, fig, campo, alto }
}

fn alto_fita(n: usize) -> f32 {
    if n > 1 {
        VAO_FITA + ALTO_MINIATURA + RODAPE
    } else {
        0.0
    }
}

/// Quanto a prévia ocupa; `None` quando a ação está mostrando a grade.
pub fn altura(p: &Painter, f: &Folha, a: &AcaoPlano, w: f32) -> Option<f32> {
    let lista = detalhes_da(a);
    let d = lista[indice(p.ctx(), a, &lista)?];
    let peca = Peca::detalhe(f, &a.referencia, d);
    Some(ALTO_CABECA + VAO_CABECA + medir(p, d, &peca, 0.0, w).alto + alto_fita(lista.len()))
}

/// A prévia em `(x, y)`, se houver uma aberta. Devolve se desenhou; `y_secao` é o topo da seção, aonde a folha rola ao abrir.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, lugar: Alvo, f: &Folha, a: &AcaoPlano, x: f32, y: f32, w: f32, y_secao: f32) -> bool {
    let ctx = ui.ctx().clone();
    let lista = detalhes_da(a);
    let Some(i) = indice(&ctx, a, &lista) else {
        return false;
    };
    if ctx.data_mut(|d| d.remove_temp::<bool>(id_rolar(&a.referencia))).is_some() {
        ui.scroll_to_rect(c.caixa(x, y_secao - 12.0, w, 1.0), Some(Align::Min));
    }
    let d = lista[i];
    let peca = Peca::detalhe(f, &a.referencia, d);
    let m = medir(c.p, d, &peca, x, w);
    nav::cabecalho(ui, c, a, d, &lista, i, &peca, x, y, w);
    let y_palco = y + ALTO_CABECA + VAO_CABECA;
    // só o que muda entra de novo ao trocar de detalhe: a figura e a coluna ao lado
    let ms = desde_que_abriu(&ctx, ("detalhe", &a.referencia, &d.id));
    let pintor = com_alfa(c.p, cascata(&ctx, ms, 0, 0.0));
    let palco = Pincel { p: &pintor, o: c.o };
    grande::figura_grande(&palco, lugar.servidor, d, &m, x, y_palco);
    blocos::lado(ui, &palco, lugar, d, &peca, &m, y_palco);
    blocos::fita(ui, c, lugar.servidor, a, &lista, i, x, y_palco + m.alto + VAO_FITA, w);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_vizinhos_param_nas_pontas() {
        assert_eq!(vizinhos(0, 3), (None, Some(1)));
        assert_eq!(vizinhos(2, 3), (Some(1), None));
    }

    #[test]
    fn a_fita_mostra_dois_antes_e_quatro_depois() {
        assert_eq!(trecho_fita(0, 3), 0..3);
        assert_eq!(trecho_fita(5, 20), 3..10);
    }

    #[test]
    fn a_figura_tem_teto_e_o_campo_tem_piso() {
        assert_eq!(medidas_figura(None, 1600.0), (FIGURA_MAX, FIGURA_MAX));
        assert_eq!(medidas_figura(None, 320.0), (180.0, CAMPO_MIN));
    }
}
