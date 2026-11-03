//! A lista da tela de agentes: os títulos das colunas, uma ação por grupo com a barra de quantos planos já estão prontos, e um cartão por plano.
//! Sem nenhum plano, no lugar da lista vem o estado vazio (`agentes_vazio.rs`); enquanto o servidor não respondeu, o esqueleto.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas::{self as pecas, Modo};
use super::super::linha::{Letra, Pincel};
use super::super::orquestrador::Pedido;
use super::super::vazio_figuras;
use super::linha::{self, colunas, ALTO, VAO};
use super::{vazio, Grupo};
use crate::i18n::{tr, tr_com};
use crate::tema::{BAIXO, MEDIO, TINTA, VERDE};
use egui::{Id, Sense, Ui};

const ALTO_GRUPO: f32 = 40.0;
const FIM_GRUPO: f32 = 10.0;
const CABECA: f32 = 14.0;
const LARGURA_BARRA_GRUPO: f32 = 120.0;

/// Quanto a lista ocupa, do título das colunas até o último cartão.
pub fn altura(grupos: &[Grupo]) -> f32 {
    let corpo = if grupos.is_empty() { vazio::ALTURA } else { grupos.iter().map(|g| ALTO_GRUPO + g.planos.len() as f32 * (ALTO + VAO) + FIM_GRUPO).sum() };
    if grupos.is_empty() { corpo + 10.0 } else { CABECA + corpo }
}

/// O título do grupo, cortado, com a barra de quantos planos já estão prontos à direita; entra um instante antes do primeiro cartão.
fn cabeca_do_grupo(ui: &Ui, c: &Pincel, g: &Grupo, indice: usize, y: f32, x0: f32, x1: f32) {
    let entra = mov::Entra::nova(c, mov::entrada(ui.ctx(), "agentes", 7 + indice));
    let c = &entra.pincel();
    let (prontos, total) = g.prontos();
    let feito = if prontos == total { VERDE } else { MEDIO };
    let conta = tr_com("{n} de {total} prontos", &[("n", &prontos.to_string()), ("total", &total.to_string())]);
    let letra = Letra::nova(10.0, feito);
    let largura = c.largura(&conta, &letra);
    c.texto(&conta, x1 - largura, y + 20.0, letra);
    let x_barra = x1 - largura - 12.0 - LARGURA_BARRA_GRUPO;
    let fracao = mov::suave(ui.ctx(), ("grupo", g.acao_id), prontos as f32 / total.max(1) as f32);
    pecas::barra(c, x_barra, y + 14.0, LARGURA_BARRA_GRUPO, 4.0, Modo::Fracao(fracao), VERDE);
    let titulo = Letra::nova(12.0, TINTA).forte();
    let (t, cortou) = c.cortado(&g.titulo, x_barra - x0 - 16.0, &titulo);
    let escrita = c.texto(&t, x0, y + 20.0, titulo);
    if cortou {
        let _ = ui.interact(c.caixa(x0, y + 8.0, escrita, 16.0), Id::unique(("plano-agentes-dica", ("grupo", g.acao_id))), Sense::hover()).on_hover_text(g.titulo.as_str());
    }
    c.fio(x0, y + ALTO_GRUPO - 6.0, x1, y + ALTO_GRUPO - 6.0, BAIXO.gamma_multiply(0.45), 1.0);
}

/// Os títulos das colunas, o estado vazio ou o esqueleto, e os cartões, com o título em `y0`. Só o que está à vista desenha. Devolve o gesto pedido e se o botão do estado vazio foi clicado.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, grupos: &[Grupo], carregando: bool, y0: f32, x0: f32, x1: f32) -> (Option<Pedido>, bool) {
    let col = colunas(x0, x1);
    if grupos.is_empty() {
        if carregando {
            vazio_figuras::esqueleto(c, x0, y0, x1 - x0, 3, ALTO, VAO);
            return (None, false);
        }
        return (None, vazio::desenhar(ui, c, y0, x0, x1));
    }
    let cabeca = mov::Entra::nova(c, mov::entrada(ui.ctx(), "agentes", 7));
    let (c_cab, letra) = (cabeca.pincel(), Letra::nova(9.5, MEDIO).trilha(0.1));
    for (nome, x) in [("Plano", col.plano), ("Situação", col.situacao), ("Depende de", col.depende), ("Área", col.area), ("Ramo do agente", col.ramo)] {
        c_cab.rotulo(tr(nome), x, y0, letra);
    }
    c_cab.fio(x0, y0 + 8.0, x1, y0 + 8.0, TINTA.gamma_multiply(0.85), 1.0);
    let mut y = y0 + CABECA;
    let (visivel, mut pedido, mut indice) = (ui.clip_rect(), None, 0);
    for g in grupos {
        cabeca_do_grupo(ui, c, g, indice, y, x0, x1);
        y += ALTO_GRUPO;
        for q in &g.planos {
            if visivel.intersects(c.caixa(0.0, y, x1 + x0, ALTO)) {
                pedido = linha::desenhar(ui, c, q, &col, indice, y, x0, x1).or(pedido);
            }
            indice += 1;
            y += ALTO + VAO;
        }
        y += FIM_GRUPO;
    }
    (pedido, false)
}
