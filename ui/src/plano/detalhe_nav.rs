//! A linha de cima da prévia: o número e o título do detalhe à esquerda; «n de N», anterior, próximo e fechar à direita.
//! As setas do teclado andam entre os detalhes e o Esc fecha, quando não há um campo de texto com o foco.

use super::super::caixinha::Lapis;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao as AcaoPlano, Detalhe};
use super::super::revisao::{self, EstadoPeca, Peca};
use super::{abrir, fechar, vizinhos, ALTO_CABECA};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{MEDIO, TINTA};
use egui::{Id, Key, Sense, Ui};

const VAO_BOTOES: f32 = 8.0;

fn botao_de(texto: &str, ativo: bool) -> Botao<'_> {
    Botao { texto, tam: 10.0, cheio: false, ativo, altura: ALTO_CABECA, pad_x: 12.0 }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn cabecalho(ui: &mut Ui, c: &Pincel, a: &AcaoPlano, d: &Detalhe, lista: &[&Detalhe], i: usize, peca: &Peca, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let (antes, depois) = vizinhos(i, lista.len());
    let ir = |k: Option<usize>| {
        if let Some(k) = k {
            abrir(&ctx, &a.referencia, &lista[k].id);
        }
    };
    let b_fechar = botao_de(tr("Fechar"), true);
    let mut x_fim = x + w - b_fechar.largura(c.p);
    if botao(ui, c.pt(x_fim, y), &b_fechar).clicked() {
        fechar(&ctx, &a.referencia);
    }
    if lista.len() > 1 {
        let b_proximo = botao_de(tr("Próximo ›"), depois.is_some());
        x_fim -= VAO_BOTOES + b_proximo.largura(c.p);
        if botao(ui, c.pt(x_fim, y), &b_proximo).clicked() {
            ir(depois);
        }
        let b_anterior = botao_de(tr("‹ Anterior"), antes.is_some());
        x_fim -= VAO_BOTOES + b_anterior.largura(c.p);
        if botao(ui, c.pt(x_fim, y), &b_anterior).clicked() {
            ir(antes);
        }
        let numero = tr_com("{n} de {total}", &[("n", &(i + 1).to_string()), ("total", &lista.len().to_string())]);
        x_fim -= 14.0 + c.texto(&numero, x_fim - 14.0, y + 17.0, Letra::nova(10.5, MEDIO).fim());
    }
    let w_id = c.texto(&format!("{}-{}", a.referencia, d.id), x, y + 17.0, Letra::nova(11.0, MEDIO));
    let x_titulo = x + w_id + 12.0;
    let reprovada = d.pede_aprovacao && peca.estado() == EstadoPeca::Reprovada;
    let letra = Letra::nova(14.0, if reprovada { MEDIO } else { TINTA }).forte();
    let (t, cortou) = c.cortado(&d.titulo, (x_fim - 16.0 - x_titulo).max(40.0), &letra);
    let w_titulo = c.texto(&t, x_titulo, y + 18.0, letra);
    if reprovada {
        revisao::risco(Lapis { p: c.p, o: c.o }, x_titulo, x_titulo + w_titulo, y + 13.0);
    }
    if cortou {
        let _ = ui.interact(c.caixa(x_titulo, y, w_titulo, ALTO_CABECA), Id::unique(("plano-detalhe-titulo", &d.id)), Sense::hover()).on_hover_text(d.titulo.as_str());
    }
    if !ui.memory(|m| m.focused().is_some()) {
        let (esq, dir, esc) = ui.input(|i| (i.key_pressed(Key::ArrowLeft), i.key_pressed(Key::ArrowRight), i.key_pressed(Key::Escape)));
        if esq {
            ir(antes);
        }
        if dir {
            ir(depois);
        }
        if esc {
            fechar(&ctx, &a.referencia);
        }
    }
}
