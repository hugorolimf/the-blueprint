//! As telas e os detalhes da ação, embutidos no bloco do texto: o número, o título, a legenda e a figura já renderizada, sem cartão.
//! A figura começa pequena: com a legenda ao lado, se couber; numa coluna estreita, com a legenda embaixo. O clique a amplia na largura do bloco e outro clique a reduz.
//! Quem usa: `acao_blocos.rs`, com as telas de cada plano. A altura sai das medidas que o servidor mandou, sem rasterizar antes.

use super::acao_estilo::{CORPO, LETRA, LETRA_MENOR, LETRA_TITULO, PASSO_LETRA};
use super::acao_movimento::com_alfa;
use super::caixinha::Lapis;
use super::figura;
use super::linha::{Letra, Pincel};
use super::modelo::{Acao as AcaoPlano, Detalhe, Diff, Figura, Folha};
use super::revisao::{self, Alvo, EstadoPeca, Meio, Peca};
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema::{MEDIO, TINTA};
use egui::{Context, CursorIcon, Id, Painter, Sense, Stroke, StrokeKind, Ui};

/// Figura alta não toma a janela inteira: passa disso, fica mais estreita e encostada à esquerda.
const FIGURA_MAX: f32 = 720.0;
/// A figura reduzida: larga o bastante para reconhecer a tela, baixa o bastante para vários planos caberem na janela.
const MINI_L: f32 = 300.0;
const MINI_COLUNA: f32 = 440.0;
const MINI_A: f32 = 190.0;
/// Abaixo disso a legenda não cabe ao lado da figura.
const LADO_A_LADO: f32 = 640.0;
/// Abaixo disso os botões de revisão não cabem na linha do título.
const CONTROLES_NA_LINHA: f32 = 500.0;
const ALTO_CONTROLES: f32 = 34.0;
/// O vão entre a figura reduzida e a legenda ao lado dela.
const VAO_MINI: f32 = 18.0;
const CABECA: f32 = 32.0;
const VAO_LEGENDA: f32 = 8.0;
const VAO_NOTA: f32 = 8.0;
/// O vão entre uma tela e a seguinte.
const VAO: f32 = 26.0;
const ALFA_REPROVADA: f32 = 0.4;

/// O que a versão pendente traz para a ação; reprovada, nada disso entra e sai da tela.
pub fn diff_vivo(a: &AcaoPlano) -> Option<&Diff> {
    a.diff.as_ref().filter(|d| revisao::estado_de(d.revisao.as_ref()) != EstadoPeca::Reprovada)
}

/// Os detalhes da ação como ficariam ao aprovar: os da árvore, com a versão nova dos que mudaram, e os que chegam.
pub fn detalhes_da(a: &AcaoPlano) -> Vec<&Detalhe> {
    let diff = diff_vivo(a);
    let mudado = |id: &str| diff.and_then(|d| d.detalhes_mudados.iter().find(|m| m.id == id));
    let novos = diff.into_iter().flat_map(|d| &d.detalhes_novos).filter(|n| !a.detalhes.iter().any(|d| d.id == n.id));
    a.detalhes.iter().map(|d| mudado(&d.id).unwrap_or(d)).chain(novos).collect()
}

/// (largura, altura) da figura nesta largura. Sem figura, o detalhe é só texto e nada é reservado.
pub fn medidas_figura(f: Option<&Figura>, largura: f32) -> (f32, f32) {
    let Some(f) = f else { return (0.0, 0.0) };
    let alta = figura::altura_para(Some(f), largura).round();
    if alta <= FIGURA_MAX {
        return (largura, alta);
    }
    (FIGURA_MAX * figura::proporcao(Some(f)), FIGURA_MAX)
}

