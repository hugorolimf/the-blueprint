//! O pincel das telas do Planejar: coordenadas da página, texto com a linha de base em `y` e corte medido de verdade.

use crate::fonte::{Peso, SUBIDA};
use crate::planta::Texto;
use crate::tema::{PAPEL, TINTA};
use egui::{vec2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

#[derive(Clone, Copy, PartialEq)]
pub enum Ancora {
    Inicio,
    Meio,
    Fim,
}

/// Como um texto é escrito.
#[derive(Clone, Copy)]
pub struct Letra {
    pub tam: f32,
    pub cor: Color32,
    pub peso: Peso,
    pub espaco: f32,
    pub ancora: Ancora,
}

impl Letra {
    pub fn nova(tam: f32, cor: Color32) -> Letra {
        Letra { tam, cor, peso: Peso::Normal, espaco: 0.06, ancora: Ancora::Inicio }
    }
    pub fn forte(self) -> Letra {
        Letra { peso: Peso::Negrito, ..self }
    }
    pub fn trilha(self, espaco: f32) -> Letra {
        Letra { espaco, ..self }
    }
    pub fn meio(self) -> Letra {
        Letra { ancora: Ancora::Meio, ..self }
    }
    pub fn fim(self) -> Letra {
        Letra { ancora: Ancora::Fim, ..self }
    }
}

/// O pincel das telas do Planejar: coordenadas da página, texto com a linha de base em `y`.
pub struct Pincel<'a> {
    pub p: &'a Painter,
    pub o: Pos2,
}

impl Pincel<'_> {
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        self.o + vec2(x, y)
    }

    pub fn caixa(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pt(x, y), vec2(w, h))
    }

    pub fn ret(&self, x: f32, y: f32, w: f32, h: f32, fundo: Option<Color32>, traco: Option<(Color32, f32)>) {
        let r = self.caixa(x, y, w, h);
        if let Some(f) = fundo {
            self.p.rect_filled(r, 0, f);
        }
        if let Some((cor, largura)) = traco {
            self.p.rect_stroke(r, 0, Stroke::new(largura, cor), StrokeKind::Middle);
        }
    }

    /// Retângulo só de contorno, tracejado `traco vao`.
    pub fn ret_tracejado(&self, x: f32, y: f32, w: f32, h: f32, cor: Color32, traco: f32, vao: f32) {
        let pts = [self.pt(x, y), self.pt(x + w, y), self.pt(x + w, y + h), self.pt(x, y + h), self.pt(x, y)];
        self.p.extend(Shape::dashed_line(&pts, Stroke::new(1.2, cor), traco, vao));
    }

    pub fn fio(&self, x1: f32, y1: f32, x2: f32, y2: f32, cor: Color32, largura: f32) {
        self.p.line_segment([self.pt(x1, y1), self.pt(x2, y2)], Stroke::new(largura, cor));
    }

    pub fn tracejada(&self, x1: f32, y1: f32, x2: f32, y2: f32, cor: Color32, tracos: &[f32], vaos: &[f32]) {
        self.p.extend(Shape::dashed_line_with_offset(&[self.pt(x1, y1), self.pt(x2, y2)], Stroke::new(1.0, cor), tracos, vaos, 0.0));
    }

    pub fn largura(&self, s: &str, l: &Letra) -> f32 {
        Texto::novo(s, l.tam).peso(l.peso).trilha(l.espaco).largura(self.p)
    }

    /// Escreve `s` com a linha de base em `y` e devolve a largura. Mede e pinta com a mesma galeria.
    pub fn texto(&self, s: &str, x: f32, y: f32, l: Letra) -> f32 {
        Texto::novo(s, l.tam).peso(l.peso).trilha(l.espaco).cor(l.cor).em_medindo(self.p, |largura| {
            let esquerda = match l.ancora {
                Ancora::Inicio => x,
                Ancora::Meio => x - largura / 2.0,
                Ancora::Fim => x - largura,
            };
            self.pt(esquerda, y - SUBIDA * l.tam)
        })
    }

    /// O mesmo texto em caixa alta (rótulos e títulos de coluna).
    pub fn rotulo(&self, s: &str, x: f32, y: f32, l: Letra) -> f32 {
        self.texto(&s.to_uppercase(), x, y, l)
    }

    /// Corta `s` com «…» para caber em `max` pixels, medindo de verdade. Devolve o texto e se cortou.
    pub fn cortado(&self, s: &str, max: f32, l: &Letra) -> (String, bool) {
        if self.largura(s, l) <= max {
            return (s.to_string(), false);
        }
        let letras: Vec<char> = s.chars().collect();
        let (mut cabe, mut nao) = (0, letras.len());
        while nao - cabe > 1 {
            let meio = (cabe + nao) / 2;
            let t: String = letras[..meio].iter().chain(['…'].iter()).collect();
            if self.largura(&t, l) <= max {
                cabe = meio;
            } else {
                nao = meio;
            }
        }
        (letras[..cabe].iter().chain(['…'].iter()).collect(), true)
    }

    /// O losango de checkpoint, cheio de tinta se feito.
    pub fn losango(&self, x: f32, y: f32, r: f32, feito: bool) {
        let pts = vec![self.pt(x, y - r), self.pt(x + r, y), self.pt(x, y + r), self.pt(x - r, y)];
        let fundo = if feito { TINTA } else { PAPEL };
        self.p.add(Shape::convex_polygon(pts, fundo, Stroke::new(1.2, TINTA)));
    }
}
