//! As perguntas antes de apagar um workspace, um blueprint ou uma ação: apagar leva junto o que está dentro, e não dá para desfazer.

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

/// A pergunta do menu do botão direito: «Excluir blueprint?» ou «Excluir ação?» (`excluirBlueprint` e `excluirAcao` de manual.js).
pub fn item(ctx: &egui::Context, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    let (ws, bp, id) = vista.excluir_item.clone()?;
    let Some(p) = doc.workspace(&ws).and_then(|w| w.get("blueprints")?.as_array()?.iter().find(|b| campo(b, "id") == bp)) else {
        vista.excluir_item = None;
        return None;
    };
    let f = id.as_deref().and_then(|a| p.get("acoes")?.as_array()?.iter().find(|x| campo(x, "id") == a));
    if id.is_some() && f.is_none() {
        vista.excluir_item = None;
        return None;
    }
    let (titulo, botao, explica) = match f {
        Some(f) => {
            let nome = if campo(f, "nome").is_empty() { tr("sem nome") } else { campo(f, "nome") };
            let tem = |c: &str| f.get(c).and_then(|l| l.as_array()).is_some_and(|l| !l.is_empty());
            let extra = if tem("planos") || tem("checkpoints") { tr(", junto com seus planos e checkpoints") } else { "" };
            (tr("Excluir ação?"), tr("Excluir ação"), tr_com("A ação \"{nome}\" será apagada{extra}. Não dá para desfazer.", &[("nome", nome), ("extra", extra)]))
        }
        None => {
            let nome = if campo(p, "nome").is_empty() { tr("Sem nome") } else { campo(p, "nome") };
            (tr("Excluir blueprint?"), tr("Excluir blueprint"), tr_com("O blueprint \"{nome}\" será apagado, junto com todas as suas ações. Não dá para desfazer.", &[("nome", nome)]))
        }
    };
    let (mut confirmou, mut fechar) = (false, false);
    egui::Window::new(texto(titulo, TINTA).strong()).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.label(texto(explica, MEDIO));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            confirmou = ui.add_enabled(!ocupado, egui::Button::new(texto(botao, ERRO))).clicked();
            fechar = ui.button(texto(tr("Cancelar"), MEDIO)).clicked();
        });
    });
    if confirmou || fechar {
        vista.excluir_item = None;
    }
    confirmou.then(|| match id {
        Some(a) => Acao::ExcluirAcao(ws, bp, a),
        None => Acao::ExcluirBlueprint(ws, bp),
    })
}

fn campo<'a>(v: &'a serde_json::Value, nome: &str) -> &'a str {
    v.get(nome).and_then(|t| t.as_str()).unwrap_or("")
}
