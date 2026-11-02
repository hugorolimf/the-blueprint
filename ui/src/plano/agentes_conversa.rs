//! A conversa com o orquestrador: o que ele faz agora (`agora`) em balões à esquerda e o que você pediu nesta janela à direita, o mais novo embaixo.
//! O servidor não guarda o que você escreveu, então os pedidos ficam só na janela e entram na ordem certa: logo depois da linha que era a mais nova quando você mandou.

use super::super::agentes_movimento as mov;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::Orquestrador;
use crate::i18n::tr;
use crate::tema::{AZUL, BAIXO, CAMPO, MEDIO, QUADRA, TINTA};
use egui::{Context, Id, Pos2, Sense, Shape, Stroke, Ui};

const ALTO: f32 = 24.0;
const VAO: f32 = 6.0;
const CORTE: f32 = 7.0;
const LARGURA_MAX: f32 = 0.82;
const GUARDADAS: usize = 3;

/// Um pedido que você mandou, e a linha mais nova do orquestrador naquele instante (a âncora da ordem).
#[derive(Clone, Debug, Default)]
pub struct Enviada {
    pub texto: String,
    pub ancora: String,
}

fn chave(ws: &str) -> Id {
    Id::unique(("plano-orq-enviadas", ws.to_string()))
}

pub fn enviadas(ctx: &Context, ws: &str) -> Vec<Enviada> {
    ctx.data(|d| d.get_temp::<Vec<Enviada>>(chave(ws))).unwrap_or_default()
}

/// Guarda o pedido que acabou de sair; `agora` é a lista do orquestrador neste quadro (a primeira linha é a mais nova).
pub fn registrar(ctx: &Context, ws: &str, texto: &str, agora: &[String]) {
    let mut lista = enviadas(ctx, ws);
    lista.push(Enviada { texto: texto.trim().to_string(), ancora: agora.first().cloned().unwrap_or_default() });
    let sobra = lista.len().saturating_sub(GUARDADAS);
    lista.drain(..sobra);
    ctx.data_mut(|d| d.insert_temp(chave(ws), lista));
}

#[derive(Clone, Copy, PartialEq)]
enum Autor {
    Orquestrador,
    Voce,
}

struct Balao<'a> {
    autor: Autor,
    texto: &'a str,
}

/// Do mais velho ao mais novo. O pedido cuja âncora já saiu da lista vai para o começo.
fn sequencia<'a>(agora: &'a [String], enviadas: &'a [Enviada]) -> Vec<Balao<'a>> {
    let mut v: Vec<Balao> = enviadas.iter().filter(|e| !agora.contains(&e.ancora)).map(|e| Balao { autor: Autor::Voce, texto: &e.texto }).collect();
    for linha in agora.iter().rev() {
        v.push(Balao { autor: Autor::Orquestrador, texto: linha });
        v.extend(enviadas.iter().filter(|e| &e.ancora == linha).map(|e| Balao { autor: Autor::Voce, texto: &e.texto }));
    }
    v
}

/// O contorno do balão, com o canto de baixo do lado de quem fala cortado.
fn contorno(c: &Pincel, x: f32, y: f32, w: f32, autor: Autor) -> Vec<Pos2> {
    let p = |dx: f32, dy: f32| c.pt(x + dx, y + dy);
    match autor {
        Autor::Orquestrador => vec![p(0.0, 0.0), p(w, 0.0), p(w, ALTO), p(CORTE, ALTO), p(0.0, ALTO - CORTE)],
        Autor::Voce => vec![p(0.0, 0.0), p(w, 0.0), p(w, ALTO - CORTE), p(w - CORTE, ALTO), p(0.0, ALTO)],
    }
}

