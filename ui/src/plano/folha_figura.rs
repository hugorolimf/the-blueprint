//! A pasta isométrica pequena que abre cada projeto na folha: uma laje com a aba levantada no fundo, desenhada em traço fino na cor da situação.
//! É a mesma câmera do kit (azimute 45°, elevação 30°), feita à mão com polígonos: sem arquivo e sem dependência.

use super::Pincel;
use crate::plano::folha::estilo::misturar;
use crate::tema::{CAMPO, CARTAO, PAPEL};
use egui::{Color32, Pos2, Shape, Stroke};

/// A laje `x0..x1`, `y0..y1`, `z0..z1`, em unidades do mundo.
struct Laje {
    x: (f32, f32),
    y: (f32, f32),
    z: (f32, f32),
}

/// A projeção isométrica: `x` anda para a direita e para baixo, `y` para a esquerda e para baixo, `z` sobe.
fn iso(origem: Pos2, escala: f32, (x, y, z): (f32, f32, f32)) -> Pos2 {
    origem + egui::vec2((x - y) * 0.866 * escala, ((x + y) * 0.5 - z) * escala)
}

/// Uma face de cada vez, de trás para frente: a esquerda, a direita e o alto, cada uma num tom.
fn laje(c: &Pincel, origem: Pos2, escala: f32, l: &Laje, tom: Color32) {
    let p = |x: f32, y: f32, z: f32| iso(origem, escala, (x, y, z));
    let (x0, x1, y0, y1, z0, z1) = (l.x.0, l.x.1, l.y.0, l.y.1, l.z.0, l.z.1);
    let traco = Stroke::new(1.0, misturar(tom, Color32::WHITE, 0.1));
    let esquerda = vec![p(x0, y1, z0), p(x1, y1, z0), p(x1, y1, z1), p(x0, y1, z1)];
    let direita = vec![p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1)];
    let alto = vec![p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)];
    c.p.add(Shape::convex_polygon(esquerda, misturar(CARTAO, tom, 0.16), traco));
    c.p.add(Shape::convex_polygon(direita, misturar(CAMPO, tom, 0.08), traco));
    c.p.add(Shape::convex_polygon(alto, misturar(PAPEL, tom, 0.34), traco));
}

/// A pasta com o centro do desenho em `(x, y)`, na cor `tom`. Cabe numa caixa de uns 34 x 26 pontos.
pub fn pasta(c: &Pincel, x: f32, y: f32, tom: Color32) {
    let escala = 1.45;
    // o corpo mede 14 x 10 x 2,6: o ponto do meio dele vai para `(x, y)`
    let origem = c.pt(x, y + 2.0) - iso(Pos2::ZERO, escala, (7.0, 5.0, 1.3)).to_vec2();
    let chao = [(0.4, 0.4), (14.8, 0.4), (14.8, 10.8), (0.4, 10.8)].map(|(a, b)| iso(origem, escala, (a, b, -0.5)));
    c.p.add(Shape::convex_polygon(chao.to_vec(), Color32::from_black_alpha(46), Stroke::NONE));
    laje(c, origem, escala, &Laje { x: (0.0, 14.0), y: (0.0, 10.0), z: (0.0, 2.6) }, tom);
    laje(c, origem, escala, &Laje { x: (0.0, 2.0), y: (0.0, 5.6), z: (2.6, 4.0) }, tom);
}
