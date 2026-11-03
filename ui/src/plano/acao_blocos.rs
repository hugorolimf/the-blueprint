//! Cada plano é um bloco, e a ficha dele mostra o que é dele; aqui fica o desenho do que é da ação inteira, na mesa dela. O plano é o título, com o que falta decidir à direita. Embaixo, duas colunas: à esquerda o que se decide (decisões e perguntas),
//! à direita o que se confere (telas e checkpoints). Tela ampliada e tabela usam a largura toda. Em largura curta, ou com um lado só, tudo fica numa coluna.
//! O que não aponta para nenhum plano é da ação inteira e abre o painel, em «Da ação»; checkpoint assim fica no fim. A tela é a exceção: ela nunca vira seção, e vai para o plano que mais combina (`dono_da_tela`). Medir e desenhar saem de `Medidas`.

use super::super::acao_estilo::{alfa, etiqueta, largura_etiqueta, LETRA_PLANO, PASSO_PLANO};
use super::super::detalhe;
use super::super::linha::{Letra, Pincel};
use super::super::modelo::{Acao, Checkpoint, Decisao, Detalhe, Folha, Pergunta, Plano, Situacao, Tabela};
use super::super::perguntas;
use super::super::revisao::{self, Alvo, EstadoPeca};
use super::obra_formas::Forma;
use super::{decisoes, planos, prancha, rotulo, tempo, Saida};
use crate::fonte::Peso;
use crate::i18n::{tr, tr_com};
use crate::planta::Texto;
use crate::tema::{AZUL, MEDIO, TINTA, VERDE};
use egui::{Painter, Ui};

/// A cabeça do bloco: o fio, o vão e, embaixo do plano, a linha do que tem nele.
const TOPO: f32 = 18.0;
const ALTO_META: f32 = 22.0;
const VAO_CABECA: f32 = 26.0;
/// O lugar de «Para aprovar», à direita do plano.
const RESERVA_ETIQUETA: f32 = 170.0;
const VAO_PECA: f32 = 28.0;
const VAO_COLUNAS: f32 = 36.0;
/// Quanto a coluna do que se decide leva: é onde há mais texto.
const FRACAO_ESQUERDA: f32 = 0.58;
/// Abaixo disso, duas colunas deixariam o texto apertado.
const LARGURA_DUAS_COLUNAS: f32 = 780.0;
/// O rótulo «Telas · n» até a primeira tela.
const ALTO_ROTULO_TELAS: f32 = 20.0;

/// Um plano e o que é dele. `plano` vazio: o que é da ação inteira.
#[derive(Default)]
pub struct Bloco<'a> {
    pub plano: Option<Plano>,
    pub telas: Vec<&'a Detalhe>,
    pub decisoes: Vec<&'a Decisao>,
    pub perguntas: Vec<&'a Pergunta>,
    pub tabelas: Vec<(&'a Tabela, bool)>,
    pub checkpoints: Vec<&'a Checkpoint>,
}

impl Bloco<'_> {
    fn vazio(&self) -> bool {
        self.plano.is_none() && self.telas.is_empty() && self.decisoes.is_empty() && self.perguntas.is_empty() && self.tabelas.is_empty() && self.checkpoints.is_empty()
    }

    /// Quanto o usuário ainda tem a decidir aqui, e se há algo a decidir.
    pub fn a_decidir(&self) -> (usize, bool) {
        let espera = |r: Option<&super::super::modelo::Revisao>| revisao::estado_de(r) == EstadoPeca::Pendente;
        let telas = self.telas.iter().filter(|d| d.pede_aprovacao);
        let decs = self.decisoes.iter().filter(|d| d.pede_aprovacao);
        let tem = telas.clone().count() + decs.clone().count() + self.perguntas.len() > 0;
        let n = telas.filter(|d| espera(d.revisao.as_ref())).count() + decs.filter(|d| espera(d.revisao.as_ref())).count() + self.perguntas.iter().filter(|q| q.resposta.is_empty()).count();
        (n, tem)
    }

    /// O que tem no bloco: uma linha embaixo do plano.
    pub fn contagem(&self) -> String {
        let itens = [("Telas {n}", self.telas.len()), ("Decisões {n}", self.decisoes.len()), ("Perguntas {n}", self.perguntas.len()), ("Checkpoints {n}", self.checkpoints.len())];
        itens.iter().filter(|(_, n)| *n > 0).map(|(t, n)| tr_com(t, &[("n", &n.to_string())])).collect::<Vec<_>>().join(" · ")
    }
}

