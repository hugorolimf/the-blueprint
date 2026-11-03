//! O controle de revisão: Aprovar, Revisar (ou Editar) e Reprovar com o mesmo `.btn` do resto do sistema (`planta::Botao`): quadrado, em caixa alta,
//! invertendo sob o mouse. «Aprovar» é o primário; enquanto a observação está aberta, o primário passa a ser a parte do meio.

use super::caixinha::Lapis;
use super::revisao::{Meio, Parte};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use egui::{Painter, Ui};

pub const ALTO: f32 = 24.0;
const VAO: f32 = 6.0;
const PARTES: [Parte; 3] = [Parte::Aprovar, Parte::Meio, Parte::Reprovar];

fn texto_da(parte: Parte, meio: Meio) -> &'static str {
    match parte {
        Parte::Aprovar if meio == Meio::Decisao => tr("Confirmar"),
        Parte::Aprovar => tr("Aprovar"),
        Parte::Meio => match meio {
            Meio::Revisar => tr("Revisar"),
            Meio::Decisao => tr("Ajustar"),
            Meio::Tela => tr("Pedir ajuste"),
            Meio::Editar(_) => tr("Editar"),
        },
        Parte::Reprovar if meio == Meio::Decisao => tr("Contestar"),
        Parte::Reprovar => tr("Reprovar"),
    }
}

fn botao_da<'a>(parte: Parte, meio: Meio, meio_ativo: bool) -> Botao<'a> {
    let cheio = if meio_ativo { parte == Parte::Meio } else { parte == Parte::Aprovar };
    Botao { texto: texto_da(parte, meio), tam: 10.0, cheio, ativo: true, altura: ALTO, pad_x: 10.0 }
}

/// As larguras das três partes e o total.
pub fn medir_partes(p: &Painter, meio: Meio) -> ([f32; 3], f32) {
    let larguras = PARTES.map(|parte| botao_da(parte, meio, false).largura(p));
    (larguras, larguras.iter().sum::<f32>() + VAO * 2.0)
}

pub const ALTO_FILEIRA: f32 = 30.0;

/// Os três verbos em botões maiores, começando em `x`, com o topo em `y`. Devolve a parte clicada.
pub fn fileira(l: Lapis, ui: &mut Ui, x: f32, y: f32, meio: Meio, meio_ativo: bool) -> Option<Parte> {
    let (mut xx, mut clicada) = (x, None);
    for parte in PARTES {
        let b = Botao { tam: 10.5, altura: ALTO_FILEIRA, pad_x: 13.0, ..botao_da(parte, meio, meio_ativo) };
        if botao(ui, l.pt(xx, y), &b).clicked() {
            clicada = Some(parte);
        }
        xx += b.largura(l.p) + VAO;
    }
    clicada
}

/// O controle preso à borda direita em `x_fim`, com o topo em `y`. Devolve a parte clicada.
pub fn controle(l: Lapis, ui: &mut Ui, x_fim: f32, y: f32, meio: Meio, meio_ativo: bool) -> Option<Parte> {
    let (larguras, total) = medir_partes(l.p, meio);
    let (mut x, mut clicada) = (x_fim - total, None);
    for (i, parte) in PARTES.into_iter().enumerate() {
        if botao(ui, l.pt(x, y), &botao_da(parte, meio, meio_ativo)).clicked() {
            clicada = Some(parte);
        }
        x += larguras[i] + VAO;
    }
    clicada
}
