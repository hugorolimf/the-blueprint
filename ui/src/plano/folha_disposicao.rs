//! Onde cada peça da folha fica, medido antes de desenhar: os projetos, as ações, as decisões soltas; o andamento do blueprint e as decisões que não são de ação nenhuma.

use super::super::caixinha::Lapis;
use super::super::escala::{LINHA_FOLHA as LINHA, X0};
use super::super::linha::{self, Letra, Pincel, ALTO_PROJETO};
use super::super::modelo::{Acao as AcaoPlano, Decisao, Folha};
use super::super::revisao::{self, Meio, Peca};
use crate::dominio::Situacao;
use crate::i18n::tr_com;
use crate::tema::{MEDIO, TINTA};
use egui::{Id, Sense, Ui};

pub const ALTO_DECISAO: f32 = 38.0;
pub const X_DECISAO: f32 = X0 + 30.0;
/// O espaço depois das ações de um projeto, até o próximo.
const VAO_PROJETO: f32 = 14.0;

/// Onde cada peça fica: `projetos` traz o meio do cabeçalho de cada um e `linhas` (projeto, ação, meio da linha).
pub struct Disposicao {
    pub projetos: Vec<f32>,
    pub linhas: Vec<(usize, usize, f32)>,
    pub y_decisoes: f32,
    pub fim: f32,
}

/// As decisões que não pertencem a ação nenhuma: sem isto, não apareceriam em lugar algum.
pub fn soltas(f: &Folha) -> impl Iterator<Item = &Decisao> {
    f.decisoes.iter().filter(|d| d.acoes.is_empty())
}

/// Empilha os projetos e as ações a partir do topo `y0`. `painel`: a altura que a ação ocupa além da linha (a ação aberta soma o painel dela aqui, e a que abre ou fecha, parte dele). `notas`: o que as observações das decisões soltas ocupam.
pub fn empilhar(f: &Folha, y0: f32, painel: &dyn Fn(&AcaoPlano) -> f32, notas: f32) -> Disposicao {
    let alturas: Vec<f32> = f.projetos.iter().flat_map(|p| &p.acoes).map(painel).collect();
    empilhar_por(f, y0, &alturas, notas)
}

/// `empilhar` com o que cada ação ocupa já medido, na ordem das linhas (todas as ações de todos os projetos): a folha mede uma vez por quadro e passa aqui.
pub fn empilhar_por(f: &Folha, y0: f32, painel: &[f32], notas: f32) -> Disposicao {
    let (mut projetos, mut linhas, mut y) = (Vec::with_capacity(f.projetos.len()), Vec::new(), y0);
    for (i, p) in f.projetos.iter().enumerate() {
        projetos.push(y + ALTO_PROJETO / 2.0);
        y += ALTO_PROJETO;
        for k in 0..p.acoes.len() {
            // o índice da linha é o da ação na folha inteira
            let extra = painel.get(linhas.len()).copied().unwrap_or(0.0);
            linhas.push((i, k, y + LINHA / 2.0));
            y += LINHA + extra;
        }
        y += VAO_PROJETO;
    }
    let y_decisoes = y;
    let n = soltas(f).count();
    if n > 0 {
        y += 24.0 + n as f32 * ALTO_DECISAO + notas;
    }
    Disposicao { projetos, linhas, y_decisoes, fim: y }
}

/// Quanto do blueprint está feito: (progresso de 0 a 1, ações feitas, ações). Planos e checkpoints pesam igual.
pub fn progresso(f: &Folha) -> (f32, usize, usize) {
    let (mut feitos, mut itens, mut prontas, mut total) = (0usize, 0usize, 0usize, 0usize);
    for a in f.projetos.iter().flat_map(|p| &p.acoes) {
        let k = a.checkpoints.iter().filter(|c| c.feito).count();
        feitos += a.numeros.prontos as usize + k;
        itens += a.numeros.planos as usize + a.checkpoints.len();
        total += 1;
        prontas += usize::from(linha::situacao(a) == Situacao::Concluido);
    }
    (if itens == 0 { 0.0 } else { feitos as f32 / itens as f32 }, prontas, total)
}

/// A situação do blueprint inteiro, para o selo: tudo pronto, algo andando, ou nada ainda.
pub fn situacao_geral(f: &Folha) -> Situacao {
    let (progresso, prontas, total) = progresso(f);
    let andando = f.projetos.iter().flat_map(|p| &p.acoes).any(|a| linha::situacao(a) != Situacao::Planejado);
    if total > 0 && prontas == total {
        Situacao::Concluido
    } else if progresso > 0.0 || andando {
        Situacao::Andamento
    } else {
        Situacao::Planejado
    }
}

/// As decisões soltas, depois dos projetos: o rótulo, o controle de revisão quando pede aprovação, a primeira linha do texto (o resto na dica) e a observação.
pub fn decisoes(ui: &mut Ui, c: &Pincel, f: &Folha, lugar: revisao::Alvo, y0: f32, x1: f32) {
    let n = soltas(f).count();
    if n == 0 {
        return;
    }
    c.rotulo(&tr_com("Decisões · {n}", &[("n", &n.to_string())]), X0, y0 + 4.0, Letra::nova(9.5, MEDIO).trilha(0.12));
    c.fio(X0, y0 + 11.0, x1, y0 + 11.0, TINTA.gamma_multiply(0.45), 1.0);
    let (letra, largura, mut y) = (Letra::nova(11.0, MEDIO), x1 - X_DECISAO, y0 + 24.0);
    for k in soltas(f) {
        let lapis = || Lapis { p: c.p, o: c.o };
        let peca = Peca::decisao(f, k);
        let forte = Letra::nova(11.5, TINTA).forte();
        c.texto(&c.cortado(&k.rotulo, largura - 270.0, &forte).0, X_DECISAO, y + 6.0, forte);
        if k.pede_aprovacao {
            revisao::canto(lapis(), ui, lugar, x1, y + 6.0, &peca, Meio::Decisao);
        }
        let (t, cortou) = c.cortado(&k.texto, largura, &letra);
        c.texto(&t, X_DECISAO, y + 22.0, letra);
        if cortou {
            let _ = ui.interact(c.caixa(X_DECISAO, y + 10.0, largura, 16.0), Id::unique(("plano-decisao", k.id.as_str())), Sense::hover()).on_hover_text(k.texto.as_str());
        }
        y += ALTO_DECISAO + revisao::nota(lapis(), ui, lugar, X_DECISAO, y + ALTO_DECISAO, largura, &peca);
    }
}