/// O painel organizado: primeiro o que é da ação, depois um bloco por plano, e no fim os checkpoints da ação.
pub struct Montagem<'a> {
    pub blocos: Vec<Bloco<'a>>,
    pub soltos: Vec<&'a Checkpoint>,
}

/// Os planos como ficariam ao aprovar: na ordem da versão pendente, com o texto novo. O plano que chega ainda não pode começar.
pub fn planos_da(a: &Acao) -> Vec<Plano> {
    let Some(d) = detalhe::diff_vivo(a).filter(|d| !d.linhas.is_empty()) else { return a.planos.clone() };
    d.linhas.iter().filter(|l| !l.referencia.is_empty()).map(|l| {
        let mut q = a.planos.iter().find(|q| q.referencia == l.referencia).cloned().unwrap_or_else(|| Plano { id: l.referencia.clone(), referencia: l.referencia.clone(), situacao: Situacao::Revisao, ..Plano::default() });
        if !l.texto.is_empty() {
            q.texto = l.texto.clone();
        }
        q
    }).collect()
}

/// O plano que mostra uma tela sem dono, contado a partir de 1; 0 só se a ação não tem plano. Nem toda proposta diz de quem é a tela, e ela precisa estar no desenho: ganha quem cita as palavras dela, depois quem fala em tela, depois o primeiro. É só onde aparecer: o `plano` guardado não muda.
fn dono_da_tela(planos: &[Plano], d: &Detalhe) -> usize {
    let texto = format!("{} {}", d.titulo, d.legenda).to_lowercase();
    let palavras: Vec<&str> = texto.split(|c: char| !c.is_alphanumeric()).filter(|w| w.chars().count() >= 4).collect();
    let nota = |q: &Plano| {
        let t = q.texto.to_lowercase();
        let citadas = palavras.iter().filter(|w| t.contains(**w)).count() * 2;
        let fala_de_tela = ["tela", "screen", "desenh", "mostr", "draw"].iter().any(|w| t.contains(w));
        citadas + usize::from(fala_de_tela) * 2 + usize::from(Forma::do_plano(q) == Forma::Tela)
    };
    // `max_by_key` fica com o último empate, e o que vale é o primeiro
    planos.iter().enumerate().rev().max_by_key(|(_, q)| nota(q)).map_or(0, |(i, _)| i + 1)
}

/// O bloco do plano `plano`, contado a partir de 1 (o primeiro bloco é o da ação); 0 se nenhum plano tem essa referência.
fn lugar(blocos: &[Bloco], plano: &str) -> usize {
    blocos.iter().skip(1).position(|b| b.plano.as_ref().is_some_and(|q| q.referencia == plano)).map_or(0, |i| i + 1)
}

/// Separa as peças por plano. Peça que aponta para um plano que não está mais aqui (a versão foi reprovada, o plano saiu) volta a ser da ação: nunca some.
pub fn montar<'a>(f: &'a Folha, a: &'a Acao) -> Montagem<'a> {
    let planos = planos_da(a);
    // as telas acham o dono antes de os planos irem para os blocos (`dono_da_tela` lê o texto deles): assim cada plano é copiado uma vez só
    let telas: Vec<(usize, &Detalhe)> = detalhe::detalhes_da(a).into_iter().map(|d| {
        let dono = match planos.iter().position(|q| q.referencia == d.plano) {
            None => dono_da_tela(&planos, d),
            Some(i) => i + 1,
        };
        (dono, d)
    }).collect();
    let mut blocos: Vec<Bloco> = std::iter::once(Bloco::default()).chain(planos.into_iter().map(|q| Bloco { plano: Some(q), ..Bloco::default() })).collect();
    for (dono, d) in telas {
        blocos[dono].telas.push(d);
    }
    for d in decisoes::da_acao(f, a) {
        let i = lugar(&blocos, &d.plano);
        blocos[i].decisoes.push(d);
    }
    for q in perguntas::da_acao(f, a) {
        let i = lugar(&blocos, &q.plano);
        blocos[i].perguntas.push(q);
    }
    for t in prancha::tabelas_da(a) {
        let i = lugar(&blocos, &t.0.plano);
        blocos[i].tabelas.push(t);
    }
    let mut soltos = Vec::new();
    for k in &a.checkpoints {
        match lugar(&blocos, &k.plano) {
            0 => soltos.push(k),
            i => blocos[i].checkpoints.push(k),
        }
    }
    blocos.retain(|b| !b.vazio());
    Montagem { blocos, soltos }
}