#[allow(clippy::too_many_arguments)]
fn balao(ui: &Ui, c: &Pincel, b: &Balao, x: f32, y: f32, w: f32, mais_novo: bool, acordado: bool) {
    let (cor, fundo, borda) = match b.autor {
        Autor::Orquestrador => (if mais_novo { TINTA } else { MEDIO }, CAMPO, BAIXO.gamma_multiply(0.75)),
        Autor::Voce => (TINTA, QUADRA.gamma_multiply(0.85), MEDIO.gamma_multiply(0.8)),
    };
    let letra = Letra::nova(10.5, cor);
    let recuo = if b.autor == Autor::Orquestrador && mais_novo && acordado { 22.0 } else { 14.0 };
    let (texto, cortou) = c.cortado(b.texto, (w * LARGURA_MAX - recuo - 12.0).max(60.0), &letra);
    let largura = (c.largura(&texto, &letra) + recuo + 12.0).round();
    let x = if b.autor == Autor::Voce { x + w - largura } else { x };
    c.p.add(Shape::convex_polygon(contorno(c, x, y, largura, b.autor), fundo, Stroke::new(1.0, borda)));
    if recuo > 14.0 {
        super::super::agentes_pecas::ponto(c, x + 12.0, y + ALTO / 2.0, AZUL, true);
    }
    c.texto(&texto, x + recuo, y + 16.0, letra);
    if cortou {
        let _ = ui.interact(c.caixa(x, y, largura, ALTO), ui.scope_id().with(("balao", b.texto)), Sense::hover()).on_hover_text(b.texto);
    }
}

/// «Digitando…»: três pontos que sobem e descem um depois do outro.
fn digitando(c: &Pincel, x: f32, y: f32) {
    let largura = 50.0;
    c.p.add(Shape::convex_polygon(contorno(c, x, y, largura, Autor::Orquestrador), CAMPO, Stroke::new(1.0, AZUL.gamma_multiply(0.6))));
    let fase = mov::fase(c.p.ctx(), 1.2);
    for i in 0..3 {
        let onda = mov::onda((fase - i as f32 * 0.14).rem_euclid(1.0));
        c.p.circle_filled(c.pt(x + 16.0 + i as f32 * 9.0, y + ALTO / 2.0 - onda * 2.5), 2.1, AZUL.gamma_multiply(0.4 + 0.6 * onda));
    }
}

/// A conversa dentro de `(x, y, w, h)`: o mais novo fica embaixo, e os mais velhos se apagam para cima.
pub fn desenhar(ui: &mut Ui, c: &Pincel, o: &Orquestrador, enviadas: &[Enviada], x: f32, y: f32, w: f32, h: f32) {
    let ctx = ui.ctx().clone();
    let lista = sequencia(&o.agora, enviadas);
    let vagas = ((h + VAO) / (ALTO + VAO)) as usize;
    let mut topo = y + h - ALTO;
    if o.acordado {
        let t = mov::chegada(&ctx, "orq-digitando", 0.3);
        digitando(&mov::Entra::nova(c, t).pincel(), x, topo);
        topo -= ALTO + VAO;
    }
    if lista.is_empty() && !o.acordado {
        let letra = Letra::nova(10.5, MEDIO);
        let largura = c.largura(tr("Nada acontecendo agora."), &letra) + 28.0;
        let topo = y + (h - ALTO) / 2.0;
        let pontos = contorno(c, x, topo, largura, Autor::Orquestrador);
        c.p.extend(Shape::dashed_line(&[pontos.clone(), vec![pontos[0]]].concat(), Stroke::new(1.0, BAIXO.gamma_multiply(0.6)), 3.0, 3.0));
        c.texto(tr("Nada acontecendo agora."), x + 14.0, topo + 16.0, letra);
        return;
    }
    let livres = vagas.saturating_sub(usize::from(o.acordado));
    for (k, b) in lista.iter().rev().take(livres).enumerate() {
        let t = mov::chegada(&ctx, ("balao", b.texto), 0.38);
        let envelhece = 1.0 - 0.24 * k as f32;
        let entra = mov::Entra::com_alfa(c, t, envelhece.max(0.25));
        balao(ui, &entra.pincel(), b, x, topo, w, k == 0, o.acordado);
        topo -= ALTO + VAO;
    }
}
