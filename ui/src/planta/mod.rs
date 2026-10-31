//! As primitivas do desenho de planta, medidas em pixels do CSS (1 ponto do egui = 1 px da página): o papel com grade e gradiente,
//! o texto com espaçamento entre letras, as marcas de canto, o tracejado e o botão `.btn`.

mod botao;
mod papel;
mod texto;

pub use botao::{botao, botao_com_largura, Botao};
pub use papel::{papel, Papel};
pub use texto::Texto;

use crate::tema::TINTA;
use egui::{vec2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

/// A marca de canto (`::before` dos cartões): um L de 24 px com 2 px de linha no canto de cima à esquerda.
pub fn canto(p: &Painter, r: Rect) {
    p.rect_filled(Rect::from_min_size(r.min, vec2(24.0, 2.0)), 0, TINTA);
    p.rect_filled(Rect::from_min_size(r.min, vec2(2.0, 24.0)), 0, TINTA);
}

pub fn borda(p: &Painter, r: Rect, cor: Color32) {
    p.rect_stroke(r, 0, Stroke::new(1.0, cor), StrokeKind::Inside);
}

/// Borda tracejada de 1 px (`border: 1px dashed`).
pub fn tracejado(p: &Painter, r: Rect, cor: Color32) {
    let r = r.shrink(0.5);
    let pontos = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    p.extend(Shape::dashed_line(&pontos, Stroke::new(1.0, cor), 3.0, 3.0));
}

/// Uma linha reta de 1 px.
pub fn linha(p: &Painter, de: Pos2, ate: Pos2, cor: Color32) {
    p.add(Shape::line_segment([de, ate], Stroke::new(1.0, cor)));
}


#[cfg(test)]
mod testes {
    use super::*;

    /// A largura de uma linha de texto monoespaçado confere com o avanço de 0,6 do tamanho que a página usa.
    #[test]
    fn largura_do_texto_bate_com_a_pagina() {
        let ctx = egui::Context::default();
        crate::fonte::instalar(&ctx);
        for ppp in [1.0f32, 1.25, 2.0] {
            ctx.set_pixels_per_point(ppp);
            ctx.run_pass(egui::RawInput::default(), |_| {}).textures_delta.clear(); // as fontes só valem a partir do primeiro quadro
            let mut medidas = Vec::new();
            ctx.run_pass(egui::RawInput::default(), |ctx| {
                let p = ctx.layer_painter(egui::LayerId::background());
                for (s, tam, trilha) in [("ROADMAP", 11.0, 0.1), ("TROCAR NOME", 11.0, 0.1), ("Escolha um workspace para continuar.", 14.0, 0.0), ("WORKSPACES", 13.0, 0.2)] {
                    medidas.push((s, Texto::novo(s, tam).trilha(trilha).largura(&p), s.chars().count() as f32 * (0.6 * tam + trilha * tam)));
                }
            })
            .textures_delta
            .clear();
            for (s, medida, esperado) in medidas {
                println!("ppp {ppp}: {s:40} medida {medida:7.2} esperado {esperado:7.2}");
                assert!((medida - esperado).abs() < 0.6, "ppp {ppp}: {s}: {medida} != {esperado}");
            }
        }
    }
}
