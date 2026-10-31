//! O papel de planta: gradiente radial e as grades de 12 e 60 px (`.entrada` e `body.sem-perfil` em web/css/entrada.css).

use crate::tema::{rgba, PAPEL};
use egui::epaint::Mesh;
use egui::{pos2, vec2, Color32, Painter, Rect, Shape};

#[derive(Clone, Copy)]
pub enum Papel {
    /// a entrada, o novo workspace e as folhas: radial de baixo para o centro e grade mais marcada
    Entrada,
    /// o fundo da tela de nome (`body.sem-perfil`): base lisa, um clareado e grade suave
    Login,
}

fn mistura(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Gradiente radial em elipse até o canto mais distante (`ellipse at cx cy`), por vértices coloridos de uma malha.
fn radial(p: &Painter, r: Rect, centro: (f32, f32), cor: impl Fn(f32) -> Color32) {
    let (cx, cy) = (r.min.x + r.width() * centro.0, r.min.y + r.height() * centro.1);
    let (rx, ry) = ((r.width() * centro.0).max(r.width() * (1.0 - centro.0)) * 2f32.sqrt(), (r.height() * centro.1).max(r.height() * (1.0 - centro.1)) * 2f32.sqrt());
    let (colunas, linhas) = (((r.width() / 32.0).ceil() as usize).clamp(4, 80), ((r.height() / 32.0).ceil() as usize).clamp(4, 120));
    let mut m = Mesh::default();
    for j in 0..=linhas {
        for i in 0..=colunas {
            let pos = pos2(r.min.x + r.width() * i as f32 / colunas as f32, r.min.y + r.height() * j as f32 / linhas as f32);
            let t = (((pos.x - cx) / rx).powi(2) + ((pos.y - cy) / ry).powi(2)).sqrt();
            m.colored_vertex(pos, cor(t));
        }
    }
    for j in 0..linhas {
        for i in 0..colunas {
            let a = (j * (colunas + 1) + i) as u32;
            let b = a + 1;
            let c = a + (colunas + 1) as u32;
            m.add_triangle(a, b, c);
            m.add_triangle(b, c + 1, c);
        }
    }
    p.add(Shape::mesh(m));
}

/// Uma linha de 1 px (horizontal ou vertical) cortada pela área visível.
fn fio(p: &Painter, r: Rect, visivel: Rect, horizontal: bool, pos: f32, cor: Color32) {
    let l = if horizontal { Rect::from_min_size(pos2(r.min.x, r.min.y + pos), vec2(r.width(), 1.0)) } else { Rect::from_min_size(pos2(r.min.x + pos, r.min.y), vec2(1.0, r.height())) };
    let l = l.intersect(visivel);
    if l.is_positive() {
        p.rect_filled(l, 0, cor);
    }
}

/// O papel inteiro da página `r` (que pode ser maior que a janela: só o que aparece é desenhado nas grades).
pub fn papel(p: &Painter, r: Rect, estilo: Papel) {
    let visivel = r.intersect(p.clip_rect());
    let (maior, menor) = match estilo {
        Papel::Entrada => {
            p.rect_filled(r, 0, PAPEL);
            radial(p, r, (0.5, 0.4), |t| if t <= 0.62 { mistura(rgb(0x1250a0), PAPEL, t / 0.62) } else { mistura(PAPEL, rgb(0x082b5e), (t - 0.62) / 0.38) });
            (rgba(0xffffff, 0x1c), rgba(0xffffff, 0x0c))
        }
        Papel::Login => {
            p.rect_filled(r, 0, rgb(0x0b3268));
            radial(p, r, (0.45, 0.35), |t| rgba(0x14509b, (0x55 as f32 * (1.0 - t / 0.7)).clamp(0.0, 255.0) as u8));
            (rgba(0xffffff, 0x12), rgba(0xffffff, 0x06))
        }
    };
    // o CSS põe a grade fina e a grossa em camadas: onde coincidem, as duas aparecem
    let (de, ate) = (((visivel.min.y - r.min.y) / 12.0).floor().max(0.0) as i32, ((visivel.max.y - r.min.y) / 12.0).ceil() as i32);
    for i in de..=ate {
        fio(p, r, visivel, true, i as f32 * 12.0, menor);
    }
    let (de, ate) = (((visivel.min.x - r.min.x) / 12.0).floor().max(0.0) as i32, ((visivel.max.x - r.min.x) / 12.0).ceil() as i32);
    for i in de..=ate {
        fio(p, r, visivel, false, i as f32 * 12.0, menor);
    }
    let (de, ate) = (((visivel.min.y - r.min.y) / 60.0).floor().max(0.0) as i32, ((visivel.max.y - r.min.y) / 60.0).ceil() as i32);
    for i in de..=ate {
        fio(p, r, visivel, true, i as f32 * 60.0, maior);
    }
    let (de, ate) = (((visivel.min.x - r.min.x) / 60.0).floor().max(0.0) as i32, ((visivel.max.x - r.min.x) / 60.0).ceil() as i32);
    for i in de..=ate {
        fio(p, r, visivel, false, i as f32 * 60.0, maior);
    }
}

fn rgb(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

