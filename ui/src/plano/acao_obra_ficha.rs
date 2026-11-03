//! A ficha de um plano: tudo o que é dele, num lugar só. O cartão com o estado e o agente, de quem ele depende e o que libera, o que ele entrega, onde mexe e como se faz, a lista de peças numeradas (`obra_lista`), que abre no lugar, e os checkpoints.
//! Serve ao mergulho e à vista expandida da parte. Plano sem nenhum detalhe não fica mudo: diz isso e oferece pedir ao orquestrador.

use super::super::acao_estilo::CORPO;
use super::super::acao_movimento::{clarao, marcar_instante};
use super::super::caixinha::{self, Clique};
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Folha, Plano};
use super::super::orquestrador::corpo_mensagem;
use super::super::revisao::Alvo;
use super::blocos::Bloco;
use super::decidir;
use super::obra_icones::{self as icones, Icone};
use super::obra_fora as fora;
use super::{bloco, lista, obra_mergulho as mergulho, obra_regra as regra, planos as cartao, rotulo, tempo, Saida};
use crate::i18n::{tr, tr_com};
use crate::planta::{botao, Botao};
use crate::tema::{AMBAR, MEDIO, VERDE};
use egui::{Context, Id, Painter, Ui};

const VAO: f32 = 14.0;
const ALTO_LIGACOES: f32 = 34.0;
const ALTO_ROTULO: f32 = 22.0;
const LETRA_INFO: f32 = 12.0;
const PASSO_INFO: f32 = 18.0;
const LINHAS_RECOLHIDO: usize = 3;
const ALTO_LER: f32 = 30.0;
const ALTO_INICIAR: f32 = 44.0;
const ALTO_SEM_DETALHES: f32 = 72.0;

/// Quem espera por este plano, em qualquer ação da folha.
fn liberados<'f>(f: &'f Folha, q: &Plano) -> Vec<&'f str> {
    f.projetos.iter().flat_map(|p| &p.acoes).flat_map(|a| &a.planos).filter(|o| o.depende_de.contains(&q.referencia)).map(|o| o.referencia.as_str()).collect()
}

fn tem_ligacoes(f: &Folha, q: &Plano) -> bool {
    !q.depende_de.is_empty() || !liberados(f, q).is_empty()
}

/// O que o plano é, além do título. O que está vazio não aparece.
fn informacoes(q: &Plano) -> Vec<(&'static str, &str)> {
    let v = [(tr("Entrega"), q.entrega.as_str()), (tr("Onde mexe"), q.area.as_str()), (tr("Como fazer"), q.instrucoes.as_str())];
    v.into_iter().filter(|(_, t)| !t.trim().is_empty()).collect()
}

fn id_ler(a: &Acao, q: &Plano) -> Id {
    Id::unique(("plano-ficha-ler", &a.referencia, &q.referencia))
}

/// Quantas linhas aparecem e se precisa do botão: texto longo vem recolhido.
fn linhas_de(ctx: &Context, a: &Acao, q: &Plano, total: usize) -> (usize, bool) {
    let aberto = ctx.data(|d| d.get_temp::<bool>(id_ler(a, q))).unwrap_or(false);
    (if aberto { total } else { total.min(LINHAS_RECOLHIDO) }, total > LINHAS_RECOLHIDO)
}

fn alto_informacoes(p: &Painter, a: &Acao, q: &Plano, w: f32) -> f32 {
    informacoes(q).iter().map(|(_, t)| {
        let (n, botao) = linhas_de(p.ctx(), a, q, bloco(t, LETRA_INFO, w, PASSO_INFO).galeria(p).rows.len());
        ALTO_ROTULO + n as f32 * PASSO_INFO + if botao { ALTO_LER } else { 0.0 } + VAO
    }).sum()
}

