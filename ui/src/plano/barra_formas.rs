//! As formas da folha que o pincel básico não tem: retângulo de canto suave, sombra, brilho, tracejado arredondado, losango com sombra, anel de progresso e o visto.
//! São métodos do `Pincel`, na mesma medida (coordenadas da página).

use crate::plano::folha::estilo::{com_alfa, meio};
use super::{Letra, Pincel};
use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, Pos2, Rect, Shape, Stroke, StrokeKind};

/// Toda forma da planta tem canto reto, como a folha original: o raio pedido não muda o desenho.
fn canto(_r: f32) -> CornerRadius {
    CornerRadius::ZERO
}

/// O contorno de um retângulo, como pontos, para poder tracejar.
fn contorno(r: Rect) -> Vec<Pos2> {
    vec![r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()]
}

impl Pincel<'_> {
    /// Retângulo de canto suave (`r`), cheio e/ou só de traço (por dentro, para não passar da caixa).
    #[allow(clippy::too_many_arguments)]
    pub fn suave(&self, x: f32, y: f32, w: f32, h: f32, r: f32, fundo: Option<Color32>, traco: Option<(Color32, f32)>) {
        let caixa = self.caixa(x, y, w, h);
        if let Some(f) = fundo {
            self.p.rect_filled(caixa, canto(r), f);
        }
        if let Some((cor, largura)) = traco {
            self.p.rect_stroke(caixa, canto(r), Stroke::new(largura, cor), StrokeKind::Inside);
        }
    }

    /// Só o contorno tracejado, de canto suave.
    #[allow(clippy::too_many_arguments)]
    pub fn suave_tracejado(&self, x: f32, y: f32, w: f32, h: f32, _r: f32, cor: Color32, traco: f32, vao: f32) {
        let caixa = self.caixa(x, y, w, h).shrink(0.6);
        self.p.extend(Shape::dashed_line(&contorno(caixa), Stroke::new(1.2, cor), traco, vao));
    }

    /// A sombra difusa de uma peça: `desfoque` e `dy` em pontos.
    #[allow(clippy::too_many_arguments)]
    pub fn sombra(&self, x: f32, y: f32, w: f32, h: f32, r: f32, cor: Color32, desfoque: u8, dy: i8) {
        let s = Shadow { offset: [0, dy], blur: desfoque, spread: 0, color: cor };
        self.p.add(s.as_shape(self.caixa(x, y, w, h), canto(r)));
    }

    /// Um brilho que respira em volta da peça.
    #[allow(clippy::too_many_arguments)]
    pub fn brilho(&self, x: f32, y: f32, w: f32, h: f32, r: f32, cor: Color32, alcance: u8) {
        let s = Shadow { offset: [0, 0], blur: alcance, spread: 1, color: cor };
        self.p.add(s.as_shape(self.caixa(x, y, w, h), canto(r)));
    }

    /// O fio claro no alto de uma peça cheia, que a deixa com volume sem sair do traço fino.
    pub fn reflexo(&self, x: f32, y: f32, w: f32, r: f32, cor: Color32) {
        if w > 2.0 * r + 2.0 {
            self.fio(x + r, y + 1.5, x + w - r, y + 1.5, cor, 1.0);
        }
    }

    /// Texto no meio vertical de `yc`.
    pub fn texto_meio(&self, s: &str, x: f32, yc: f32, l: Letra) -> f32 {
        self.texto(s, x, yc + meio(l.tam), l)
    }

    /// A etiqueta: texto num fundo de canto suave, com o meio em `yc` e o começo em `x`. Devolve a largura.
    #[allow(clippy::too_many_arguments)]
    pub fn etiqueta(&self, s: &str, x: f32, yc: f32, l: Letra, fundo: Color32, traco: Option<Color32>, pad: f32) -> f32 {
        let w = self.largura(s, &l) + pad * 2.0;
        let h = l.tam + 8.0;
        self.suave(x, yc - h / 2.0, w, h, h / 2.0 - 1.0, Some(fundo), traco.map(|c| (c, 1.0)));
        self.texto_meio(s, x + pad, yc, l);
        w
    }

    /// O losango de checkpoint: sombra leve, e cheio da cor `feito` quando concluído.
    pub fn marco(&self, x: f32, y: f32, r: f32, feito: Option<Color32>, vazio: Color32, traco: Color32) {
        let pts = |dy: f32, k: f32| vec![self.pt(x, y - r * k + dy), self.pt(x + r * k, y + dy), self.pt(x, y + r * k + dy), self.pt(x - r * k, y + dy)];
        self.p.add(Shape::convex_polygon(pts(1.8, 1.05), com_alfa(Color32::BLACK, 0.28), Stroke::NONE));
        let (fundo, borda) = feito.map_or((vazio, traco), |c| (c, c));
        self.p.add(Shape::convex_polygon(pts(0.0, 1.0), fundo, Stroke::new(1.3, borda)));
    }

    /// Um arco de círculo (`parte` de 0 a 1 da volta, a partir do alto) de `largura` pontos; com `fundo`, o círculo inteiro apagado por baixo.
    #[allow(clippy::too_many_arguments)]
    pub fn anel(&self, x: f32, y: f32, raio: f32, largura: f32, parte: f32, cor: Color32, fundo: Option<Color32>) {
        let centro = self.pt(x, y);
        if let Some(f) = fundo {
            self.p.circle_stroke(centro, raio, Stroke::new(largura, f));
        }
        let parte = parte.clamp(0.0, 1.0);
        if parte <= 0.0 {
            return;
        }
        let passos = ((parte * 48.0).ceil() as usize).max(2);
        let pts: Vec<Pos2> = (0..=passos)
            .map(|k| {
                let a = -std::f32::consts::FRAC_PI_2 + parte * std::f32::consts::TAU * k as f32 / passos as f32;
                centro + egui::vec2(raio * a.cos(), raio * a.sin())
            })
            .collect();
        self.p.add(Shape::line(pts, Stroke::new(largura, cor)));
    }

    /// O visto, com o meio em `(x, y)`, de lado `s`.
    pub fn visto(&self, x: f32, y: f32, s: f32, cor: Color32, largura: f32) {
        let pts = vec![self.pt(x - s * 0.5, y + s * 0.02), self.pt(x - s * 0.12, y + s * 0.4), self.pt(x + s * 0.55, y - s * 0.38)];
        self.p.add(Shape::line(pts, Stroke::new(largura, cor)));
    }

    /// Um ponto cheio, com o halo opcional (`halo` de 0 a 1) em volta.
    pub fn ponto(&self, x: f32, y: f32, raio: f32, cor: Color32, halo: f32) {
        if halo > 0.0 {
            self.p.circle_filled(self.pt(x, y), raio + 2.0 + 2.5 * halo, com_alfa(cor, 0.22 * halo));
        }
        self.p.circle_filled(self.pt(x, y), raio, cor);
    }
}
