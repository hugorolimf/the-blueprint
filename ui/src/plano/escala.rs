//! A escala da régua da folha (`folha.mjs:16`): datas para x, recortada em `XR`, e o desenho da régua. Na folha, a régua ocupa a largura toda.

use super::folha::estilo::com_alfa;
use super::linha::{Letra, Pincel};
use super::modelo::Folha;
use crate::documento::{dia, iso_do_dia};
use crate::i18n::{tr, tr_com};
use crate::tema::{MEDIO, PAPEL, TINTA};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Hoje {data}", "Today {data}"),
    ("JAN", "JAN"),
    ("FEV", "FEB"),
    ("MAR", "MAR"),
    ("ABR", "APR"),
    ("MAI", "MAY"),
    ("JUN", "JUN"),
    ("JUL", "JUL"),
    ("AGO", "AUG"),
    ("SET", "SEP"),
    ("OUT", "OCT"),
    ("NOV", "NOV"),
    ("DEZ", "DEC"),
];

/// A margem da folha, o começo da régua e a altura de uma ação fechada.
pub const X0: f32 = 70.0;
pub const XR: f32 = 646.0;
pub const LINHA: f32 = 46.0;
/// A altura de uma ação na folha: o rótulo, a barra e o respiro.
pub const LINHA_FOLHA: f32 = 64.0;
/// Quanto a régua ocupa depois da margem direita: `X1 = largura - MARGEM`.
pub const MARGEM: f32 = 86.0;
/// Abaixo disso as marcas semanais se tocariam: fica só o traço, sem o dia.
const SEMANA_COM_DIA: f32 = 18.0;
const MESES: [&str; 12] = ["JAN", "FEV", "MAR", "ABR", "MAI", "JUN", "JUL", "AGO", "SET", "OUT", "NOV", "DEZ"];

/// `2026-11-01` vira `01/11`; o que não tem esse formato fica vazio.
pub fn curta(iso: &str) -> String {
    match (iso.get(8..10), iso.get(5..7)) {
        (Some(d), Some(m)) => format!("{d}/{m}"),
        _ => String::new(),
    }
}

/// A régua: uma semana de passado e o hoje perto do começo, para o plano usar a largura toda.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Escala {
    pub x0: f32,
    pub x1: f32,
    pub hoje: i64,
    ini: f64,
    fim: f64,
}

impl Escala {
    /// `datas` são os dias que têm de caber: o fim das ações e os checkpoints.
    pub fn nova(hoje: i64, datas: impl Iterator<Item = i64>, x0: f32, x1: f32) -> Escala {
        let ultimo = datas.fold(hoje + 30, i64::max);
        let fim = (ultimo + 2) as f64;
        let ini = hoje as f64 - (fim - hoje as f64) * 0.08;
        Escala { x0, x1, hoje, ini, fim }
    }

    /// A escala da régua da folha: a mesma janela de meses da régua original, para as barras caírem nas marcas dela.
    pub fn da_janela(j: &crate::dominio::Janela, hoje: i64, x0: f32, x1: f32) -> Escala {
        Escala { x0, x1, hoje, ini: j.t0 as f64, fim: j.t1 as f64 }
    }

    /// A escala de uma folha. Ação `semData` traz datas inventadas pelo servidor, que não contam; checkpoint com data conta sempre.
    pub fn de_folha(f: &Folha, x0: f32, x1: f32) -> Escala {
        let hoje = dia(&f.hoje).or_else(|| dia(&crate::documento::hoje())).unwrap_or(0);
        let acoes = f.projetos.iter().flat_map(|p| &p.acoes);
        let fins = acoes.clone().filter(|a| !a.sem_data).filter_map(|a| dia(&a.ate));
        let marcos = acoes.flat_map(|a| &a.checkpoints).filter_map(|c| dia(&c.data));
        Escala::nova(hoje, fins.chain(marcos), x0, x1)
    }

    pub fn x(&self, d: i64) -> f32 {
        self.x0 + (((d as f64 - self.ini) / (self.fim - self.ini)) as f32) * (self.x1 - self.x0)
    }

    pub fn x_iso(&self, iso: &str) -> Option<f32> {
        dia(iso).map(|d| self.x(d))
    }

    /// O trecho `de..ate` que aparece na régua, cortado em `XR` e em `X1`; `None` se fica todo de fora. Datas invertidas são toleradas.
    pub fn recorte(&self, de: i64, ate: i64) -> Option<(f32, f32)> {
        let (a, b) = (self.x(de.min(ate)), self.x(de.max(ate)));
        (b >= self.x0 && a <= self.x1).then(|| (a.max(self.x0), b.min(self.x1)))
    }

    /// Onde escrever «sem data».
    pub fn x_sem_data(&self) -> f32 {
        self.x(self.hoje) + 8.0
    }

    /// As marcas da régua em ordem: os dias de 1º de mês e as segundas-feiras dentro dela.
    fn marcas(&self) -> impl Iterator<Item = (i64, String)> + '_ {
        (self.ini.ceil() as i64..=self.fim.floor() as i64).map(|d| (d, iso_do_dia(d)))
    }
}

/// O dia 0 é 01/01/1970, uma quinta-feira.
fn segunda(d: i64) -> bool {
    (d + 3).rem_euclid(7) == 0
}

/// A régua: meses, marcas semanais, uma grade leve até `y_fim` e a etiqueta do hoje, com a faixa do dia e a linha tracejada.
pub fn regua(c: &Pincel, e: &Escala, y: f32, y_fim: f32) {
    c.fio(e.x0, y, e.x1, y, com_alfa(TINTA, 0.9), 1.0);
    let cabe_dia = e.x(e.hoje + 7) - e.x(e.hoje) >= SEMANA_COM_DIA;
    for (d, iso) in e.marcas() {
        let x = e.x(d);
        if iso.get(8..10) == Some("01") {
            c.fio(x, y - 9.0, x, y_fim, com_alfa(TINTA, 0.1), 1.0);
            c.fio(x, y - 9.0, x, y, TINTA, 1.0);
            let mes = iso.get(5..7).and_then(|m| m.parse::<usize>().ok()).and_then(|m| MESES.get(m.wrapping_sub(1)));
            if let (Some(mes), true) = (mes, x + 30.0 < e.x1) {
                c.texto(tr(*mes), x + 5.0, y - 11.0, Letra::nova(10.0, TINTA).trilha(0.12).forte());
            }
        } else if segunda(d) {
            c.fio(x, y, x, y_fim, com_alfa(TINTA, 0.045), 1.0);
            c.fio(x, y - 4.0, x, y, MEDIO, 1.0);
            if cabe_dia {
                c.texto(iso.get(8..10).unwrap_or(""), x, y + 14.0, Letra::nova(8.5, MEDIO).meio());
            }
        }
    }
    hoje(c, e, y, y_fim);
}

/// O hoje: o eixo tracejado e a etiqueta cheia logo abaixo da régua, como na folha original (`folha/regua.rs`).
fn hoje(c: &Pincel, e: &Escala, y: f32, y_fim: f32) {
    let xh = e.x(e.hoje);
    c.tracejada(xh, y, xh, y_fim, TINTA, &[16.0, 2.0], &[4.0, 4.0]);
    c.ret(xh - 46.0, y + 8.0, 92.0, 20.0, Some(TINTA), None);
    let rotulo = tr_com("Hoje {data}", &[("data", &curta(&iso_do_dia(e.hoje)))]).to_uppercase();
    c.texto(&rotulo, xh, y + 22.0, Letra::nova(11.0, PAPEL).forte().meio());
}
