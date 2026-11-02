//! A moldura da folha do Planejar: as bordas com as marcas de prancha, o título grande com a nota, os links, o aviso e a legenda (`moldura` de `folha.mjs`).
//! Os botões ficam em `folha_botoes.rs` e o selo da folha em `folha_selo.rs`. Este arquivo também entra em `agentes`, `chegou` e `detalhe` (`#[path]`), que usam
//! `botoes`, `legenda`, `selo`, `moldura` e `Topo` com a assinatura de sempre: por isso os caminhos de fora são `crate::plano::folha::…`. Os textos ficam em `folha::TEXTOS`.

use super::super::escala::{MARGEM, X0, XR};
use crate::folha::cena::Cena;
use crate::folha::pagina::Medidas;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::Folha;
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::menu::{self, Alvo};
use crate::plano::folha::botoes_topo::{self, Item, TITULO_MAX};
use crate::plano::folha::estilo::{self, com_alfa, ANDANDO, ESPERA, FEITO, PLANEJADO};
use crate::telas::Acao;
use crate::tema::{AMBAR, BAIXO, CARTAO, MEDIO, PAPEL, TINTA};
use egui::{Id, Sense, Shape, Stroke, Ui};
use serde_json::Value;

pub use botoes_topo::Topo;

/// Onde ficam a linha de base do título e do subtítulo.
const Y_TITULO: f32 = 72.0;
const Y_SUBTITULO: f32 = 92.0;

/// A moldura da folha original, nas telas que não têm a régua de meses: duas colunas.
pub fn moldura(c: &Pincel, w: f32, h: f32) {
    let r = w - MARGEM;
    crate::folha::pagina::moldura(&Cena { p: c.p, o: c.o }, &Medidas { w, h, l: X0, r, meses: vec![X0, (X0 + r) / 2.0, r] });
}

/// A legenda dos símbolos, à esquerda do selo e sem passar de `x_fim`: cada situação com a peça que a barra usa.
pub fn legenda_ate(c: &Pincel, h: f32, x_fim: f32) {
    let (y, letra) = (h - 62.0, Letra::nova(10.0, MEDIO));
    let rotulos = [tr("concluída"), tr("em andamento"), tr("planejada"), tr("aguardando"), tr("checkpoint"), tr("proposta")];
    let mut x = X0;
    for (i, rotulo) in rotulos.into_iter().enumerate() {
        let w = 30.0 + c.largura(rotulo, &letra) + 22.0;
        if x + w - 22.0 > x_fim {
            break;
        }
        let ya = y - 6.0;
        match i {
            0 => c.suave(x, ya, 22.0, 12.0, 0.0, Some(com_alfa(FEITO, 0.82)), Some((FEITO, 1.0))),
            1 => {
                c.suave(x, ya, 22.0, 12.0, 0.0, Some(com_alfa(ANDANDO, 0.13)), Some((com_alfa(ANDANDO, 0.55), 1.0)));
                c.suave(x, ya, 12.0, 12.0, 0.0, Some(com_alfa(ANDANDO, 0.9)), None);
            }
            2 => c.suave(x, ya, 22.0, 12.0, 0.0, Some(com_alfa(CARTAO, 0.55)), Some((PLANEJADO, 1.2))),
            3 => c.suave_tracejado(x, ya, 22.0, 12.0, 0.0, ESPERA, 4.0, 3.0),
            4 => c.marco(x + 11.0, y, 5.0, None, PAPEL, TINTA),
            _ => c.suave_tracejado(x, ya, 22.0, 12.0, 0.0, TINTA, 4.0, 3.0),
        }
        c.texto_meio(rotulo, x + 30.0, y, letra);
        x += w;
    }
}

/// A legenda das outras telas do Planejar: cabe ao lado do selo delas, então mostra só as primeiras situações.
pub fn legenda(c: &Pincel, h: f32) {
    legenda_ate(c, h, X0 + 340.0);
}

