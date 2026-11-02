//! O diff da ação (`design/planejar/acao-planos.mjs:47`) no jeito de um diff de git: um cabeçalho com o título, o autor e os números (+1, −1…),
//! as linhas que saíram em vermelho e as que entraram em verde (fundo suave, barra e sinal), a palavra exata destacada, e o controle de revisão ao lado.
//! «Editar» troca a linha nova de cada plano alterado por um campo; o rascunho fica na memória do egui, por diff, e vai junto ao aprovar.
//! Quem usa: `altura` antes de empilhar e `desenhar` no lugar, os dois com `y` no topo do cabeçalho.

use super::acao_estilo::{alfa, cartao, etiqueta, mix, raio, tracejado, CORPO, RAIO};
use super::acao_movimento::{cascata, com_alfa, desde_que_abriu, hover};
use super::caixinha::{Clique, Lapis};
use super::cartao_caixa;
use super::linha::Pincel;
use super::modelo::{Acao, Diff};
use super::revisao::{self, Alvo, EstadoPeca, Meio, Peca};
use crate::fonte::{familia, Peso, SUBIDA};
use crate::i18n::tr;
use crate::planta::Texto;
use crate::tema::{AMBAR, BAIXO, CAMPO, MEDIO, TINTA};
use egui::{vec2, CornerRadius, FontId, Rect, Sense, Stroke, Ui};

#[path = "diff_itens.rs"]
mod itens;
#[path = "diff_quebra.rs"]
mod quebra;
use itens::{altura_item, editaveis, id_edicao, ler_edicao, medidor, montar, pedacos, Edicao, Item, Sinal, MARGEM_DIR, PASSO, PASSO_NOTA, TAM, TAM_NOTA, X_TEXTO};
pub use itens::textos_editados;
use quebra::Fileira;

pub const TEXTOS: &[(&str, &str)] = &[
    ("Plano alterado", "Plan changed"),
    ("Ação removida", "Action removed"),
    ("agente", "agent"),
    ("{n} alterado", "{n} changed"),
    ("{n} alterados", "{n} changed"),
    ("+{n} detalhe", "+{n} detail"),
    ("+{n} detalhes", "+{n} details"),
    ("prazo", "dates"),
    ("sem data", "no date"),
    ("Detalhe: {t}", "Detail: {t}"),
    ("Também mudou: {c}", "Also changed: {c}"),
    ("área", "area"),
    ("dependências", "dependencies"),
    ("instruções", "instructions"),
    ("nome", "name"),
    ("texto", "text"),
    ("checkpoints", "checkpoints"),
    ("tabelas", "tables"),
    ("Plano feito: volta a não feito ao aprovar.", "Completed plan: it goes back to not completed when approved."),
    ("Aprovar vale a versão nova; reprovar volta ao estado original. O que mudou só pode ser iniciado depois disso.", "Approving keeps the new version; rejecting goes back to the original. What changed can only start after that."),
    ("Aprovar remove a ação; reprovar a mantém como está.", "Approving removes the action; rejecting keeps it as it is."),
];

const X_SINAL: f32 = 15.0;
const X_ID: f32 = 31.0;
const MAX_ID: f32 = 42.0;
/// O cabeçalho do diff, onde mora o título e o controle.
const CABECA: f32 = 38.0;

fn rodape(removida: bool, w: f32) -> Texto<'static> {
    let t = if removida { tr("Aprovar remove a ação; reprovar a mantém como está.") } else { tr("Aprovar vale a versão nova; reprovar volta ao estado original. O que mudou só pode ser iniciado depois disso.") };
    Texto::novo(t, TAM_NOTA).caixa(w - 4.0).lh(PASSO_NOTA).cor(MEDIO)
}

/// O que vem depois do cabeçalho até o fim do bloco: o corpo, a observação e o rodapé.
fn altura_total(p: &egui::Painter, corpo: f32, nota: f32, removida: bool, w: f32) -> f32 {
    let linhas = rodape(removida, w).galeria(p).rows.len().max(1) as f32;
    CABECA + corpo + 4.0 + if nota > 0.0 { 8.0 + nota } else { 0.0 } + 8.0 + linhas * PASSO_NOTA + 6.0
}

