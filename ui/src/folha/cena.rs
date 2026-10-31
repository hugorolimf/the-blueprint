//! As peças do desenho da folha, nas coordenadas do SVG da página: o traço é centrado na geometria, o texto tem a linha de base em `y`,
//! o espaçamento de `.06em` e o contorno de 3 px na cor do papel que o CSS põe atrás das letras (`paint-order: stroke`).

use crate::fonte::Peso;
use crate::planta::Texto;
use crate::tema::{BAIXO, MEDIO, PAPEL, TINTA};
use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

/// O quanto a linha de base fica abaixo do topo da letra, em múltiplos do tamanho (a JetBrains Mono sobe 1,02 do corpo).
const SUBIDA: f32 = 1.02;

#[derive(Clone, Copy, PartialEq)]
pub enum Ancora {
    Inicio,
    Meio,
    Fim,
}

/// O traço das classes `.ln`, `.ln.m` e `.ln.l` de web/css/base.css (`.guia` e `.eixo`, tracejados, entram com a linha do tempo).
#[derive(Clone, Copy)]
pub enum Fio {
    Ln,
    Medio,
    Baixo,
}

pub struct Cena<'a> {
    pub p: &'a Painter,
    pub o: Pos2,
}

impl Cena<'_> {
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        self.o + vec2(x, y)
    }

    pub fn retangulo(&self, x: f32, y: f32, w: f32, h: f32, fundo: Option<Color32>, traco: Option<(Color32, f32)>) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w, h));
        if let Some(f) = fundo {
            self.p.rect_filled(r, 0, f);
        }
        if let Some((cor, largura)) = traco {
            self.p.rect_stroke(r, 0, Stroke::new(largura, cor), StrokeKind::Middle);
        }
    }

    /// Retângulo só de contorno, com traço contínuo (`ln`) ou tracejado `6 4` (as ações a fazer).
    pub fn moldura(&self, x: f32, y: f32, w: f32, h: f32, cor: Color32, largura: f32, tracejado: bool) {
        if !tracejado {
            return self.retangulo(x, y, w, h, None, Some((cor, largura)));
        }
        let pts = [self.pt(x, y), self.pt(x + w, y), self.pt(x + w, y + h), self.pt(x, y + h), self.pt(x, y)];
        self.p.extend(Shape::dashed_line(&pts, Stroke::new(largura, cor), 6.0, 4.0));
    }

    /// Segmentos soltos `(x1, y1, x2, y2)` com o traço de `fio`.
    pub fn fios(&self, fio: Fio, segmentos: &[(f32, f32, f32, f32)]) {
        let cor = match fio {
            Fio::Ln => TINTA,
            Fio::Medio => MEDIO,
            Fio::Baixo => BAIXO,
        };
        for &(x1, y1, x2, y2) in segmentos {
            self.p.line_segment([self.pt(x1, y1), self.pt(x2, y2)], Stroke::new(1.0, cor));
        }
    }

    /// Fio com traço e espessura livres (`stroke-width` do `style` da página).
    pub fn fio(&self, cor: Color32, largura: f32, de: (f32, f32), ate: (f32, f32)) {
        self.p.line_segment([self.pt(de.0, de.1), self.pt(ate.0, ate.1)], Stroke::new(largura, cor));
    }

    /// Escreve `s` com a linha de base em `y`. Devolve a largura. `cap`: caixa alta (a classe `cap`).
    #[allow(clippy::too_many_arguments)]
    pub fn texto(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32, ancora: Ancora, cap: bool) -> f32 {
        let s = if cap { s.to_uppercase() } else { s.to_string() };
        let t = Texto::novo(s.as_str(), tam).peso(peso).trilha(0.06);
        let largura = t.largura(self.p);
        let esquerda = match ancora {
            Ancora::Inicio => x,
            Ancora::Meio => x - largura / 2.0,
            Ancora::Fim => x - largura,
        };
        let topo = self.pt(esquerda, y - SUBIDA * tam);
        // o contorno de 3 px na cor do papel: o texto corta a grade sem apagar o fundo em volta
        let halo = Texto::novo(s.as_str(), tam).peso(peso).trilha(0.06).cor(PAPEL);
        let g = halo.galeria(self.p);
        for (dx, dy) in [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (-1.1, -1.1), (1.1, -1.1), (-1.1, 1.1), (1.1, 1.1)] {
            self.p.galley(topo + vec2(dx, dy), g.clone(), PAPEL);
        }
        t.cor(cor).em(self.p, topo);
        largura
    }

    /// A legenda da marca-d'água: `legenda-marca` (espaçamento .16em) com a opacidade de `.marca-fundo` (18%), centrada em `x`, sem contorno.
    pub fn texto_apagado(&self, s: &str, x: f32, y: f32, tam: f32) {
        let t = Texto::novo(s, tam).trilha(0.16).cor(MEDIO.gamma_multiply(0.18));
        let largura = t.largura(self.p);
        t.em(self.p, self.pt(x - largura / 2.0, y - SUBIDA * tam));
    }

    /// O hachurado de `pattern#hach`: linhas a 45° espaçadas de 6 px, em `#9cc4ee` com 1,5 px.
    pub fn hachura(&self, x: f32, y: f32, w: f32, h: f32) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w, h));
        let p = self.p.with_clip_rect(r.intersect(self.p.clip_rect()));
        let passo = 6.0 * 2f32.sqrt();
        let mut k = -h;
        while k < w + h {
            p.line_segment([pos2(r.min.x + k, r.max.y), pos2(r.min.x + k + h, r.min.y)], Stroke::new(1.5, MEDIO));
            k += passo;
        }
    }

    /// O losango de checkpoint (`.dm`): cheio de tinta se feito.
    pub fn losango(&self, x: f32, yc: f32, feito: bool) {
        let pts = vec![self.pt(x, yc - 5.0), self.pt(x + 4.0, yc), self.pt(x, yc + 5.0), self.pt(x - 4.0, yc)];
        let (fundo, traco) = if feito { (TINTA, PAPEL) } else { (PAPEL, TINTA) };
        self.p.add(Shape::convex_polygon(pts, fundo, Stroke::new(1.4, traco)));
    }
}
