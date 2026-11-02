//! O que acompanha a figura na prévia: a decisão e a legenda ao lado dela, e a fita com as vizinhas embaixo.

use super::super::caixinha::Lapis;
use super::super::cartao_ladrilho::{self, Ladrilho};
use super::super::linha::Pincel;
use super::super::modelo::{Acao as AcaoPlano, Detalhe};
use super::super::revisao::{self, Alvo, Meio, Peca};
use super::{abrir, fechar, texto_legenda, trecho_fita, y_legenda, Medidas, ALTO_CONTROLE, ALTO_MINIATURA, VAO_CONTROLE};
use egui::{Id, Sense, Ui};

const VAO_MINIATURAS: f32 = 10.0;
/// Com poucos vizinhos as miniaturas não crescem até encher a folha.
const LARGURA_MINIATURA: f32 = 220.0;

/// A coluna ao lado da figura: a decisão no topo, sempre no mesmo lugar ao andar entre os detalhes, e a legenda embaixo.
pub(super) fn lado(ui: &mut Ui, c: &Pincel, lugar: Alvo, d: &Detalhe, peca: &Peca, m: &Medidas, y: f32) {
    let l = Lapis { p: c.p, o: c.o };
    if d.pede_aprovacao {
        revisao::canto(l, ui, lugar, m.x_lado + m.w_lado, y + 16.0, peca, Meio::Revisar);
        revisao::nota(l, ui, lugar, m.x_lado, y + ALTO_CONTROLE + VAO_CONTROLE, m.w_lado, peca);
    }
    if d.legenda.is_empty() {
        return;
    }
    let y_texto = y + y_legenda(c.p, d, peca, m.w_lado);
    let t = texto_legenda(d, m.w_lado);
    let g = t.galeria(c.p);
    t.em(c.p, c.pt(m.x_lado, y_texto));
    if g.elided {
        let _ = ui.interact(c.caixa(m.x_lado, y_texto, m.w_lado, g.size().y), Id::unique(("plano-detalhe-legenda", &d.id)), Sense::hover()).on_hover_text(d.legenda.as_str());
    }
}

/// A fita com as vizinhas: um clique troca a figura; na atual, volta à grade. Só existe com mais de um detalhe.
#[allow(clippy::too_many_arguments)]
pub(super) fn fita(ui: &Ui, c: &Pincel, servidor: &str, a: &AcaoPlano, lista: &[&Detalhe], i: usize, x: f32, y: f32, w: f32) {
    if lista.len() < 2 {
        return;
    }
    let trecho = trecho_fita(i, lista.len());
    let n = trecho.len() as f32;
    let largura = ((w - (n - 1.0) * VAO_MINIATURAS) / n).min(LARGURA_MINIATURA);
    for (pos, k) in trecho.enumerate() {
        let (v, atual) = (lista[k], k == i);
        let xx = x + pos as f32 * (largura + VAO_MINIATURAS);
        let ladrilho = Ladrilho { rotulo: format!("{}-{}", a.referencia, v.id), titulo: &v.titulo, figura: v.figura.as_ref(), estado: v.pede_aprovacao.then(|| revisao::estado_de(v.revisao.as_ref())), atual };
        if cartao_ladrilho::desenhar(ui, c, servidor, &format!("fita:{}/{}", a.referencia, v.id), &ladrilho, (xx, y, largura, ALTO_MINIATURA), 1.0).clicked() {
            if atual {
                fechar(ui.ctx(), &a.referencia);
            } else {
                abrir(ui.ctx(), &a.referencia, &v.id);
            }
        }
    }
}
