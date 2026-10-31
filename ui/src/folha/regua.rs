//! A régua de meses do blueprint aberto (`regua` em web/js/render.js): a linha, as marcas, o nome de cada mês, o ano, as semanas, os guias
//! pontilhados e o eixo e a etiqueta de «hoje».

use super::cena::{Ancora, Cena};
use crate::documento::{dia, iso_do_dia};
use crate::dominio::{indice_mes, mes_iso, Janela};
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::tema::{MEDIO, PAPEL, TENUE, TINTA};

pub struct Regua<'a> {
    pub janela: &'a Janela,
    pub l: f32,
    pub r: f32,
    pub y: f32,
    /// o y onde os guias terminam (`yBand - 20`)
    pub fundo: f32,
    pub hoje: i64,
}

impl Regua<'_> {
    pub fn x(&self, d: i64) -> f32 {
        self.l + (d - self.janela.t0) as f32 / (self.janela.t1 - self.janela.t0).max(1) as f32 * (self.r - self.l)
    }

    /// O x de um dia fracionário (`X(dia + 0,5)`: o meio do dia).
    pub fn xf(&self, d: f32) -> f32 {
        self.l + (d - self.janela.t0 as f32) / (self.janela.t1 - self.janela.t0).max(1) as f32 * (self.r - self.l)
    }

    /// O x do começo do mês `m` (0 a NM).
    pub fn mes(&self, m: usize) -> f32 {
        self.x(dia(&format!("{}-01", mes_iso(self.janela.inicio + m as i64))).unwrap_or(self.janela.t0))
    }
}

pub fn desenhar(c: &Cena, g: &Regua) {
    let j = g.janela;
    let nm = (j.fim - j.inicio + 1) as usize;
    let meses: Vec<&str> = tr("JAN|FEV|MAR|ABR|MAI|JUN|JUL|AGO|SET|OUT|NOV|DEZ").split('|').collect();
    let (ymin, mes_inicial) = (j.inicio.div_euclid(12), j.inicio.rem_euclid(12));
    c.fio(TINTA, 1.0, (g.l, g.y), (g.r, g.y));
    for m in 0..=nm {
        let x = g.mes(m);
        c.fio(TINTA, 1.0, (x, g.y - 10.0), (x, g.y));
        c.tracejado(TENUE, 1.0, (x, g.y), (x, g.fundo), &[2.0, 5.0]);
        if m < nm {
            let meio = (x + g.mes(m + 1)) / 2.0;
            let nome = meses.get(((mes_inicial + m as i64) % 12) as usize).copied().unwrap_or("");
            c.texto_sublinhado(nome, meio, g.y - 16.0, 11.0, Peso::Normal, TINTA, Ancora::Meio, false);
            if m == 0 || (mes_inicial + m as i64) % 12 == 0 {
                c.texto(&(ymin + (mes_inicial + m as i64).div_euclid(12)).to_string(), x + 3.0, g.y - 34.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
            }
        }
    }
    let mut d = j.t0 + 7;
    while d < j.t1 {
        let x = g.x(d);
        c.fio(MEDIO, 1.0, (x, g.y - 4.0), (x, g.y));
        d += 7;
    }
    if g.hoje >= j.t0 && g.hoje < j.t1 {
        let xt = g.x(g.hoje);
        c.tracejado(TINTA, 1.0, (xt, g.y), (xt, g.fundo), &[16.0, 4.0, 2.0, 4.0]);
        c.retangulo(xt - 46.0, g.y + 8.0, 92.0, 20.0, Some(TINTA), None);
        let iso = iso_do_dia(g.hoje);
        c.texto_liso(&format!("{} {}/{}", tr("HOJE"), &iso[8..10], &iso[5..7]), xt, g.y + 22.0, 11.0, Peso::Negrito, PAPEL, Ancora::Meio);
    }
    let _ = indice_mes;
}
