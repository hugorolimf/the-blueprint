//! A faixa de indicadores da tela de agentes: cinco cartões com o número grande, o rótulo e o traço de cor do estado, e o cartão do limite
//! com `−`, `+`, `Configurar` e as vagas (as que estão em uso acendem). Só desenha e devolve o gesto do limite.

use super::super::agentes_movimento as mov;
use super::super::agentes_pecas as pecas;
use super::super::caixinha::botao_mini;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::Agentes;
use super::super::orquestrador::{alternar_folhinha, Pedido};
use super::limite_novo;
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, AZUL, BAIXO, CARTAO, MEDIO, TINTA, VERDE, VERMELHO};
use egui::{vec2, Color32, Rect, Stroke, StrokeKind, Ui};
use serde_json::json;

pub const ALTURA: f32 = 78.0;
const LARGURA_LIMITE: f32 = 312.0;
const VAO: f32 = 10.0;
const MAX_CARTAO: f32 = 280.0;

fn fundo() -> Color32 {
    CARTAO.gamma_multiply(0.85)
}

/// Um indicador: o traço de cor no topo (apagado quando é zero), o número que anda até o valor e o rótulo.
#[allow(clippy::too_many_arguments)]
fn indicador(ui: &Ui, c: &Pincel, rotulo: &'static str, valor: u32, cor: Color32, pulsa: bool, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx();
    let cor_viva = mov::suavizar(ctx, ("kpi-cor", rotulo), if valor > 0 { cor } else { MEDIO.gamma_multiply(0.5) });
    pecas::cartao(c, x, y, w, ALTURA, fundo(), BAIXO.gamma_multiply(0.5), Some(cor.gamma_multiply(if valor > 0 { 0.95 } else { 0.28 })));
    let numero = mov::suave(ctx, ("kpi", rotulo), valor as f32).round() as u32;
    c.texto(&numero.to_string(), x + 16.0, y + 45.0, Letra::nova(30.0, cor_viva).forte());
    c.texto(tr(rotulo), x + 16.0, y + 65.0, Letra::nova(10.5, MEDIO));
    if pulsa && valor > 0 {
        pecas::ponto(c, x + w - 18.0, y + 20.0, cor, true);
    }
}

/// O botão pequeno de `−` ou `+`; fica sem ação na ponta do limite.
fn passo(ui: &mut Ui, c: &Pincel, texto: &str, ativo: bool, x: f32, y: f32) -> bool {
    botao(ui, c.pt(x, y), &Botao { ativo, ..botao_mini(texto) }).clicked()
}

/// As vagas do limite: uma por agente que pode trabalhar ao mesmo tempo; as em uso acendem e respiram.
fn vagas(c: &Pincel, limite: u32, em_uso: u32, x: f32, y: f32, largura: f32) {
    let n = limite.clamp(1, 16) as usize;
    let lado = ((largura - (n - 1) as f32 * 4.0) / n as f32).clamp(5.0, 12.0);
    let respira = if em_uso > 0 { 0.78 + 0.22 * mov::onda(mov::fase(c.p.ctx(), 1.8)) } else { 1.0 };
    for i in 0..n {
        let aceso = (i as u32) < em_uso;
        let t = c.p.ctx().animate_bool_with_time(egui::Id::unique(("vaga", i)), aceso, 0.35);
        let r = Rect::from_min_size(c.pt(x + i as f32 * (lado + 4.0), y + (24.0 - lado) / 2.0), vec2(lado, lado));
        c.p.rect_filled(r, 0, AZUL.gamma_multiply(t * respira));
        c.p.rect_stroke(r, 0, Stroke::new(1.0, BAIXO.lerp_to_gamma(AZUL, t)), StrokeKind::Inside);
    }
}

/// O cartão do limite: «Agentes ao mesmo tempo», o número com `−` e `+`, as vagas e `Configurar`. Devolve o limite novo, se mudou.
fn limite_card(ui: &mut Ui, c: &Pincel, a: &Agentes, limite: Option<u32>, ws: &str, x: f32, y: f32, w: f32) -> Option<Pedido> {
    pecas::cartao(c, x, y, w, ALTURA, fundo(), BAIXO.gamma_multiply(0.5), Some(MEDIO.gamma_multiply(0.45)));
    c.rotulo(tr("Agentes ao mesmo tempo"), x + 14.0, y + 22.0, Letra::nova(9.5, MEDIO).trilha(0.1));
    let configurar = botao_mini(tr("Configurar"));
    if botao(ui, c.pt(x + w - 10.0 - configurar.largura(c.p), y + 8.0), &configurar).clicked() {
        alternar_folhinha(ui.ctx());
    }
    let delta = |d: i32| limite.and_then(|l| limite_novo(l, d));
    let (menos, mais) = (botao_mini("−"), botao_mini("+"));
    let (lm, ym) = (menos.largura(c.p), y + 40.0);
    let mut pedido = None;
    if passo(ui, c, "−", delta(-1).is_some(), x + 14.0, ym) {
        pedido = delta(-1);
    }
    let x_mais = x + 14.0 + lm + 38.0;
    if passo(ui, c, "+", delta(1).is_some(), x_mais, ym) {
        pedido = delta(1);
    }
    c.texto(&limite.map_or_else(|| "—".to_string(), |l| l.to_string()), x + 14.0 + lm + 19.0, ym + 17.0, Letra::nova(15.0, TINTA).forte().meio());
    if let Some(l) = limite {
        let xv = x_mais + mais.largura(c.p) + 16.0;
        vagas(c, l, a.numeros.trabalhando, xv, ym, x + w - 14.0 - xv);
    }
    pedido.map(|l| ("limite", json!({"workspaceId": ws, "limite": l})))
}

/// A faixa inteira em `(x0, y)` até `x1`: os cinco indicadores à esquerda e o limite à direita.
#[allow(clippy::too_many_arguments)]
pub fn faixa(ui: &mut Ui, c: &Pincel, a: &Agentes, limite: Option<u32>, ws: &str, x0: f32, y: f32, x1: f32) -> Option<Pedido> {
    let n = &a.numeros;
    let lista = [(n.trabalhando, "trabalhando", AZUL, true), (n.fila, "na fila", MEDIO, false), (n.esperando, "esperando", AMBAR, false), (n.parados, "parados", VERMELHO, false), (n.prontos, "prontos", VERDE, false)];
    let largura = (((x1 - x0) - LARGURA_LIMITE - VAO * 5.0) / 5.0).clamp(120.0, MAX_CARTAO);
    for (i, (valor, rotulo, cor, pulsa)) in lista.into_iter().enumerate() {
        let t = mov::entrada(ui.ctx(), "agentes", 1 + i);
        let entra = mov::Entra::nova(c, t);
        indicador(ui, &entra.pincel(), rotulo, valor, cor, pulsa, x0 + i as f32 * (largura + VAO), y, largura);
    }
    let t = mov::entrada(ui.ctx(), "agentes", 6);
    let entra = mov::Entra::nova(c, t);
    mov::com_opacidade(ui, t, |ui| limite_card(ui, &entra.pincel(), a, limite, ws, x1 - LARGURA_LIMITE, y, LARGURA_LIMITE))
}