fn peca_de<'a>(acao: &'a Acao, d: &'a Diff, e: &Edicao) -> Peca<'a> {
    let mut peca = Peca::diff(&acao.referencia, d);
    if e.aberta && peca.estado() == EstadoPeca::Pendente {
        peca.textos = textos_editados(&d.linhas, &e.textos);
    }
    peca
}

/// Quanto o diff ocupa, para empilhar antes de desenhar. Zero sem diff.
pub fn altura(p: &egui::Painter, acao: &Acao, w: f32) -> f32 {
    let Some(d) = &acao.diff else { return 0.0 };
    let chave = format!("diff:{}", acao.referencia);
    let e = ler_edicao(p.ctx(), &chave, &d.hash);
    let peca = peca_de(acao, d, &e);
    let editando = e.aberta && peca.estado() == EstadoPeca::Pendente;
    let itens = montar(p, acao, d, w, editando.then_some(&e));
    let corpo: f32 = itens.iter().map(|i| altura_item(p, i, w)).sum();
    altura_total(p, corpo, revisao::altura_nota(p, &peca, w), d.removida, w)
}

/// O que `desenhar` devolve. `ver_agentes` fica com quem chama, porque a folha que ele percorre está emprestada do mesmo `Estado`.
pub struct Saida {
    pub altura: f32,
    pub ver_agentes: bool,
}

/// Desenha o diff da ação em `(x, y)`, com `y` no topo do cabeçalho.
pub fn desenhar(l: Lapis, ui: &mut Ui, alvo: Alvo, acao: &Acao, x: f32, y: f32, w: f32) -> Saida {
    let Some(d) = &acao.diff else { return Saida { altura: 0.0, ver_agentes: false } };
    let ctx = ui.ctx().clone();
    let chave = format!("diff:{}", acao.referencia);
    let mut e = ler_edicao(&ctx, &chave, &d.hash);
    let meio = if editaveis(d) { Meio::Editar(e.aberta) } else { Meio::Revisar };
    let peca = peca_de(acao, d, &e);
    let estado = peca.estado();
    let editando = e.aberta && estado == EstadoPeca::Pendente;
    let itens = montar(l.p, acao, d, w, editando.then_some(&e));
    let corpo: f32 = itens.iter().map(|i| altura_item(l.p, i, w)).sum();
    cartao(l.p, Rect::from_min_size(l.pt(x, y), vec2(w, CABECA + corpo + 2.0)), CAMPO, alfa(BAIXO, 0.7));
    cabeca(l, d, (x, y, w));
    if revisao::canto(l, ui, alvo, x + w - 10.0, y + 25.0, &peca, meio) {
        e.aberta = !e.aberta;
        ctx.data_mut(|m| m.insert_temp(id_edicao(&chave), e.clone()));
        ctx.request_repaint();
    }
    let ms = desde_que_abriu(&ctx, ("acao-painel", &acao.referencia));
    let (mut yy, mut ver_agentes, mut mudou) = (y + CABECA + 1.0, false, false);
    for (i, item) in itens.iter().enumerate() {
        let h = altura_item(l.p, item, w);
        // cada linha chega depois da anterior: o diff se escreve de cima para baixo
        let pintor = com_alfa(l.p, cascata(&ctx, ms, 1, 90.0 + 38.0 * i.min(14) as f32));
        let li = Lapis { p: &pintor, o: l.o };
        match item {
            Item::Linha { sinal, id, fileiras, caixinha } => {
                fundo_e_id(li, ui, (&chave, i), *sinal, id, (x, yy, w, h), i + 1 == itens.len());
                pintar_fileiras(li, *sinal, fileiras, x + X_TEXTO, yy, estado);
                if let Some(plano) = caixinha.as_ref().and_then(|r| acao.planos.iter().find(|q| &q.referencia == r)) {
                    match cartao_caixa::caixa(li, ui, x + w - 26.0, yy + 4.0, plano) {
                        Some(Clique::Marcar) => super::caixinha::marcar(&ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &[plano.id.as_str()]),
                        Some(Clique::Agentes) => ver_agentes = true,
                        None => {}
                    }
                }
            }
            Item::Editor { sinal, id, referencia, texto } => {
                fundo_e_id(li, ui, (&chave, i), *sinal, id, (x, yy, w, h), false);
                let area = Rect::from_min_size(l.pt(x + X_TEXTO, yy + 3.0), vec2(w - X_TEXTO - MARGEM_DIR, h - 6.0));
                tracejado(li.p, area.expand2(vec2(3.0, -1.0)), alfa(AMBAR, 0.75), 3.0);
                let mut rascunho = texto.clone();
                let editor = egui::TextEdit::multiline(&mut rascunho).id(id_edicao(&chave).with(referencia)).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(TAM, familia(Peso::Normal))).text_color(TINTA).desired_rows(1).desired_width(area.width());
                if ui.put(area, editor).changed() {
                    // um plano é uma linha só: o Enter não abre outra
                    e.textos.insert(referencia.clone(), rascunho.replace(['\n', '\r'], " "));
                    mudou = true;
                }
            }
            Item::Nota(fileiras) => {
                for (k, f) in fileiras.iter().enumerate() {
                    li.texto(&f.texto, x + X_TEXTO, yy + 11.0 + k as f32 * PASSO_NOTA, TAM_NOTA, Peso::Normal, AMBAR);
                }
            }
        }
        yy += h;
    }
    if mudou {
        ctx.data_mut(|m| m.insert_temp(id_edicao(&chave), e));
    }
    yy += 4.0;
    let h_nota = revisao::altura_nota(l.p, &peca, w);
    if h_nota > 0.0 {
        yy += 8.0;
        revisao::nota(l, ui, alvo, x, yy, w, &peca);
        yy += h_nota;
    }
    rodape(d.removida, w).em(l.p, l.pt(x + 2.0, yy + 8.0));
    Saida { altura: altura_total(l.p, corpo, h_nota, d.removida, w), ver_agentes }
}