/// O selo das outras telas, em células `(rótulo, valor, largura)`, como numa prancha. A folha tem o seu, com situação e progresso (`folha_selo.rs`).
pub fn selo(c: &Pincel, x1: f32, h: f32, celulas: &[(&str, String, f32)]) {
    let total: f32 = celulas.iter().map(|k| k.2).sum();
    let (mut x, y) = (x1 - total, h - 90.0);
    c.sombra(x, y, total, 60.0, 0.0, com_alfa(egui::Color32::BLACK, 0.35), 14, 4);
    c.ret(x, y, total, 60.0, Some(com_alfa(CARTAO, 0.96)), Some((com_alfa(TINTA, 0.95), 1.0)));
    for (i, (rotulo, valor, largura)) in celulas.iter().enumerate() {
        if i > 0 {
            c.fio(x, y + 8.0, x, y + 52.0, com_alfa(MEDIO, 0.5), 1.0);
        }
        c.rotulo(rotulo, x + 12.0, y + 19.0, Letra::nova(9.0, MEDIO).trilha(0.14));
        let letra = Letra { peso: Peso::Negrito, ..Letra::nova(13.0, TINTA) };
        c.texto(&c.cortado(valor, largura - 22.0, &letra).0, x + 12.0, y + 43.0, letra);
        x += largura;
    }
}

/// Os botões das outras telas.
pub fn botoes(ui: &mut Ui, c: &Pincel, x1: f32, lista: &[(Topo, String)]) -> Option<Topo> {
    let itens: Vec<Item> = lista.iter().map(|(topo, rotulo)| Item { topo: *topo, rotulo: rotulo.clone() }).collect();
    botoes_topo::botoes(ui, c, x1, &itens, X0 + 416.0)
}

/// O ícone de nota do blueprint: folha com a ponta dobrada.
fn nota(c: &Pincel, x: f32, y: f32, cor: egui::Color32) {
    let corpo = vec![c.pt(x, y), c.pt(x + 9.0, y), c.pt(x + 14.0, y + 5.0), c.pt(x + 14.0, y + 16.0), c.pt(x, y + 16.0)];
    c.p.add(Shape::convex_polygon(corpo, PAPEL, Stroke::new(1.2, cor)));
    c.fio(x + 9.0, y, x + 9.0, y + 5.0, cor, 1.2);
    c.fio(x + 9.0, y + 5.0, x + 14.0, y + 5.0, cor, 1.2);
    c.fio(x + 3.0, y + 9.0, x + 11.0, y + 9.0, cor, 1.2);
    c.fio(x + 3.0, y + 12.5, x + 11.0, y + 12.5, cor, 1.2);
}

/// Os links seguros (`http`/`https`), lado a lado a partir de `x`, sem passar de `x_fim`; o que não cabe vira «+N links». Abrem no navegador e têm menu.
pub fn links(ui: &Ui, c: &Pincel, x: f32, y: f32, x_fim: f32, lista: &[Value], chave: &str) {
    let campo = |l: &Value, k: &str| l.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let seguros = lista.iter().map(|l| (campo(l, "url"), [campo(l, "nome"), campo(l, "rotulo"), campo(l, "url")].into_iter().find(|t| !t.is_empty()).unwrap_or_default())).filter(|(u, _)| u.starts_with("http://") || u.starts_with("https://"));
    let (letra, mut px, mut sobram) = (Letra::nova(10.0, TINTA), x, 0usize);
    for (url, rotulo) in seguros {
        let texto = format!("↗ {}", c.cortado(&rotulo, 130.0, &letra).0.to_uppercase());
        let w = c.largura(&texto, &letra);
        if sobram > 0 || px + w + 60.0 > x_fim {
            sobram += 1;
            continue;
        }
        let area = c.caixa(px, y - 11.0, w, 15.0);
        let r = ui.interact(area, ui.scope_id().with(("plano-link", chave, url.as_str())), Sense::click());
        menu::marcar(ui, area, || Alvo::Link(url.clone()));
        let luz = estilo::sobre(ui.ctx(), Id::unique(("plano-link-sobre", chave, url.as_str())), r.hovered());
        c.texto(&texto, px, y, Letra { cor: estilo::misturar(MEDIO, TINTA, luz), ..letra });
        c.fio(px, y + 3.5, px + w * (0.3 + 0.7 * luz), y + 3.5, com_alfa(BAIXO, 0.6 + 0.4 * luz), 1.0);
        if r.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if r.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }
        px += w + 14.0;
    }
    if sobram > 0 {
        c.texto(&tr_com("+{n} links", &[("n", &sobram.to_string())]), px, y, Letra::nova(10.0, MEDIO));
    }
}