/// (largura, altura) da figura reduzida. Figura alta fica mais estreita, para não passar da altura.
pub fn medidas_mini(f: Option<&Figura>, w: f32) -> (f32, f32) {
    let Some(f) = f else { return (0.0, 0.0) };
    let l = if em_coluna(w) { w.min(MINI_COLUNA) } else { MINI_L };
    let alta = figura::altura_para(Some(f), l).round();
    if alta <= MINI_A {
        return (l, alta);
    }
    (MINI_A * figura::proporcao(Some(f)), MINI_A)
}

fn id_ampliada(a: &AcaoPlano, d: &Detalhe) -> Id {
    Id::unique(("plano-tela-ampliada", &a.referencia, &d.id))
}

fn em_coluna(w: f32) -> bool {
    w < LADO_A_LADO
}

fn alto_controles(d: &Detalhe, w: f32) -> f32 {
    if d.pede_aprovacao && w < CONTROLES_NA_LINHA { ALTO_CONTROLES } else { 0.0 }
}

pub fn ampliada(ctx: &Context, a: &AcaoPlano, d: &Detalhe) -> bool {
    ctx.data(|m| m.get_temp::<bool>(id_ampliada(a, d))).unwrap_or(false)
}

/// Sem figura ou ampliada, a legenda ocupa a largura toda.
fn reduzida(ctx: &Context, a: &AcaoPlano, d: &Detalhe) -> bool {
    d.figura.is_some() && !ampliada(ctx, a, d)
}

fn largura_legenda(ctx: &Context, a: &AcaoPlano, d: &Detalhe, w: f32) -> f32 {
    if reduzida(ctx, a, d) && !em_coluna(w) { w - medidas_mini(d.figura.as_ref(), w).0 - VAO_MINI } else { w }
}

fn texto_legenda(d: &Detalhe, w: f32) -> Texto<'_> {
    Texto::novo(&d.legenda, LETRA).trilha(0.03).caixa(w).lh(PASSO_LETRA).cor(CORPO)
}

fn alto_legenda(p: &Painter, d: &Detalhe, w: f32) -> f32 {
    if d.legenda.is_empty() {
        return 0.0;
    }
    texto_legenda(d, w).galeria(p).rows.len() as f32 * PASSO_LETRA + VAO_LEGENDA
}

fn alto_nota(p: &Painter, d: &Detalhe, peca: &Peca, w: f32) -> f32 {
    let nota = if d.pede_aprovacao { revisao::altura_nota(p, peca, w) } else { 0.0 };
    if nota > 0.0 { nota + VAO_NOTA } else { 0.0 }
}

fn altura_de(p: &Painter, f: &Folha, a: &AcaoPlano, d: &Detalhe, w: f32) -> f32 {
    let peca = Peca::detalhe(f, &a.referencia, d);
    let legenda = alto_legenda(p, d, largura_legenda(p.ctx(), a, d, w));
    let corpo = if reduzida(p.ctx(), a, d) {
        let mini = medidas_mini(d.figura.as_ref(), w).1;
        if em_coluna(w) { mini + if legenda > 0.0 { VAO_LEGENDA + legenda } else { 0.0 } } else { legenda.max(mini) }
    } else {
        legenda + medidas_figura(d.figura.as_ref(), w).1
    };
    CABECA + alto_controles(d, w) + alto_nota(p, d, &peca, w) + corpo + VAO
}

/// Quanto estas telas ocupam, para empilhar antes de desenhar. Zero sem telas.
pub fn altura(p: &Painter, f: &Folha, a: &AcaoPlano, lista: &[&Detalhe], w: f32) -> f32 {
    lista.iter().map(|d| altura_de(p, f, a, d, w)).sum()
}

/// As telas a partir de `(x, y)`, na largura `w`, uma embaixo da outra.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &AcaoPlano, lista: &[&Detalhe], x: f32, y: f32, w: f32) {
    let mut yy = y;
    for d in lista {
        tela(ui, c, alvo, f, a, d, x, yy, w);
        yy += altura_de(c.p, f, a, d, w);
    }
}

