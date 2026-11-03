//! A mesa da ação: flutua sobre o palco como a ficha de um plano, e traz o que não é de nenhuma construção: o texto, o pedido de ajuste, quem está trabalhando, o que é «Da ação», os checkpoints soltos e os recados.
//! Em janela larga ela abre em duas colunas: à esquerda, a ação (a linha de cima com o fechar, o número e o nome; o texto, os ajustes e os agentes); à direita, «Da ação», à vista sem rolar. Estreita, ou embaixo do palco, uma coluna só. Medir e desenhar saem do mesmo `arranjo`.
//! O ajuste vai ao orquestrador como pedido, e o que ele fizer volta como proposta para revisar.

use super::super::acao_estilo::CORPO;
use super::super::acao_movimento::{clarao, marcar_instante, mistura};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha};
use super::super::orquestrador::corpo_mensagem;
use super::super::revisao::Alvo;
use super::blocos::{Bloco, Montagem};
use super::{bloco, decidir, lista, movimentos, obra_mergulho as mergulho, prancha, rotulo, tempo, Saida};
use crate::fonte::{familia, Peso};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{BAIXO, CAMPO, MEDIO, TINTA, VERDE};
use egui::{vec2, Context, FontId, Id, Painter, Rect, Sense, Stroke, StrokeKind, TextEdit, Ui};

const ALTO_BOTAO: f32 = 30.0;
const VAO: f32 = 18.0;
const LETRA_SOBRE: f32 = 12.5;
const PASSO_SOBRE: f32 = 19.0;
const ALTO_ROTULO: f32 = 24.0;
const ALTO_CAMPO: f32 = 32.0;
const ALTO_LEGENDA: f32 = 22.0;
/// A linha de cima: o número e o nome da ação ao lado do fechar, para não gastar uma linha só com o botão.
const LETRA_NOME: f32 = 13.0;
const BASE_NOME: f32 = 20.0;
const VAO_NOME: f32 = 14.0;
/// Abaixo desta largura, duas colunas apertariam o texto.
const LARGURA_DUAS_COLUNAS: f32 = 600.0;
const VAO_COLUNAS: f32 = 28.0;
const FRACAO_ESQUERDA: f32 = 0.5;
/// A coluna da direita desce um pouco: o rótulo dela fica na altura do nome da ação.
const DESCE_DIREITA: f32 = 8.0;
/// O campo do ajuste não fica mais estreito que isto: antes, o botão desce para a linha de baixo.
const CAMPO_MIN: f32 = 160.0;
const VAO_BOTAO: f32 = 8.0;

fn id(a: &Acao) -> Id {
    Id::unique(("plano-obra-mesa-acao", &a.referencia))
}

pub fn aberta(ctx: &Context, a: &Acao) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id(a))).unwrap_or(false)
}

/// De 0 a 1, para ela chegar deslizando.
pub fn quanto(ctx: &Context, a: &Acao) -> f32 {
    mistura(ctx, ("plano-obra-mesa-acao-abre", &a.referencia), aberta(ctx, a), 0.5)
}

/// A mesa e a ficha de um plano ocupam o mesmo lugar: abrir uma fecha a outra.
pub fn abrir(ctx: &Context, a: &Acao) {
    mergulho::sair(ctx, a);
    ctx.data_mut(|d| d.insert_temp(id(a), true));
    crate::anima::mexer(ctx, 400.0);
}

pub fn fechar(ctx: &Context, a: &Acao) {
    ctx.data_mut(|d| d.remove_temp::<bool>(id(a)));
    crate::anima::mexer(ctx, 400.0);
}

pub fn alternar(ctx: &Context, a: &Acao) {
    if aberta(ctx, a) {
        fechar(ctx, a);
    } else {
        abrir(ctx, a);
    }
}

/// O que a mesa traz, cada parte com o vão de baixo incluído.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parte {
    Cabeca,
    Sobre,
    Ajuste,
    Agentes,
    DaAcao,
    Soltos,
}

