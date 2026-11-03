//! A régua do blueprint aberto (`regua` em web/js/render.js): a linha, as marcas dos passos e os guias pontilhados. As ações ficam em escada, então
//! a régua conta passos na ordem do blueprint, e não tempo.

use super::cena::Cena;
use crate::dominio::Janela;
use crate::tema::{TENUE, TINTA};
use egui::Ui;

/// Quantas marcas a régua e a moldura mostram, por mais ações que o blueprint tenha.
const MARCAS_MAX: i64 = 12;

pub struct Regua<'a> {
    pub janela: &'a Janela,
    pub l: f32,
    pub r: f32,
    pub y: f32,
    /// o y onde os guias terminam (`yBand - 20`)
    pub fundo: f32,
}

impl Regua<'_> {
    pub fn x(&self, d: i64) -> f32 {
        self.l + (d - self.janela.t0) as f32 / (self.janela.t1 - self.janela.t0).max(1) as f32 * (self.r - self.l)
    }

    /// O x das marcas, sempre com o começo e o fim da régua.
    pub fn marcas(&self) -> Vec<f32> {
        let n = (self.janela.t1 - self.janela.t0).max(1);
        let salto = ((n + MARCAS_MAX - 1) / MARCAS_MAX).max(1);
        let mut xs: Vec<f32> = (0..n).step_by(salto as usize).map(|k| self.x(self.janela.t0 + k)).collect();
        xs.push(self.x(self.janela.t1));
        xs
    }
}

/// Desenha a régua. Na abertura (barras.js): a linha corre da esquerda e os guias descem um depois do outro.
pub fn desenhar(c: &Cena, ui: &Ui, g: &Regua) {
    let ab = crate::anima::atual(ui.ctx());
    let passo = |atraso: f32, dur: f32| ab.map_or(1.0, |a| a.passo(atraso, dur, crate::anima::BARRA));
    c.fio(TINTA, 1.0, (g.l, g.y), (g.l + (g.r - g.l) * passo(0.0, 500.0), g.y));
    let marcas = g.marcas();
    for (m, &x) in marcas.iter().enumerate() {
        c.fio(TINTA, 1.0, (x, g.y - 10.0), (x, g.y));
        let k = passo(150.0 + 350.0 * m as f32 / marcas.len().max(1) as f32, 450.0);
        if k > 0.0 {
            Cena { p: &super::barra::opaco(c.p, k), o: c.o }.tracejado(TENUE, 1.0, (x, g.y), (x, g.y + (g.fundo - g.y) * k), &[2.0, 5.0]);
        }
    }
}
