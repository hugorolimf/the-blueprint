//! A folha de observações (`abreFolhaDaAcao` de web/js/iniciar-acao.js): o roteiro da ação, ou a observação do blueprint, que o usuário lê e edita ali mesmo.

use super::{texto, Acao, Vista};
use crate::documento::Documento;
use crate::i18n::tr;
use crate::tema::{MEDIO, PAPEL, TINTA};
use egui::{Align2, Frame, Margin, Stroke};

pub fn folha(ctx: &egui::Context, vista: &mut Vista, doc: &Documento, ocupado: bool) -> Option<Acao> {
    let (ws, bp, acao) = vista.nota.clone()?;
    let nome_bp = doc.workspace(&ws).and_then(|w| w.get("blueprints")?.as_array()?.iter().find(|b| b.get("id").and_then(|i| i.as_str()) == Some(bp.as_str())).cloned());
    let Some(b) = nome_bp else {
        vista.nota = None;
        return None;
    };
    let nome_de = |v: &serde_json::Value| v.get("nome").and_then(|n| n.as_str()).filter(|n| !n.is_empty()).unwrap_or("").to_string();
    let (rotulo, titulo) = match &acao {
        Some(a) => (tr("Instruções da ação"), b.get("acoes").and_then(|l| l.as_array()).and_then(|l| l.iter().find(|x| x.get("id").and_then(|i| i.as_str()) == Some(a.as_str()))).map(&nome_de).unwrap_or_default()),
        None => (tr("Observação do blueprint"), nome_de(&b)),
    };
    let titulo = if titulo.is_empty() { tr("Sem nome").to_string() } else { titulo };
    let moldura = Frame::NONE.fill(PAPEL).stroke(Stroke::new(2.0, TINTA)).inner_margin(Margin::symmetric(22, 18));
    let (mut salvar, mut fechar) = (false, false);
    egui::Window::new("nota-folha").title_bar(false).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).frame(moldura).show(ctx, |ui| {
        ui.set_width(520.0);
        ui.label(texto(rotulo.to_uppercase(), MEDIO).size(10.0));
        ui.label(texto(titulo.to_uppercase(), TINTA).size(15.0).strong());
        ui.add_space(8.0);
        let dica = if acao.is_some() { tr("O que fazer nesta ação, passo a passo: o roteiro, as decisões, as dependências e os riscos…") } else { tr("Anotações do blueprint…") };
        ui.add(egui::TextEdit::multiline(&mut vista.nota_texto).desired_rows(10).desired_width(f32::INFINITY).hint_text(dica));
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            salvar = ui.add_enabled(!ocupado, egui::Button::new(texto(tr("Salvar"), TINTA))).clicked();
            fechar = ui.button(texto(tr("Fechar"), MEDIO)).clicked();
        });
    });
    let esc = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    if salvar || fechar || esc {
        vista.nota = None;
    }
    salvar.then(|| Acao::SalvarObs(ws, bp, acao, std::mem::take(&mut vista.nota_texto)))
}
