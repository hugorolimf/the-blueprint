//! O blueprint aberto: a folha `n` do workspace, com a régua de meses e as seções (`folhaBlueprint` em web/js/render.js). Sem projetos há uma
//! seção, a do blueprint, que nasce aberta; com projetos há uma seção por projeto (e o Geral), que nascem recolhidas.

use super::cena::{Ancora, Cena};
use super::disposicao::{self, FOLGA};
use super::pagina::{self, corta, Carimbo, Medidas, BAIXO_FAIXA, IL};
use super::regua::{self, Regua};
use super::secao::{self, Secao};
use crate::documento::{self, Documento, RAIZ};
use crate::dominio::{self, Acao as AcaoDom};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::planta::{self, Papel};
use crate::telas::{Acao, Tela, Vista};
use crate::tema::{MEDIO, TINTA};
use egui::{vec2, Sense, Ui};
use serde_json::Value;

const Y_REGUA: f32 = 204.0;
/// O vão entre duas seções.
const GAP_SECAO: f32 = 28.0;

fn texto(v: &Value, campo: &str) -> String {
    v.get(campo).and_then(Value::as_str).unwrap_or("").to_string()
}

/// O que cada seção mostra, antes de medir: a chave de aberto/fechado, o número, o nome e as ações.
struct Modelo {
    chave: String,
    numero: String,
    nome: String,
    descricao: String,
    idxs: Vec<usize>,
    principal: bool,
    aberto: bool,
}

fn data_curta(dias: i64) -> String {
    let iso = documento::iso_do_dia(dias);
    format!("{}/{}", &iso[8..10], &iso[5..7])
}

fn modelos(vista: &Vista, p: &Value, pi: usize, acoes: &[AcaoDom]) -> Vec<Modelo> {
    let bp = texto(p, "id");
    match dominio::grupos(p, acoes) {
        None => {
            let chave = format!("{bp}|");
            vec![Modelo { aberto: vista.abertas.get(&chave).copied().unwrap_or(true), chave, numero: format!("{:02}", pi + 1), nome: texto(p, "nome"), descricao: texto(p, "descricao"), idxs: (0..acoes.len()).collect(), principal: true }]
        }
        Some(gs) => gs.into_iter().enumerate().map(|(k, g)| {
            let chave = format!("{bp}|{}", g.id);
            let (ini, fim) = (g.acoes.iter().map(|i| acoes[*i].a).min(), g.acoes.iter().map(|i| acoes[*i].b).max());
            let descricao = if !g.descricao.is_empty() { g.descricao.clone() } else if let (Some(a), Some(b)) = (ini, fim) { format!("{} → {}", data_curta(a), data_curta(b)) } else { String::new() };
            Modelo { aberto: vista.abertas.get(&chave).copied().unwrap_or(false), chave, numero: format!("{:02}", k + 1), nome: if g.id.is_empty() { tr("Geral").to_string() } else { g.nome }, descricao, idxs: g.acoes, principal: false }
        }).collect(),
    }
}

