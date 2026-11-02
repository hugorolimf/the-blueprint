//! As tabelas e a prancha de detalhes da ação aberta (`tabelas`, `ladrilho` e `prancha` de `tela-acao.mjs`). A altura sai das medidas que o servidor mandou,
//! sem rasterizar antes de empilhar. A grade é responsiva: quantas colunas couberem (de 2 a 6), com a figura larga ocupando duas.

use super::super::acao_estilo::{alfa, CORPO};
use super::super::acao_movimento::{cascata, desde_que_abriu};
use super::super::cartao_ladrilho::{self, Ladrilho};
use super::super::detalhe;
use super::super::figura;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Detalhe, Folha, Tabela};
use super::super::revisao::{self, Alvo};
use super::rotulo;
use crate::i18n::tr_com;
use crate::tema::{BAIXO, MEDIO, TINTA};
use egui::{Painter, Sense, Ui};

const LARGURA_MIN: f32 = 236.0;
const VAO: f32 = 14.0;
const ALTO_LINHA: f32 = 26.0;
const ALTO_ROTULO: f32 = 24.0;
const CASCATA_MAX: usize = 14;

/// Onde um ladrilho fica na prancha, a partir do topo dela. `alto` é o da figura; o rodapé soma `RODAPE`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lugar {
    pub indice: usize,
    pub x: f32,
    pub y: f32,
    pub largura: f32,
    pub alto: f32,
    pub larga: bool,
}

/// Quantas colunas cabem em `w`: de 2 a 6.
pub fn colunas(w: f32) -> usize {
    (((w + VAO) / (LARGURA_MIN + VAO)).floor() as usize).clamp(2, 6)
}

/// A prancha: as colunas que couberem; figura muito larga ocupa duas e fica baixa. As normais vêm primeiro, para cada linha ter uma altura só. Devolve os lugares e a altura total.
pub fn dispor(itens: &[&Detalhe], w: f32) -> (Vec<Lugar>, f32) {
    let n = colunas(w);
    let u = (w - VAO * (n as f32 - 1.0)) / n as f32;
    let alto_normal = (u * 0.62).round().clamp(96.0, 170.0);
    let alto_larga = (alto_normal * 0.48).round();
    let (mut col, mut y, mut alt, mut antes) = (0usize, 0.0, 0.0f32, None::<bool>);
    let mut lugares = Vec::with_capacity(itens.len());
    let eh_larga = |d: &Detalhe| figura::larga(d.figura.as_ref());
    let ordem = (0..itens.len()).filter(|i| !eh_larga(itens[*i])).chain((0..itens.len()).filter(|i| eh_larga(itens[*i])));
    for indice in ordem {
        let larga = eh_larga(itens[indice]);
        let (span, alto) = if larga { (2usize, alto_larga) } else { (1usize, alto_normal) };
        if (antes.is_some_and(|a| a != larga) && col > 0) || col + span > n {
            y += alt + cartao_ladrilho::RODAPE + VAO;
            col = 0;
            alt = 0.0;
        }
        antes = Some(larga);
        lugares.push(Lugar { indice, x: col as f32 * (u + VAO), y, largura: span as f32 * u + (span as f32 - 1.0) * VAO, alto, larga });
        col += span;
        alt = alt.max(alto);
    }
    if col > 0 {
        y += alt + cartao_ladrilho::RODAPE + VAO;
    }
    (lugares, y)
}

/// Os detalhes em grupos, na ordem em que aparecem; sem grupo é o de nome vazio.
pub fn grupos(a: &Acao) -> Vec<(&str, Vec<&Detalhe>)> {
    let mut lista: Vec<(&str, Vec<&Detalhe>)> = Vec::new();
    for d in &a.detalhes {
        match lista.iter_mut().find(|(nome, _)| *nome == d.grupo) {
            Some((_, itens)) => itens.push(d),
            None => lista.push((&d.grupo, vec![d])),
        }
    }
    lista
}

/// Quanto a seção de detalhes ocupa: a prévia aberta ou a grade. Zero sem detalhes.
pub fn altura_detalhes(p: &Painter, f: &Folha, a: &Acao, w: f32) -> f32 {
    if a.detalhes.is_empty() {
        return 0.0;
    }
    let corpo = detalhe::altura(p, f, a, w).unwrap_or_else(|| grupos(a).iter().map(|(nome, itens)| (if nome.is_empty() { 0.0 } else { 22.0 }) + dispor(itens, w).1).sum::<f32>());
    ALTO_ROTULO + 4.0 + corpo
}