/// O número e o grupo à esquerda, o título ao lado e o controle de revisão à direita. O «+» é o mesmo do diff: esta tela ainda não foi aprovada.
#[allow(clippy::too_many_arguments)]
fn cabeca(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &AcaoPlano, d: &Detalhe, peca: &Peca, x: f32, y: f32, w: f32) {
    let sinal = if a.detalhes.iter().any(|v| v.id == d.id) { "" } else { "+ " };
    let grupo = if d.grupo.is_empty() { String::new() } else { format!("{} · ", d.grupo) };
    let w_id = c.texto(&format!("{sinal}{grupo}{}-{}", a.referencia, d.id), x, y + 19.0, Letra::nova(LETRA_MENOR, MEDIO));
    let reprovada = d.pede_aprovacao && peca.estado() == EstadoPeca::Reprovada;
    let na_linha = d.pede_aprovacao && alto_controles(d, w) == 0.0;
    let reserva = if na_linha { revisao::medir_partes(c.p, Meio::Tela).1 + 16.0 } else { 0.0 };
    let x_titulo = x + w_id + 10.0;
    let letra = Letra::nova(LETRA_TITULO, if reprovada { MEDIO } else { TINTA }).forte();
    let (titulo, cortou) = c.cortado(&d.titulo, (x + w - reserva - x_titulo).max(40.0), &letra);
    let w_titulo = c.texto(&titulo, x_titulo, y + 19.0, letra);
    let lapis = Lapis { p: c.p, o: c.o };
    if reprovada {
        revisao::risco(lapis, x_titulo, x_titulo + w_titulo, y + 14.0);
    }
    if cortou {
        let _ = ui.interact(c.caixa(x_titulo, y, w_titulo, CABECA), Id::unique(("plano-detalhe-titulo", &a.referencia, &d.id)), Sense::hover()).on_hover_text(d.titulo.as_str());
    }
    if d.pede_aprovacao {
        revisao::canto(lapis, ui, alvo, x + w, if na_linha { y + 19.0 } else { y + CABECA + 18.0 }, peca, Meio::Tela);
    }
}

#[allow(clippy::too_many_arguments)]
fn tela(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &AcaoPlano, d: &Detalhe, x: f32, y: f32, w: f32) {
    let peca = Peca::detalhe(f, &a.referencia, d);
    cabeca(ui, c, alvo, a, d, &peca, x, y, w);
    let mut yy = y + CABECA + alto_controles(d, w);
    let nota = alto_nota(c.p, d, &peca, w);
    if nota > 0.0 {
        revisao::nota(Lapis { p: c.p, o: c.o }, ui, alvo, x, yy, w, &peca);
        yy += nota;
    }
    let pequena = reduzida(ui.ctx(), a, d);
    let w_legenda = largura_legenda(ui.ctx(), a, d, w);
    let (w_fig, alto) = if pequena { medidas_mini(d.figura.as_ref(), w) } else { medidas_figura(d.figura.as_ref(), w) };
    // a legenda vem ao lado da figura reduzida, embaixo dela numa coluna estreita e acima da figura ampliada
    let (x_legenda, y_legenda, y_figura) = match (pequena, em_coluna(w)) {
        (true, true) => (x, yy + alto + VAO_LEGENDA, yy),
        (true, false) => (x + w_fig + VAO_MINI, yy, yy),
        (false, _) => (x, yy, yy + alto_legenda(c.p, d, w_legenda)),
    };
    if !d.legenda.is_empty() {
        texto_legenda(d, w_legenda).em(c.p, c.pt(x_legenda, y_legenda));
    }
    if alto > 0.0 {
        figura_clicavel(ui, c, alvo, a, d, &peca, (x, y_figura, w_fig, alto), pequena);
    }
}

