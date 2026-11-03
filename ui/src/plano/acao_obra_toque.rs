//! O que o mouse tocou na cena: pela chave do sólido, um duto, uma construção ou uma peça; e a dica que cada um mostra.

use super::super::super::modelo::Plano;
use super::super::dutos::{self, Duto};
use super::super::figura::Peca;
use super::super::obra_figura::Clique;
use super::super::obra_pecas::PecaObra;
use crate::i18n::{tr, tr_com};

/// O que esta chave é: o módulo aceso e a dica do mouse.
pub fn o_que_e(planos: &[&Plano], pecas: &[Vec<PecaObra>], chave: u32) -> (Option<usize>, Option<String>) {
    if let Some(u) = duto_da_chave(planos, chave) {
        let t = tr_com("{para} depende de {de}", &[("para", &planos[u.para].referencia), ("de", &planos[u.de].referencia)]);
        return (None, Some(t));
    }
    match chave {
        0 => (None, None),
        c => {
            let (i, j) = ((c / 1000) as usize - 1, (c % 1000) as usize);
            if j == 0 {
                (Some(i), Some(format!("{}\n\n{}", planos[i].texto, tr("Clique para aproximar"))))
            } else {
                (Some(i), pecas[i].get(j - 1).map(|p| {
                    let dica = if p.tipo == Peca::Tela { format!("\n\n{}", tr("Clique para aproximar; mais um clique entra na tela.")) } else { String::new() };
                    format!("{} {}{dica}", p.numero, p.nome)
                }))
            }
        }
    }
}

pub fn duto_da_chave(planos: &[&Plano], chave: u32) -> Option<Duto> {
    let n = chave.checked_sub(1_000_000)? as usize;
    dutos::todos(planos).into_iter().find(|u| u.n == n)
}

pub fn clique_da_chave(planos: &[&Plano], pecas: &[Vec<PecaObra>], chave: u32) -> Option<Clique> {
    if let Some(u) = duto_da_chave(planos, chave) {
        return Some(Clique::Duto(u.de));
    }
    if chave == 0 {
        return None;
    }
    let (i, j) = ((chave / 1000) as usize - 1, (chave % 1000) as usize);
    if j == 0 {
        return Some(Clique::Modulo(i));
    }
    pecas[i].get(j - 1).map(|p| Clique::Peca(i, p.chave.clone()))
}