/// A seção de detalhes com o rótulo em `(x, y)`: a prévia, se houver uma aberta; senão, a grade, em que um clique abre a prévia.
#[allow(clippy::too_many_arguments)]
pub fn detalhes(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, x: f32, y: f32, w: f32) {
    if a.detalhes.is_empty() {
        return;
    }
    let n = a.detalhes.len().to_string();
    let titulo = tr_com("Detalhes · {n}", &[("n", &n)]);
    rotulo(c, x, y, &titulo, w);
    if detalhe::desenhar(ui, c, alvo, f, a, x, y + ALTO_ROTULO + 4.0, w, y) {
        return;
    }
    let servidor = alvo.servidor;
    let ctx = ui.ctx().clone();
    let ms = desde_que_abriu(&ctx, ("acao-painel", &a.referencia));
    let (mut yy, mut ordem) = (y + ALTO_ROTULO + 4.0, 0usize);
    for (nome, itens) in grupos(a) {
        if !nome.is_empty() {
            let w_nome = c.texto(nome, x, yy + 12.0, Letra::nova(11.0, TINTA).forte());
            c.texto(&format!("{}", itens.len()), x + w_nome + 8.0, yy + 12.0, Letra::nova(10.0, MEDIO));
            yy += 22.0;
        }
        let (lugares, alto) = dispor(&itens, w);
        for l in lugares {
            let d = itens[l.indice];
            let ladrilho = Ladrilho { rotulo: format!("{}-{}", a.referencia, d.id), titulo: &d.titulo, figura: d.figura.as_ref(), estado: d.pede_aprovacao.then(|| revisao::estado_de(d.revisao.as_ref())), atual: false };
            let entra = cascata(&ctx, ms, 2, 120.0 + 45.0 * ordem.min(CASCATA_MAX) as f32);
            ordem += 1;
            let chave = format!("{}/{}", a.referencia, d.id);
            if cartao_ladrilho::desenhar(ui, c, servidor, &chave, &ladrilho, (x + l.x, yy + l.y, l.largura, l.alto), entra).clicked() {
                detalhe::abrir(&ctx, &a.referencia, &d.id);
            }
        }
        yy += alto;
    }
}

/// Quanto as tabelas ocupam.
pub fn altura_tabelas(a: &Acao) -> f32 {
    a.tabelas.iter().map(|t| ALTO_ROTULO - 4.0 + ALTO_LINHA * (t.linhas.len() as f32 + 1.0) + 22.0).sum()
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
pub fn tabelas(ui: &Ui, c: &Pincel, a: &Acao, x: f32, y: f32, w: f32) {
    let mut yy = y;
    for (k, t) in a.tabelas.iter().enumerate() {
        rotulo(c, x, yy, if t.titulo.is_empty() { crate::i18n::tr("Tabela") } else { &t.titulo }, w);
        yy += ALTO_ROTULO - 4.0;
        let partes = fracoes(t);
        let mut cols = Vec::with_capacity(partes.len());
        partes.iter().fold(0.0f32, |acum, f| {
            cols.push(acum);
            acum + f * w
        });
        let cabeca = Letra::nova(9.5, MEDIO).trilha(0.1);
        for (i, nome) in t.colunas.iter().enumerate() {
            c.rotulo(&c.cortado(nome, partes[i] * w - 20.0, &cabeca).0, x + cols[i], yy + 14.0, cabeca);
        }
        yy += ALTO_LINHA;
        for (n, linha) in t.linhas.iter().enumerate() {
            if n > 0 {
                c.fio(x, yy - 4.0, x + w, yy - 4.0, alfa(BAIXO, 0.55), 1.0);
            }
            for (i, v) in linha.iter().enumerate().take(partes.len()) {
                let letra = Letra::nova(11.0, CORPO);
                let max = partes[i] * w - 20.0;
                let (corte, cortou) = c.cortado(v, max, &letra);
                c.texto(&corte, x + cols[i], yy + 15.0, letra);
                if cortou {
                    let _ = ui.interact(c.caixa(x + cols[i], yy, max, 18.0), ui.scope_id().with(("plano-tabela", &a.referencia, k, n, i)), Sense::hover()).on_hover_text(v.as_str());
                }
            }
            yy += ALTO_LINHA;
        }
        yy += 22.0 - 4.0 + 4.0;
    }
}
