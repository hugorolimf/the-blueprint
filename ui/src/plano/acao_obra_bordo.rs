//! O painel de bordo, no canto do palco, e a linha do próximo passo: tudo o que era mesa e aba agora é figura e clique em cima da obra.
//! Cada casa voa até a peça que falta. O título abre a mesa da ação, e «Liberar» inicia só quem já pode; com decisão em aberto, confirma antes.

use super::super::acao_estilo::alfa;
use super::super::acao_movimento::{clarao, marcar_instante};
use super::super::caixinha;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha, Plano, Situacao};
use super::super::revisao::Alvo;
use super::blocos::{self, Bloco, Montagem};
use super::figura::{feito, Peca};
use super::movimentos::figura_viva;
use super::obra_pecas::pecas_de;
use super::obra_regra::{self as regra, Casa, Passo};
use super::{mesa_acao, obra_icones as icones, obra_mergulho as mergulho};
use super::obra_icones::Icone;
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, AZUL, CAMPO, DESTAQUE, MEDIO, TINTA, VERDE, VERMELHO};
use egui::{Color32, Context, CursorIcon, Id, Pos2, Sense, Ui};

const LARGURA: f32 = 450.0;
const ALTO_TITULO: f32 = 26.0;
const ALTO_CASAS: f32 = 38.0;
const ALTO_CONFIRMA: f32 = 70.0;
const VAO_CASAS: f32 = 10.0;
pub const ALTO_PROXIMO: f32 = 26.0;

fn id_confirma(a: &Acao) -> Id {
    Id::unique(("plano-bordo-confirma", &a.referencia))
}

/// Leva o olhar até o que a casa pede: à peça, voando; o que é da ação inteira está na mesa dela.
pub fn ir(ctx: &Context, a: &Acao, casa: &Casa) {
    match &casa.plano {
        Some(r) => {
            mergulho::entrar(ctx, a, r);
            mergulho::escolher(ctx, a, &casa.chave);
        }
        None => mesa_acao::abrir(ctx, a),
    }
}

fn liberar(ctx: &Context, a: &Acao, f: &Folha, alvo: Alvo) {
    caixinha::marcar(ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &regra::liberaveis(f, &a.planos));
    marcar_instante(ctx, ("plano-bordo-liberou", &a.referencia));
    ctx.data_mut(|d| d.remove_temp::<bool>(id_confirma(a)));
}

fn confirmacao(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Montagem, pendentes: usize, x: f32, y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    c.ret(x, y, w, ALTO_CONFIRMA, Some(CAMPO), Some((TINTA, 1.2)));
    c.texto(&tr_com("Faltam {n} decisões. Liberar mesmo assim?", &[("n", &pendentes.to_string())]), x + 10.0, y + 22.0, Letra::nova(11.5, TINTA).forte());
    let sim = Botao { texto: tr("Liberar mesmo assim"), tam: 10.0, cheio: true, ativo: true, altura: 28.0, pad_x: 12.0 };
    let nao = Botao { texto: tr("Decidir primeiro"), tam: 10.0, cheio: false, ativo: true, altura: 28.0, pad_x: 12.0 };
    let (ws, wn) = (sim.largura(c.p), nao.largura(c.p));
    if botao(ui, c.pt(x + w - ws - 10.0, y + 34.0), &sim).clicked() {
        liberar(&ctx, a, f, alvo);
    }
    if botao(ui, c.pt(x + w - ws - wn - 18.0, y + 34.0), &nao).clicked() {
        ctx.data_mut(|d| d.remove_temp::<bool>(id_confirma(a)));
        if let Some(casa) = regra::casas(m.blocos.iter()).iter().find(|k| k.pede > 0 && k.tipo != Peca::Checkpoint) {
            ir(&ctx, a, casa);
        }
    }
}

