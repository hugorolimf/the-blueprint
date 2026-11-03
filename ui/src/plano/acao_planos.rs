//! Os planos da ação aberta, como as linhas da folha original (`itens` de `folha/acoes.rs`): a caixinha que chama o agente e o texto inteiro,
//! e, para quem já mexeu, o estado com o indicador; a área e a dependência ficam na dica. O texto clareia com o mouse em cima.

use super::super::acao_estilo::{LETRA_PLANO};
use super::super::caixinha::{self, Clique};
use super::super::linha::Pincel;
use super::super::modelo::{Acao, Folha, Plano, Situacao};
use super::super::revisao::Alvo;
use super::estado::{self, situacao_de};
use super::obra_icones::{self as icones, Icone};
use super::{bloco, obra_regra};
use crate::i18n::tr_com;
use crate::menu::{self, Alvo as AlvoMenu};
use crate::planta::Texto;
use crate::plano::cartao_caixa;
use crate::fonte::Peso;
use crate::tema::{AMBAR, MEDIO, TINTA};
use egui::{CursorIcon, Painter, Pos2, Sense, Ui};

const PASSO: f32 = super::super::acao_estilo::PASSO_PLANO;
/// Depois da caixinha, o número do plano.
const X_NUMERO: f32 = 24.0;
const VAO_NUMERO: f32 = 10.0;
const LETRA_NUMERO: f32 = 12.0;
const MARGEM_DIR: f32 = 4.0;
const ALTO_FILEIRA: f32 = 18.0;

fn numero(q: &Plano) -> Texto<'_> {
    Texto::novo(&q.referencia, LETRA_NUMERO).cor(MEDIO)
}

fn x_texto(p: &Painter, q: &Plano) -> f32 {
    X_NUMERO + numero(q).largura(p) + VAO_NUMERO
}

fn texto_plano<'a>(p: &Painter, q: &'a Plano, w: f32) -> Texto<'a> {
    let cor = if q.feito || q.situacao == Situacao::Pronto { MEDIO } else { TINTA };
    bloco(&q.texto, LETRA_PLANO, w - x_texto(p, q) - MARGEM_DIR, PASSO).peso(Peso::Negrito).cor(cor)
}

fn tem_fileira(q: &Plano) -> bool {
    !matches!(q.situacao, Situacao::Nao | Situacao::Revisao | Situacao::Outra)
}

pub fn altura_plano(p: &Painter, q: &Plano, w: f32) -> f32 {
    let linhas = texto_plano(p, q, w).galeria(p).rows.len().max(1) as f32;
    (linhas * PASSO + 8.0).max(24.0) + if tem_fileira(q) { ALTO_FILEIRA } else { 0.0 }
}

/// Onde o texto do plano começa, para o que vem embaixo dele alinhar.
pub fn recuo(p: &Painter, q: &Plano) -> f32 {
    x_texto(p, q)
}

/// O que o clique na caixinha pede a quem tem a folha.
pub enum Pede {
    Agentes,
    /// leva ao plano que falta iniciar
    Ver(String, Pos2),
}

/// O cadeado no lugar da caixinha: o plano anterior nem começou. O clique leva até ele.
fn cadeado(ui: &mut Ui, c: &Pincel, q: &Plano, faltam: &[&str], x: f32, y: f32) -> Option<Pede> {
    let area = c.caixa(x - 3.0, y - 3.0, 20.0, 20.0);
    icones::desenhar(c.p, area.center(), icones::LADO, Icone::Cadeado, AMBAR);
    let r = ui.interact(area, ui.scope_id().with(("plano-cadeado", &q.id)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    let clicou = r.clicked();
    let _ = r.on_hover_text(tr_com("Inicie {lista} antes: este plano depende dele.", &[("lista", &faltam.join(", "))]));
    clicou.then(|| Pede::Ver(faltam[0].to_string(), area.center()))
}

/// Uma linha de plano com o topo em `y`. Devolve o que o clique pede.
#[allow(clippy::too_many_arguments)]
pub fn cartao_plano(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, q: &Plano, (x, y, w): (f32, f32, f32)) -> Option<Pede> {
    let ctx = ui.ctx().clone();
    let alto = altura_plano(c.p, q, w);
    let t = texto_plano(c.p, q, w);
    let g = t.galeria(c.p);
    let x_t = x_texto(c.p, q);
    let area = c.caixa(x, y, w, alto);
    let resposta = ui.interact(area, ui.scope_id().with(("plano-texto", &q.id)), Sense::hover());
    menu::marcar(ui, area, || AlvoMenu::Item { ws: alvo.ws.into(), bp: alvo.bp.into(), acao: a.id.clone(), tipo: "planos", id: q.id.clone(), texto: q.texto.clone(), feito: q.feito });
    let t = if resposta.hovered() { t.cor(egui::Color32::WHITE) } else { t };
    numero(q).em(c.p, c.pt(x + X_NUMERO, y + 8.0));
    t.em(c.p, c.pt(x + x_t, y + 4.0));
    if let Some(dica) = dica_plano(q) {
        let _ = resposta.on_hover_text(dica);
    }
    let faltam = obra_regra::sem_iniciar(f, q);
    let mut pede = None;
    if !faltam.is_empty() && caixinha::clique_de(q) == Some(Clique::Marcar) {
        pede = cadeado(ui, c, q, &faltam, x, y + 8.0);
    } else {
        match cartao_caixa::caixa(super::lapis(c), ui, x, y + 8.0, q) {
            Some(Clique::Marcar) => caixinha::marcar(&ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &[q.id.as_str()]),
            Some(Clique::Agentes) => pede = Some(Pede::Agentes),
            None => {}
        }
    }
    if tem_fileira(q) {
        let linhas = g.rows.len().max(1) as f32;
        fileira(ui, c, q, (x + x_t, y + 4.0 + linhas * PASSO, w - x_t - MARGEM_DIR));
    }
    pede
}

/// A fileira de baixo do cartão: só o estado, quando ele diz algo além da caixinha.
fn fileira(ui: &Ui, c: &Pincel, q: &Plano, (x, y, w): (f32, f32, f32)) {
    estado::desenhar(ui, c, q, x + w, y + ALTO_FILEIRA / 2.0, w);
}

/// O que a dica do cartão traz: o que o agente recebe além do texto.
fn dica_plano(q: &Plano) -> Option<String> {
    let mut partes: Vec<String> = Vec::with_capacity(4);
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
