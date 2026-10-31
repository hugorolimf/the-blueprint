//! A seção do blueprint aberto (`cabecalhoSecao` e `secao` em web/js): o cabeçalho com o número, o nome, o progresso e os links, e cada ação com
//! a barra, a cota de dias, as caixas dos planos e os losangos dos checkpoints (tudo marcável com o clique).

use super::cena::{Ancora, Cena};
use super::disposicao::{Disposicao, CHAR};
use super::pagina::corta;
use super::regua::Regua;
use crate::dominio::{self, Acao};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::pesquisa::ResBlueprint;
use crate::tema::{rgba, MEDIO, PAPEL, TINTA};
use egui::{vec2, Rect, Sense, Ui};
use serde_json::Value;

pub struct Secao<'a> {
    pub ws: &'a str,
    pub p: &'a Value,
    /// as ações desta seção (o blueprint inteiro, ou as de um projeto)
    pub acoes: &'a [&'a Acao],
    pub numero: String,
    pub nome: String,
    pub descricao: String,
    pub total: usize,
    pub ok: bool,
    pub aberto: bool,
    /// a seção do blueprint sem projetos tem a nota e os links; a de um projeto não
    pub principal: bool,
    pub resumo: dominio::Resumo,
    /// o filtro «Pendentes» está ligado (muda a mensagem de seção sem ações visíveis)
    pub pendentes: bool,
    /// a consulta e o que casou (só pesquisando): o que não casou esmaece
    pub pesq: Option<(&'a ResBlueprint, &'a [String])>,
    pub disp: &'a Disposicao,
    pub regua: &'a Regua<'a>,
    /// o topo da seção (`y0`)
    pub y: f32,
    pub colw: f32,
    pub r: f32,
    pub l: f32,
}

fn texto(v: &Value, campo: &str) -> String {
    v.get(campo).and_then(Value::as_str).unwrap_or("").to_string()
}