fn desenhar_informacoes(ui: &mut Ui, c: &Pincel, a: &Acao, q: &Plano, x: f32, mut y: f32, w: f32) {
    let ctx = ui.ctx().clone();
    for (nome, t) in informacoes(q) {
        rotulo(c, x, y + 10.0, nome, w);
        y += ALTO_ROTULO;
        let texto = bloco(t, LETRA_INFO, w, PASSO_INFO).cor(CORPO);
        let (n, pede_botao) = linhas_de(&ctx, a, q, texto.galeria(c.p).rows.len());
        let alto = n as f32 * PASSO_INFO;
        texto.em(&c.p.with_clip_rect(c.caixa(x, y, w, alto).intersect(c.p.clip_rect())), c.pt(x, y));
        y += alto;
        if pede_botao {
            let aberto = ctx.data(|d| d.get_temp::<bool>(id_ler(a, q))).unwrap_or(false);
            let b = Botao { texto: if aberto { tr("Ler menos") } else { tr("Ler tudo") }, tam: 10.0, cheio: false, ativo: true, altura: 24.0, pad_x: 9.0 };
            if botao(ui, c.pt(x, y + 3.0), &b).clicked() {
                ctx.data_mut(|d| d.insert_temp(id_ler(a, q), !aberto));
                crate::anima::mexer(&ctx, 250.0);
            }
            y += ALTO_LER;
        }
        y += VAO;
    }
}

/// Nada a mostrar além do título.
fn mudo(q: &Plano, b: &Bloco) -> bool {
    informacoes(q).is_empty() && decidir::todas(b).is_empty() && b.checkpoints.is_empty()
}

