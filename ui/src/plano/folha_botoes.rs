//! Os botões do topo da folha: os mesmos `.btn` da folha original (`planta::Botao`, 34 px, a 8 px um do outro, alinhados a 64 px da borda).
//! A primeira fileira é a de sempre; a segunda, no lugar da busca, traz o que pede atenção.

use super::super::modelo::Folha;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::planta::{self, botao, Botao, Texto};
use crate::tema::{rgba, AMBAR, MEDIO, PAPEL};
use egui::{vec2, Rect, Sense, Ui};

use super::super::linha::Pincel;

const Y_BOTOES: f32 = 44.0;
const ALTO: f32 = 34.0;
const VAO: f32 = 8.0;
/// A fileira da segunda linha desce isto (a mesma distância da busca original).
const PASSO_FILEIRA: f32 = 42.0;
/// Os botões passam da margem da régua até aqui: `w - 64` contra `w - 86`.
pub const ALEM_DA_REGUA: f32 = 22.0;
/// Onde o título acaba.
pub const TITULO_MAX: f32 = 520.0;

/// Os botões do topo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Topo {
    Aguardando,
    Chegou,
    Agentes,
    FolhaAntiga,
    Voltar,
    Web,
}

pub struct Item {
    pub topo: Topo,
    pub rotulo: String,
}

fn item(topo: Topo, rotulo: impl Into<String>) -> Item {
    Item { topo, rotulo: rotulo.into() }
}

/// O rótulo do «✦ Planejar»: os espaços abrem o lugar da estrela, como na folha original.
fn rotulo_planejar() -> String {
    format!("  {}", tr("Planejar"))
}

/// A primeira fileira: a da folha original.
pub fn fileira_principal() -> Vec<Item> {
    vec![item(Topo::Voltar, tr("← Blueprints")), item(Topo::Web, rotulo_planejar()), item(Topo::Web, tr("Compartilhar")), item(Topo::Web, tr("Apresentar"))]
}

/// A segunda fileira. `Aguardando` e `O que chegou` só existem quando há o que mostrar.
pub fn fileira_atencao(f: &Folha, trabalhando: Option<u32>) -> Vec<Item> {
    let mut b = Vec::with_capacity(3);
    if f.numeros.aprovar > 0 {
        b.push(item(Topo::Aguardando, tr_com("Aguardando você · {n}", &[("n", &f.numeros.aprovar.to_string())])));
    }
    if f.pendente.is_some() {
        b.push(item(Topo::Chegou, tr("O que chegou")));
    }
    let agora = trabalhando.filter(|n| *n > 0);
    b.push(item(Topo::Agentes, agora.map_or_else(|| tr("Agentes").to_string(), |n| tr_com("Agentes · {n}", &[("n", &n.to_string())]))));
    b
}

fn largura(ui: &Ui, i: &Item) -> f32 {
    Botao::barra(&i.rotulo).largura(ui.painter())
}

/// A largura de uma fileira inteira.
pub fn largura_fileira(ui: &Ui, lista: &[Item]) -> f32 {
    lista.iter().map(|i| largura(ui, i) + VAO).sum::<f32>() - VAO
}

/// O «Aguardando você»: o mesmo botão, em âmbar, porque pede a pessoa.
fn botao_ambar(ui: &mut Ui, pos: egui::Pos2, rotulo: &str) -> bool {
    let p = ui.painter().clone();
    let r = Rect::from_min_size(pos, vec2(Botao::barra(rotulo).largura(&p), ALTO));
    let resposta = ui.interact(r, ui.scope_id().with(("btn-ambar", rotulo)), Sense::click());
    let (fundo, cor) = if resposta.hovered() { (AMBAR, PAPEL) } else { (rgba(0x0c3a78, 0xcc), AMBAR) };
    p.rect_filled(r, 0, fundo);
    planta::borda(&p, r, if resposta.hovered() { AMBAR } else { MEDIO });
    let t = Texto::novo(rotulo.to_uppercase(), 11.0).trilha(0.1).peso(Peso::Negrito).cor(cor);
    let w = t.largura(&p);
    t.na_linha(&p, r.min.x + (r.width() - w) / 2.0, r.min.y, r.height());
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta.clicked()
}

/// Desenha uma fileira alinhada à direita em `direita`. Devolve o clicado.
fn fileira(ui: &mut Ui, c: &Pincel, direita: f32, y: f32, lista: &[Item]) -> Option<Topo> {
    let (mut x, mut clicado) = (direita, None);
    for i in lista.iter().rev() {
        x -= largura(ui, i);
        let pos = c.pt(x, y);
        let clicou = if i.topo == Topo::Aguardando {
            botao_ambar(ui, pos, &i.rotulo)
        } else {
            let r = botao(ui, pos, &Botao::barra(&i.rotulo));
            if i.rotulo == rotulo_planejar() {
                let p = ui.painter().clone();
                crate::folha::controles::estrela(&p, pos + vec2(19.85, 17.0), r.hovered() || r.has_focus());
            }
            r.clicked()
        };
        if clicou {
            clicado = Some(i.topo);
        }
        x -= VAO;
    }
    clicado
}

/// As duas fileiras da folha. Devolve o clicado.
pub fn fileiras(ui: &mut Ui, c: &Pincel, x1: f32, principal: &[Item], atencao: &[Item]) -> Option<Topo> {
    let direita = x1 + ALEM_DA_REGUA;
    let a = fileira(ui, c, direita, Y_BOTOES, principal);
    let b = fileira(ui, c, direita, Y_BOTOES + PASSO_FILEIRA, atencao);
    a.or(b)
}

/// Os botões das outras telas: os que não cabem à direita de `minimo` descem para a segunda fileira. Devolve o clicado.
pub fn botoes(ui: &mut Ui, c: &Pincel, x1: f32, lista: &[Item], minimo: f32) -> Option<Topo> {
    let direita = x1 + ALEM_DA_REGUA;
    let (mut x, mut y, mut clicado) = (direita, Y_BOTOES, None);
    for i in lista.iter().rev() {
        let w = largura(ui, i);
        if x - w < minimo && y == Y_BOTOES {
            (x, y) = (direita, Y_BOTOES + PASSO_FILEIRA);
        }
        x -= w;
        if botao(ui, c.pt(x, y), &Botao::barra(&i.rotulo)).clicked() {
            clicado = Some(i.topo);
        }
        x -= VAO;
    }
    clicado
}
