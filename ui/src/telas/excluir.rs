//! A pergunta «Excluir workspace?»: apagar leva junto os blueprints e as ações dele, e não dá para desfazer.

use super::{texto, Acao, Vista};
use crate::documento::Documento;
use crate::i18n::{tr, tr_com, trn};
use crate::tema::{ERRO, MEDIO, TINTA};

pub fn dialogo(ctx: &egui::Context, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    let id = vista.excluir.clone()?;
    let Some(w) = doc.workspaces().into_iter().find(|w| w.id == id) else {
        vista.excluir = None;
        return None;
    };
    let mut acao = None;
    let mut fechar = false;
    egui::Window::new(texto(tr("Excluir workspace?"), TINTA).strong()).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        ui.set_width(380.0);
        let extra = if w.blueprints > 0 { tr_com(", junto com {blueprints} e todas as ações", &[("blueprints", &trn(w.blueprints, "{n} blueprint", "{n} blueprints"))]) } else { String::new() };
        ui.label(texto(tr_com("O workspace \"{nome}\" será apagado{extra}. Não dá para desfazer.", &[("nome", &w.nome), ("extra", &extra)]), MEDIO));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.add_enabled(!ocupado, egui::Button::new(texto(tr("Excluir workspace"), ERRO))).clicked() {
                acao = Some(Acao::Excluir(id.clone()));
                fechar = true;
            }
            if ui.button(texto(tr("Cancelar"), MEDIO)).clicked() {
                fechar = true;
            }
        });
    });
    if fechar {
        vista.excluir = None;
    }
    acao
}
