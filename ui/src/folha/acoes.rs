//! As ações de uma seção (`secao` em web/js/render.js): recolhidas, só a barra fina; abertas, a barra, a cota de dias, as caixas dos planos e os
//! losangos dos checkpoints, tudo marcável com o clique. Pesquisando, o que não casou esmaece e o trecho achado ganha o realce.

use super::barra;
use super::cena::{Ancora, Cena, Fio};
use super::disposicao::{CHAR, PH};
use super::pagina::corta;
use super::secao::Secao;
use crate::agentes::Lugar;
use crate::dominio::{Acao, Situacao};
use crate::fonte::Peso;
use crate::i18n::{tr, trn};
use crate::menu::Alvo;
use crate::pesquisa::{self, ResAcao};
use crate::telas::Acao as Pedido;
use crate::tema::{MEDIO, PAPEL, TINTA};
use egui::{vec2, Rect, Sense, Ui};
use serde_json::Value;

fn texto(v: &Value, campo: &str) -> String {
    v.get(campo).and_then(Value::as_str).unwrap_or("").to_string()
}

/// As ações abertas, com as caixas marcáveis. Devolve o pedido de marcar, se houve clique.
pub fn desenhar(c: &Cena, ui: &mut Ui, s: &Secao) -> Option<Pedido> {
    let mut pedido = None;
    let j = s.regua.janela;
    if s.total == 0 {
        c.texto(&tr("Sem ações · abra Editar para adicionar"), s.l + 40.0, s.y + PH + 8.0, 11.0, Peso::Normal, MEDIO, Ancora::Inicio, true);
    }
    if s.total > 0 && s.acoes.is_empty() {
        let aviso = if s.pendentes { tr("Nada pendente neste blueprint.") } else { tr("Nenhuma ação neste período.") };
        c.texto(aviso, s.l, s.y + PH + 30.0, 12.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
    }
    for pos in &s.disp.posicoes {
        let f = s.acoes[pos.i];
        // pesquisando: a ação que não casou esmaece (18%); a que casou fica inteira
        let ra: Option<&ResAcao> = s.pesq.and_then(|(r, _)| r.acoes.iter().find(|a| a.id == f.id));
        // abrir, recolher ou trocar o período: a ação anda até o lugar novo, a barra muda de largura e o andamento cresce (anima::deslize)
        let (topo, bx, bw) = if s.aberto { (PH + s.disp.linha_y[pos.linha], pos.bx0, pos.bw) } else { (PH - 12.0 + s.disp.linha_y[pos.linha], pos.x0, (pos.x1 - pos.x0).max(6.0)) };
        let (v, nasce) = crate::anima::deslize(ui.ctx(), ("acao", f.id.as_str()), &[topo, bx, bw, f.pct]);
        let fraco = barra::opaco(c.p, if s.pesq.is_some() && ra.is_none() { 0.16 * nasce } else { nasce });
        let ce = Cena { p: &fraco, o: c.o };
        let c = &ce;
        let area = if s.aberto {
            let (x1, fy) = (v[1] + v[2], s.y + v[0]);
            let (de, ate) = (pos.lx.min(v[1]), (pos.lx + pos.lw).max(x1 + 56.0).max(pos.lx + pos.bloco_w));
            Rect::from_min_max(c.pt(de, fy - 6.0), c.pt(ate, fy + pos.alt - 20.0))
        } else {
            Rect::from_min_size(c.pt(v[1], s.y + v[0] - 7.0), vec2(v[2], 24.0))
        };
        crate::menu::marcar(ui, area, || Alvo::Acao { ws: s.ws.into(), bp: texto(s.p, "id"), id: f.id.clone(), rotulo: pos.rotulo.clone(), status: f.status, itens: f.itens });
        if !s.aberto {
            compacta(c, s, f, s.y + v[0], v[1], v[2], v[3]);
            continue;
        }
        let (x0, w, pct) = (v[1], v[2], v[3]);
        let x1 = x0 + w;
        let fy = s.y + v[0];
        let (by, bh) = (fy + 20.0, 26.0);
        s.ancoras.borrow_mut().insert(Lugar::Barra(f.id.clone()), (x0, by, w, bh));
        if let (Some((_, termos)), Some(a)) = (s.pesq, ra) {
            if a.propria.as_ref().is_some_and(|o| o.contains(&"nome")) {
                c.realce(&pos.rotulo, pos.lx, fy + 12.0, 11.0, Peso::Normal, 0.06, true, &pesquisa::trechos(&pos.rotulo, termos));
            }
        }
        c.texto(&pos.rotulo, pos.lx, fy + 12.0, 11.0, Peso::Normal, if ra.is_some_and(|a| a.propria.is_some()) { egui::Color32::WHITE } else { TINTA }, Ancora::Inicio, true);
        let nx = pos.lx + pos.rotulo.chars().count() as f32 * (CHAR + 0.35) + 16.0;
        c.nota(nx, fy - 1.0, false);
        let area_nota = Rect::from_min_size(c.pt(nx - 3.0, fy - 4.0), vec2(22.0, 22.0));
        crate::menu::marcar(ui, area_nota, || Alvo::Nota { ws: s.ws.into(), bp: texto(s.p, "id"), acao: Some(f.id.clone()) });
        if super::secao::clique_nota(c, ui, area_nota, ("nota-acao", f.id.as_str())) {
            *s.aberta.borrow_mut() = Some(Pedido::AbrirNota(s.ws.into(), texto(s.p, "id"), Some(f.id.clone())));
        }
        // a barra: cheia de tinta fina (concluída), com o andamento (em andamento) ou só o contorno (a fazer); na abertura ela se desenha
        let vez = barra::Vez::da(c, fy, fy + 112.0);
        barra::caixa(c, x0, by, w, bh, f.status, vez);
        barra::miolo(c, x0 + 1.0, by + 1.0, w - 2.0, bh - 2.0, f.status, pct, vez);
        if f.status == Situacao::Concluido && w >= 100.0 {
            barra::selo(c, (x0 + 12.0).max(x1 - 17.0), by + bh / 2.0, vez);
        }
        let xp = (x1 + 20.0).min(s.r + 4.0);
        c.texto(&format!("{}%", (f.pct * 100.0).round() as i32), xp, by + 17.0, 12.0, Peso::Negrito, TINTA, Ancora::Inicio, false);
        if f.itens.1 > 0 {
            c.texto(&format!("{}/{}", f.itens.0, f.itens.1), xp, by + 31.0, 9.0, Peso::Normal, MEDIO, Ancora::Inicio, false);
        }
        for m in f.checkpoints.iter().filter(|m| m.data.is_some_and(|d| d >= j.t0 && d < j.t1)) {
            let d = m.data.unwrap_or(0);
            let mx = x0 + ((d as f32 + 0.5 - f.a as f32) / ((f.b + 1 - f.a) as f32)) * (x1 - x0);
            barra::losango(c, x0, x1, mx, by + bh / 2.0, 7.0, m.feito, vez);
        }
        // a cota de dias: duas hastes, a linha e os traços oblíquos das pontas (abre da esquerda na abertura)
        let cy = fy + 72.0;
        let pincel_cota = barra::cota(c, x0, x1, by + bh, cy + 24.0, vez);
        let c = &Cena { p: &pincel_cota, o: c.o };
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
            if let Some(p) = itens(&ce, ui, s, pos, f, fy + 112.0, ra) {
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
fn itens(c: &Cena, ui: &mut Ui, s: &Secao, pos: &super::disposicao::Posicao, f: &Acao, base: f32, ra: Option<&ResAcao>) -> Option<Pedido> {
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
            s.ancoras.borrow_mut().insert(Lugar::Plano(t.id.clone()), (cx0 - 4.0, yy - 15.0, colw - 12.0, altura));
            let area = Rect::from_min_size(c.pt(cx0 - 4.0, yy - 15.0), vec2(colw - 12.0, altura));
            crate::menu::marcar(ui, area, || Alvo::Item { ws: s.ws.into(), bp: texto(s.p, "id"), acao: f.id.clone(), tipo: "planos", id: t.id.clone(), texto: t.texto.clone(), feito: t.feito });
            let (clicou, sobre) = marcar(ui, area, &t.id);
            // pesquisando: o item que casou fica inteiro e branco; os outros de uma ação «que só tem» itens achados esmaecem
            let achou = ra.and_then(|a| a.planos.iter().find(|(i, _)| *i == t.id));
            let mut fraco = c.p.clone();
            fraco.set_opacity(0.16);
            let ci = Cena { p: if s.pesq.is_some() && achou.is_none() && ra.is_some_and(|a| a.propria.is_none()) { &fraco } else { c.p }, o: c.o };
            ci.retangulo(cx0, yy - 11.0, 13.0, 13.0, Some(if t.feito { TINTA } else { PAPEL }), Some((TINTA, if sobre { 2.6 } else { 1.3 })));
            if t.feito {
                ci.p.add(egui::Shape::line(vec![ci.pt(cx0 + 3.0, yy - 4.0), ci.pt(cx0 + 6.0, yy - 1.0), ci.pt(cx0 + 11.0, yy - 7.0)], egui::Stroke::new(2.0, PAPEL)));
            }
            let cor = if sobre || achou.is_some() { egui::Color32::WHITE } else if t.feito { MEDIO.gamma_multiply(0.8) } else { TINTA };
            for (n, linha) in linhas.iter().enumerate() {
                if let (Some((_, termos)), Some((_, onde))) = (s.pesq, achou) {
                    if onde.contains(&"nome") {
                        ci.realce(linha, cx0 + 22.0, yy + n as f32 * 14.0, 11.0, Peso::Normal, 0.06, false, &pesquisa::trechos(linha, termos));
                    }
                }
                ci.texto(linha, cx0 + 22.0, yy + n as f32 * 14.0, 11.0, Peso::Normal, cor, Ancora::Inicio, false);
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
            let area = Rect::from_min_size(c.pt(cx0 - 4.0, yy - 15.0), vec2(colw - 12.0, 22.0));
            crate::menu::marcar(ui, area, || Alvo::Item { ws: s.ws.into(), bp: texto(s.p, "id"), acao: f.id.clone(), tipo: "checkpoints", id: m.id.clone(), texto: m.nome.clone(), feito: m.feito });
            let (clicou, sobre) = marcar(ui, area, &m.id);
            let achou = ra.and_then(|a| a.checkpoints.iter().find(|(i, _)| *i == m.id));
            let mut fraco = c.p.clone();
            fraco.set_opacity(0.16);
            let ci = Cena { p: if s.pesq.is_some() && achou.is_none() && ra.is_some_and(|a| a.propria.is_none()) { &fraco } else { c.p }, o: c.o };
            ci.losango_r(cx0 + 6.5, yy - 5.5, 6.5, m.feito, if sobre { 2.6 } else { 1.4 });
            let nome = if m.nome.is_empty() { tr("sem nome").to_string() } else { m.nome.clone() };
            let data = m.data.map(|d| format!(" · {}", data_curta(d))).unwrap_or_default();
            let cor = if sobre || achou.is_some() { egui::Color32::WHITE } else if m.feito { MEDIO.gamma_multiply(0.8) } else { TINTA };
            let linha = corta(&format!("{nome}{data}"), maxc);
            if let (Some((_, termos)), Some((_, onde))) = (s.pesq, achou) {
                if onde.contains(&"nome") {
                    ci.realce(&linha, cx0 + 22.0, yy, 11.0, Peso::Normal, 0.06, false, &pesquisa::trechos(&linha, termos));
                }
            }
            ci.texto(&linha, cx0 + 22.0, yy, 11.0, Peso::Normal, cor, Ancora::Inicio, false);
            if clicou {
                pedido = Some(Pedido::Marcar(s.ws.into(), texto(s.p, "id"), f.id.clone(), "checkpoints", m.id.clone()));
            }
        }
    }
    pedido
}

/// A ação da seção recolhida: só a barra fina na linha dela, com o hachurado ou o andamento e os checkpoints (`!s.aberto` em render.js).
#[allow(clippy::too_many_arguments)]
fn compacta(c: &Cena, s: &Secao, f: &Acao, by: f32, x0: f32, w: f32, pct: f32) {
    s.ancoras.borrow_mut().insert(Lugar::Barra(f.id.clone()), (x0, by, w, 10.0));
    let vez = barra::Vez::da(c, by - 7.0, by + 17.0);
    barra::caixa(c, x0, by, w, 10.0, f.status, vez);
    barra::miolo(c, x0 + 1.0, by + 1.0, w - 2.0, 8.0, f.status, pct, vez);
    let j = s.regua.janela;
    for m in f.checkpoints.iter().filter(|m| m.data.is_some_and(|d| d >= j.t0 && d < j.t1)) {
        let mx = s.regua.xf(m.data.unwrap_or(0) as f32 + 0.5).clamp(s.l, s.r);
        barra::losango(c, x0, x0 + w, mx, by + 5.0, 4.5, m.feito, vez);
    }
}