/// O cabeçalho: o círculo com o número, o nome, a descrição, o progresso geral, a linha, o «− Recolher»/«+ Abrir» e os links.
/// Devolve `true` se a pessoa clicou para abrir ou recolher a seção (o cabeçalho inteiro ou o botão).
pub fn cabecalho(c: &Cena, ui: &mut Ui, s: &Secao) -> bool {
    let (l, r, yb) = (s.l, s.r, s.y);
    let nome = corta(&if s.nome.is_empty() { tr("Sem nome").to_string() } else { s.nome.clone() }, 48);
    let descricao = corta(&s.descricao, 70);
    let obs = s.principal && !texto(s.p, "obs").trim().is_empty();
    let area = Rect::from_min_size(c.pt(l - 8.0, yb - 8.0), vec2(r - l + 16.0, 86.0));
    let resposta = ui.interact(area, ui.scope_id().with(("secao", s.numero.as_str())), Sense::click());
    if resposta.hovered() {
        c.retangulo(l - 8.0, yb - 8.0, r - l + 16.0, 86.0, Some(rgba(0xffffff, 0x14)), None);
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    c.p.circle_stroke(c.pt(l + 14.0, yb + 20.0), 14.0, egui::Stroke::new(1.0, TINTA));
    c.texto(&s.numero, l + 14.0, yb + 25.0, 13.0, Peso::Negrito, TINTA, Ancora::Meio, false);
    c.texto(&nome, l + 40.0, yb + 19.0, 17.0, Peso::Negrito, TINTA, Ancora::Inicio, true);
    c.texto(&descricao, l + 40.0, yb + 38.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    if obs {
        c.nota(l + 40.0 + nome.chars().count() as f32 * 11.6 + 10.0, yb + 3.0, false);
    }
    let (resumo, total, ok) = (&s.resumo, s.total, s.ok);
    let px = (l + 40.0 + (nome.chars().count() as f32 * 11.6 + if obs { 30.0 } else { 0.0 }).max(descricao.chars().count() as f32 * 7.2) + 64.0).min(r - 330.0);
    let horas = if resumo.horas > 0.0 { format!(" · {}", tr_com("{horas} h de desenvolvimento", &[("horas", &resumo.horas.to_string())])) } else { String::new() };
    let contagem = format!("{}{horas}", tr_com("{concl}/{total} ações concluídas", &[("concl", &resumo.concl.to_string()), ("total", &total.to_string())]));
    if total == 0 {
        c.texto(&tr("Situação"), px, yb + 14.0, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.texto(&tr("SEM AÇÕES AINDA"), px, yb + 32.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
    } else if ok || resumo.futuro {
        c.texto(&format!("{}{}", tr("Situação"), if ok { format!(" · {contagem}") } else { String::new() }), px, yb + 14.0, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.texto(&if ok { tr("✓ CONCLUÍDO · 100%") } else { tr("PLANEJADO") }, px, yb + 32.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
    } else {
        c.texto(&format!("{} · {contagem}", tr("Progresso geral")), px, yb + 14.0, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.moldura(px, yb + 21.0, 190.0, 10.0, TINTA, 1.0, false);
        c.retangulo(px, yb + 21.0, 190.0 * resumo.geral, 10.0, Some(TINTA.gamma_multiply(0.9)), None);
        c.texto(&format!("{}%", (resumo.geral * 100.0).round() as i32), px + 200.0, yb + 31.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
    }
    c.fio(MEDIO, 1.0, (l, yb + 58.0), (if total > 0 { r - 144.0 } else { r }, yb + 58.0));
    if total > 0 {
        c.retangulo(r - 136.0, yb + 44.0, 132.0, 30.0, Some(PAPEL), Some((TINTA, 1.2)));
        let rotulo = if s.aberto { tr("− Recolher").to_string() } else { format!("+ {}", trn(total, "Abrir {n} ação", "Abrir {n} ações")) };
        c.texto_liso(&rotulo.to_uppercase(), r - 70.0, yb + 63.0, 11.0, Peso::Normal, TINTA, Ancora::Meio);
    }
    if s.principal {
        links(c, ui, s, px);
    }
    resposta.clicked()
}

/// Os links do blueprint, em até duas linhas à direita (`↗ REPOSITÓRIO`); o que não cabe vira «+N LINKS».
fn links(c: &Cena, ui: &mut Ui, s: &Secao, px: f32) {
    let seguros: Vec<(String, String)> = s.p.get("links").and_then(Value::as_array).map_or(&[][..], Vec::as_slice).iter().filter_map(|l| {
        let url = texto(l, "url");
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return None;
        }
        let rotulo = [texto(l, "nome"), texto(l, "rotulo"), url.clone()].into_iter().find(|t| !t.is_empty()).unwrap_or_default();
        Some((url, rotulo))
    }).collect();
    let vaga = s.r - 6.0 - (px + 280.0);
    let (mut grupos, mut usado, mut g, mut livres): (Vec<Vec<(String, String, f32)>>, f32, usize, usize) = (vec![vec![], vec![]], 0.0, 0, 0);
    for (url, rotulo) in seguros {
        let txt = format!("↗ {}", corta(&rotulo, 18));
        let w = txt.chars().count() as f32 * CHAR + 12.0;
        if usado > 0.0 && usado + 8.0 + w > vaga {
            g += 1;
            usado = 0.0;
        }
        if g > 1 || w > vaga {
            livres += 1;
            continue;
        }
        grupos[g].push((url, txt, w));
        usado += if usado > 0.0 { 8.0 } else { 0.0 } + w;
    }
    for (i, linha) in grupos.iter().enumerate() {
        let mut x = s.r - 6.0 - (linha.iter().map(|c| c.2).sum::<f32>() + 8.0 * linha.len().saturating_sub(1) as f32);
        for (url, txt, w) in linha {
            let ry = s.y + 2.0 + i as f32 * 20.0;
            let area = Rect::from_min_size(c.pt(x, ry), vec2(*w, 18.0));
            let resposta = ui.interact(area, ui.scope_id().with(("link", url.as_str())), Sense::click());
            c.texto_sublinhado(txt, x + 6.0, ry + 13.0, 11.0, Peso::Normal, TINTA, Ancora::Inicio, true);
            if resposta.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if resposta.clicked() {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
            }
            x += w + 8.0;
        }
    }
    if livres > 0 {
        c.texto(&trn(livres, "+{n} LINK · VEJA EM EDITAR", "+{n} LINKS · VEJA EM EDITAR"), s.r - 6.0, s.y + 55.0, 10.0, Peso::Normal, MEDIO, Ancora::Fim, false);
    }
}