/// Quanto o usuário ainda tem a decidir na ação inteira.
pub fn pendentes(m: &Montagem) -> usize {
    m.blocos.iter().map(|b| b.a_decidir().0).sum()
}

/// A soma do que existe, com o vão só entre um e outro.
fn pilha(alturas: &[f32]) -> f32 {
    let (soma, n) = alturas.iter().filter(|h| **h > 0.0).fold((0.0, 0), |(s, n), h| (s + h, n + 1));
    soma + VAO_PECA * (n as f32 - 1.0).max(0.0)
}

/// Onde cada coisa do bloco fica, nesta largura: medir e desenhar saem daqui.
struct Medidas<'b, 'a> {
    plano: f32,
    ampliadas: Vec<&'a Detalhe>,
    pequenas: Vec<&'a Detalhe>,
    alto_ampliadas: f32,
    alto_tabelas: f32,
    /// decisões e perguntas
    esquerda: [f32; 2],
    /// telas e checkpoints
    direita: [f32; 2],
    w_e: f32,
    w_d: f32,
    x_d: f32,
    duas: bool,
    b: &'b Bloco<'a>,
}

impl Medidas<'_, '_> {
    fn corpo(&self) -> f32 {
        let (e, d) = (pilha(&self.esquerda), pilha(&self.direita));
        if self.duas { e.max(d) } else { pilha(&[e, d]) }
    }
    fn total(&self) -> f32 {
        let corpo = self.corpo();
        let baixo = pilha(&[self.alto_ampliadas, corpo, self.alto_tabelas]);
        TOPO + self.plano + ALTO_META + if baixo > 0.0 { VAO_CABECA + baixo } else { 0.0 }
    }
}

fn medir<'b, 'a>(p: &Painter, f: &Folha, a: &Acao, b: &'b Bloco<'a>, w: f32) -> Medidas<'b, 'a> {
    let (ampliadas, pequenas): (Vec<&Detalhe>, Vec<&Detalhe>) = b.telas.iter().copied().partition(|d| detalhe::ampliada(p.ctx(), a, d));
    let tem_e = !b.decisoes.is_empty() || !b.perguntas.is_empty();
    let tem_d = !pequenas.is_empty() || !b.checkpoints.is_empty();
    let duas = tem_e && tem_d && w >= LARGURA_DUAS_COLUNAS;
    let w_e = if duas { (w - VAO_COLUNAS) * FRACAO_ESQUERDA } else { w };
    let (x_d, w_d) = if duas { (w_e + VAO_COLUNAS, w - w_e - VAO_COLUNAS) } else { (0.0, w) };
    let telas = if pequenas.is_empty() { 0.0 } else { ALTO_ROTULO_TELAS + detalhe::altura(p, f, a, &pequenas, w_d) };
    Medidas {
        plano: b.plano.as_ref().map_or(PASSO_PLANO, |q| planos::altura_plano(p, q, w - RESERVA_ETIQUETA)),
        alto_ampliadas: detalhe::altura(p, f, a, &ampliadas, w),
        alto_tabelas: prancha::altura_tabelas(&b.tabelas),
        esquerda: [decisoes::altura(p, f, &b.decisoes, w_e), perguntas::altura(p, &b.perguntas, w_e)],
        direita: [telas, tempo::altura(p, &b.checkpoints, w_d)],
        ampliadas,
        pequenas,
        w_e,
        w_d,
        x_d,
        duas,
        b,
    }
}

