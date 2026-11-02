//! A tela de agentes (`design/planejar/tela-agentes.mjs`): o cartão do orquestrador, os indicadores e o limite, os avisos e um cartão por plano, de todos os blueprints do workspace.
//! Quem calcula é o servidor (`planejar/agentes`): aqui só se escolhe a figura e o que cada linha oferece, e os gestos saem depois do desenho. Cada peça fica no seu arquivo (`mod` abaixo).

use super::agentes_movimento as mov;
use super::escala::{MARGEM, X0};
use super::linha::Pincel;
use super::modelo::Agentes;
use super::orquestrador::{self, Pedido};
use super::Sub;
use crate::documento::{Documento, RAIZ};
use crate::i18n::{tr, tr_com};
use crate::menu::{self, Alvo};
use crate::planta::{self, Papel};
use crate::telas::{Acao, Tela, Vista};
use egui::{vec2, Sense, Ui};
use serde_json::Value;

// a moldura, a legenda, o selo e os botões são os da folha; lá o módulo é privado de `folha.rs`, então entra aqui pelo mesmo arquivo, como em `detalhe.rs`
#[path = "folha_topo.rs"]
#[allow(dead_code)]
mod cabeca;
use cabeca::{botoes, legenda, moldura, selo, Topo};

#[path = "agentes_regras.rs"]
mod regras;
pub use regras::*;

#[path = "agentes_aviso.rs"]
mod aviso;
#[path = "agentes_cartoes.rs"]
mod cartoes;
#[path = "agentes_conversa.rs"]
mod conversa;
#[path = "agentes_demo.rs"]
pub mod demo;
#[path = "agentes_figuras.rs"]
pub mod figuras;
#[path = "agentes_linha.rs"]
mod linha;
#[path = "agentes_orq.rs"]
mod orq;
#[path = "agentes_tabela.rs"]
mod tabela;
#[path = "agentes_vazio.rs"]
mod vazio;

pub const TEXTOS: &[(&str, &str)] = &[
    ("Não iniciado", "Not started"),
    ("Revise a mudança", "Review the change"),
    ("Trabalhando", "Working"),
    ("Na fila", "In queue"),
    ("Esperando plano", "Waiting for a plan"),
    ("Esperando área", "Waiting for the area"),
    ("Parado", "Stopped"),
    ("Pediu ajuda", "Asked for help"),
    ("O agente sumiu", "The agent disappeared"),
    ("O agente falhou", "The agent failed"),
    ("Pronto", "Done"),
    (" · sem sinal", " · no signal"),
    ("Sem sinal do agente há mais de 2 minutos.", "No signal from the agent for over 2 minutes."),
    ("1 subagente trabalhando.", "1 subagent working."),
    ("{n} subagentes trabalhando.", "{n} subagents working."),
    ("Espera {por}.", "Waiting for {por}."),
    ("Terminou sem concluir o plano.", "It ended without completing the plan."),
    ("Os ramos de que este plano parte conflitam entre si.", "The branches this plan starts from conflict with each other."),
    ("O repositório ainda não tem nenhum commit.", "The repository has no commits yet."),
    ("O plano saiu do blueprint.", "The plan was removed from the blueprint."),
    ("Integrar", "Integrate"),
    ("Retomar", "Resume"),
    ("Cancelar", "Cancel"),
    ("Juntar {ramo} em {destino}.", "Merge {ramo} into {destino}."),
    ("A pasta principal não está em nenhum ramo.", "The main folder is not on any branch."),
    ("Plano", "Plan"),
    ("Situação", "Status"),
    ("Depende de", "Depends on"),
    ("Área", "Area"),
    ("Ramo do agente", "Agent branch"),
    ("Tempo", "Time"),
    ("trabalhando", "working"),
    ("na fila", "in the queue"),
    ("esperando", "waiting"),
    ("parados", "stopped"),
    ("prontos", "done"),
    ("Agentes ao mesmo tempo", "Agents at the same time"),
    ("Avisos · {n}", "Warnings · {n}"),
    ("{n} de {total} prontos", "{n} of {total} done"),
    ("nesta janela", "in this window"),
    ("Contado desde que esta janela viu o plano nesta situação.", "Counted since this window saw the plan in this status."),
    ("agora", "now"),
    ("{n} min", "{n} min"),
    ("{h} h {m} min", "{h} h {m} min"),
    ("{n} d", "{n} d"),
    ("Nenhum agente por aqui", "No agents here"),
    ("Marque a caixinha de um plano na folha para chamar um agente.", "Check a plan's box on the sheet to call an agent."),
    ("Marque a caixinha de um plano na folha", "Check a plan's box on the sheet"),
    ("O orquestrador prepara o agente daquele plano", "The orchestrator prepares the agent for that plan"),
    ("O agente aparece aqui e começa a trabalhar", "The agent shows up here and starts working"),
    ("Abrir a folha", "Open the sheet"),
];

const LARGURA_MIN: f32 = 1120.0;
const ALTURA_MIN: f32 = 560.0;
const Y_FAIXA: f32 = 118.0;
const VAO: f32 = 14.0;

