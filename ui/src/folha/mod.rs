//! A folha do workspace: a lista de blueprints, desenhada como a prancha de plotagem da página (`capa` em web/js/render.js), em coordenadas
//! do SVG dela. O blueprint aberto (a linha do tempo das ações) ainda é da versão web: o clique no cartão abre lá.

mod acoes;
mod barra;
mod blueprint;
mod cartao;
mod cena;
pub(crate) mod claude;
mod controles;
mod disposicao;
mod figuras;
mod movimento;
mod pagina;
mod regua;
mod secao;

pub use blueprint::tela as blueprint_aberto;

use crate::documento::{self, Documento, RAIZ};
use crate::dominio;
use crate::emblema;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::pesquisa;
use crate::planta::{self, Papel};
use crate::telas::{Acao, Tela, Vista};
use crate::tema::{rgba, MEDIO, TINTA};
use cena::{Ancora, Cena};
use egui::{vec2, Rect, Sense, Ui};
use pagina::{corta, Carimbo, Medidas, IL};

const TH: f32 = 152.0;
const GAP: f32 = 24.0;
const Y_MAPA: f32 = 128.0;

fn texto(v: &serde_json::Value, campo: &str) -> String {
    v.get(campo).and_then(|v| v.as_str()).unwrap_or("").to_string()
}

