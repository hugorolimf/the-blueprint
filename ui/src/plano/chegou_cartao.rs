//! O cartão de um plano em «O que chegou»: o novo vem como resumo (título, quantos planos, quem mandou e quando) com `Colocar` cheio; o alterado traz a contagem do diff
//! e `Ver diff`; o removido diz que sai do plano ao aprovar. O traço à esquerda e as fichas dão a cor; o texto diz o mesmo por escrito.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::agentes_tempo as tempo;
use super::super::caixinha::botao_mini;
use super::super::linha::{Letra, Pincel};
use super::lista::{Item, Oferta, Tipo};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, BAIXO, CAMPO, MEDIO, TINTA, VERDE, VERMELHO};
use egui::{Color32, Id, Sense, Ui};

pub const ALTO_NOVO: f32 = 84.0;
pub const ALTO_OUTRO: f32 = 64.0;
pub const VAO: f32 = 8.0;

pub fn alto(tipo: Tipo) -> f32 {
    if tipo == Tipo::Novo {
        ALTO_NOVO
    } else {
        ALTO_OUTRO
    }
}

/// A cor do tipo: o novo é tinta, o alterado âmbar, o removido vermelho.
pub fn cor(tipo: Tipo) -> Color32 {
    match tipo {
        Tipo::Novo => TINTA,
        Tipo::Alterado => AMBAR,
        Tipo::Removido => VERMELHO,
    }
}

/// A cor de cada pedaço do diff: o que entra é verde, o que sai vermelho, o que muda âmbar.
pub fn cor_da_ficha(texto: &str) -> Color32 {
    match texto.chars().next() {
        Some('+') if !texto.contains("det") => VERDE,
        Some('−') => VERMELHO,
        Some('~') => AMBAR,
        _ => MEDIO,
    }
}

/// O cartão com o topo em `y`, de `x` a `x + w`; `indice` é a vez dele na cascata. Devolve o botão clicado.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, item: &Item, tipo: Tipo, indice: usize, agente: &str, numero: i64, x: f32, y: f32, w: f32) -> Option<Oferta> {
    let ctx = ui.ctx().clone();
    let t = mov::entrada(&ctx, "chegou", 1 + indice);
    let sobre = ui.rect_contains_pointer(c.caixa(x, y, w, alto(tipo)));
    let sobre = ctx.animate_bool_with_time(Id::unique(("chegou-sobre", &item.referencia)), sobre, 0.15);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let (h, tom) = (alto(tipo), cor(tipo));
    pecas::cartao(c, x, y, w, h, CAMPO.gamma_multiply(0.55 + 0.25 * sobre), BAIXO.gamma_multiply(0.5 + 0.3 * sobre), None);
    c.ret(x, y, 3.0, h, Some(tom), None);

    let b = item.oferta.map(|o| match o {
        Oferta::VerDiff => botao_mini(tr("Ver diff")),
        Oferta::Colocar => Botao { cheio: true, ..botao_mini(tr("Colocar")) },
    });
    let reserva = b.as_ref().map_or(0.0, |b| b.largura(c.p) + 12.0);
    let largura_ref = pecas::ficha(c, x + 14.0, y + 10.0, &item.referencia, tom);
    let forte = Letra::nova(11.5, TINTA).forte();
    let (nome, cortou) = c.cortado(&item.nome, w - 14.0 - largura_ref - 8.0 - reserva - 12.0, &forte);
    let escrita = c.texto(&nome, x + 14.0 + largura_ref + 8.0, y + 22.0, forte);
    if cortou {
        let _ = ui.interact(c.caixa(x + 14.0, y + 8.0, largura_ref + 8.0 + escrita, 18.0), Id::unique(("plano-chegou-item", &item.referencia)), Sense::hover()).on_hover_text(item.nome.as_str());
    }
    let mut xf = x + 14.0;
    for (texto, cor) in &item.fichas {
        xf += pecas::ficha(c, xf, y + 36.0, texto, *cor) + 6.0;
    }
    if item.fichas.is_empty() {
        c.texto(tr("ainda sem planos"), xf, y + 48.0, Letra::nova(10.0, MEDIO));
    }
    if tipo == Tipo::Novo {
        let quem = if agente.is_empty() { tr("O agente").to_string() } else { agente.to_string() };
        let visto = tempo::escrito(tempo::visto_ha(&ctx, ("chegou", numero)));
        let frase = tr_com("enviado por {agente} · visto há {tempo}", &[("agente", &quem), ("tempo", &visto)]);
        let letra = Letra::nova(9.5, MEDIO);
        c.texto(&c.cortado(&frase, w - 28.0, &letra).0, x + 14.0, y + 70.0, letra);
    }
    let b = b?;
    let topo = y + 8.0;
    let clicou = mov::com_opacidade(ui, t, |ui| botao(ui, c.pt(x + w - 12.0 - b.largura(c.p), topo), &b).clicked());
    clicou.then_some(item.oferta).flatten().filter(|_| t > 0.6)
}
