//! O texto com tamanho, peso, cor e espaçamento entre letras, medido como a página mede (`letter-spacing`, `line-height`).

use crate::fonte::{familia, Peso};
use crate::tema::TINTA;
use egui::text::{LayoutJob, TextFormat};
use egui::{pos2, Align2, Color32, FontId, Galley, Painter, Pos2, Rect};
use std::borrow::Cow;
use std::sync::Arc;

/// Um texto com tamanho, peso, cor e espaçamento entre letras (`letter-spacing`, em px). `lh` é a altura de linha do CSS.
pub struct Texto<'a> {
    s: Cow<'a, str>,
    tam: f32,
    peso: Peso,
    cor: Color32,
    trilha: f32,
    largura: f32,
    linhas: usize,
    lh: Option<f32>,
    centrado: bool,
}

impl<'a> Texto<'a> {
    pub fn novo(s: impl Into<Cow<'a, str>>, tam: f32) -> Self {
        Texto { s: s.into(), tam, peso: Peso::Normal, cor: TINTA, trilha: 0.0, largura: f32::INFINITY, linhas: usize::MAX, lh: None, centrado: false }
    }
    pub fn peso(mut self, p: Peso) -> Self {
        self.peso = p;
        self
    }
    pub fn cor(mut self, c: Color32) -> Self {
        self.cor = c;
        self
    }
    /// `letter-spacing` em múltiplos do tamanho (`.1` = `.1em`).
    pub fn trilha(mut self, em: f32) -> Self {
        self.trilha = em * self.tam;
        self
    }
    pub fn caixa(mut self, largura: f32) -> Self {
        self.largura = largura;
        self
    }
    pub fn linhas(mut self, n: usize) -> Self {
        self.linhas = n;
        self
    }
    pub fn lh(mut self, px: f32) -> Self {
        self.lh = Some(px);
        self
    }
    /// `text-align: center` dentro da `caixa`.
    pub fn centrado(mut self) -> Self {
        self.centrado = true;
        self
    }
    pub fn galeria(&self, p: &Painter) -> Arc<Galley> {
        let mut job = LayoutJob::default();
        // o egui encolhe/arredonda o avanço de cada letra e a página não (a JetBrains Mono avança 0,6 do tamanho): a diferença volta no espaçamento
        let ajuste = 0.6 * self.tam - avanco_real(p, self.tam, self.peso);
        job.append(&self.s, 0.0, TextFormat { font_id: FontId::new(self.tam, familia(self.peso)), color: self.cor, extra_letter_spacing: self.trilha + ajuste, line_height: self.lh, ..Default::default() });
        if self.centrado {
            job.halign = egui::Align::Center;
        }
        // na página o espaço que fecha a linha fica pendurado; no egui ele conta na largura: soma um avanço para o texto quebrar no mesmo lugar
        job.wrap.max_width = if self.largura.is_finite() { self.largura + 0.6 * self.tam } else { self.largura };
        job.wrap.max_rows = self.linhas;
        job.wrap.overflow_character = Some('…');
        p.layout_job(job)
    }
    /// A largura como o CSS a conta: o espaçamento entre letras também vem depois da última.
    pub fn largura(&self, p: &Painter) -> f32 {
        self.galeria(p).size().x + self.trilha
    }
    pub fn altura(&self, p: &Painter) -> f32 {
        self.galeria(p).size().y
    }
    /// Pinta com o canto de cima à esquerda em `pos`.
    pub fn em(&self, p: &Painter, pos: Pos2) -> Rect {
        let g = self.galeria(p);
        let r = Rect::from_min_size(pos, g.size());
        p.galley(pos, g, self.cor);
        r
    }
    /// Pinta numa caixa de linha de altura `altura` (como o `line-height` do CSS): o texto fica no meio dela.
    pub fn na_linha(&self, p: &Painter, x: f32, topo: f32, altura: f32) -> Rect {
        let g = self.galeria(p);
        self.em(p, pos2(x, topo + (altura - g.size().y) / 2.0))
    }
    /// Pinta com o canto de cima à direita em `(direita, topo)` na caixa de linha.
    pub fn na_linha_dir(&self, p: &Painter, direita: f32, topo: f32, altura: f32) -> Rect {
        let l = self.largura(p);
        self.na_linha(p, direita - l, topo, altura)
    }
    /// Pinta o bloco (de `caixa` de largura) centralizado em `cx`, com o topo em `topo`.
    pub fn centrado_em(&self, p: &Painter, cx: f32, topo: f32) -> Rect {
        let g = self.galeria(p);
        let r = Rect::from_min_size(pos2(cx - self.largura.min(g.size().x.max(1.0)) / 2.0, topo), g.size());
        p.galley(pos2(cx, topo), g, self.cor);
        r
    }
    pub fn centro(&self, p: &Painter, centro: Pos2) -> Rect {
        let g = self.galeria(p);
        let r = Align2::CENTER_CENTER.anchor_size(centro, g.size());
        p.galley(r.min, g, self.cor);
        r
    }
}

/// O avanço de uma letra como o egui a mede neste tamanho e escala (medido numa linha de 16 letras, porque cada uma é arredondada).
fn avanco_real(p: &Painter, tam: f32, peso: Peso) -> f32 {
    let mut job = LayoutJob::default();
    job.append("MMMMMMMMMMMMMMMM", 0.0, TextFormat { font_id: FontId::new(tam, familia(peso)), ..Default::default() });
    p.layout_job(job).size().x / 16.0
}

