//! O Claude na folha, a parte de interação (`abreNotaRobo` e `#robo-agente` de web/js/robo.js): a folhinha que abre ao clicar na figura, com o que o agente faz agora
//! (só leitura: o app não manda trabalho a ninguém), e a faixa fixa embaixo com a última fala. «Concluído» dispensa o agente que terminou ou está parado.

use super::{texto, Tela, Vista};
use crate::agentes::{self, Agente};
use crate::documento::Documento;
use crate::i18n::tr;
use crate::tema::{ERRO, MEDIO, PAPEL, TINTA};
use egui::{Align2, Frame, Margin, Stroke};
use serde_json::Value;

fn estado_texto(a: &Agente) -> String {
    match agentes::estado(a) {
        Some("lendo") => tr("Lendo"),
        Some("escrevendo") => tr("Escrevendo"),
        Some("testando") => tr("Testando"),
        Some("aguardando") => tr("Aguardando"),
        Some("bloqueado") => tr("Bloqueado"),
        Some("concluido") => tr("Concluído"),
        Some("ocioso") => tr("Parado"),
        _ => tr("Trabalhando"),
    }
    .to_string()
}

/// «Blueprint › Ação › Plano» do que o agente faz, ou «neste workspace».
fn onde(doc: &Documento, a: &Agente) -> String {
    let nome = |v: Option<&Value>| v.and_then(|v| v.get("nome").or_else(|| v.get("texto"))).and_then(Value::as_str).map(str::to_string);
    let achar = |lista: Option<&Value>, id: &str| lista.and_then(Value::as_array).and_then(|l| l.iter().find(|x| x.get("id").and_then(Value::as_str) == Some(id)).cloned());
    let bp = doc.workspace(&a.workspace_id).and_then(|w| achar(w.get("blueprints"), &a.blueprint_id));
    let acao = a.acao_id.as_deref().and_then(|id| achar(bp.as_ref().and_then(|b| b.get("acoes")), id));
    let plano = a.plano_id.as_deref().and_then(|id| achar(acao.as_ref().and_then(|f| f.get("planos")), id));
    let partes: Vec<String> = [nome(bp.as_ref()), nome(acao.as_ref()), nome(plano.as_ref())].into_iter().flatten().collect();
    if partes.is_empty() { tr("neste workspace").to_string() } else { partes.join(" › ") }
}

fn moldura() -> Frame {
    Frame::NONE.fill(PAPEL).stroke(Stroke::new(2.0, TINTA)).inner_margin(Margin::symmetric(22, 18)).shadow(egui::Shadow { offset: [0, 20], blur: 56, spread: 0, color: egui::Color32::from_rgba_unmultiplied(2, 19, 46, 0xe6) })
}

/// A folhinha do agente clicado.
pub fn folhinha(ctx: &egui::Context, vista: &mut Vista, doc: &Documento) {
    let Some(nome) = vista.agente_aberto.clone() else { return };
    let ativos = agentes::ativos(&vista.agentes, &vista.agente_ws(), agentes::agora_ms(), vista.agentes_vida, &vista.vistos);
    let Some(a) = ativos.iter().find(|a| a.agente == nome).cloned() else {
        vista.agente_aberto = None;
        return;
    };
    let concluiu = a.estado.as_deref() == Some("concluido");
    let sub = a.papel.is_some() && agentes::principal(&ativos).is_some_and(|p| ativos[p].agente != a.agente);
    let parado = matches!(a.estado.as_deref(), Some("aguardando" | "ocioso"));
    let papel = match a.papel.as_deref() {
        Some("explorador") => " · Explorador",
        Some("construtor") => " · Construtor",
        Some("revisor") => " · Revisor",
        _ => "",
    };
    let rotulo = if concluiu { if sub { tr("Subagente terminou uma tarefa") } else { tr("Agente concluiu") } } else { tr("Agente trabalhando") };
    let mut fechar = false;
    let mut visto = false;
    egui::Window::new("agente-folhinha").title_bar(false).collapsible(false).resizable(false).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).frame(moldura()).show(ctx, |ui| {
        ui.set_width(430.0);
        ui.label(texto(rotulo.to_uppercase(), MEDIO).size(10.0));
        ui.label(texto(a.agente.to_uppercase(), TINTA).size(15.0).strong());
        ui.add_space(8.0);
        ui.label(texto(format!("{}{papel}", estado_texto(&a)), TINTA));
        ui.label(texto(format!("{}: {}", if concluiu { if sub { tr("Última tarefa") } else { tr("Mensagem de conclusão") } } else { tr("Agora") }, a.mensagem.as_deref().unwrap_or("")), TINTA));
        ui.label(texto(onde(doc, &a), MEDIO));
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if (concluiu && !sub || parado) && ui.button(texto(tr("Concluído"), TINTA)).clicked() {
                visto = true;
            }
            if ui.button(texto(tr("Fechar"), MEDIO)).clicked() {
                fechar = true;
            }
        });
    });
    if visto {
        vista.vistos.insert(agentes::chave_visto(&a));
    }
    if visto || fechar || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        vista.agente_aberto = None;
    }
}

/// A faixa fixa embaixo, à esquerda: o último agente que avisou neste workspace e o que ele disse. Clicar leva ao blueprint dele.
pub fn faixa(ctx: &egui::Context, vista: &mut Vista, doc: &Documento) {
    let ativos = agentes::ativos(&vista.agentes, &vista.agente_ws(), agentes::agora_ms(), vista.agentes_vida, &vista.vistos);
    let Some(a) = ativos.iter().filter(|a| a.estado.as_deref() != Some("ocioso")).max_by_key(|a| agentes::em_ms(&a.em)) else { return };
    let erro = agentes::estado(a) == Some("bloqueado");
    let titulo = if erro { tr("Agente travou") } else if agentes::estado(a) == Some("concluido") { tr("Agente concluiu") } else { tr("Agente trabalhando") };
    let cor = if erro { ERRO } else { TINTA };
    let ir = egui::Area::new(faixa_id()).anchor(Align2::LEFT_BOTTOM, [16.0, -16.0]).order(egui::Order::Foreground).show(ctx, |ui| {
        Frame::NONE.fill(egui::Color32::from_rgba_unmultiplied(0x08, 0x2b, 0x5e, 0xf5)).stroke(Stroke::new(1.0, cor)).inner_margin(Margin::symmetric(14, 8)).show(ui, |ui| {
            ui.set_max_width(560.0);
            ui.add(egui::Label::new(texto(format!("{} · {}", titulo.to_uppercase(), onde(doc, a).to_uppercase()), MEDIO).size(10.0)).truncate());
            ui.add(egui::Label::new(texto(a.mensagem.clone().unwrap_or_default(), cor).size(11.0)).truncate());
        });
    });
    if ctx.input(|i| i.pointer.primary_clicked()) && ir.response.rect.contains(ctx.input(|i| i.pointer.interact_pos()).unwrap_or_default()) {
        vista.tela = Tela::Blueprint(a.workspace_id.clone(), a.blueprint_id.clone());
    }
}

#[allow(deprecated)]
fn faixa_id() -> egui::Id {
    egui::Id::new("agente-faixa")
}
