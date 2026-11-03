//! O palco do «Novo workspace»: uma margem de prancha com réguas e miras, a figura do Blueprint se montando e o carimbo com o que foi digitado.

use crate::emblema;
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::planta::{self, Texto};
use crate::tema::{rgba, BAIXO, MEDIO, TINTA};
use egui::{pos2, vec2, Painter, Rect, Ui};

const MARGEM: f32 = 24.0;
const PASSO: f32 = 18.0;
const CARIMBO_W: f32 = 440.0;
const LINHA: f32 = 34.0;
const ROTULO_W: f32 = 92.0;
/// A proporção dos quadros da figura (700 por 500).
const FIGURA: f32 = 1.4;

pub fn desenhar(ui: &Ui, r: Rect, nome: &str, descricao: &str, montagem: f32) {
    let p = ui.painter().clone();
    reguas(&p, r);
    let dentro = Rect::from_min_max(r.min + vec2(MARGEM, MARGEM), r.max);
    miras(&p, dentro);
    // a figura fica acima do carimbo, centrada
    let altura = (dentro.width() - 16.0) / FIGURA;
    let altura = altura.min(dentro.height() - 2.0 * LINHA - 32.0).min(600.0).max(120.0);
    let tam = vec2((altura * FIGURA).round(), altura.round());
    let vao = dentro.height() - 2.0 * LINHA - 16.0 - tam.y;
    let figura = Rect::from_min_size(pos2((dentro.center().x - tam.x / 2.0).round(), (dentro.min.y + vao / 2.0).round()), tam);
    // sai do traço para o sólido, com a curva da casa: rápido no começo
    emblema::pintar_passo(ui, figura, 1.0 - (1.0 - montagem).powi(3));
    carimbo(&p, dentro, nome, descricao);
}

/// As margens de prancha: letras em cima, números do lado e um traço maior a cada cinco.
fn reguas(p: &Painter, r: Rect) {
    let ate = |tam: f32| ((tam - MARGEM) / PASSO).floor() as usize;
    for i in 0..=ate(r.width()) {
        let x = r.min.x + MARGEM + i as f32 * PASSO;
        planta::linha(p, pos2(x, r.min.y + 18.0 - if i % 5 == 0 { 9.0 } else { 4.0 }), pos2(x, r.min.y + 18.0), BAIXO.gamma_multiply(0.7));
        if i % 5 == 0 && (x + 18.0) < r.max.x {
            let letra = char::from(b'A' + ((i / 5) % 26) as u8).to_string();
            Texto::novo(letra, 9.0).cor(BAIXO).na_linha(p, x + 5.0, r.min.y - 2.0, 12.0);
        }
    }
    for i in 0..=ate(r.height()) {
        let y = r.min.y + MARGEM + i as f32 * PASSO;
        planta::linha(p, pos2(r.min.x + 18.0 - if i % 5 == 0 { 9.0 } else { 4.0 }, y), pos2(r.min.x + 18.0, y), BAIXO.gamma_multiply(0.7));
        if i % 5 == 0 && (y + 18.0) < r.max.y {
            Texto::novo((i / 5 + 1).to_string(), 9.0).cor(BAIXO).na_linha(p, r.min.x, y + 3.0, 12.0);
        }
    }
}

fn miras(p: &Painter, r: Rect) {
    const T: f32 = 13.0;
    for (canto, dx, dy) in [(r.left_top(), T, T), (r.right_top(), -T, T), (r.left_bottom(), T, -T), (r.right_bottom(), -T, -T)] {
        planta::linha(p, canto, canto + vec2(dx, 0.0), MEDIO);
        planta::linha(p, canto, canto + vec2(0.0, dy), MEDIO);
    }
}

/// O carimbo de prancha, no canto de baixo, com o que já foi digitado.
fn carimbo(p: &Painter, dentro: Rect, nome: &str, descricao: &str) {
    let w = CARIMBO_W.min(dentro.width() - 32.0);
    let r = Rect::from_min_size(pos2(dentro.max.x - 16.0 - w, dentro.max.y - 16.0 - 2.0 * LINHA), vec2(w, 2.0 * LINHA));
    p.rect_filled(r, 0, rgba(0x082b5e, 0xd9));
    planta::borda(p, r, BAIXO);
    planta::linha(p, pos2(r.min.x + ROTULO_W, r.min.y), pos2(r.min.x + ROTULO_W, r.max.y), BAIXO);
    for i in 1..2 {
        let y = r.min.y + i as f32 * LINHA;
        planta::linha(p, pos2(r.min.x, y), pos2(r.max.x, y), BAIXO);
    }
    let util = w - ROTULO_W - 20.0;
    let (nome, descricao) = (nome.trim(), descricao.trim().replace('\n', " "));
    for (i, rotulo) in [tr("Workspace"), tr("Descrição")].into_iter().enumerate() {
        let y = r.min.y + i as f32 * LINHA;
        Texto::novo(rotulo.to_uppercase(), 9.0).cor(MEDIO).trilha(0.14).na_linha(p, r.min.x + 10.0, y, LINHA);
        let x = r.min.x + ROTULO_W + 10.0;
        match i {
            0 if nome.is_empty() => Texto::novo(tr("Sem nome"), 14.0).peso(Peso::Medio).cor(BAIXO),
            0 => Texto::novo(nome, 14.0).peso(Peso::Medio).cor(TINTA),
            _ if descricao.is_empty() => Texto::novo("—", 12.0).cor(MEDIO),
            _ => Texto::novo(descricao.as_str(), 12.0).cor(MEDIO),
        }
        .caixa(util)
        .linhas(1)
        .na_linha(p, x, y, LINHA);
    }
}