/// A figura e o clique que a amplia ou reduz. As linhas de baixo andam junto, como ao abrir a ação.
#[allow(clippy::too_many_arguments)]
fn figura_clicavel(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &AcaoPlano, d: &Detalhe, peca: &Peca, (x, y, w, h): (f32, f32, f32, f32), pequena: bool) {
    let nome = if d.titulo.is_empty() { &d.id } else { &d.titulo };
    let area = c.caixa(x, y, w, h);
    if d.pede_aprovacao && peca.estado() == EstadoPeca::Reprovada {
        figura::desenhar(&com_alfa(c.p, ALFA_REPROVADA), alvo.servidor, d.figura.as_ref(), nome, area);
    } else {
        figura::desenhar(c.p, alvo.servidor, d.figura.as_ref(), nome, area);
    }
    let id = id_ampliada(a, d);
    let resposta = ui.interact(area, id.with("clique"), Sense::click());
    if resposta.hovered() {
        c.p.rect_stroke(area, 0, Stroke::new(1.0, MEDIO), StrokeKind::Inside);
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if resposta.clicked() {
        ui.ctx().data_mut(|m| m.insert_temp(id, pequena));
        crate::anima::mexer(ui.ctx(), 350.0);
    }
    let _ = resposta.on_hover_text(tr(if pequena { "Clique para ampliar" } else { "Clique para reduzir" }));
}

#[cfg(test)]
mod tests {
    use super::super::modelo::{EstadoRevisao, Revisao};
    use super::*;

    fn det(id: &str, titulo: &str) -> Detalhe {
        Detalhe { id: id.into(), titulo: titulo.into(), ..Detalhe::default() }
    }

    fn acao(estado: Option<EstadoRevisao>) -> AcaoPlano {
        let revisao = estado.map(|estado| Revisao { estado, ..Revisao::default() });
        let diff = Diff { detalhes_novos: vec![det("02", "nova")], detalhes_mudados: vec![det("01", "refeita")], revisao, ..Diff::default() };
        AcaoPlano { detalhes: vec![det("01", "antiga")], diff: Some(diff), ..AcaoPlano::default() }
    }

    fn titulos(a: &AcaoPlano) -> Vec<String> {
        detalhes_da(a).iter().map(|d| d.titulo.clone()).collect()
    }

    #[test]
    fn a_versao_pendente_ja_aparece_na_acao() {
        assert_eq!(titulos(&acao(None)), ["refeita", "nova"]);
    }

    #[test]
    fn reprovada_volta_ao_que_estava() {
        assert_eq!(titulos(&acao(Some(EstadoRevisao::Reprovada))), ["antiga"]);
    }

    #[test]
    fn sem_figura_nada_e_reservado() {
        assert_eq!(medidas_figura(None, 800.0), (0.0, 0.0));
    }

    #[test]
    fn a_figura_reduzida_nao_passa_da_altura() {
        assert_eq!(medidas_mini(None, 800.0), (0.0, 0.0));
        let larga = Figura { largura: Some(1000), altura: Some(500), ..Figura::default() };
        assert_eq!(medidas_mini(Some(&larga), 800.0), (MINI_L, 150.0));
        assert_eq!(medidas_mini(Some(&larga), 500.0), (MINI_A * 2.0, MINI_A), "numa coluna, ocupa a coluna até o teto");
        let alta = Figura { largura: Some(500), altura: Some(1000), ..Figura::default() };
        assert_eq!(medidas_mini(Some(&alta), 800.0), (MINI_A * 0.5, MINI_A));
    }

    #[test]
    fn a_figura_alta_tem_teto_e_mantem_a_proporcao() {
        let baixa = Figura { largura: Some(1000), altura: Some(500), ..Figura::default() };
        assert_eq!(medidas_figura(Some(&baixa), 800.0), (800.0, 400.0));
        let alta = Figura { largura: Some(500), altura: Some(1000), ..Figura::default() };
        assert_eq!(medidas_figura(Some(&alta), 800.0), (FIGURA_MAX * 0.5, FIGURA_MAX));
    }
}