/// Uma coluna da mesa: onde começa, a largura e as partes, de cima para baixo.
#[derive(Debug, PartialEq)]
struct Coluna {
    x: f32,
    y: f32,
    w: f32,
    partes: Vec<Parte>,
}

/// As larguras das duas colunas, ou `None` para uma só: embaixo do palco, ou estreita demais para dividir.
fn colunas(w: f32, larga: bool) -> Option<(f32, f32)> {
    if !larga || w < LARGURA_DUAS_COLUNAS {
        return None;
    }
    let esq = ((w - VAO_COLUNAS) * FRACAO_ESQUERDA).floor();
    Some((esq, w - VAO_COLUNAS - esq))
}

/// Onde fica cada parte; toda parte entra uma vez, e a vazia não ocupa nada. Em duas colunas, a da esquerda fala da ação e a da direita traz «Da ação»; sem nada da ação inteira, a da direita fica com os agentes, para nenhuma das duas ficar vazia.
fn arranjo(tem_da_acao: bool, w: f32, larga: bool) -> Vec<Coluna> {
    use Parte::*;
    let Some((we, wd)) = colunas(w, larga) else {
        return vec![Coluna { x: 0.0, y: 0.0, w, partes: vec![Cabeca, Sobre, Ajuste, Agentes, DaAcao, Soltos] }];
    };
    let (esq, dir) = if tem_da_acao { (vec![Cabeca, Sobre, Ajuste, Agentes, Soltos], vec![DaAcao]) } else { (vec![Cabeca, Sobre, Ajuste], vec![DaAcao, Agentes, Soltos]) };
    vec![Coluna { x: 0.0, y: 0.0, w: we, partes: esq }, Coluna { x: we + VAO_COLUNAS, y: DESCE_DIREITA, w: wd, partes: dir }]
}

fn da_acao_de<'m, 'a>(m: &'m Montagem<'a>) -> Option<&'m Bloco<'a>> {
    m.blocos.iter().find(|b| b.plano.is_none())
}

/// Há algo da ação inteira para mostrar: peças, checkpoints ou tabelas.
fn tem_da_acao(m: &Montagem) -> bool {
    da_acao_de(m).is_some_and(|b| !decidir::todas(b).is_empty() || !b.checkpoints.is_empty() || !b.tabelas.is_empty())
}

fn alto_sobre(p: &Painter, a: &Acao, w: f32) -> f32 {
    if a.texto.is_empty() { 0.0 } else { bloco(&a.texto, LETRA_SOBRE, w, PASSO_SOBRE).galeria(p).rows.len() as f32 * PASSO_SOBRE + VAO }
}

fn enviar(ativo: bool) -> Botao<'static> {
    Botao { texto: tr("Pedir ao orquestrador"), tam: 10.5, cheio: true, ativo, altura: ALTO_CAMPO, pad_x: 14.0 }
}

/// O botão de enviar cabe ao lado do campo? Senão desce para a linha de baixo.
fn lado_a_lado(p: &Painter, w: f32) -> bool {
    w - enviar(true).largura(p) - VAO_BOTAO >= CAMPO_MIN
}

fn alto_ajuste(p: &Painter, w: f32) -> f32 {
    let embaixo = if lado_a_lado(p, w) { 0.0 } else { ALTO_CAMPO + VAO_BOTAO };
    ALTO_ROTULO + ALTO_CAMPO + embaixo + ALTO_LEGENDA + VAO
}

/// O que é da ação inteira, e não de uma construção: as peças na mesma lista da ficha, os checkpoints e as tabelas.
fn alto_da_acao(p: &Painter, f: &Folha, a: &Acao, b: &Bloco, w: f32) -> f32 {
    [lista::altura(p, f, a, b, w), tempo::altura(p, &b.checkpoints, w), prancha::altura_tabelas(&b.tabelas)].iter().filter(|h| **h > 0.0).map(|h| h + VAO).sum()
}

