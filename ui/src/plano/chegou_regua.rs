//! A régua de «O que chegou»: a linha e uma marca por passo da escada.

use super::super::escala::Escala;
use super::super::linha::Pincel;
use crate::tema::TINTA;

const Y_REGUA: f32 = 138.0;

pub fn regua(c: &Pincel, e: &Escala) {
    c.fio(e.x0, Y_REGUA, e.x1, Y_REGUA, TINTA, 1.0);
    for i in 0..=e.total().max(1) {
        c.fio(e.x(i), Y_REGUA - 9.0, e.x(i), Y_REGUA, TINTA, 1.0);
    }
}