pub fn lista(ui: &mut Ui, vista: &mut Vista, doc: &Documento, id: &str, servidor: &str) -> Option<Acao> {
    let Some(ws) = doc.workspace(id) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let blueprints = ws.get("blueprints").and_then(|v| v.as_array()).map_or(&[][..], Vec::as_slice);
    let (empresa, titulo) = (if id == RAIZ { texto(ws, "empresa") } else { texto(ws, "nome") }, texto(ws, "titulo"));
    let hoje_iso = documento::hoje();
    let hoje = documento::dia(&hoje_iso).unwrap_or(0);
    let todas: Vec<dominio::Acao> = blueprints.iter().flat_map(dominio::acoes_de).collect();
    let janela = dominio::janela(&todas, hoje);
    let nm = (janela.fim - janela.inicio + 1) as usize;
    let termos = pesquisa::termos(&vista.consulta);
    // com consulta, só os blueprints que casam, cada um com o texto de onde casou (`pesqLista`)
    let achados: Vec<(&serde_json::Value, Option<String>)> = blueprints.iter().filter_map(|p| {
        if termos.is_empty() {
            return Some((p, None));
        }
        let r = pesquisa::blueprint(p, &termos, None);
        if !r.casa() {
            return None;
        }
        let mut partes = Vec::new();
        if r.proprio && r.total() == 0 {
            partes.push(tr("nome ou descrição").to_string());
        }
        for (n, um, varios) in [(r.n_acoes, "{n} ação", "{n} ações"), (r.n_planos, "{n} plano", "{n} planos"), (r.n_checkpoints, "{n} checkpoint", "{n} checkpoints")] {
            if n > 0 {
                partes.push(trn(n, um, varios));
            }
        }
        if r.proprio && r.total() > 0 {
            partes.insert(0, tr("nome ou descrição").to_string());
        }
        Some((p, Some(tr_com("ACHOU: {onde}", &[("onde", &partes.join(" · ").to_uppercase())]))))
    }).collect();
    let mapa: Vec<&serde_json::Value> = achados.iter().map(|(p, _)| *p).collect();

    let (viewport_h, viewport_w) = (ui.available_height(), ui.available_width());
    let mut acao = None;
    let mut abrir_web: Option<String> = None;
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    if let Some(y) = vista.rolagem.take() {
        area = area.vertical_scroll_offset(y);
    }
    area.show(ui, |ui| {
        let w = viewport_w.max(960.0);
        let (l, r) = (IL + 34.0, w - IL - 50.0);
        let min_cartao = if mapa.len() > 1 { 400.0 } else { 320.0 };
        let colunas = (((r - l + GAP) / (min_cartao + GAP)).floor() as usize).clamp(1, 3);
        let tw = (r - l - (colunas - 1) as f32 * GAP) / colunas as f32;
        let map_end = Y_MAPA + 48.0 + ((mapa.len() + 1).div_ceil(colunas)) as f32 * (TH + GAP) - GAP;
        let minima = viewport_h.max(560.0);
        let h = minima.max(map_end + 150.0 + pagina::BAIXO_FAIXA + IL);
        let (pagina_r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let p = ui.painter().clone();
        planta::papel(&p, pagina_r, Papel::Entrada);
        let c = Cena { p: &p, o: pagina_r.min };
        let x_mes = |m: usize| {
            let t = documento::dia(&format!("{}-01", dominio::mes_iso(janela.inicio + m as i64))).unwrap_or(0);
            l + (t - janela.t0) as f32 / (janela.t1 - janela.t0).max(1) as f32 * (r - l)
        };
        let medidas = Medidas { w, h, l, r, meses: (0..=nm).map(x_mes).collect() };
        pagina::moldura(&c, &medidas);
        // a marca do workspace atrás de tudo, «construída» conforme o progresso médio dos blueprints
        let comacoes: Vec<&serde_json::Value> = blueprints.iter().filter(|p| p.get("acoes").and_then(|v| v.as_array()).is_some_and(|a| !a.is_empty())).collect();
        let geral = if comacoes.is_empty() { 0.0 } else { comacoes.iter().map(|p| dominio::resumo(&dominio::acoes_de(p)).geral).sum::<f32>() / comacoes.len() as f32 };
        let lado = (w * 0.55).min(minima * 0.55).min(minima).round();
        let larg = (lado * 1.35).round();
        let marca = Rect::from_min_size(c.pt(w / 2.0 - larg / 2.0, h / 2.0 - lado / 2.0), vec2(larg, lado - 18.0));
        let icone = texto(ws, "icone");
        emblema::pintar_fundo(ui, marca, if icone.is_empty() { "blueprint" } else { &icone }, geral);
        let legenda = tr_com("{pct}% CONSTRUÍDA", &[("pct", &((geral * 100.0).round() as i32).to_string())]);
        c.texto_apagado(&legenda, w / 2.0, h / 2.0 + lado / 2.0 - 3.0, 10.0);
        c.texto(&format!("{empresa} · {titulo}"), l, IL + 36.0, 18.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
        c.texto(&tr("Blueprints · cada um abre o seu próprio blueprint"), l, IL + 56.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.texto(&tr("Clique para abrir o blueprint · segure e arraste para reordenar"), l, Y_MAPA + 6.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        let ancoras: claude::Ancoras = Default::default();
        for (i, p) in mapa.iter().enumerate() {
            let (col, linha) = (i % colunas, i / colunas);
            let m = cartao::Medidas { x: l + col as f32 * (tw + GAP), y: Y_MAPA + 48.0 + linha as f32 * (TH + GAP), w: tw, h: TH };
            ancoras.borrow_mut().insert(crate::agentes::Lugar::Cab(texto(p, "id")), (m.x, m.y, m.w, m.h));
            let area = Rect::from_min_size(c.pt(m.x, m.y), vec2(m.w, m.h));
            let resposta = ui.interact(area, ui.scope_id().with(("bp", texto(p, "id"))), Sense::click());
            let onde = achados.get(i).and_then(|(_, o)| o.as_deref());
            let arte = cartao::desenhar(&c, &m, p, hoje, resposta.hovered() || resposta.has_focus(), onde.map(|o| (o, termos.as_slice())));
            emblema::pintar_construcao(ui, Rect::from_min_size(c.pt(arte.x, arte.y), vec2(arte.w, arte.h)), arte.progresso);
            if resposta.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resposta.clicked() {
                vista.tela = Tela::Blueprint(id.to_string(), texto(p, "id"));
            }
        }
        // o Claude: uma figura por agente no canto de cima do cartão do blueprint em que ele trabalha
        let ags = crate::agentes::ativos(&vista.agentes, id, crate::agentes::agora_ms(), vista.agentes_vida, &vista.vistos);
        for p in mapa.iter() {
            let bp = texto(p, "id");
            let deste: Vec<_> = ags.iter().filter(|a| a.blueprint_id == bp).cloned().collect();
            if let Some(n) = claude::desenhar(&c, ui, &bp, "lista", &deste, &ancoras, "cartao") {
                vista.agente_aberto = Some(n);
            }
        }
        // «novo blueprint»: o cartão tracejado depois do último
        let (col, linha) = (mapa.len() % colunas, mapa.len() / colunas);
        let (nx, ny) = (l + col as f32 * (tw + GAP), Y_MAPA + 48.0 + linha as f32 * (TH + GAP));
        let area = Rect::from_min_size(c.pt(nx, ny), vec2(tw, TH));
        let resposta = ui.interact(area, ui.scope_id().with("novo-bp"), Sense::click());
        let realce = resposta.hovered() || resposta.has_focus();
        c.retangulo(nx, ny, tw, TH, Some(if realce { rgba(0x1d4880, 0x66) } else { rgba(0x113360, 0x40) }), None);
        c.moldura(nx, ny, tw, TH, if realce { TINTA } else { crate::tema::BAIXO }, 1.0, true);
        let (art_w, art_x) = ((tw * 0.24).round().clamp(76.0, 108.0), nx + 12.0);
        emblema::pintar_construcao(ui, Rect::from_min_size(c.pt(art_x, ny + 26.0), vec2(art_w, TH - 52.0)), 1.0);
        let (texto_x, mais_x) = (art_x + art_w + 14.0, nx + tw - 56.0);
        let texto_w = mais_x - 12.0 - texto_x;
        c.texto(&corta(&tr("NOVO BLUEPRINT"), (texto_w / 7.2).floor() as usize), texto_x, ny + 68.0, 12.0, Peso::Normal, TINTA, Ancora::Inicio, true);
        c.texto(&corta(&tr("ADICIONAR BLUEPRINT"), (texto_w / 6.0).floor() as usize), texto_x, ny + 94.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.retangulo(mais_x, ny + TH / 2.0 - 18.0, 36.0, 36.0, Some(if realce { rgba(0x1d4880, 0xff) } else { rgba(0x113360, 0x40) }), Some((if realce { TINTA } else { crate::tema::BAIXO }, 1.0)));
        let cm = ny + TH / 2.0;
        c.fio(MEDIO, 2.0, (mais_x + 18.0, cm - 9.0), (mais_x + 18.0, cm + 9.0));
        c.fio(MEDIO, 2.0, (mais_x + 9.0, cm), (mais_x + 27.0, cm));
        if resposta.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if resposta.clicked() {
            vista.novo_bp = Some(crate::telas::NovoBp::em(id));
        }
        // o rodapé
        let (feitas, n_acoes) = (todas.iter().filter(|f| f.status == dominio::Situacao::Concluido).count(), todas.len());
        let progresso = if blueprints.is_empty() { 0.0 } else { blueprints.iter().map(dominio::progresso).sum::<f32>() / blueprints.len() as f32 };
        let data = hoje_iso.split('-').rev().collect::<Vec<_>>().join("/");
        pagina::rodape(&c, &medidas, &tr("Clique num blueprint para abrir o blueprint dele."), &Carimbo { empresa: &empresa, tela: &tr("Lista de blueprints"), data: &data, progresso, folha: 1, folhas: 1 + blueprints.len(), legenda: true, feitas, acoes: n_acoes });
        let titulo_fim = l + planta::Texto::novo(format!("{empresa} · {titulo}").to_uppercase(), 18.0).peso(Peso::Negrito).trilha(0.06).largura(&p);
        let sub_fim = l + planta::Texto::novo(tr("Blueprints · cada um abre o seu próprio blueprint").to_uppercase(), 11.0).trilha(0.06).largura(&p);
        controles::lista(ui, vista, &c, w, id, titulo_fim.max(sub_fim) + 24.0 - c.o.x, mapa.len(), !termos.is_empty(), &mut abrir_web);
    });
    if let Some(destino) = abrir_web {
        ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/{destino}")));
    }
    acao.take()
}