fn alto_parte(p: &Painter, f: &Folha, a: &Acao, m: &Montagem, parte: Parte, w: f32) -> f32 {
    match parte {
        Parte::Cabeca => ALTO_BOTAO + VAO,
        Parte::Sobre => alto_sobre(p, a, w),
        Parte::Ajuste => alto_ajuste(p, w),
        Parte::Agentes => movimentos::altura(p, a, w) + VAO,
        Parte::DaAcao => da_acao_de(m).map_or(0.0, |b| alto_da_acao(p, f, a, b, w)),
        Parte::Soltos => {
            let h = tempo::altura(p, &m.soltos, w);
            if h > 0.0 { h + VAO } else { 0.0 }
        }
    }
}

/// A altura da mesa nesta largura: a da coluna mais alta. `larga` é a mesa que flutua sobre o palco, a única que pode abrir em duas colunas.
pub fn altura(p: &Painter, f: &Folha, a: &Acao, m: &Montagem, w: f32, larga: bool) -> f32 {
    let alto_coluna = |k: &Coluna| k.y + k.partes.iter().map(|parte| alto_parte(p, f, a, m, *parte, k.w)).sum::<f32>();
    arranjo(tem_da_acao(m), w, larga).iter().map(alto_coluna).fold(0.0, f32::max)
}

/// A linha de cima: o fechar e, ao lado, o número e o nome da ação. Nome longo é cortado, e o mouse mostra inteiro.
fn cabeca(ui: &mut Ui, c: &Pincel, a: &Acao, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    let fechar_b = Botao { texto: tr("← Fechar a mesa"), tam: 10.0, cheio: false, ativo: true, altura: ALTO_BOTAO, pad_x: 12.0 };
    if botao(ui, c.pt(x, y), &fechar_b).clicked() {
        fechar(&ctx, a);
    }
    let xn = x + fechar_b.largura(c.p) + VAO_NOME;
    let wn = c.texto(&a.referencia, xn, y + BASE_NOME, Letra::nova(LETRA_NOME, TINTA).forte());
    let l = Letra::nova(LETRA_NOME, TINTA);
    let xt = xn + wn + 8.0;
    let (nome, cortou) = c.cortado(&a.nome, (x + w - xt).max(0.0), &l);
    c.texto(&nome, xt, y + BASE_NOME, l);
    if cortou {
        let area = c.caixa(xn, y, (x + w - xn).max(1.0), ALTO_BOTAO);
        let _ = ui.interact(area, ui.scope_id().with(("plano-mesa-nome", &a.referencia)), Sense::hover()).on_hover_text(a.nome.as_str());
    }
}