pub fn tela(ui: &mut Ui, vista: &mut Vista, doc: &Documento, id: &str, bp: &str, servidor: &str) -> Option<Acao> {
    let Some(ws) = doc.workspace(id) else {
        vista.tela = Tela::Entrada;
        return None;
    };
    let blueprints = ws.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let Some((pi, p)) = blueprints.iter().enumerate().find(|(_, b)| texto(b, "id") == bp) else {
        vista.tela = Tela::Workspace(id.to_string());
        return None;
    };
    let empresa = if id == RAIZ { texto(ws, "empresa") } else { texto(ws, "nome") };
    let hoje_iso = documento::hoje();
    let hoje = documento::dia(&hoje_iso).unwrap_or(0);
    let acoes = dominio::acoes_de(p);
    let janela = dominio::janela(&acoes, hoje);
    let nm = (janela.fim - janela.inicio + 1) as usize;
    let modelos = modelos(vista, p, pi, &acoes);
    let (viewport_h, viewport_w) = (ui.available_height(), ui.available_width());
    let (mut pedido, mut alternar, mut comando) = (None, None::<String>, None);
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    if let Some(y) = vista.rolagem.take() {
        area = area.vertical_scroll_offset(y);
    }
    area.show(ui, |ui| {
        let w = viewport_w.max(960.0);
        let (l, r) = (IL + 34.0, w - IL - 50.0);
        let colw = (((r - l - 30.0) / 2.0).floor()).clamp(240.0, 430.0);
        let y0 = Y_REGUA + 90.0;
        // mede cada seção e empilha: a próxima começa depois da altura da anterior (sem a folga) e do vão
        let subconjuntos: Vec<Vec<&AcaoDom>> = modelos.iter().map(|m| m.idxs.iter().map(|k| &acoes[*k]).collect()).collect();
        let mut disps = Vec::new();
        let mut ys = Vec::new();
        let mut y = y0;
        for (m, refs) in modelos.iter().zip(&subconjuntos) {
            let d = disposicao::dispor(refs, m.aberto, |f| dominio::numero_da_acao(p, f, pi, &acoes), &janela, l, r, colw);
            ys.push(y);
            y += d.alto - FOLGA + GAP_SECAO;
            disps.push(d);
        }
        let alto = if modelos.is_empty() { disposicao::PH + 36.0 + FOLGA } else { y - y0 - GAP_SECAO + FOLGA };
        let h = viewport_h.max(560.0).max(y0 + alto + 40.0 + BAIXO_FAIXA + IL);
        let (pagina_r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina_r, Papel::Entrada);
        let c = Cena { p: &painter, o: pagina_r.min };
        let regua = Regua { janela: &janela, l, r, y: Y_REGUA, fundo: h - IL - BAIXO_FAIXA - 20.0, hoje };
        let medidas = Medidas { w, h, l, r, meses: (0..=nm).map(|m| regua.mes(m)).collect() };
        pagina::moldura(&c, &medidas);
        let titulo = format!("{empresa} · {}", texto(ws, "titulo"));
        c.texto(&titulo, l, IL + 36.0, 18.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
        let sub = tr_com("Folha {n} de {total} · Blueprint {bp} · {nome}", &[("n", &(pi + 2).to_string()), ("total", &(1 + blueprints.len()).to_string()), ("bp", &format!("{:02}", pi + 1)), ("nome", &corta(&texto(p, "nome"), 60))]);
        c.texto(&sub, l, IL + 56.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        regua::desenhar(&c, &regua);
        for (k, m) in modelos.iter().enumerate() {
            let refs = &subconjuntos[k];
            let total = if m.principal { p.get("acoes").and_then(Value::as_array).map_or(0, Vec::len) } else { refs.len() };
            let s = Secao { ws: id, p, acoes: refs, numero: m.numero.clone(), nome: m.nome.clone(), descricao: m.descricao.clone(), total, ok: m.principal && dominio::concluido(p), aberto: m.aberto, principal: m.principal, resumo: dominio::resumo(refs.iter().copied()), disp: &disps[k], regua: &regua, y: ys[k], colw, r, l };
            if secao::cabecalho(&c, ui, &s) {
                alternar = Some(m.chave.clone());
            }
            if let Some(a) = secao::acoes(&c, ui, &s, hoje) {
                pedido = Some(a);
            }
        }
        let (feitas, total) = (acoes.iter().filter(|f| f.status == dominio::Situacao::Concluido).count(), acoes.len());
        let data = hoje_iso.split('-').rev().collect::<Vec<_>>().join("/");
        let nome = texto(p, "nome");
        let dica = tr("Marque planos e checkpoints clicando neles · passe o mouse sobre uma ação para ver os detalhes.");
        pagina::rodape(&c, &medidas, &dica, &Carimbo { empresa: &empresa, tela: if nome.is_empty() { tr("Sem nome") } else { &nome }, data: &data, progresso: dominio::progresso(p), folha: pi + 2, folhas: 1 + blueprints.len(), legenda: false, feitas, acoes: total });
        // a barra cheia só cabe se sobrar espaço depois do título e do subtítulo (24 px de respiro)
        let titulo_fim = l + crate::planta::Texto::novo(titulo.to_uppercase(), 18.0).peso(Peso::Negrito).trilha(0.06).largura(&painter);
        let sub_fim = l + crate::planta::Texto::novo(sub.to_uppercase(), 11.0).trilha(0.06).largura(&painter);
        let mut abrir_web = None;
        comando = super::controles::blueprint(ui, vista, &c, w, id, titulo_fim.max(sub_fim) + 24.0 - c.o.x, &mut abrir_web);
        if let Some(destino) = abrir_web {
            ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/{destino}")));
        }
    });
    if let Some(chave) = alternar {
        let aberto = modelos.iter().find(|m| m.chave == chave).is_some_and(|m| m.aberto);
        vista.abertas.insert(chave, !aberto);
    }
    // «expandir tudo» e «recolher tudo»
    if let Some(c) = comando {
        for m in &modelos {
            vista.abertas.insert(m.chave.clone(), c == "expandir");
        }
    }
    pedido
}
