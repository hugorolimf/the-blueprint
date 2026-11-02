//! A régua de «O que chegou», sem os títulos das colunas, que não existem aqui (`colunas: false` do protótipo): os meses, as semanas e o hoje tracejado.

use super::super::escala::{curta, Escala};
use super::super::linha::{Letra, Pincel};
use crate::documento::iso_do_dia;
use crate::i18n::{tr, tr_com};
use crate::tema::{MEDIO, PAPEL, TINTA};

const MESES: [&str; 12] = ["JAN", "FEV", "MAR", "ABR", "MAI", "JUN", "JUL", "AGO", "SET", "OUT", "NOV", "DEZ"];
const Y_REGUA: f32 = 138.0;

/// A régua desta tela, sem os títulos das colunas, que não existem aqui (`colunas: false` do protótipo): `escala::regua` sempre os escreve, e cairiam sobre as datas.
pub fn regua(c: &Pincel, e: &Escala, y_fim: f32) {
    let y = Y_REGUA;
    c.fio(e.x0, y, e.x1, y, TINTA, 1.0);
    let (xh, por_dia) = (e.x(e.hoje), e.x(e.hoje + 1) - e.x(e.hoje));
    let (ini, fim) = (e.hoje - ((xh - e.x0) / por_dia).floor() as i64, e.hoje + ((e.x1 - xh) / por_dia).floor() as i64);
    let cabe_dia = por_dia * 7.0 >= 18.0;
    for d in ini..=fim {
        let (x, iso) = (e.x(d), iso_do_dia(d));
        if iso.get(8..10) == Some("01") {
            c.fio(x, y - 9.0, x, y, TINTA, 1.0);
            let mes = iso.get(5..7).and_then(|m| m.parse::<usize>().ok()).and_then(|m| MESES.get(m.wrapping_sub(1)));
            if let (Some(mes), true) = (mes, x + 30.0 < e.x1) {
                c.texto(tr(*mes), x + 4.0, y - 11.0, Letra::nova(10.0, TINTA).trilha(0.1));
            }
        } else if (d + 3).rem_euclid(7) == 0 {
            c.fio(x, y - 4.0, x, y, MEDIO, 1.0);
            if cabe_dia {
                c.texto(iso.get(8..10).unwrap_or(""), x, y + 12.0, Letra::nova(8.5, MEDIO).meio());
            }
        }
    }
    c.tracejada(xh, y, xh, y_fim, TINTA.gamma_multiply(0.8), &[12.0, 2.0], &[3.0, 3.0]);
    let rotulo = tr_com("Hoje {data}", &[("data", &curta(&iso_do_dia(e.hoje)))]).to_uppercase();
    let letra = Letra::nova(9.5, PAPEL).forte().trilha(0.1).meio();
    let largura = c.largura(&rotulo, &letra) + 12.0;
    c.ret(xh - largura / 2.0, y - 24.0, largura, 15.0, Some(TINTA), None);
    c.texto(&rotulo, xh, y - 13.0, letra);
}