/// O que a tela pede depois de desenhar.
#[derive(Default)]
struct Saida {
    pedidos: Vec<Pedido>,
    topo: Option<Topo>,
    abrir_folha: bool,
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, ws: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let Some(w) = doc.workspace(ws) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    demo::aplicar(&mut vista.plano);
    demo::semear(ui.ctx(), ws);
    let rolagem = vista.rolagem.take();
    let saida = desenhar(ui, vista, w, ws, bp, rolagem);
    let ctx = ui.ctx().clone();
    for (rota, corpo) in saida.pedidos {
        vista.plano.gesto(&ctx, servidor, rota, &corpo);
    }
    match saida.topo {
        Some(Topo::Voltar) => vista.plano.tela = Sub::Folha,
        Some(Topo::Web) => ctx.open_url(egui::OpenUrl::new_tab(format!("{servidor}/#/c/{ws}"))),
        _ => {}
    }
    if saida.abrir_folha {
        vista.plano.tela = Sub::Folha;
    }
    None
}

fn desenhar(ui: &mut Ui, vista: &Vista, w: &Value, ws: &str, bp: &str, rolagem: Option<f32>) -> Saida {
    let (alto, largura) = (ui.available_height(), ui.available_width().max(LARGURA_MIN));
    let x1 = largura - MARGEM;
    let estado = &vista.plano;
    mov::abrir(ui.ctx(), "agentes");
    let nome_ws = w.get(if ws == RAIZ { "empresa" } else { "nome" }).and_then(Value::as_str).unwrap_or("");
    let nome_bp = estado.folha.as_ref().map_or("", |f| f.blueprint.nome.as_str());
    let vazio = Agentes::default();
    let a = estado.agentes.as_ref().unwrap_or(&vazio);
    let limite = estado.agentes.as_ref().map(|a| a.limite).or_else(|| estado.folha.as_ref().map(|f| f.limite));
    let grupos = agrupar(&a.planos, w, bp);
    let aberta = orquestrador::folhinha_aberta(ui.ctx());
    // medido antes de desenhar: a altura da página depende de tudo o que vem abaixo da faixa
    let y_cartoes = Y_FAIXA + orq::ALTURA + VAO;
    let y_folhinha = y_cartoes + cartoes::ALTURA + VAO;
    let y_avisos = y_folhinha + if aberta { orquestrador::ALTURA_CONFIG + VAO } else { 0.0 };
    let y_tabela = y_avisos + if a.avisos.is_empty() { 22.0 } else { aviso::altura(a.avisos.len()) + 30.0 };
    let fim = y_tabela + tabela::altura(&grupos);
    let mut saida = Saida::default();
    let mut area = egui::ScrollArea::both().id_salt("plano-agentes").auto_shrink([false, false]);
    if let Some(y) = rolagem {
        area = area.vertical_scroll_offset(y);
    }
    area.show(ui, |ui| {
        let h = alto.max(ALTURA_MIN).max(fim + 120.0);
        let (pagina, _) = ui.allocate_exact_size(vec2(largura, h), Sense::hover());
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina, Papel::Entrada);
        menu::marcar(ui, pagina, || Alvo::Folha { ws: ws.into(), bp: bp.into(), nome: nome_bp.to_string(), chaves: Vec::new(), aberto: false });
        let c = Pincel { p: &painter, o: pagina.min };
        moldura(&c, largura, h);
        aviso::titulo(ui, &c, nome_bp, nome_ws, &estado.aviso, x1);
        let trabalhando = a.numeros.trabalhando;
        saida.pedidos.extend(orq::faixa(ui, &c, &a.orquestrador, nome_ws, trabalhando, X0, Y_FAIXA, x1, ws, bp));
        saida.pedidos.extend(cartoes::faixa(ui, &c, a, estado.agentes.as_ref().and(limite), ws, X0, y_cartoes, x1));
        if aberta {
            saida.pedidos.extend(orquestrador::folhinha(ui, &c, estado.config.as_ref(), X0, y_folhinha, x1));
        }
        if !a.avisos.is_empty() {
            aviso::desenhar(ui, &c, &a.avisos, y_avisos, x1);
        }
        let (pedido, folha) = tabela::desenhar(ui, &c, &grupos, estado.agentes.is_none(), y_tabela, X0, x1);
        saida.pedidos.extend(pedido);
        saida.abrir_folha = folha;
        legenda(&c, h);
        selo(&c, x1, h, &[
            (tr("Workspace"), nome_ws.to_string(), 140.0),
            (tr("Blueprint"), nome_bp.to_string(), 210.0),
            (tr("Agentes"), limite.map_or_else(|| trabalhando.to_string(), |l| tr_com("{n} de {total}", &[("n", &trabalhando.to_string()), ("total", &l.to_string())])), 110.0),
        ]);
        let topo = [(Topo::Voltar, tr("Folha").to_string()), (Topo::Web, tr("Planejar").to_string()), (Topo::Web, tr("Compartilhar").to_string())];
        saida.topo = botoes(ui, &c, x1, &topo);
    });
    saida
}
