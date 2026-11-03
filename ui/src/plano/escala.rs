//! A escala da folha (`folha.mjs:16`): a posição de cada ação na escada, na ordem do plano, cada uma começando onde a anterior termina.

use super::modelo::{Acao, Folha};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

/// A margem da folha, o começo da régua e a altura de uma ação fechada.
pub const X0: f32 = 70.0;
pub const XR: f32 = 646.0;
pub const LINHA: f32 = 46.0;
/// A altura de uma ação na folha: a miniatura da obra e o respiro.
pub const LINHA_FOLHA: f32 = 104.0;
/// Quanto a régua ocupa depois da margem direita: `X1 = largura - MARGEM`.
pub const MARGEM: f32 = 86.0;

/// A escada: um passo por ação, de largura igual, em `x0..x1`.
#[derive(Clone, Debug, PartialEq)]
pub struct Escala {
    pub x0: f32,
    pub x1: f32,
    passos: Arc<HashMap<String, usize>>,
}

/// O passo de cada ação, contado uma vez por folha (`Folha::geracao`): a folha e «O que chegou» pedem a escala a cada quadro.
fn passos_da(f: &Folha) -> Arc<HashMap<String, usize>> {
    thread_local! {
        static ULTIMA: RefCell<Option<(u64, Arc<HashMap<String, usize>>)>> = const { RefCell::new(None) };
    }
    let contar = || -> Arc<HashMap<String, usize>> { Arc::new(f.projetos.iter().flat_map(|p| &p.acoes).enumerate().map(|(i, a)| (a.referencia.clone(), i)).collect()) };
    if f.geracao == 0 {
        return contar();
    }
    ULTIMA.with(|u| {
        let mut u = u.borrow_mut();
        match u.as_ref() {
            Some((g, passos)) if *g == f.geracao => passos.clone(),
            _ => {
                let passos = contar();
                *u = Some((f.geracao, passos.clone()));
                passos
            }
        }
    })
}

impl Escala {
    /// As ações de todos os projetos da folha, na ordem em que aparecem.
    pub fn de_folha(f: &Folha, x0: f32, x1: f32) -> Escala {
        Escala { x0, x1, passos: passos_da(f) }
    }

    /// Quantos passos a escada tem.
    pub fn total(&self) -> usize {
        self.passos.len()
    }

    /// O x do começo do passo `i`.
    pub fn x(&self, i: usize) -> f32 {
        self.x0 + i as f32 / self.total().max(1) as f32 * (self.x1 - self.x0)
    }

    /// O trecho da ação na escada. `None` se ela não está na folha.
    pub fn trecho(&self, a: &Acao) -> Option<(f32, f32)> {
        let i = *self.passos.get(&a.referencia)?;
        Some((self.x(i), self.x(i + 1)))
    }
}