/// O cabeçalho do diff: o ícone ±, o título, o autor e os números do resumo, cada um com a sua cor.
fn cabeca(l: Lapis, d: &Diff, (x, y, w): (f32, f32, f32)) {
    let faixa = Rect::from_min_size(l.pt(x + 1.0, y + 1.0), vec2(w - 2.0, CABECA - 1.0));
    l.p.rect_filled(faixa, CornerRadius { nw: RAIO, ne: RAIO, sw: 0, se: 0 }, alfa(MEDIO, 0.09));
    l.fio(alfa(BAIXO, 0.6), 1.0, (x + 1.0, y + CABECA), (x + w - 1.0, y + CABECA));
    let c = Pincel { p: l.p, o: l.o };
    // o ícone: um quadrado com + e −
    let cx = l.pt(x + 17.0, y + 19.0);
    l.p.rect_stroke(Rect::from_center_size(cx, vec2(15.0, 15.0)), raio(), Stroke::new(1.2, alfa(TINTA, 0.85)), egui::StrokeKind::Inside);
    l.p.line_segment([cx + vec2(-3.5, -2.2), cx + vec2(3.5, -2.2)], Stroke::new(1.2, alfa(TINTA, 0.85)));
    l.p.line_segment([cx + vec2(0.0, -5.7), cx + vec2(0.0, 1.3)], Stroke::new(1.2, alfa(TINTA, 0.85)));
    l.p.line_segment([cx + vec2(-3.5, 4.0), cx + vec2(3.5, 4.0)], Stroke::new(1.2, alfa(TINTA, 0.85)));
    let nome = if d.removida { tr("Ação removida") } else { tr("Plano alterado") };
    let largura = Texto::novo(nome, 12.0).peso(Peso::Negrito).trilha(0.06).largura(l.p);
    l.texto(nome, x + 34.0, y + 24.0, 12.0, Peso::Negrito, TINTA);
    let agente = if d.agente.is_empty() { tr("agente") } else { d.agente.as_str() };
    let mut cx = x + 34.0 + largura + 12.0;
    let livre = x + w - revisao::medir_partes(l.p, Meio::Editar(false)).1 - 24.0;
    let letra = super::linha::Letra::nova(TAM_NOTA, MEDIO).trilha(0.06);
    let texto = c.cortado(agente, (livre - cx).max(30.0).min(170.0), &letra).0;
    cx += c.texto(&texto, cx, y + 23.0, letra) + 12.0;
    for (texto, cor) in pedacos(&d.resumo) {
        let wc = c.largura(&texto, &super::linha::Letra::nova(9.5, MEDIO).trilha(0.04)) + 12.0;
        if cx + wc > livre {
            break;
        }
        etiqueta(&c, cx, y + 11.0, &texto, cor, true);
        cx += wc + 6.0;
    }
}

