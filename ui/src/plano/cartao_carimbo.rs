//! O que fica da revisão depois de decidida: o resultado (APROVADO, EM REVISÃO, REPROVADO) com ícone, cor e texto;
//! o «desfazer», a situação escrita com o ponto que respira enquanto espera você.

use super::acao_estilo::{mix, ponto};
use super::acao_movimento::{hover, pulso_breve};
use super::caixinha::Lapis;
use super::cartao_botao::{icone, Icone};
use super::linha::Pincel;
use super::revisao::{rotulo, EstadoPeca};
use crate::fonte::{Peso, SUBIDA};
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema::{BAIXO, MEDIO, TINTA, VERMELHO};
use egui::{vec2, Color32, Rect, Sense, Ui};

const MS_POR_DIA: i64 = 86_400_000;

fn data_de(em: i64) -> Option<String> {
    (em > 0).then(|| {
        let iso = crate::documento::iso_do_dia(em / MS_POR_DIA);
        format!("{}/{}", &iso[8..10], &iso[5..7])
    })
}

fn icone_do(e: EstadoPeca) -> Icone {
    match e {
        EstadoPeca::Aprovada => Icone::Check,
        EstadoPeca::Reprovada => Icone::Xis,
        _ => Icone::Lapis,
    }
}

/// O resultado da revisão, na linha do título: o ícone e o texto em caixa alta, na cor do estado. `x` é a borda direita. Devolve a borda esquerda.
pub fn carimbo(l: Lapis, x: f32, y: f32, estado: EstadoPeca, em: i64) -> f32 {
    let (rot, cor) = rotulo(estado);
    let texto = data_de(em).map_or_else(|| rot.to_string(), |d| format!("{rot} · {d}")).to_uppercase();
    let t = Texto::novo(texto, 10.0).peso(Peso::Negrito).trilha(0.1).cor(cor);
    let x0 = x - t.largura(l.p) - 16.0;
    icone(l.p, l.pt(x0 + 5.0, y - 3.5), icone_do(estado), cor);
    t.em(l.p, l.pt(x0 + 16.0, y - SUBIDA * 10.0));
    x0
}

/// A situação escrita, com o ponto na cor (e o halo que respira enquanto espera você): nunca só a cor. `fim`: `x` é a borda direita. `curta`: pendente vira «Revisar».
pub fn situacao(l: Lapis, x: f32, y: f32, estado: EstadoPeca, fim: bool, curta: bool) {
    // o que espera você não se escreve: os botões à vista já dizem
    if estado == EstadoPeca::Pendente && !curta {
        return;
    }
    let (rot, cor) = rotulo(estado);
    let t = if curta && estado == EstadoPeca::Pendente { tr("Revisar") } else { rot };
    let x0 = if fim { x - Texto::novo(t, 10.0).largura(l.p) - 12.0 } else { x };
    let halo = if estado == EstadoPeca::Pendente { pulso_breve(l.p.ctx(), ("situacao", x as i32, y as i32), 2.4, 6.0) } else { 0.0 };
    ponto(l.p, l.pt(x0 + 3.5, y - 3.5), cor, 3.0, halo);
    l.texto(t, x0 + 12.0, y, 10.0, Peso::Normal, cor);
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