/// O plano mudo diz que é mudo e oferece o caminho: pedir os detalhes a quem escreveu.
fn sem_detalhes(ui: &mut Ui, c: &Pincel, alvo: Alvo, a: &Acao, q: &Plano, x: f32, y: f32) {
    let ctx = ui.ctx().clone();
    c.texto(tr("O agente ainda não detalhou este plano."), x, y + 12.0, Letra::nova(LETRA_INFO, MEDIO));
    let pedido = tr_com("Detalhe o plano {id} ({nome}): o que ele entrega, onde mexe e como fazer.", &[("id", &q.referencia), ("nome", &q.texto)]);
    let corpo = corpo_mensagem(alvo.ws, alvo.bp, &pedido);
    let b = Botao { texto: tr("Pedir detalhes ao orquestrador"), tam: 10.5, cheio: false, ativo: corpo.is_some(), altura: 30.0, pad_x: 12.0 };
    if let Some(corpo) = corpo.filter(|_| botao(ui, c.pt(x, y + 24.0), &b).clicked()) {
        alvo.estado.gesto(&ctx, alvo.servidor, "orquestrador", &corpo);
        marcar_instante(&ctx, ("plano-ficha-pediu", &a.referencia, &q.referencia));
    }
    if clarao(&ctx, ("plano-ficha-pediu", &a.referencia, &q.referencia), 3.0) > 0.0 {
        c.texto(tr("Pedido enviado ao orquestrador"), x + b.largura(c.p) + 10.0, y + 43.0, Letra::nova(10.5, VERDE));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn altura(p: &Painter, f: &Folha, a: &Acao, q: &Plano, b: &Bloco, w: f32, _completa: bool) -> f32 {
    let pecas = lista::altura(p, f, a, b, w);
    let checks = tempo::altura(p, &b.checkpoints, w);
    alto_iniciar(q) + cartao::altura_plano(p, q, w) + VAO
        + if tem_ligacoes(f, q) { ALTO_LIGACOES } else { 0.0 }
        + alto_informacoes(p, a, q, w)
        + if mudo(q, b) { ALTO_SEM_DETALHES } else { 0.0 }
        + if pecas > 0.0 { pecas + VAO } else { 0.0 }
        + if checks > 0.0 { checks + VAO } else { 0.0 }
}

fn alto_iniciar(q: &Plano) -> f32 {
    if caixinha::clique_de(q) == Some(Clique::Marcar) { ALTO_INICIAR } else { 0.0 }
}

/// O botão de iniciar: o plano anterior nem começou, e ele vira o caminho até lá. Com decisão em aberto, ainda dá para iniciar, mas o botão não é o principal e diz o que falta.
#[allow(clippy::too_many_arguments)]
fn iniciar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, q: &Plano, b: &Bloco, x: f32, y: f32, completa: bool) -> Option<Saida> {
    if alto_iniciar(q) == 0.0 {
        return None;
    }
    let ctx = ui.ctx().clone();
    let faltam = regra::sem_iniciar(f, q);
    if let Some(r) = faltam.first() {
        let t = tr_com("Inicie {ref} antes", &[("ref", r)]);
        let bt = Botao { texto: &t, tam: 10.5, cheio: false, ativo: true, altura: 32.0, pad_x: 14.0 };
        icones::desenhar(c.p, c.pt(x + 8.0, y + 16.0), icones::LADO, Icone::Cadeado, AMBAR);
        let clique = botao(ui, c.pt(x + 24.0, y), &bt);
        let saiu = clique.clicked().then(|| mergulho::ir_ao_plano(&ctx, f, a, r, clique.rect.center(), completa)).flatten();
        let _ = clique.on_hover_text(tr_com("Inicie {lista} antes: este plano depende dele.", &[("lista", &faltam.join(", "))]));
        return saiu;
    }
    let n = b.a_decidir().0;
    let com_aberto = tr_com("Iniciar com {n} em aberto", &[("n", &n.to_string())]);
    let bt = Botao { texto: if n > 0 { &com_aberto } else { tr("Iniciar plano") }, tam: 10.5, cheio: n == 0, ativo: true, altura: 32.0, pad_x: 14.0 };
    if botao(ui, c.pt(x, y), &bt).clicked() {
        caixinha::marcar(&ctx, alvo.estado, alvo.servidor, alvo.ws, alvo.bp, &[q.id.as_str()]);
    }
    None
}

/// Quem este plano espera e quem espera por ele, como caminhos para andar pela folha inteira: o clique leva até lá, mesmo em outra ação.
#[allow(clippy::too_many_arguments)]
fn ligacoes(ui: &mut Ui, c: &Pincel, f: &Folha, a: &Acao, q: &Plano, x: f32, y: f32, w: f32, completa: bool) -> Option<Saida> {
    let ctx = ui.ctx().clone();
    let espera: Vec<&str> = q.depende_de.iter().map(String::as_str).collect();
    let mut xx = x;
    let mut saida = None;
    for (nome, refs) in [(tr("Depende de"), espera), (tr("Libera"), liberados(f, q))] {
        if refs.is_empty() {
            continue;
        }
        xx += c.rotulo(nome, xx, y + 17.0, Letra::nova(10.0, MEDIO)) + 8.0;
        for r in refs {
            let t = if fora::pronto(f, r) { format!("{r} ✓") } else { r.to_string() };
            let b = Botao { texto: &t, tam: 10.0, cheio: false, ativo: true, altura: 26.0, pad_x: 9.0 };
            if xx + b.largura(c.p) > x + w {
                break;
            }
            let clique = botao(ui, c.pt(xx, y), &b);
            if clique.clicked() {
                saida = mergulho::ir_ao_plano(&ctx, f, a, r, clique.rect.center(), completa);
            }
            xx += b.largura(c.p) + 6.0;
        }
        xx += 12.0;
    }
    saida
}

#[allow(clippy::too_many_arguments)]
pub fn desenhar(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, q: &Plano, b: &Bloco, x: f32, y: f32, w: f32, completa: bool) -> Option<Saida> {
    let ctx = ui.ctx().clone();
    let mut saida = iniciar(ui, c, alvo, f, a, q, b, x, y, completa);
    let y = y + alto_iniciar(q);
    match cartao::cartao_plano(ui, c, alvo, f, a, q, (x, y, w)) {
        Some(cartao::Pede::Agentes) => saida = Some(Saida::Agentes),
        Some(cartao::Pede::Ver(r, de)) => saida = mergulho::ir_ao_plano(&ctx, f, a, &r, de, completa).or(saida),
        None => {}
    }
    let mut yy = y + cartao::altura_plano(c.p, q, w) + VAO;
    if tem_ligacoes(f, q) {
        saida = ligacoes(ui, c, f, a, q, x, yy, w, completa).or(saida);
        yy += ALTO_LIGACOES;
    }
    desenhar_informacoes(ui, c, a, q, x, yy, w);
    yy += alto_informacoes(c.p, a, q, w);
    if mudo(q, b) {
        sem_detalhes(ui, c, alvo, a, q, x, yy);
        yy += ALTO_SEM_DETALHES;
    }
    let pecas = lista::altura(c.p, f, a, b, w);
    if pecas > 0.0 {
        lista::desenhar(ui, c, alvo, f, a, Some(q), tr("Peças"), b, x, yy, w);
        yy += pecas + VAO;
    }
    tempo::desenhar(ui, c, alvo, a, &b.checkpoints, (x, yy, w));
    saida
}