/// O que o subtítulo de «O que chegou» diz, vindo do documento.
pub struct Cabeca {
    pub workspace: String,
    pub folha: usize,
    pub folhas: usize,
}

/// O título de «O que chegou»: o nome e, embaixo, o workspace e a folha.
pub fn titulo(ui: &Ui, c: &Pincel, f: &Folha, cabeca: &Cabeca, ws: &str, bp: &str) -> Option<Acao> {
    let sub = format!("{} · {}", cabeca.workspace, tr_com("Folha {n} de {total}", &[("n", &cabeca.folha.to_string()), ("total", &cabeca.folhas.to_string())])).trim_start_matches(" · ").to_string();
    let nome = if f.blueprint.nome.is_empty() { tr("Sem nome") } else { f.blueprint.nome.as_str() };
    titulo_folha(ui, c, f, nome, &sub, TITULO_MAX, ws, bp)
}

/// O título da folha, como na original: 18 px em negrito e caixa alta; embaixo, 11 px no tom médio. O nome é cortado, e a dica traz o inteiro e o resumo. Devolve o pedido de abrir a nota.
#[allow(clippy::too_many_arguments)]
pub fn titulo_folha(ui: &Ui, c: &Pincel, f: &Folha, titulo: &str, sub: &str, max: f32, ws: &str, bp: &str) -> Option<Acao> {
    let letra_sub = Letra::nova(11.0, MEDIO);
    c.texto(&c.cortado(&sub.to_uppercase(), max, &letra_sub).0, X0, Y_SUBTITULO, letra_sub);
    let letra = Letra::nova(18.0, TINTA).forte();
    let (corte, _) = c.cortado(&titulo.to_uppercase(), max - if f.blueprint.tem_nota { 30.0 } else { 0.0 }, &letra);
    let largura = c.texto(&corte, X0, Y_TITULO, letra);
    let dica = if f.blueprint.resumo.is_empty() { titulo.to_string() } else { format!("{titulo}\n{}", f.blueprint.resumo) };
    let _ = ui.interact(c.caixa(X0, Y_TITULO - 20.0, largura, 26.0), Id::unique(("plano-titulo", bp)), Sense::hover()).on_hover_text(dica);
    if !f.blueprint.tem_nota {
        return None;
    }
    let x = X0 + largura + 12.0;
    let area = c.caixa(x - 4.0, Y_TITULO - 20.0, 24.0, 24.0);
    menu::marcar(ui, area, || Alvo::Nota { ws: ws.into(), bp: bp.into(), acao: None });
    let resposta = ui.interact(area, Id::unique(("plano-nota", bp)), Sense::click());
    let luz = estilo::sobre(ui.ctx(), Id::unique(("plano-nota-sobre", bp)), resposta.hovered());
    nota(c, x, Y_TITULO - 16.0, estilo::misturar(TINTA, egui::Color32::WHITE, luz));
    if resposta.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resposta.clicked().then(|| Acao::AbrirNota(ws.into(), bp.into(), None))
}

/// O que o último gesto respondeu, na coluna esquerda, cortado com a frase inteira na dica.
pub fn escrever_aviso(ui: &Ui, c: &Pincel, aviso: &str, y: f32) {
    let letra = Letra::nova(10.5, AMBAR);
    let (t, cortou) = c.cortado(aviso, XR - X0 - 8.0, &letra);
    let largura = c.texto(&t, X0, y, letra);
    if cortou {
        let _ = ui.interact(c.caixa(X0, y - 12.0, largura, 16.0), Id::unique("plano-aviso"), Sense::hover()).on_hover_text(aviso);
    }
}
