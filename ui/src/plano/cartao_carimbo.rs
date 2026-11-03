//! O que fica da revisão depois de decidida: o resultado (APROVADO, EM REVISÃO, REPROVADO) com ícone, cor e texto;
//! e o «desfazer».

use super::acao_estilo::mix;
use super::acao_movimento::hover;
use super::caixinha::Lapis;
use super::cartao_botao::{icone, Icone};
use super::linha::Pincel;
use super::revisao::{rotulo_de, EstadoPeca, Meio};
use crate::fonte::{Peso, SUBIDA};
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema::{BAIXO, MEDIO, TINTA, VERMELHO};
use egui::{vec2, Color32, Rect, Sense, Ui};

fn icone_do(e: EstadoPeca) -> Icone {
    match e {
        EstadoPeca::Aprovada => Icone::Check,
        EstadoPeca::Reprovada => Icone::Xis,
        _ => Icone::Lapis,
    }
}

/// O resultado da revisão, na linha do título: o ícone e o texto em caixa alta, na cor do estado. `x` é a borda direita. Devolve a borda esquerda.
pub fn carimbo(l: Lapis, x: f32, y: f32, estado: EstadoPeca, meio: Meio) -> f32 {
    let (rot, cor) = rotulo_de(estado, meio);
    let texto = rot.to_uppercase();
    let t = Texto::novo(texto, 10.0).peso(Peso::Negrito).trilha(0.1).cor(cor);
    let x0 = x - t.largura(l.p) - 16.0;
    icone(l.p, l.pt(x0 + 5.0, y - 3.5), icone_do(estado), cor);
    t.em(l.p, l.pt(x0 + 16.0, y - SUBIDA * 10.0));
    x0
}

/// O risco sobre a peça reprovada.
pub fn risco(l: Lapis, x0: f32, x1: f32, y: f32) {
    l.fio(VERMELHO.gamma_multiply(0.7), 1.0, (x0, y), (x1, y));
}

/// O «desfazer» ao lado do selo; `x` é a borda direita. A cor e o sublinhado entram no hover. Devolve se foi clicado.
pub fn desfazer(l: Lapis, ui: &mut Ui, chave: &str, x: f32, y: f32) -> bool {
    let t = tr("desfazer");
    let largura = Texto::novo(t, 9.5).largura(l.p);
    let r = Rect::from_min_size(l.pt(x - largura, y - 12.0), vec2(largura, 16.0));
    let resposta = ui.interact(r, ui.scope_id().with(("plano-revisao-desfazer", chave)), Sense::click());
    let h = hover(ui.ctx(), ("desfazer", chave), resposta.hovered());
    let cor: Color32 = mix(MEDIO, TINTA, h);
    l.texto(t, x - largura, y, 9.5, Peso::Normal, cor);
    let c = Pincel { p: l.p, o: l.o };
    c.fio(x - largura, y + 3.5, x - largura + largura * (0.35 + 0.65 * h), y + 3.5, mix(BAIXO, TINTA, h), 1.0);
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta.clicked()
}
