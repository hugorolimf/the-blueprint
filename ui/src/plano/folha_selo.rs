//! O selo no canto da folha, em células, como numa prancha: workspace, blueprint, a situação, o progresso num anel que enche e a folha. Sem espaço, as células
//! menos importantes saem. O nome do workspace vem do documento (`Cabeca`): a raiz sem nome ganha «Meu workspace».

use super::estilo::{com_alfa, entrada, ANDANDO, FEITO};
use super::super::linha::{Letra, Pincel};
use crate::dominio::Situacao;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::tema::{CARTAO, MEDIO, TINTA};
use egui::Id;

/// O que o selo mostra.
pub struct Dados<'a> {
    pub workspace: &'a str,
    pub blueprint: &'a str,
    pub situacao: Situacao,
    /// de 0 a 1
    pub progresso: f32,
    pub feitas: usize,
    pub acoes: usize,
    pub folha: usize,
    pub folhas: usize,
}

const ALTO: f32 = 68.0;

/// Uma célula e a largura que pede; `urgencia` maior sai por último.
#[derive(Clone, Copy, PartialEq)]
enum Celula {
    Workspace,
    Blueprint,
    Situacao,
    Progresso,
    Folha,
}

fn pedido(c: Celula) -> (f32, u8) {
    match c {
        Celula::Workspace => (190.0, 2),
        Celula::Blueprint => (250.0, 5),
        Celula::Situacao => (142.0, 3),
        Celula::Progresso => (168.0, 4),
        Celula::Folha => (92.0, 1),
    }
}

fn texto_da_situacao(s: Situacao) -> (&'static str, egui::Color32) {
    match s {
        Situacao::Concluido => (tr("Concluído"), FEITO),
        Situacao::Andamento => (tr("Em andamento"), ANDANDO),
        Situacao::Planejado => (tr("Planejado"), MEDIO),
    }
}

/// O selo com o canto de baixo à direita em `(x1, h - 30)`, sem passar de `x_min` para a esquerda. Devolve onde ele começa.
pub fn selo(c: &Pincel, x1: f32, h: f32, x_min: f32, d: &Dados) -> f32 {
    let mut celulas = vec![Celula::Workspace, Celula::Blueprint, Celula::Situacao, Celula::Progresso, Celula::Folha];
    let total = |l: &[Celula]| l.iter().map(|k| pedido(*k).0).sum::<f32>();
    while total(&celulas) > x1 - x_min && celulas.len() > 1 {
        if let Some(i) = (0..celulas.len()).min_by_key(|i| pedido(celulas[*i]).1) {
            celulas.remove(i);
        }
    }
    let largura = total(&celulas).min(x1 - x_min).max(100.0);
    let (inicio, y) = (x1 - largura, h - 30.0 - ALTO);
    let mut x = inicio;
    c.sombra(x, y, largura, ALTO, 0.0, com_alfa(egui::Color32::BLACK, 0.35), 14, 4);
    c.ret(x, y, largura, ALTO, Some(com_alfa(CARTAO, 0.96)), Some((com_alfa(TINTA, 0.95), 1.0)));
    let chave = d.blueprint.to_string();
    for (i, k) in celulas.iter().enumerate() {
        let w = if i + 1 == celulas.len() { x1 - x } else { pedido(*k).0.min(x1 - x) };
        if i > 0 {
            c.fio(x, y + 8.0, x, y + ALTO - 8.0, com_alfa(MEDIO, 0.5), 1.0);
        }
        celula(c, *k, x, y, w, d, &chave);
        x += w;
    }
    inicio
}

fn rotulo(c: &Pincel, s: &str, x: f32, y: f32) {
    c.rotulo(s, x + 12.0, y + 20.0, Letra::nova(9.0, MEDIO).trilha(0.14));
}

fn valor(c: &Pincel, s: &str, x: f32, y: f32, w: f32) {
    let letra = Letra { peso: Peso::Negrito, ..Letra::nova(14.0, TINTA) };
    c.texto(&c.cortado(s, w - 24.0, &letra).0, x + 12.0, y + 48.0, letra);
}

fn celula(c: &Pincel, k: Celula, x: f32, y: f32, w: f32, d: &Dados, chave: &str) {
    match k {
        Celula::Workspace => {
            rotulo(c, tr("Workspace"), x, y);
            valor(c, d.workspace, x, y, w);
        }
        Celula::Blueprint => {
            rotulo(c, tr("Blueprint"), x, y);
            valor(c, d.blueprint, x, y, w);
        }
        Celula::Situacao => {
            rotulo(c, tr("Situação"), x, y);
            let (texto, tom) = texto_da_situacao(d.situacao);
            c.etiqueta(texto, x + 12.0, y + 43.0, Letra::nova(10.0, tom).forte(), com_alfa(tom, 0.14), Some(com_alfa(tom, 0.45)), 9.0);
        }
        Celula::Progresso => {
            rotulo(c, tr("Progresso"), x, y);
            let ctx = c.p.ctx();
            let parte = ctx.animate_value_with_time(Id::unique(("folha-selo-anel", chave)), d.progresso, 0.6) * entrada(ctx, chave, 250.0, 900.0);
            let tom = texto_da_situacao(d.situacao).1;
            c.anel(x + 28.0, y + 44.0, 13.0, 3.6, parte, tom, Some(com_alfa(tom, 0.2)));
            let pct = format!("{}%", (parte * 100.0).round() as i32);
            c.texto(&pct, x + 52.0, y + 43.0, Letra { peso: Peso::Negrito, ..Letra::nova(16.0, TINTA) });
            let sub = tr_com("{n} de {total}", &[("n", &d.feitas.to_string()), ("total", &d.acoes.to_string())]);
            c.texto(&format!("{sub} {}", tr("Ações").to_lowercase()), x + 52.0, y + 57.0, Letra::nova(9.0, MEDIO));
        }
        Celula::Folha => {
            rotulo(c, tr("Folha"), x, y);
            valor(c, &tr_com("{n} de {total}", &[("n", &d.folha.to_string()), ("total", &d.folhas.to_string())]), x, y, w);
        }
    }
}
