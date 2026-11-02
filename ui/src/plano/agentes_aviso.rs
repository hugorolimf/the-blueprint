//! O cabeçalho da tela de agentes (nome do blueprint, workspace e o que o último gesto respondeu) e a caixa de avisos em âmbar, em duas colunas.

use super::super::agentes_movimento as mov;
use super::super::escala::X0;
use super::super::linha::{Letra, Pincel};
use crate::i18n::{tr, tr_com};
use crate::planta;
use crate::tema::{AMBAR, CARTAO, TINTA, MEDIO};
use egui::{Id, Sense, Shape, Stroke, Ui};

const MAX_AVISOS: usize = 6;
const PASSO_AVISO: f32 = 19.0;

/// A altura da caixa de avisos em duas colunas; os que não cabem ficam na dica do título.
pub fn altura(n: usize) -> f32 {
    34.0 + n.min(MAX_AVISOS).div_ceil(2) as f32 * PASSO_AVISO
}

/// O triângulo de atenção, em traço fino.
fn atencao(c: &Pincel, x: f32, y: f32) {
    let pontos = vec![c.pt(x, y + 11.0), c.pt(x + 12.0, y + 11.0), c.pt(x + 6.0, y)];
    c.p.add(Shape::convex_polygon(pontos, AMBAR.gamma_multiply(0.14), Stroke::new(1.2, AMBAR)));
    c.fio(x + 6.0, y + 4.0, x + 6.0, y + 7.0, AMBAR, 1.2);
}

pub fn desenhar(ui: &Ui, c: &Pincel, lista: &[String], y: f32, x1: f32) {
    let t = mov::entrada(ui.ctx(), "agentes", 7);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let (w, h) = (x1 - X0, altura(lista.len()));
    c.ret(X0, y, w, h, Some(CARTAO.gamma_multiply(0.85)), Some((MEDIO, 1.0)));
    planta::canto(c.p, c.caixa(X0, y, w, h));
    atencao(c, X0 + 16.0, y + 9.0);
    let titulo = tr_com("Avisos · {n}", &[("n", &lista.len().to_string())]);
    let largura = c.rotulo(&titulo, X0 + 36.0, y + 20.0, Letra::nova(10.0, AMBAR).trilha(0.12));
    if lista.len() > MAX_AVISOS {
        let _ = ui.interact(c.caixa(X0 + 36.0, y + 8.0, largura, 16.0), Id::unique("plano-avisos"), Sense::hover()).on_hover_text(lista.join("\n"));
    }
    let (coluna, letra) = ((w - 52.0) / 2.0, Letra::nova(10.5, TINTA));
    for (i, texto) in lista.iter().take(MAX_AVISOS).enumerate() {
        let (x, yy) = (X0 + 18.0 + (i % 2) as f32 * (coluna + 16.0), y + 42.0 + (i / 2) as f32 * PASSO_AVISO);
        let (corte, cortou) = c.cortado(texto, coluna - 8.0, &letra);
        c.p.circle_filled(c.pt(x, yy - 3.5), 1.6, AMBAR.gamma_multiply(0.8));
        let largura = c.texto(&corte, x + 10.0, yy, letra);
        if cortou {
            let _ = ui.interact(c.caixa(x, yy - 11.0, largura + 10.0, 15.0), Id::unique(("plano-aviso", i)), Sense::hover()).on_hover_text(texto.as_str());
        }
    }
}

/// O nome do blueprint, o workspace e o que o último gesto respondeu: a recusa de integrar tem de aparecer aqui.
pub fn titulo(ui: &Ui, c: &Pincel, nome: &str, workspace: &str, aviso: &str, x1: f32) {
    let nome = if nome.is_empty() { tr("Sem nome") } else { nome };
    let letra = Letra::nova(18.0, TINTA).forte().trilha(0.06);
    c.texto(&c.cortado(&nome.to_uppercase(), 400.0, &letra).0, X0, 62.0, letra);
    let letra = Letra::nova(10.5, MEDIO).trilha(0.1);
    c.texto(&c.cortado(&format!("{workspace} · {}", tr("Agentes")).trim_start_matches(" · ").to_uppercase(), 400.0, &letra).0, X0, 82.0, letra);
    if aviso.is_empty() {
        return;
    }
    let letra = Letra::nova(10.5, AMBAR);
    let (t, cortou) = c.cortado(aviso, x1 - X0, &letra);
    let largura = c.texto(&t, X0, 104.0, letra);
    if cortou {
        let _ = ui.interact(c.caixa(X0, 93.0, largura, 15.0), Id::unique("plano-agentes-aviso"), Sense::hover()).on_hover_text(aviso);
    }
}
