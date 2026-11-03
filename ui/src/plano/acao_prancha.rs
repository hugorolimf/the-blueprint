//! As tabelas da ação aberta (`tabelas` de `tela-acao.mjs`). As telas e os detalhes moram em `detalhe.rs`.

use super::super::acao_estilo::{alfa, CORPO, LETRA_MENOR};
use super::super::detalhe;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Tabela};
use super::rotulo;
use crate::tema::{BAIXO, MEDIO};
use egui::{Sense, Ui};

const ALTO_LINHA: f32 = 28.0;
const ALTO_ROTULO: f32 = 24.0;

/// As tabelas da ação: as aprovadas e, com `true`, as que a versão pendente traz ou refaz.
pub fn tabelas_da(a: &Acao) -> Vec<(&Tabela, bool)> {
    let novas: Vec<&Tabela> = detalhe::diff_vivo(a).into_iter().flat_map(|d| &d.tabelas_novas).collect();
    // a tabela refeita aparece uma vez só, na versão nova
    let antigas = a.tabelas.iter().filter(|t| !novas.iter().any(|n| n.titulo == t.titulo));
    antigas.map(|t| (t, false)).chain(novas.iter().map(|t| (*t, true))).collect()
}

/// Quanto as tabelas ocupam.
pub fn altura_tabelas(lista: &[(&Tabela, bool)]) -> f32 {
    lista.iter().map(|(t, _)| ALTO_ROTULO - 4.0 + ALTO_LINHA * (t.linhas.len() as f32 + 1.0) + 22.0).sum()
}

/// As frações de cada coluna: as do plano quando servem; senão, iguais. Nunca passam da largura.
pub fn fracoes(t: &Tabela) -> Vec<f32> {
    let n = t.colunas.len();
    let soma: f64 = t.larguras.iter().sum();
    if t.larguras.len() != n || soma <= 0.0 || t.larguras.iter().any(|l| *l < 0.0) {
        return vec![1.0 / n.max(1) as f32; n];
    }
    let escala = if soma > 1.0 { soma } else { 1.0 };
    t.larguras.iter().map(|l| (*l / escala) as f32).collect()
}

/// As tabelas a partir de `(x, y)`, só com fios: o título em caixa alta, o cabeçalho e um fio fino entre as linhas.
pub fn tabelas(ui: &Ui, c: &Pincel, lista: &[(&Tabela, bool)], x: f32, y: f32, w: f32) {
    let mut yy = y;
    for (k, (t, nova)) in lista.iter().copied().enumerate() {
        let nome: &str = if t.titulo.is_empty() { crate::i18n::tr("Tabela") } else { &t.titulo };
        rotulo(c, x, yy, &format!("{}{nome}", if nova { "+ " } else { "" }), w);
        yy += ALTO_ROTULO - 4.0;
        let partes = fracoes(t);
        let mut cols = Vec::with_capacity(partes.len());
        partes.iter().fold(0.0f32, |acum, f| {
            cols.push(acum);
            acum + f * w
        });
        let cabeca = Letra::nova(10.5, MEDIO).trilha(0.1);
        for (i, nome) in t.colunas.iter().enumerate() {
            c.rotulo(&c.cortado(nome, partes[i] * w - 20.0, &cabeca).0, x + cols[i], yy + 14.0, cabeca);
        }
        yy += ALTO_LINHA;
        for (n, linha) in t.linhas.iter().enumerate() {
            if n > 0 {
                c.fio(x, yy - 4.0, x + w, yy - 4.0, alfa(BAIXO, 0.55), 1.0);
            }
            for (i, v) in linha.iter().enumerate().take(partes.len()) {
                let letra = Letra::nova(LETRA_MENOR + 1.0, CORPO);
                let max = partes[i] * w - 20.0;
                let (corte, cortou) = c.cortado(v, max, &letra);
                c.texto(&corte, x + cols[i], yy + 15.0, letra);
                if cortou {
                    let _ = ui.interact(c.caixa(x + cols[i], yy, max, 18.0), ui.scope_id().with(("plano-tabela", &t.titulo, k, n, i)), Sense::hover()).on_hover_text(v.as_str());
                }
            }
            yy += ALTO_LINHA;
        }
        yy += 22.0 - 4.0 + 4.0;
    }
}
