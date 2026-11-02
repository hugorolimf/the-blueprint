//! A régua de meses do blueprint aberto (`regua` em web/js/render.js): a linha, as marcas, o nome de cada mês, o ano, as semanas, os guias
//! pontilhados e o eixo e a etiqueta de «hoje».

use super::cena::{Ancora, Cena};
use crate::documento::{dia, iso_do_dia};
use crate::dominio::{mes_iso, Janela};
use crate::fonte::Peso;
use crate::i18n::tr;
use crate::tema::{rgba, MEDIO, PAPEL, TENUE, TINTA};
use egui::{vec2, Rect, Sense, Ui};

pub struct Regua<'a> {
    pub janela: &'a Janela,
    pub l: f32,
    pub r: f32,
    pub y: f32,
    /// o y onde os guias terminam (`yBand - 20`)
    pub fundo: f32,
    pub hoje: i64,
    /// o trecho `(de, ate)` em y em que o eixo do hoje não passa: a ação aberta, para ele não cortar os textos
    pub sem_eixo: Option<(f32, f32)>,
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

/// Desenha a régua. Os nomes dos meses são clicáveis (`.lk`): devolve o mês `aaaa-mm` clicado. `aguardando`: o mês inicial já marcado, aguardando o fim.
pub fn desenhar(c: &Cena, ui: &mut Ui, g: &Regua, aguardando: Option<&str>) -> Option<String> {
    let mut clicado = None;
    let j = g.janela;
    let nm = (j.fim - j.inicio + 1) as usize;
    let meses: Vec<&str> = tr("JAN|FEV|MAR|ABR|MAI|JUN|JUL|AGO|SET|OUT|NOV|DEZ").split('|').collect();
    let (ymin, mes_inicial) = (j.inicio.div_euclid(12), j.inicio.rem_euclid(12));
    // na abertura (barras.js): a linha corre da esquerda, os guias descem um depois do outro, o «hoje» desce e a etiqueta surge por último
    let ab = crate::anima::atual(ui.ctx());
    let passo = |atraso: f32, dur: f32| ab.map_or(1.0, |a| a.passo(atraso, dur, crate::anima::BARRA));
    c.fio(TINTA, 1.0, (g.l, g.y), (g.l + (g.r - g.l) * passo(0.0, 500.0), g.y));
    for m in 0..=nm {
        let x = g.mes(m);
        c.fio(TINTA, 1.0, (x, g.y - 10.0), (x, g.y));
        let k = passo(150.0 + 350.0 * m as f32 / nm.max(1) as f32, 450.0);
        if k > 0.0 {
            Cena { p: &super::barra::opaco(c.p, k), o: c.o }.tracejado(TENUE, 1.0, (x, g.y), (x, g.y + (g.fundo - g.y) * k), &[2.0, 5.0]);
        }
        if m < nm {
            let meio = (x + g.mes(m + 1)) / 2.0;
            let nome = meses.get(((mes_inicial + m as i64) % 12) as usize).copied().unwrap_or("");
            let iso = mes_iso(j.inicio + m as i64);
            let area = Rect::from_min_size(c.pt(x, g.y - 32.0), vec2(g.mes(m + 1) - x, 30.0));
            let resposta = ui.interact(area, ui.scope_id().with(("mes", iso.as_str())), Sense::click());
            if aguardando == Some(iso.as_str()) {
                c.retangulo(x, g.y - 32.0, g.mes(m + 1) - x, 30.0, Some(rgba(0xffffff, 0x33)), None);
            }
            if resposta.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resposta.clicked() {
                clicado = Some(iso);
            }
            c.texto_sublinhado(nome, meio, g.y - 16.0, 11.0, if resposta.hovered() { Peso::Negrito } else { Peso::Normal }, TINTA, Ancora::Meio, false);
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
        let fim = g.y + (g.fundo - g.y) * passo(650.0, 500.0);
        match g.sem_eixo {
            Some((de, ate)) if de < fim => {
                c.tracejado(TINTA, 1.0, (xt, g.y), (xt, de), &[16.0, 4.0, 2.0, 4.0]);
                if ate < fim {
                    c.tracejado(TINTA, 1.0, (xt, ate), (xt, fim), &[16.0, 4.0, 2.0, 4.0]);
                }
            }
            _ => c.tracejado(TINTA, 1.0, (xt, g.y), (xt, fim), &[16.0, 4.0, 2.0, 4.0]),
        }
        let k = passo(1000.0, 280.0);
        if k > 0.0 {
            let (s, ce) = (0.6 + 0.4 * k, Cena { p: &super::barra::opaco(c.p, k), o: c.o });
            ce.retangulo(xt - 46.0 * s, g.y + 18.0 - 10.0 * s, 92.0 * s, 20.0 * s, Some(TINTA), None);
            let iso = iso_do_dia(g.hoje);
            ce.texto_liso(&format!("{} {}/{}", tr("HOJE"), &iso[8..10], &iso[5..7]), xt, g.y + 22.0, 11.0, Peso::Negrito, PAPEL, Ancora::Meio);
        }
    }
    clicado
}
