//! O estado vazio da tela de agentes: a folha com a caixinha que se marca e o agente que aparece, desenhados em traço isométrico, e os três passos
//! que se acendem na mesma batida da figura. Diz o que fazer em vez de deixar um vazio; o botão leva de volta à folha, onde mora a caixinha.

use super::super::agentes_movimento as mov;
use super::super::linha::{Letra, Pincel};
use super::super::vazio_figuras::{self, AGENTES_TAM};
use crate::i18n::tr;
use crate::planta::{self, botao, Botao, Texto};
use crate::tema::{AZUL, BAIXO, CARTAO, MEDIO, PAPEL, TINTA};
use egui::{Stroke, Ui};

pub const ALTURA: f32 = 292.0;
const K: f32 = 27.0;
const LARGURA_TEXTO: f32 = 390.0;
const VAO_FIGURA: f32 = 36.0;
/// Onde cada passo termina na volta da figura (0 a 1), a mesma de `vazio_figuras::agentes`.
const PASSOS: [(&str, f32); 3] = [("Marque a caixinha de um plano na folha", 0.38), ("O orquestrador prepara o agente daquele plano", 0.6), ("O agente aparece aqui e começa a trabalhar", 1.0)];

fn passo_aceso(c: &Pincel) -> usize {
    if !mov::ligado() {
        return 2;
    }
    let t = mov::fase(c.p.ctx(), 7.0);
    PASSOS.iter().position(|(_, fim)| t < *fim).unwrap_or(2)
}

/// Os passos numerados: o aceso tem o número cheio, os outros só o contorno.
fn passos(c: &Pincel, x: f32, y: f32) {
    let aceso = passo_aceso(c);
    for (i, (texto, _)) in PASSOS.iter().enumerate() {
        let yy = y + i as f32 * 26.0;
        let ligado = ctx_t(c, i, aceso);
        let cor = MEDIO.lerp_to_gamma(TINTA, ligado);
        let centro = c.pt(x + 9.0, yy - 4.0);
        c.p.circle(centro, 9.0, TINTA.gamma_multiply(ligado), Stroke::new(1.0, cor));
        c.texto(&(i + 1).to_string(), x + 9.0, yy, Letra::nova(9.5, if ligado > 0.5 { PAPEL } else { cor }).forte().meio());
        c.texto(tr(*texto), x + 28.0, yy, Letra::nova(11.0, cor));
    }
}

fn ctx_t(c: &Pincel, i: usize, aceso: usize) -> f32 {
    c.p.ctx().animate_bool_with_time(egui::Id::unique(("agentes-vazio-passo", i)), i == aceso, 0.25)
}

/// O painel vazio com o topo em `y`, de `x0` a `x1`. Devolve se o botão foi clicado.
pub fn desenhar(ui: &mut Ui, c: &Pincel, y: f32, x0: f32, x1: f32) -> bool {
    let ctx = ui.ctx().clone();
    let t = mov::entrada(&ctx, "agentes", 8);
    let entra = mov::Entra::nova(c, t);
    let c = &entra.pincel();
    let w = x1 - x0;
    let area = c.caixa(x0, y, w, ALTURA);
    c.p.rect_filled(area, 0, CARTAO.gamma_multiply(0.4));
    planta::tracejado(c.p, area, BAIXO.gamma_multiply(0.7));
    planta::canto(c.p, area);
    let fig_w = AGENTES_TAM.0 * K;
    let grupo = fig_w + VAO_FIGURA + LARGURA_TEXTO;
    let xs = x0 + ((w - grupo) / 2.0).max(16.0);
    vazio_figuras::agentes(c.p, c.pt(xs + fig_w / 2.0, y + ALTURA / 2.0 + 2.0), K);

    let xt = xs + fig_w + VAO_FIGURA;
    c.rotulo(tr("Agentes"), xt, y + 50.0, Letra::nova(9.5, AZUL).trilha(0.16));
    c.texto(tr("Nenhum agente por aqui"), xt, y + 78.0, Letra::nova(19.0, TINTA).forte());
    Texto::novo(tr("Marque a caixinha de um plano na folha para chamar um agente."), 11.5).cor(MEDIO).caixa(LARGURA_TEXTO).linhas(2).lh(17.0).em(c.p, c.pt(xt, y + 92.0));
    passos(c, xt, y + 150.0);
    let abrir = Botao { cheio: true, ..super::super::caixinha::botao_mini(tr("Abrir a folha")) };
    mov::com_opacidade(ui, t, |ui| botao(ui, c.pt(xt, y + ALTURA - 62.0), &abrir).clicked())
}
