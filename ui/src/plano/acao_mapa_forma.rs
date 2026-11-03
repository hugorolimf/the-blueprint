//! Onde cada plano fica na planta da ação, pelas dependências: conta pura, sem desenho.

use super::super::modelo::Plano;

/// O lugar de uma laje: a coluna e a linha, que pode ser fracionada para centralizar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lugar {
    pub col: usize,
    pub lin: f32,
}

pub fn indice(q: &[&Plano], d: &str) -> Option<usize> {
    q.iter().position(|p| p.referencia == d || p.id == d)
}

/// O nível é a maior corrente de dependências até o plano. Dependência de fora da ação não conta, e um ciclo não trava: são até `n` passadas e o nível tem teto.
/// As dependências viram índices uma vez só, e a passada que não muda nada encerra: a seguinte daria o mesmo.
pub fn niveis(q: &[&Plano]) -> Vec<usize> {
    let deps: Vec<Vec<usize>> = q.iter().map(|p| p.depende_de.iter().filter_map(|d| indice(q, d)).collect()).collect();
    let mut n = vec![0; q.len()];
    for _ in 0..q.len() {
        let mut mudou = false;
        for i in 0..q.len() {
            let maior = deps[i].iter().map(|j| n[*j] + 1).max().unwrap_or(0).min(q.len() - 1);
            mudou |= maior != n[i];
            n[i] = maior;
        }
        if !mudou {
            break;
        }
    }
    n
}

/// Cada nível é uma coluna, e cada plano do nível é uma linha, centralizada na faixa. Sem nenhuma dependência, os planos só correm em fila; o que não couber quebra para a faixa de baixo.
pub fn dispor(q: &[&Plano], cabem: usize) -> Vec<Lugar> {
    let cabem = cabem.max(1);
    let n = niveis(q);
    let topo = n.iter().copied().max().unwrap_or(0);
    if topo == 0 {
        return (0..q.len()).map(|i| Lugar { col: i % cabem, lin: (i / cabem) as f32 }).collect();
    }
    let no_nivel = |v: usize| n.iter().filter(|x| **x == v).count();
    let faixa = (0..=topo).map(no_nivel).max().unwrap_or(1);
    let mut vistos = vec![0; topo + 1];
    n.iter().map(|&v| {
        let i = vistos[v];
        vistos[v] += 1;
        Lugar { col: v % cabem, lin: (v / cabem * faixa) as f32 + i as f32 + (faixa - no_nivel(v)) as f32 / 2.0 }
    }).collect()
}

#[cfg(test)]
#[path = "acao_mapa_testes.rs"]
mod tests;
