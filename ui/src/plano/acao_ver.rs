//! O que o usuário abre no painel da ação (texto de plano, «ver tudo»): fica na memória do egui, porque a folha é refeita a cada resposta do servidor.

use super::super::acao_estilo::mix;
use super::super::acao_movimento::hover;
use super::super::linha::{Letra, Pincel};
use crate::i18n::tr;
use crate::tema::{BAIXO, MEDIO, TINTA};
use egui::{Context, Id, Sense, Ui};

fn id_de(chave: &str) -> Id {
    Id::unique(("plano-acao-aberto", chave))
}

/// A chave de um texto do painel: `tipo` separa ação, plano e decisão, que têm ids independentes.
pub fn chave_ver(tipo: &str, id: &str) -> String {
    format!("{tipo}:{id}")
}

pub fn alternado(ctx: &Context, chave: &str) -> bool {
    ctx.data(|d| d.get_temp::<bool>(id_de(chave))).unwrap_or(false)
}

pub fn alternar(ctx: &Context, chave: &str) {
    let aberto = alternado(ctx, chave);
    ctx.data_mut(|d| d.insert_temp(id_de(chave), !aberto));
    crate::anima::mexer(ctx, 200.0);
}

/// «ver tudo» na ponta da linha `y`, só para o texto cortado; depois de aberto, «ver menos». `x_fim` é a borda direita. A cor e o sublinhado entram no hover.
pub fn ver_tudo(ui: &Ui, c: &Pincel, chave: &str, x_fim: f32, y: f32, cortou: bool) {
    let aberto = alternado(ui.ctx(), chave);
    if !aberto && !cortou {
        return;
    }
    let t = tr(if aberto { "ver menos" } else { "ver tudo" });
    let letra = Letra::nova(9.5, MEDIO).fim();
    let largura = c.largura(t, &letra);
    let r = ui.interact(c.caixa(x_fim - largura - 4.0, y - 11.0, largura + 8.0, 17.0), ui.scope_id().with(("plano-ver-tudo", chave)), Sense::click());
    let h = hover(ui.ctx(), ("ver-tudo", chave), r.hovered());
    c.texto(t, x_fim, y, Letra { cor: mix(MEDIO, TINTA, h), ..letra });
    c.fio(x_fim - largura, y + 3.5, x_fim - largura + largura * (0.3 + 0.7 * h), y + 3.5, mix(BAIXO, TINTA, h), 1.0);
    if r.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if r.clicked() {
        alternar(ui.ctx(), chave);
    }
}
