//! A seção do blueprint aberto (`cabecalhoSecao` e `secao` em web/js): o cabeçalho com o número, o nome, o progresso e os links, e cada ação com
//! a barra, a cota de dias, as caixas dos planos e os losangos dos checkpoints (tudo marcável com o clique).

use super::cena::{Ancora, Cena, Fio};
use super::disposicao::{Disposicao, CHAR, PH};
use super::pagina::corta;
use super::regua::Regua;
use crate::dominio::{self, Acao, Situacao};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com, trn};
use crate::telas::Acao as Pedido;
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

/// As ações abertas, com as caixas marcáveis. Devolve o pedido de marcar, se houve clique.
pub fn acoes(c: &Cena, ui: &mut Ui, s: &Secao, hoje: i64) -> Option<Pedido> {
    let mut pedido = None;
    let j = s.regua.janela;
    if s.total == 0 {
        c.texto(&tr("Sem ações · abra Editar para adicionar"), s.l + 40.0, s.y + PH + 8.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    }
    for pos in &s.disp.posicoes {
        let f = s.acoes[pos.i];
        if !s.aberto {
            compacta(c, s, pos, f, hoje);
            continue;
        }
        let (x0, x1, w) = (pos.bx0, pos.bx0 + pos.bw, pos.bw);
        let fy = s.y + PH + s.disp.linha_y[pos.linha];
        let (by, bh) = (fy + 20.0, 26.0);
        c.texto(&pos.rotulo, pos.lx, fy + 12.0, 11.0, Peso::Normal, TINTA, Ancora::Inicio, true);
        c.nota(pos.lx + pos.rotulo.chars().count() as f32 * (CHAR + 0.35) + 16.0, fy - 1.0, false);
        // a barra: cheia de tinta fina (concluída), com o andamento (em andamento) ou tracejada (a fazer)
        match f.status {
            Situacao::Planejado => c.moldura(x0, by, w, bh, MEDIO, 1.2, false),
            Situacao::Andamento => c.retangulo(x0, by, w, bh, Some(PAPEL), Some((TINTA, 1.8))),
            Situacao::Concluido => c.retangulo(x0, by, w, bh, Some(PAPEL), Some((TINTA, 1.2))),
        }
        if f.status == Situacao::Concluido {
            c.hachura(x0 + 1.0, by + 1.0, w - 2.0, bh - 2.0);
            if w >= 100.0 {
                let (cx, cy) = ((x0 + 12.0).max(x1 - 17.0), by + bh / 2.0);
                c.p.circle(c.pt(cx, cy), 8.0, PAPEL, egui::Stroke::new(1.0, TINTA));
                c.fios(Fio::Ln, &[(cx - 3.6, cy, cx - 1.0, cy + 3.0), (cx - 1.0, cy + 3.0, cx + 3.6, cy - 3.0)]);
            }
        } else if f.status == Situacao::Andamento {
            c.retangulo(x0 + 1.0, by + 1.0, ((w - 2.0) * f.pct).max(0.0), bh - 2.0, Some(TINTA.gamma_multiply(0.9)), None);
        }
        let xp = (x1 + 20.0).min(s.r + 4.0);
        c.texto(&format!("{}%", (f.pct * 100.0).round() as i32), xp, by + 17.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
        if f.itens.1 > 0 {
            c.texto(&format!("{}/{}", f.itens.0, f.itens.1), xp, by + 31.0, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
        }
        for m in f.checkpoints.iter().filter(|m| m.data.is_some_and(|d| d >= j.t0 && d < j.t1)) {
            let d = m.data.unwrap_or(0);
            let mx = x0 + ((d as f32 + 0.5 - f.a as f32) / ((f.b + 1 - f.a) as f32)) * (x1 - x0);
            c.losango_r(mx, by + bh / 2.0, 7.0, m.feito, 1.4);
        }
        // a cota de dias: duas hastes, a linha e os traços oblíquos das pontas
        let cy = fy + 72.0;
        c.fios(Fio::Medio, &[(x0, by + bh + 2.0, x0, cy + 6.0), (x1, by + bh + 2.0, x1, cy + 6.0), (x0, cy, x1, cy), (x0 - 4.0, cy + 4.0, x0 + 4.0, cy - 4.0), (x1 - 4.0, cy + 4.0, x1 + 4.0, cy - 4.0)]);
        c.texto(&trn((f.b - f.a + 1) as usize, "{n} DIA", "{n} DIAS"), (x0 + x1) / 2.0, cy - 5.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
        let (ini, fim) = (data_curta(f.a), data_curta(f.b));
        if x1 - x0 < 104.0 {
            c.texto(&format!("{ini} → {fim}"), (x0 + x1) / 2.0, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Meio, false);
        } else {
            c.texto(&ini, x0, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
            c.texto(&fim, x1, cy + 18.0, 10.0, Peso::Normal, MEDIO, Ancora::Fim, false);
        }
        if !f.planos.is_empty() || !f.checkpoints.is_empty() {
            if let Some(p) = itens(c, ui, s, pos, f, fy + 112.0) {
                pedido = Some(p);
            }
        }
    }
    pedido
}

fn data_curta(dias: i64) -> String {
    let iso = crate::documento::iso_do_dia(dias);
    format!("{}/{}", &iso[8..10], &iso[5..7])
}

/// As duas colunas de baixo: os planos (caixas) e os checkpoints (losangos), cada item clicável.
fn itens(c: &Cena, ui: &mut Ui, s: &Secao, pos: &super::disposicao::Posicao, f: &Acao, base: f32) -> Option<Pedido> {
    let mut pedido = None;
    let colw = s.colw;
    let maxc = ((colw - 28.0) / 7.3).floor() as usize;
    let cabeca = |x: f32, titulo: &str, feitos: usize, total: usize| {
        c.texto(&format!("{titulo} · {feitos}/{total}"), x, base, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
        c.fio(super::cena::BAIXO_COR, 1.0, (x, base + 7.0), (x + colw - 20.0, base + 7.0));
    };
    let mut cx0 = pos.lx;
    let marcar = |ui: &mut Ui, area: Rect, id: &str| {
        let r = ui.interact(area, ui.scope_id().with(("item", id)), Sense::click());
        if r.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        (r.clicked(), r.hovered())
    };
    if !f.planos.is_empty() {
        cabeca(cx0, &tr("Planos"), f.planos.iter().filter(|t| t.feito).count(), f.planos.len());
        let mut deslocamento = 0.0;
        for (i, t) in f.planos.iter().enumerate() {
            let yy = base + 30.0 + deslocamento;
            let linhas = &pos.linhas_planos[i];
            let altura = (linhas.len() as f32 * 14.0 + 8.0).max(22.0);
            deslocamento += altura;
            let (clicou, sobre) = marcar(ui, Rect::from_min_size(c.pt(cx0 - 4.0, yy - 15.0), vec2(colw - 12.0, altura)), &t.id);
            c.retangulo(cx0, yy - 11.0, 13.0, 13.0, Some(if t.feito { TINTA } else { PAPEL }), Some((TINTA, if sobre { 2.6 } else { 1.3 })));
            if t.feito {
                c.p.add(egui::Shape::line(vec![c.pt(cx0 + 3.0, yy - 4.0), c.pt(cx0 + 6.0, yy - 1.0), c.pt(cx0 + 11.0, yy - 7.0)], egui::Stroke::new(2.0, PAPEL)));
            }
            let cor = if sobre { egui::Color32::WHITE } else if t.feito { MEDIO.gamma_multiply(0.8) } else { TINTA };
            for (n, linha) in linhas.iter().enumerate() {
                c.texto_liso(linha, cx0 + 22.0, yy + n as f32 * 14.0, 11.0, Peso::Normal, cor, Ancora::Inicio);
            }
            if clicou {
                pedido = Some(Pedido::Marcar(s.ws.into(), texto(s.p, "id"), f.id.clone(), "planos", t.id.clone()));
            }
        }
        cx0 += colw + 30.0;
    }
    if !f.checkpoints.is_empty() {
        cabeca(cx0, &tr("Checkpoints"), f.checkpoints.iter().filter(|m| m.feito).count(), f.checkpoints.len());
        for (i, m) in f.checkpoints.iter().enumerate() {
            let yy = base + 30.0 + i as f32 * 22.0;
            let (clicou, sobre) = marcar(ui, Rect::from_min_size(c.pt(cx0 - 4.0, yy - 15.0), vec2(colw - 12.0, 22.0)), &m.id);
            c.losango_r(cx0 + 6.5, yy - 5.5, 6.5, m.feito, if sobre { 2.6 } else { 1.4 });
            let nome = if m.nome.is_empty() { tr("sem nome").to_string() } else { m.nome.clone() };
            let data = m.data.map(|d| format!(" · {}", data_curta(d))).unwrap_or_default();
            let cor = if sobre { egui::Color32::WHITE } else if m.feito { MEDIO.gamma_multiply(0.8) } else { TINTA };
            c.texto_liso(&corta(&format!("{nome}{data}"), maxc), cx0 + 22.0, yy, 11.0, Peso::Normal, cor, Ancora::Inicio);
            if clicou {
                pedido = Some(Pedido::Marcar(s.ws.into(), texto(s.p, "id"), f.id.clone(), "checkpoints", m.id.clone()));
            }
        }
    }
    let _ = rgba;
    pedido
}

/// A ação da seção recolhida: só a barra fina na linha dela, com o hachurado ou o andamento e os checkpoints (`!s.aberto` em render.js).
fn compacta(c: &Cena, s: &Secao, pos: &super::disposicao::Posicao, f: &Acao, _hoje: i64) {
    let (x0, w) = (pos.x0, (pos.x1 - pos.x0).max(6.0));
    let by = s.y + PH - 12.0 + s.disp.linha_y[pos.linha];
    match f.status {
        Situacao::Planejado => c.moldura(x0, by, w, 10.0, MEDIO, 1.2, false),
        Situacao::Andamento => c.retangulo(x0, by, w, 10.0, Some(PAPEL), Some((TINTA, 1.8))),
        Situacao::Concluido => c.retangulo(x0, by, w, 10.0, Some(PAPEL), Some((TINTA, 1.2))),
    }
    match f.status {
        Situacao::Concluido => c.hachura(x0 + 1.0, by + 1.0, w - 2.0, 8.0),
        Situacao::Andamento => c.retangulo(x0 + 1.0, by + 1.0, ((w - 2.0) * f.pct).max(0.0), 8.0, Some(TINTA.gamma_multiply(0.9)), None),
        Situacao::Planejado => {}
    }
    let j = s.regua.janela;
    for m in f.checkpoints.iter().filter(|m| m.data.is_some_and(|d| d >= j.t0 && d < j.t1)) {
        let mx = s.regua.xf(m.data.unwrap_or(0) as f32 + 0.5).clamp(s.l, s.r);
        c.losango_r(mx, by + 5.0, 4.5, m.feito, 1.4);
    }
}