/// O fundo da linha, a barra, o sinal e o id, com o id inteiro no hover quando não cabe. O fundo clareia com o mouse em cima.
fn fundo_e_id(l: Lapis, ui: &mut Ui, chave: (&str, usize), sinal: Sinal, id: &str, (x, y, w, h): (f32, f32, f32, f32), ultima: bool) {
    let cor = sinal.cor();
    let area = Rect::from_min_size(l.pt(x + 1.0, y), vec2(w - 2.0, h));
    let resposta = ui.interact(area, ui.scope_id().with(("plano-diff-linha", chave)), Sense::hover());
    let sobre = hover(ui.ctx(), ("diff-linha", chave), resposta.hovered());
    if sinal != Sinal::Neutro {
        l.p.rect_filled(area, 0, alfa(cor, 0.105 + 0.07 * sobre));
        l.p.rect_filled(Rect::from_min_size(area.min, vec2(3.0, h)), 0, alfa(cor, 0.95));
    } else if sobre > 0.01 {
        l.p.rect_filled(area, 0, alfa(TINTA, 0.04 * sobre));
    }
    if !ultima {
        l.fio(alfa(BAIXO, 0.16), 1.0, (x + 1.0, y + h), (x + w - 1.0, y + h));
    }
    l.fio(alfa(BAIXO, 0.22), 1.0, (x + X_TEXTO - 9.0, y + 3.0), (x + X_TEXTO - 9.0, y + h - 3.0));
    l.texto(sinal.letra(), x + X_SINAL, y + PASSO, TAM, Peso::Negrito, cor);
    let t = Texto::novo(id, TAM_NOTA).trilha(0.06).caixa(MAX_ID).linhas(1).cor(mix(MEDIO, TINTA, sobre));
    let r = t.em(l.p, l.pt(x + X_ID, y + PASSO - SUBIDA * TAM_NOTA));
    if t.galeria(l.p).elided {
        ui.interact(r, ui.scope_id().with(("plano-diff-id", chave)), Sense::hover()).on_hover_text(id);
    }
}

/// As linhas quebradas: primeiro o destaque, na cor do sinal e só sobre a palavra; a reprovada fica riscada, em tom médio.
fn pintar_fileiras(l: Lapis, sinal: Sinal, fileiras: &[Fileira], x: f32, y: f32, estado: EstadoPeca) {
    let medir = medidor(l.p, TAM);
    let ate = |f: &Fileira, n: usize| medir(&f.texto.chars().take(n).collect::<String>());
    let corpo = if matches!(estado, EstadoPeca::Aprovada | EstadoPeca::Reprovada) { MEDIO } else { CORPO };
    for (k, f) in fileiras.iter().enumerate() {
        let base = y + PASSO + k as f32 * PASSO;
        for &(a, b) in &f.trechos {
            let x0 = x + if a == 0 { 0.0 } else { ate(f, a) };
            let r = Rect::from_min_size(l.pt(x0 - 2.0, base - 11.5), vec2(ate(f, b) - (x0 - x) + 4.0, PASSO - 1.0));
            l.p.rect_filled(r, 2.0, alfa(sinal.cor(), 0.34));
            l.p.line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(1.0, alfa(sinal.cor(), 0.9)));
        }
        l.texto(&f.texto, x, base, TAM, Peso::Normal, if sinal == Sinal::Neutro { corpo } else { crate::tema::TINTA });
        if estado == EstadoPeca::Reprovada {
            revisao::risco(l, x, x + medir(&f.texto), base - 4.0);
        }
    }
}
