//! O topo da ação aberta, direto no papel: o rótulo e a barra já estão na linha da ação, e a cota de dias embaixo dela. Aqui ficam o «− Recolher»,
//! o resumo da ação (duas linhas, o resto em «ver tudo») e os links.

use super::super::acao_estilo::CORPO;
use super::super::linha::Pincel;
use super::super::modelo::Acao as AcaoPlano;
use super::ver::{alternado, chave_ver, ver_tudo};
use super::{bloco, links, tem_links, Geometria, Saida};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use egui::{Context, Painter, Ui};

const PASSO_TEXTO: f32 = 18.0;
const ALTO_LINKS: f32 = 22.0;
/// O lugar da cota de dias, que a linha da ação desenha.
const ALTO_COTA: f32 = 36.0;
/// A altura do botão: a mesma dos filtros da folha original.
const ALTO_BOTAO: f32 = 27.0;
/// O lugar do botão à direita do resumo, com o «ver tudo».
const RESERVA_TEXTO: f32 = 190.0;
/// A medida do resumo: linhas maiores que isso cansam a leitura.
const MEDIDA: f32 = 680.0;
const LARGURA_BOTAO: f32 = 120.0;
const VAO_DEPOIS: f32 = 26.0;

fn recolher<'a>() -> Botao<'a> {
    Botao { texto: tr("− Recolher"), tam: 10.0, cheio: false, ativo: true, altura: ALTO_BOTAO, pad_x: 12.0 }
}

fn texto_acao<'a>(ctx: &Context, a: &'a AcaoPlano, g: &Geometria) -> crate::planta::Texto<'a> {
    let linhas = if alternado(ctx, &chave_ver("a", &a.id)) { usize::MAX } else { 2 };
    bloco(&a.texto, 11.5, (g.w - RESERVA_TEXTO).min(MEDIDA), PASSO_TEXTO, linhas).cor(CORPO)
}

pub fn altura(p: &Painter, a: &AcaoPlano, g: &Geometria) -> f32 {
    let texto = if a.texto.is_empty() { 0.0 } else { texto_acao(p.ctx(), a, g).galeria(p).rows.len() as f32 * PASSO_TEXTO };
    let ligacoes = if tem_links(&a.links) { ALTO_LINKS } else { 0.0 };
    ALTO_COTA + texto.max(ALTO_BOTAO) + ligacoes + VAO_DEPOIS
}

/// O resumo, os links e o botão. Devolve o que o clique em «Recolher» pede.
pub fn desenhar(ui: &mut Ui, c: &Pincel, a: &AcaoPlano, g: &Geometria, y: f32) -> Option<Saida> {
    let mut saida = None;
    let topo = y + ALTO_COTA;
    let b = recolher();
    let x_botao = g.x + g.w - b.largura(c.p);
    if botao(ui, c.pt(x_botao, topo), &b).clicked() {
        // as linhas de baixo sobem andando, como ao fechar pela linha da ação
        crate::anima::mexer(ui.ctx(), 350.0);
        super::super::detalhe::fechar(ui.ctx(), &a.referencia);
        saida = Some(Saida::Recolher);
    }
    let mut abaixo = topo;
    if !a.texto.is_empty() {
        let t = texto_acao(ui.ctx(), a, g);
        let galeria = t.galeria(c.p);
        t.em(c.p, c.pt(g.x, abaixo));
        let alto = galeria.rows.len() as f32 * PASSO_TEXTO;
        ver_tudo(ui, c, &chave_ver("a", &a.id), (g.x + g.w - LARGURA_BOTAO).min(g.x + MEDIDA + 64.0), abaixo + alto - 5.0, galeria.elided);
        abaixo += alto;
    }
    abaixo = abaixo.max(topo + ALTO_BOTAO);
    if tem_links(&a.links) {
        links(ui, c, g.x, abaixo + 14.0, g.x + g.w, &a.links, &a.id);
    }
    saida
}
