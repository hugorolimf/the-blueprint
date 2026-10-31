//! O blueprint aberto: a folha `n` do workspace, com a régua de meses, a seção do blueprint e as ações com planos e checkpoints
//! (`folhaBlueprint` em web/js/render.js). Só o caso sem projetos e com a seção aberta; blueprint com projetos abre a versão web.

use super::cena::{Ancora, Cena};
use super::disposicao;
use super::pagina::{self, corta, Carimbo, Medidas, BAIXO_FAIXA, IL};
use super::regua::{self, Regua};
use super::secao::{self, Secao};
use crate::documento::{self, Documento, RAIZ};
use crate::dominio;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::planta::{self, Papel};
use crate::telas::{Acao, Tela, Vista};
use crate::tema::{MEDIO, TINTA};
use egui::{vec2, Sense, Ui};
use serde_json::Value;

const Y_REGUA: f32 = 204.0;

fn texto(v: &Value, campo: &str) -> String {
    v.get(campo).and_then(Value::as_str).unwrap_or("").to_string()
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
    let (viewport_h, viewport_w) = (ui.available_height(), ui.available_width());
    let mut pedido = None;
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    if let Some(y) = vista.rolagem.take() {
        area = area.vertical_scroll_offset(y);
    }
    area.show(ui, |ui| {
        let w = viewport_w.max(960.0);
        let (l, r) = (IL + 34.0, w - IL - 50.0);
        let colw = (((r - l - 30.0) / 2.0).floor()).clamp(240.0, 430.0);
        let y0 = Y_REGUA + 90.0;
        let numero = |f: &dominio::Acao| format!("{}.{}", pi + 1, f.idx + 1);
        let disp = disposicao::dispor(&acoes, numero, &janela, l, r, colw);
        let h = viewport_h.max(560.0).max(y0 + disp.alto + 40.0 + BAIXO_FAIXA + IL);
        let (pagina_r, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let painter = ui.painter().clone();
        planta::papel(&painter, pagina_r, Papel::Entrada);
        let c = Cena { p: &painter, o: pagina_r.min };
        let regua = Regua { janela: &janela, l, r, y: Y_REGUA, fundo: h - IL - BAIXO_FAIXA - 20.0, hoje };
        let medidas = Medidas { w, h, l, r, meses: (0..=nm).map(|m| regua.mes(m)).collect() };
        pagina::moldura(&c, &medidas);
        c.texto(&format!("{empresa} · {}", texto(ws, "titulo")), l, IL + 36.0, 18.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
        let sub = tr_com("Folha {n} de {total} · Blueprint {bp} · {nome}", &[("n", &(pi + 2).to_string()), ("total", &(1 + blueprints.len()).to_string()), ("bp", &format!("{:02}", pi + 1)), ("nome", &corta(&texto(p, "nome"), 60))]);
        c.texto(&sub, l, IL + 56.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        regua::desenhar(&c, &regua);
        let secao = Secao { ws: id, p, pi, acoes: &acoes, disp: &disp, regua: &regua, y: y0, colw, r, l };
        secao::cabecalho(&c, ui, &secao);
        pedido = secao::acoes(&c, ui, &secao, hoje);
        let (feitas, total) = (acoes.iter().filter(|f| f.status == dominio::Situacao::Concluido).count(), acoes.len());
        let data = hoje_iso.split('-').rev().collect::<Vec<_>>().join("/");
        let nome = texto(p, "nome");
        let dica = tr("Marque planos e checkpoints clicando neles · passe o mouse sobre uma ação para ver os detalhes.");
        pagina::rodape(&c, &medidas, &dica, &Carimbo { empresa: &empresa, tela: if nome.is_empty() { tr("Sem nome") } else { &nome }, data: &data, progresso: dominio::progresso(p), folha: pi + 2, folhas: 1 + blueprints.len(), legenda: false, feitas, acoes: total });
        // a barra cheia só cabe se sobrar espaço depois do título e do subtítulo (24 px de respiro)
        let titulo_fim = l + crate::planta::Texto::novo(format!("{empresa} · {}", texto(ws, "titulo")).to_uppercase(), 18.0).peso(Peso::Negrito).trilha(0.06).largura(&painter);
        let sub_fim = l + crate::planta::Texto::novo(sub.to_uppercase(), 11.0).trilha(0.06).largura(&painter);
        let mut abrir_web = None;
        super::controles::blueprint(ui, vista, &c, w, id, titulo_fim.max(sub_fim) + 24.0 - c.o.x, &mut abrir_web);
        if let Some(destino) = abrir_web {
            ui.ctx().open_url(egui::OpenUrl::new_tab(format!("{servidor}/{destino}")));
        }
    });
    pedido
}