/// O painel, encostado em `x_dir` e `y_fim`.
#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Montagem, planos: &[&Plano], x_dir: f32, y_fim: f32, w_palco: f32) {
    let ctx = ui.ctx().clone();
    let w = LARGURA.min(w_palco);
    let h = ALTO_TITULO + ALTO_CASAS;
    let (x, y) = (x_dir - w, y_fim - h);
    let chave = a.referencia.as_str();
    c.ret(x, y, w, h, Some(CAMPO.gamma_multiply(0.96)), Some((TINTA, 1.2)));
    c.fio(x, y + ALTO_TITULO, x + w, y + ALTO_TITULO, TINTA, 1.0);

    let area = c.caixa(x, y, w, ALTO_TITULO);
    let titulo = ui.interact(area, ui.scope_id().with(("plano-bordo-titulo", chave)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
    if titulo.hovered() {
        c.p.rect_filled(area, 0, alfa(TINTA, 0.12));
    }
    let forte = Letra::nova(10.5, TINTA).forte();
    c.texto(&c.cortado(&format!("{} · {}", a.referencia, a.nome), w - 34.0, &forte).0, x + 10.0, y + 17.0, forte);
    c.texto(">", x + w - 16.0, y + 17.0, Letra::nova(11.0, MEDIO));
    if titulo.on_hover_text(tr("Abrir a mesa da ação")).clicked() {
        mesa_acao::alternar(&ctx, a);
    }

    let ym = y + ALTO_TITULO + ALTO_CASAS * 0.5;
    let mut xx = x + 10.0;
    let feitos = planos.iter().filter(|q| feito(q)).count();
    icones::desenhar(c.p, c.pt(xx + 7.0, ym), icones::LADO, Icone::Obra, if feitos == planos.len() { VERDE } else { MEDIO });
    let de_pe = format!("{feitos}/{}", planos.len());
    let dica = tr_com("{n} de {total} de pé", &[("n", &feitos.to_string()), ("total", &planos.len().to_string())]);
    let wp = c.texto(&de_pe, xx + 18.0, ym + 4.0, Letra::nova(11.0, TINTA).forte());
    let _ = ui.interact(c.caixa(xx, ym - 11.0, 18.0 + wp, 22.0), ui.scope_id().with(("plano-bordo-feito", chave)), Sense::hover()).on_hover_text(dica);
    xx += 18.0 + wp + VAO_CASAS + 4.0;
    for (i, casa) in regra::casas(m.blocos.iter()).iter().enumerate() {
        if icones::casa(ui, c, (chave, "bordo", i), casa, xx, ym) {
            ir(&ctx, a, casa);
        }
        xx += icones::largura_casa(c, casa) + VAO_CASAS;
    }
    let trabalhando: Vec<&&Plano> = planos.iter().filter(|q| q.situacao == Situacao::Trabalhando).collect();
    if let Some(q) = trabalhando.first() {
        let area = c.caixa(xx, ym - 13.0, 44.0, 26.0);
        figura_viva(ui, q, c.caixa(xx, ym - 11.0, 26.0, 20.0));
        c.texto(&trabalhando.len().to_string(), xx + 28.0, ym + 4.0, Letra::nova(11.0, AZUL).forte());
        let r = ui.interact(area, ui.scope_id().with(("plano-bordo-agentes", chave)), Sense::click()).on_hover_cursor(CursorIcon::PointingHand);
        if r.on_hover_text(tr_com("Agentes · {n} trabalhando", &[("n", &trabalhando.len().to_string())])).clicked() {
            mergulho::entrar(&ctx, a, &q.referencia);
        }
    }

    let n = regra::liberaveis(f, &a.planos).len();
    let pendentes = blocos::pendentes(m);
    let texto = tr_com("Liberar {n}", &[("n", &n.to_string())]);
    let b = Botao { texto: &texto, tam: 10.0, cheio: n > 0, ativo: n > 0, altura: 26.0, pad_x: 10.0 };
    let bw = b.largura(c.p);
    let liberou = clarao(&ctx, ("plano-bordo-liberou", chave), 2.0);
    if liberou > 0.0 {
        c.p.rect_filled(c.caixa(x + w - bw - 14.0, y + ALTO_TITULO, bw + 14.0, ALTO_CASAS), 0, alfa(VERDE, 0.3 * liberou));
    }
    if botao(ui, c.pt(x + w - bw - 7.0, ym - 13.0), &b).clicked() {
        if pendentes > 0 {
            let aberta = ctx.data(|d| d.get_temp::<bool>(id_confirma(a))).unwrap_or(false);
            ctx.data_mut(|d| d.insert_temp(id_confirma(a), !aberta));
        } else {
            liberar(&ctx, a, f, alvo);
        }
    }
    if ctx.data(|d| d.get_temp::<bool>(id_confirma(a))).unwrap_or(false) {
        confirmacao(ui, c, alvo, f, a, m, pendentes, x, y - ALTO_CONFIRMA - 6.0, w);
    }
}

/// O que dizer do passo: a frase, a cor e o sinal.
fn frase(passo: &Passo, lista: &[(&Plano, &Bloco)]) -> (String, Color32, Icone) {
    let r = |i: usize| lista[i].0.referencia.as_str();
    match passo {
        Passo::Parou(i) => (tr_com("{ref} parou: veja o motivo", &[("ref", r(*i))]), VERMELHO, Icone::Parou),
        Passo::Decidir(i, chave) => {
            let pecas = pecas_de(lista[*i].1);
            let Some(p) = pecas.iter().find(|p| p.chave == *chave) else { return (String::new(), AZUL, Icone::Fila) };
            let modelo = match p.tipo {
                Peca::Pergunta => "Responder {id}: {nome}",
                Peca::Tela => "Aprovar {id}: {nome}",
                _ => "Decidir {id}: {nome}",
            };
            (tr_com(modelo, &[("id", &p.numero), ("nome", &p.nome)]), DESTAQUE, Icone::Peca(p.tipo, p.marca))
        }
        Passo::Iniciar(i) => (tr_com("Iniciar {ref}", &[("ref", r(*i))]), DESTAQUE, Icone::Iniciar),
        Passo::EmObra(i) => (tr_com("{ref} em obra", &[("ref", r(*i))]), DESTAQUE, Icone::Obra),
        Passo::Espera(r) => (tr_com("Esperando {ref}, de outra ação", &[("ref", r)]), AMBAR, Icone::Cadeado),
        Passo::Aguarda => (tr("Esperando os agentes").to_string(), MEDIO, Icone::Fila),
        Passo::Pronta => (tr("Tudo de pé").to_string(), VERDE, Icone::Visto),
    }
}

/// A linha do próximo passo, no alto do palco: uma frase e um clique. Devolve o plano de fora que ela pediu.
#[allow(clippy::too_many_arguments)]
pub fn proximo(ui: &mut Ui, c: &Pincel, f: &Folha, a: &Acao, lista: &[(&Plano, &Bloco)], x: f32, y: f32, max: f32) -> Option<(String, Pos2)> {
    let ctx = ui.ctx().clone();
    let passo = regra::proximo(f, lista);
    let (texto, cor, icone) = frase(&passo, lista);
    let cabeca = Letra::nova(10.0, cor);
    let l = Letra::nova(11.0, cor);
    let rotulo = tr("Próximo");
    let w_cabeca = c.rotulo(rotulo, -10000.0, -10000.0, cabeca) + 10.0;
    let corte = c.cortado(&texto, (max - w_cabeca - icones::LADO - 34.0).max(40.0), &l).0;
    let w = 10.0 + w_cabeca + icones::LADO + 6.0 + c.largura(&corte, &l) + 12.0;
    let area = c.caixa(x, y, w, ALTO_PROXIMO);
    let leva = !matches!(passo, Passo::Aguarda | Passo::Pronta);
    let r = ui.interact(area, ui.scope_id().with(("plano-obra-proximo", &a.referencia)), if leva { Sense::click() } else { Sense::hover() });
    c.ret(x, y, w, ALTO_PROXIMO, Some(alfa(CAMPO, if r.hovered() && leva { 1.0 } else { 0.85 })), None);
    c.fio(x, y, x, y + ALTO_PROXIMO, cor, 2.0);
    c.rotulo(rotulo, x + 10.0, y + 17.0, cabeca);
    icones::desenhar(c.p, c.pt(x + 10.0 + w_cabeca + icones::LADO * 0.5, y + ALTO_PROXIMO * 0.5), icones::LADO, icone, cor);
    c.texto(&corte, x + 10.0 + w_cabeca + icones::LADO + 6.0, y + 17.0, l);
    if leva {
        ctx.set_cursor_icon(if r.hovered() { CursorIcon::PointingHand } else { ctx.output(|o| o.cursor_icon) });
    }
    let clicou = r.clicked();
    let de = area.center();
    let _ = r.on_hover_text(texto);
    if !clicou {
        return None;
    }
    match &passo {
        Passo::Parou(i) | Passo::Iniciar(i) | Passo::EmObra(i) => mergulho::entrar(&ctx, a, &lista[*i].0.referencia),
        Passo::Decidir(i, chave) => {
            mergulho::entrar(&ctx, a, &lista[*i].0.referencia);
            mergulho::escolher(&ctx, a, chave);
        }
        Passo::Espera(r) => return Some((r.clone(), de)),
        Passo::Aguarda | Passo::Pronta => {}
    }
    None
}