fn ajuste(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &Acao, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    rotulo(c, x, y + 10.0, tr("Ajustes"), w);
    let y = y + ALTO_ROTULO;
    let id = Id::unique(("plano-mesa-ajuste", &a.referencia));
    let mut texto = ctx.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let pedido = tr_com("Ajuste na ação {id} ({nome}): {texto}", &[("id", &a.referencia), ("nome", &a.nome), ("texto", texto.trim())]);
    let pronto = if texto.trim().is_empty() { None } else { corpo_mensagem(alvo.ws, alvo.bp, &pedido) };
    let envia = enviar(pronto.is_some());
    // coluna estreita: o campo fica com a linha inteira e o botão desce
    let lado = lado_a_lado(c.p, w);
    let w_campo = if lado { w - envia.largura(c.p) - VAO_BOTAO } else { w };
    let r = c.caixa(x, y, w_campo, ALTO_CAMPO);
    c.p.rect_filled(r, 0, CAMPO);
    c.p.rect_stroke(r, 0, Stroke::new(1.0, BAIXO), StrokeKind::Inside);
    let dica = egui::RichText::new(tr("O que ajustar?")).size(11.0).color(MEDIO);
    let editor = TextEdit::singleline(&mut texto).id(id.with("campo")).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(11.0, familia(Peso::Normal))).text_color(TINTA).hint_text(dica);
    let resposta = ui.put(Rect::from_min_size(c.pt(x + 12.0, y + 8.0), vec2(w_campo - 24.0, 16.0)), editor);
    let enter = resposta.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let (xb, yb) = if lado { (x + w_campo + VAO_BOTAO, y) } else { (x, y + ALTO_CAMPO + VAO_BOTAO) };
    let clicou = botao(ui, c.pt(xb, yb), &envia).clicked();
    if let Some(corpo) = pronto.filter(|_| enter || clicou) {
        alvo.estado.gesto(&ctx, alvo.servidor, "orquestrador", &corpo);
        texto.clear();
        marcar_instante(&ctx, ("plano-mesa-enviado", &a.referencia));
    }
    ctx.data_mut(|d| d.insert_temp(id, texto));
    let enviado = clarao(&ctx, ("plano-mesa-enviado", &a.referencia), 3.0) > 0.0;
    let (legenda, cor) = if enviado { (tr("Pedido enviado ao orquestrador"), VERDE) } else { (tr("O orquestrador propõe e você revisa"), MEDIO) };
    c.texto(legenda, x, yb + ALTO_CAMPO + 15.0, Letra::nova(10.5, cor));
}

#[allow(clippy::too_many_arguments)]
fn da_acao(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, b: &Bloco, x: f32, mut y: f32, w: f32) {
    let pecas = lista::altura(c.p, f, a, b, w);
    if pecas > 0.0 {
        lista::desenhar(ui, c, alvo, f, a, None, tr("Da ação"), b, x, y, w);
        y += pecas + VAO;
    }
    let checks = tempo::altura(c.p, &b.checkpoints, w);
    if checks > 0.0 {
        tempo::desenhar(ui, c, alvo, a, &b.checkpoints, (x, y, w));
        y += checks + VAO;
    }
    if !b.tabelas.is_empty() {
        prancha::tabelas(ui, c, &b.tabelas, x, y, w);
    }
}

#[allow(clippy::too_many_arguments)]
fn desenhar_parte(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Montagem, parte: Parte, x: f32, y: f32, w: f32) -> Option<Saida> {
    match parte {
        Parte::Cabeca => cabeca(ui, c, a, x, y, w),
        Parte::Sobre => {
            if !a.texto.is_empty() {
                bloco(&a.texto, LETRA_SOBRE, w, PASSO_SOBRE).cor(CORPO).em(c.p, c.pt(x, y));
            }
        }
        Parte::Ajuste => ajuste(ui, c, alvo, a, x, y, w),
        Parte::Agentes => return movimentos::desenhar(ui, c, a, x, y, w),
        Parte::DaAcao => {
            if let Some(b) = da_acao_de(m) {
                da_acao(ui, c, alvo, f, a, b, x, y, w);
            }
        }
        Parte::Soltos => tempo::desenhar(ui, c, alvo, a, &m.soltos, (x, y, w)),
    }
    None
}

/// Desenha a mesa com o topo em `y`, no mesmo arranjo que `altura` mediu para esta largura.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Montagem, x: f32, y: f32, w: f32, larga: bool) -> Option<Saida> {
    let mut saida = None;
    for k in arranjo(tem_da_acao(m), w, larga) {
        let mut yy = y + k.y;
        for parte in k.partes {
            // medida antes de desenhar, como `altura` mediu: um clique que abre uma peça só muda a conta no quadro seguinte
            let alto = alto_parte(c.p, f, a, m, parte, k.w);
            saida = desenhar_parte(ui, c, alvo, f, a, m, parte, x + k.x, yy, k.w).or(saida);
            yy += alto;
        }
    }
    saida
}

#[cfg(test)]
#[path = "acao_obra_mesa_testes.rs"]
mod testes;
