//! Um workspace aberto. A folha do blueprint (o desenho, as ações, os planos) ainda é da versão web: aqui fica o cabeçalho, a lista e a
//! ponte para abrir lá, até a folha ser migrada.

use super::{Acao, Tela, Vista};
use crate::documento::{Documento, RAIZ};
use crate::fonte::Peso;
use crate::i18n::{tr, trn};
use crate::planta::{self, botao, Botao, Papel, Texto};
use crate::tema::{BAIXO, MEDIO};
use egui::{pos2, vec2, Sense, Ui};

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, id: &str, servidor: &str) -> Option<Acao> {
    let Some(ws) = doc.workspace(id) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let pagina = ui.available_rect_before_wrap();
    ui.allocate_rect(pagina, Sense::hover());
    let p = ui.painter().clone();
    planta::papel(&p, pagina, Papel::Entrada);
    let (x, y) = (pagina.min.x + 64.0, pagina.min.y + 44.0);
    if botao(ui, pos2(x, y), &Botao::entrada(&format!("← {}", tr("Workspaces")))).clicked() {
        vista.tela = Tela::Entrada;
    }
    let nome = ws.get(if id == RAIZ { "empresa" } else { "nome" }).and_then(|v| v.as_str()).unwrap_or("");
    Texto::novo(nome, 42.0).peso(Peso::Medio).trilha(-0.04).na_linha(&p, x, y + 70.0, 67.2);
    let mut topo = y + 150.0;
    if let Some(d) = ws.get("descricao").and_then(|v| v.as_str()).filter(|d| !d.is_empty()) {
        topo += Texto::novo(d, 14.0).cor(MEDIO).lh(22.4).caixa(pagina.width() - 128.0).em(&p, pos2(x, topo)).height() + 16.0;
    }
    let blueprints = ws.get("blueprints").and_then(|v| v.as_array()).map_or(&[][..], Vec::as_slice);
    if blueprints.is_empty() {
        Texto::novo(tr("sem blueprints"), 14.0).cor(BAIXO).na_linha(&p, x, topo, 22.4);
        topo += 30.0;
    }
    for b in blueprints {
        let acoes = b.get("acoes").and_then(|v| v.as_array()).map_or(0, Vec::len);
        let nome = b.get("nome").and_then(|v| v.as_str()).unwrap_or("");
        Texto::novo(format!("{nome}  ·  {}", trn(acoes, "{n} ação", "{n} ações")), 14.0).na_linha(&p, x, topo, 22.4);
        topo += 26.0;
    }
    topo += 24.0;
    Texto::novo(tr("A folha do blueprint, as ações e os planos ainda abrem na versão web."), 12.0).cor(MEDIO).na_linha(&p, x, topo, 19.2);
    let alvo = if id == RAIZ { "#/c/sm".to_string() } else { format!("#/c/{id}") };
    if botao(ui, pos2(x, topo + 30.0), &Botao::entrada(tr("Abrir na versão web"))).clicked() {
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/{alvo}")));
    }
    let _ = vec2(0.0, 0.0);
    None
}
