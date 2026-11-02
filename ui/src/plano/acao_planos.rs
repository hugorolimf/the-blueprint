//! Os planos da ação aberta, como as linhas da folha original (`itens` de `folha/acoes.rs`): a caixinha que chama o agente e o texto (duas linhas; abre ao clique),
//! e, para quem já mexeu, o estado com o indicador; a área e a dependência ficam na dica. O texto clareia com o mouse em cima.

use super::super::acao_estilo::CORPO;
use super::super::caixinha::{self, Clique};
use super::super::linha::Pincel;
use super::super::modelo::{Acao, Plano, Situacao};
use super::super::revisao::Alvo;
use super::estado::{self, situacao_de};
use super::ver::{alternado, alternar, chave_ver};
use super::{bloco, rotulo, Saida};
use crate::i18n::tr_com;
use crate::menu::{self, Alvo as AlvoMenu};
use crate::planta::Texto;
use crate::plano::cartao_botao::{botao, Botao, Icone};
use crate::plano::cartao_caixa;
use crate::tema::{AZUL, MEDIO};
use egui::{Context, Painter, Sense, Ui};

const PASSO: f32 = 15.0;
const X_TEXTO: f32 = 24.0;
const MARGEM_DIR: f32 = 4.0;
const ALTO_FILEIRA: f32 = 18.0;
const ALTO_INICIAR: f32 = 34.0;
const ALTO_ROTULO: f32 = 22.0;
const LINHAS_FECHADO: usize = 2;
const LINHAS_ABERTO: usize = 6;

fn texto_plano<'a>(ctx: &Context, q: &'a Plano, w: f32) -> Texto<'a> {
    let linhas = if alternado(ctx, &chave_ver("p", &q.id)) { LINHAS_ABERTO } else { LINHAS_FECHADO };
    let cor = if q.feito || q.situacao == Situacao::Pronto { MEDIO } else { CORPO };
    bloco(&q.texto, 11.0, w - X_TEXTO - MARGEM_DIR, PASSO, linhas).cor(cor)
}

fn tem_fileira(q: &Plano) -> bool {
    !matches!(q.situacao, Situacao::Nao | Situacao::Revisao | Situacao::Outra)
}

fn altura_plano(p: &Painter, q: &Plano, w: f32) -> f32 {
    let linhas = texto_plano(p.ctx(), q, w).galeria(p).rows.len().max(1) as f32;
    (linhas * PASSO + 8.0).max(24.0) + if tem_fileira(q) { ALTO_FILEIRA } else { 0.0 }
}

/// Quantos planos estão prontos, como o rótulo mostra.
pub fn prontos(a: &Acao) -> usize {
    a.planos.iter().filter(|q| q.feito || q.situacao == Situacao::Pronto).count()
}

/// Quanto a lista de planos ocupa, com «Iniciar os N» quando há mais de um livre.
pub fn altura_planos(p: &Painter, a: &Acao, w: f32) -> f32 {
    let iniciar = if caixinha::livres(&a.planos).len() > 1 { ALTO_INICIAR } else { 0.0 };
    ALTO_ROTULO + a.planos.iter().map(|q| altura_plano(p, q, w)).sum::<f32>() + iniciar
}

/// A lista de planos com o rótulo em `(x, y)`. Marcar vai direto ao servidor; ir a Agentes volta em `Saida`.
pub fn planos(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &Acao, x: f32, y: f32, w: f32) -> Option<Saida> {
    rotulo(c, x, y, &tr_com("Planos · {feitos}/{total}", &[("feitos", &prontos(a).to_string()), ("total", &a.planos.len().to_string())]), w);
    let (mut yy, mut saida) = (y + ALTO_ROTULO - 2.0, None);
    for q in &a.planos {
        if cartao_plano(ui, c, alvo, a, q, (x, yy, w)) {
            saida = Some(Saida::Agentes);
        }
        yy += altura_plano(c.p, q, w);
    }
    let livres = caixinha::livres(&a.planos);
    if livres.len() > 1 {
        let texto = tr_com("Iniciar os {n}", &[("n", &livres.len().to_string())]);
        let b = Botao::novo(&texto).icone(Icone::Direita).cor(AZUL);
        if botao(ui, ("iniciar-todos", &a.id), c.pt(x, yy + 4.0), &b).clicked() {
            caixinha::marcar(ui.ctx(), alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &livres);
        }
    }
    saida
}

/// Uma linha de plano com o topo em `y`. Devolve se pediu para ver em Agentes.
fn cartao_plano(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &Acao, q: &Plano, (x, y, w): (f32, f32, f32)) -> bool {
    let ctx = ui.ctx().clone();
    let alto = altura_plano(c.p, q, w);
    let chave = chave_ver("p", &q.id);
    let t = texto_plano(&ctx, q, w);
    let g = t.galeria(c.p);
    let pode_abrir = g.elided || alternado(&ctx, &chave);
    let area = c.caixa(x, y, w, alto);
    let resposta = ui.interact(area, ui.scope_id().with(("plano-texto", &q.id)), if pode_abrir { Sense::click() } else { Sense::hover() });
    menu::marcar(ui, area, || AlvoMenu::Item { ws: alvo.ws.into(), bp: alvo.bp.into(), acao: a.id.clone(), tipo: "planos", id: q.id.clone(), texto: q.texto.clone(), feito: q.feito });
    let t = if resposta.hovered() { t.cor(egui::Color32::WHITE) } else { t };
    t.em(c.p, c.pt(x + X_TEXTO, y + 4.0));
    if pode_abrir && resposta.hovered() {
        ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let clicou = resposta.clicked();
    if let Some(dica) = dica_plano(q, g.elided) {
        let _ = resposta.on_hover_text(dica);
    }
    if clicou && pode_abrir {
        alternar(&ctx, &chave);
    }
    let mut pede_agentes = false;
    match cartao_caixa::caixa(super::lapis(c), ui, x, y + 4.0, q) {
        Some(Clique::Marcar) => caixinha::marcar(&ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &[q.id.as_str()]),
        Some(Clique::Agentes) => pede_agentes = true,
        None => {}
    }
    if tem_fileira(q) {
        let linhas = g.rows.len().max(1) as f32;
        fileira(ui, c, q, (x + X_TEXTO, y + 4.0 + linhas * PASSO, w - X_TEXTO - MARGEM_DIR));
    }
    pede_agentes
}

/// A fileira de baixo do cartão: só o estado, quando ele diz algo além da caixinha.
fn fileira(ui: &Ui, c: &Pincel, q: &Plano, (x, y, w): (f32, f32, f32)) {
    estado::desenhar(ui, c, q, x + w, y + ALTO_FILEIRA / 2.0, w);
}

/// O que a dica do cartão traz: o texto inteiro, se cortou, e o que o agente recebe além dele.
fn dica_plano(q: &Plano, cortou: bool) -> Option<String> {
    let mut partes: Vec<String> = Vec::with_capacity(5);
    if cortou {
        partes.push(q.texto.clone());
    }
    if let Some((estado, _)) = situacao_de(q).filter(|_| q.situacao == Situacao::Revisao) {
        partes.push(estado.to_string());
    }
    if !q.area.is_empty() {
        partes.push(q.area.clone());
    }
    if !q.depende_de.is_empty() {
        partes.push(tr_com("Depende de {lista}", &[("lista", &q.depende_de.join(", "))]));
    }
    if !q.instrucoes.is_empty() {
        partes.push(q.instrucoes.clone());
    }
    (!partes.is_empty()).then(|| partes.join("\n\n"))
}