/// Quanto um bloco ocupa nesta largura.
pub fn altura_bloco(p: &Painter, f: &Folha, a: &Acao, b: &Bloco, w: f32) -> f32 {
    medir(p, f, a, b, w).total()
}

/// A cabeça do bloco: o fio de cima, o plano como título e, à direita, quanto falta decidir. Embaixo, o que tem no bloco.
#[allow(clippy::too_many_arguments)]
fn cabeca(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, m: &Medidas, x: f32, y: f32, w: f32) -> Option<Saida> {
    let b = m.b;
    c.fio(x, y, x + w, y, alfa(TINTA, 0.32), 1.0);
    let topo = y + TOPO;
    let mut saida = None;
    let mut recuo = 0.0;
    match &b.plano {
        Some(q) => {
            recuo = planos::recuo(c.p, q);
            if matches!(planos::cartao_plano(ui, c, alvo, f, a, q, (x, topo, w - RESERVA_ETIQUETA)), Some(planos::Pede::Agentes)) {
                saida = Some(Saida::Agentes);
            }
        }
        None => {
            Texto::novo(tr("Da ação"), LETRA_PLANO).peso(Peso::Negrito).cor(TINTA).em(c.p, c.pt(x, topo));
        }
    }
    let (n, tem) = b.a_decidir();
    if tem {
        let (texto, cor) = if n > 0 { (tr_com("Para aprovar: {n}", &[("n", &n.to_string())]), AZUL) } else { (tr("Tudo decidido").to_string(), VERDE) };
        etiqueta(c, x + w - largura_etiqueta(c, &texto), topo + 5.0, &texto, Some(cor), n > 0);
    }
    c.texto(&b.contagem(), x + recuo, topo + m.plano + 12.0, Letra::nova(11.0, MEDIO));
    saida
}

#[allow(clippy::too_many_arguments)]
pub fn um_bloco(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &Acao, b: &Bloco, x: f32, y: f32, w: f32) -> Option<Saida> {
    let m = medir(c.p, f, a, b, w);
    let saida = cabeca(ui, c, alvo, f, a, &m, x, y, w);
    let mut yy = y + TOPO + m.plano + ALTO_META + VAO_CABECA;
    if m.alto_ampliadas > 0.0 {
        detalhe::desenhar(ui, c, alvo, f, a, &m.ampliadas, x, yy, w);
        yy += m.alto_ampliadas + VAO_PECA;
    }
    let corpo = m.corpo();
    // à esquerda, o que se decide
    let mut ye = yy;
    if m.esquerda[0] > 0.0 {
        decisoes::desenhar(ui, c, alvo, f, &b.decisoes, x, ye, m.w_e);
        ye += m.esquerda[0] + VAO_PECA;
    }
    if m.esquerda[1] > 0.0 {
        perguntas::desenhar(ui, c, alvo, &b.perguntas, x, ye, m.w_e);
    }
    // à direita, o que se confere; em uma coluna só, vem depois
    let mut yd = if m.duas || pilha(&m.esquerda) == 0.0 { yy } else { yy + pilha(&m.esquerda) + VAO_PECA };
    let xd = x + m.x_d;
    if m.direita[0] > 0.0 {
        rotulo(c, xd, yd, &tr_com("Telas · {n}", &[("n", &m.pequenas.len().to_string())]), m.w_d);
        detalhe::desenhar(ui, c, alvo, f, a, &m.pequenas, xd, yd + ALTO_ROTULO_TELAS, m.w_d);
        yd += m.direita[0] + VAO_PECA;
    }
    if m.direita[1] > 0.0 {
        tempo::desenhar(ui, c, alvo, a, &b.checkpoints, (xd, yd, m.w_d));
    }
    if m.alto_tabelas > 0.0 {
        let y_tab = yy + if corpo > 0.0 { corpo + VAO_PECA } else { 0.0 };
        prancha::tabelas(ui, c, &b.tabelas, x, y_tab, w);
    }
    saida
}

#[cfg(test)]
#[path = "acao_blocos_testes.rs"]
mod tests;
