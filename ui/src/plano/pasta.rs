//! A faixa de `semPasta`, no topo da folha: campo de caminho e `Ligar pasta` (`gesto/pasta`). Sem pasta, a caixinha só daria 409.
//! Fica à vista enquanto o servidor disser `semPasta`: quando a pasta liga, a folha refeita a tira. Recusa volta em `Estado::aviso`, e o caminho fica no campo para corrigir.

use super::caixinha::Lapis;
use super::folha::estilo::com_alfa;
use super::linha::{figura, Letra, Pincel};
use super::modelo::Folha;
use super::Estado;
use crate::fonte::{familia, Peso};
use crate::i18n::tr;
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, BAIXO, CAMPO, CARTAO, MEDIO, TINTA};
use egui::{vec2, FontId, Id, Rect, Stroke, StrokeKind, Ui};
use serde_json::{json, Value};

pub const TEXTOS: &[(&str, &str)] = &[
    ("Ligue a pasta do projeto", "Link the project folder"),
    ("Os agentes trabalham na pasta ligada aqui. Sem ela, nenhum plano pode começar.", "Agents work in the folder linked here. Without one, no plan can start."),
    ("Caminho completo da pasta", "Full path of the folder"),
    ("Ligar pasta", "Link folder"),
];

/// A altura da faixa; a folha soma o vão que quiser depois dela.
pub const ALTURA: f32 = 52.0;
const CAMPO_ALTO: f32 = 32.0;
/// O quanto o texto e o campo recuam, para a pasta caber à esquerda.
const RECUO: f32 = 56.0;

/// Quanto a faixa ocupa nesta folha.
pub fn altura(folha: &Folha) -> f32 {
    if folha.sem_pasta {
        ALTURA
    } else {
        0.0
    }
}

/// O corpo de `gesto/pasta`, ou `None` com o campo vazio. Quem confere se é pasta é o servidor.
pub fn corpo_ligar(ws: &str, bp: &str, caminho: &str) -> Option<Value> {
    let caminho = caminho.trim();
    (!caminho.is_empty()).then(|| json!({"workspaceId": ws, "blueprintId": bp, "caminho": caminho}))
}

fn id_campo(bp: &str) -> Id {
    Id::unique(("plano-pasta", bp))
}

/// Desenha a faixa em `(x, y)` com a largura `w`, se a folha pede. Devolve a altura ocupada, a mesma de `altura`.
#[allow(clippy::too_many_arguments)]
pub fn faixa(l: Lapis, ui: &mut Ui, estado: &Estado, servidor: &str, ws: &str, bp: &str, sem_pasta: bool, x: f32, y: f32, w: f32) -> f32 {
    if !sem_pasta {
        return 0.0;
    }
    let ctx = ui.ctx().clone();
    let p = Pincel { p: l.p, o: l.o };
    p.ret(x, y, w, ALTURA, Some(com_alfa(CARTAO, 0.96)), Some((MEDIO, 1.0)));
    figura::pasta(&p, x + 30.0, y + ALTURA / 2.0, AMBAR);
    let titulo = tr("Ligue a pasta do projeto");
    let letra = Letra::nova(11.0, AMBAR).forte();
    let w_titulo = p.texto_meio(&titulo.to_uppercase(), x + RECUO, y + ALTURA / 2.0, letra);
    let _ = ui.interact(p.caixa(x, y, RECUO + w_titulo, ALTURA), id_campo(bp).with("dica"), egui::Sense::hover()).on_hover_text(tr("Os agentes trabalham na pasta ligada aqui. Sem ela, nenhum plano pode começar."));
    let x_campo = x + RECUO + w_titulo + 16.0;

    let id = id_campo(bp);
    let mut caminho = ctx.data(|d| d.get_temp::<String>(id)).unwrap_or_default();
    let pronto = corpo_ligar(ws, bp, &caminho);
    let b = Botao { texto: tr("Ligar pasta"), tam: 11.0, cheio: true, ativo: pronto.is_some(), altura: CAMPO_ALTO, pad_x: 14.0 };
    let largura_botao = b.largura(l.p);
    let topo = y + (ALTURA - CAMPO_ALTO) / 2.0;
    let r = Rect::from_min_size(l.pt(x_campo, topo), vec2(x + w - 16.0 - largura_botao - 8.0 - x_campo, CAMPO_ALTO));
    l.p.rect_filled(r, 0, CAMPO);
    l.p.rect_stroke(r, 0, Stroke::new(1.0, com_alfa(BAIXO, 0.9)), StrokeKind::Inside);
    let dica = egui::RichText::new(tr("Caminho completo da pasta")).size(12.0).color(BAIXO);
    let editor = egui::TextEdit::singleline(&mut caminho).id(id.with("campo")).frame(egui::Frame::NONE).margin(egui::Margin::ZERO).font(FontId::new(12.0, familia(Peso::Normal))).text_color(TINTA).hint_text(dica).desired_width(r.width() - 20.0);
    let campo = ui.put(r.shrink2(vec2(10.0, 8.0)), editor);
    let enter = campo.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let clicou = botao(ui, l.pt(x + w - 16.0 - largura_botao, topo), &b).clicked();
    ctx.data_mut(|d| d.insert_temp(id, caminho));
    if let (true, Some(corpo)) = (enter || clicou, pronto) {
        estado.gesto(&ctx, servidor, "pasta", &corpo);
    }
    ALTURA
}
