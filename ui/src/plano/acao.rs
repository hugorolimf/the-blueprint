//! A ação aberta no lugar, desenhada direto no papel da folha, sem cartão, como a seção aberta da folha original (`folha/acoes.rs`): tudo que se decide do plano se decide aqui. A altura é medida antes de empilhar, e `altura` e `painel` usam a mesma conta.
//! Quem usa: `empilhar` soma `ocupa` na ação aberta; `painel` desenha logo abaixo da linha dela; e `aplicar` faz o que o painel devolver, depois de a folha sair de empréstimo.
//! O painel abre crescendo (a altura anda junto com as linhas de baixo) e os blocos chegam em cascata; cada bloco mora no seu arquivo `acao_*.rs`.

use super::acao_estilo::rotulo_secao;
use super::acao_movimento::{cascata, com_alfa, desde_que_abriu};
use super::diff;
use super::escala::X0;
use super::folha::links;
use super::linha::{self, Pincel};
use super::modelo::{Acao as AcaoPlano, Folha};
use super::perguntas;
use super::revisao::Alvo;
use super::{Estado, Sub};
use crate::anima;
use crate::menu;
use crate::planta::Texto;
use crate::telas::Acao as Pedido;
use egui::{vec2, Painter, Ui};

#[path = "acao_cabeca.rs"]
mod cabecalho;
#[path = "acao_decisoes.rs"]
mod decisoes;
#[path = "acao_estado.rs"]
mod estado;
#[path = "acao_linha_tempo.rs"]
mod tempo;
#[path = "acao_planos.rs"]
mod planos;
#[path = "acao_prancha.rs"]
mod prancha;
#[path = "acao_ver.rs"]
mod ver;

pub const TEXTOS: &[(&str, &str)] = &[
    ("− Recolher", "− Collapse"),
    ("Recolher", "Collapse"),
    ("ver tudo", "see all"),
    ("ver menos", "see less"),
    ("sem data", "no date"),
    ("Planos · {feitos}/{total}", "Plans · {feitos}/{total}"),
    ("Checkpoints · {feitos}/{total}", "Checkpoints · {feitos}/{total}"),
    ("Decisões · {n}", "Decisions · {n}"),
    ("Detalhes · {n}", "Details · {n}"),
    ("Tabela", "Table"),
    ("Passou da data sem ser feito", "Past the date and not done"),
    ("Depende de {lista}", "Depends on {lista}"),
    ("Livre", "Free"),
    ("Revise a mudança", "Review the change"),
    ("Trabalhando", "Working"),
    ("Na fila", "In queue"),
    ("Esperando plano", "Waiting for a plan"),
    ("Esperando área", "Waiting for the area"),
    ("Parado", "Stopped"),
    ("O agente sumiu", "The agent disappeared"),
    ("Falhou ao iniciar", "Failed to start"),
    ("Pediu ajuda", "Asked for help"),
    ("Marcado como feito", "Marked as done"),
    ("O agente concluiu e marcou", "The agent finished and marked it"),
];

/// A largura máxima do conteúdo: em janela larga os botões continuam perto do que decidem.
const LARGURA_MAX: f32 = 1440.0;
/// O vão entre as duas colunas, o mesmo da folha original.
const VAO_COLUNAS: f32 = 30.0;
/// O vão antes de cada seção: maior que o de dentro dela.
const VAO_SECAO: f32 = 26.0;
const VAO_ABAIXO: f32 = 18.0;

/// O que o painel pede e só quem tem `vista` pode fazer. Marcar e revisar já foram ao servidor.
#[derive(Debug)]
pub enum Saida {
    Recolher,
    Agentes,
    Pedido(Pedido),
}

/// Faz o que o painel pediu; o que for de `app` volta como pedido.
pub fn aplicar(plano: &mut Estado, _a: &AcaoPlano, s: Saida) -> Option<Pedido> {
    match s {
        Saida::Recolher => plano.aberta = None,
        Saida::Agentes => plano.tela = Sub::Agentes,
        Saida::Pedido(p) => return Some(p),
    }
    None
}

/// As alturas de cada parte: medir e desenhar saem desta mesma conta.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Partes {
    pub cabeca: f32,
    /// o diff com o vão depois dele; zero sem diff
    pub diff: f32,
    /// a lista normal; zero com diff, porque o diff a substitui
    pub planos: f32,
    pub checkpoints: f32,
    pub decisoes: f32,
    pub perguntas: f32,
    pub tabelas: f32,
    pub detalhes: f32,
}

impl Partes {
    /// As duas colunas: planos e checkpoints, lado a lado como na folha original. Vale a maior.
    pub fn colunas(&self) -> f32 {
        self.planos.max(self.checkpoints)
    }
    /// Uma seção e o vão antes dela; zero quando ela não existe.
    fn secao(alto: f32) -> f32 {
        if alto > 0.0 { alto + VAO_SECAO } else { 0.0 }
    }
    pub fn total(&self) -> f32 {
        self.cabeca + self.diff + self.colunas() + Self::secao(self.decisoes) + Self::secao(self.perguntas) + Self::secao(self.tabelas) + Self::secao(self.detalhes) + 6.0
    }
}

/// As medidas do painel nesta largura da folha.
#[derive(Clone, Copy)]
pub(super) struct Geometria {
    pub x: f32,
    pub w: f32,
    /// a largura de cada coluna e onde a segunda começa
    pub coluna: f32,
    pub x_direita: f32,
}

impl Geometria {
    fn de(x1: f32) -> Geometria {
        let (x, w) = (X0, (x1 - X0).min(LARGURA_MAX));
        let coluna = (w - VAO_COLUNAS) / 2.0;
        Geometria { x, w, coluna, x_direita: x + coluna + VAO_COLUNAS }
    }
}

/// A letra do bloco de várias linhas: mesma trilha do resto do painel, e altura de linha fixa para a conta fechar.
pub fn bloco(s: &str, tam: f32, max: f32, passo: f32, linhas: usize) -> Texto<'_> {
    Texto::novo(s, tam).trilha(0.03).caixa(max.max(1.0)).lh(passo).linhas(linhas)
}

/// O título de uma seção: marca, caixa alta e um fio que some.
pub fn rotulo(c: &Pincel, x: f32, y: f32, t: &str, w: f32) {
    rotulo_secao(c, x, y, t, w);
}

fn lapis<'a>(c: &Pincel<'a>) -> super::caixinha::Lapis<'a> {
    super::caixinha::Lapis { p: c.p, o: c.o }
}

/// Os links que a folha abre: só `http` e `https`, como `folha::links`.
pub fn tem_links(lista: &[serde_json::Value]) -> bool {
    lista.iter().any(|l| l.get("url").and_then(serde_json::Value::as_str).is_some_and(|u| u.starts_with("http://") || u.starts_with("https://")))
}

fn medir(p: &Painter, f: &Folha, a: &AcaoPlano, g: &Geometria) -> Partes {
    let com_diff = a.diff.is_some();
    let coluna = g.coluna;
    Partes {
        cabeca: cabecalho::altura(p, a, g),
        diff: if com_diff { diff::altura(p, a, g.w) + 22.0 } else { 0.0 },
        planos: if com_diff { 0.0 } else { planos::altura_planos(p, a, coluna) },
        checkpoints: tempo::altura(a),
        decisoes: decisoes::altura(p, f, a, g.w),
        perguntas: perguntas::altura(p, f, a, g.w),
        tabelas: prancha::altura_tabelas(a),
        detalhes: prancha::altura_detalhes(p, f, a, g.w),
    }
}

/// A altura do painel inteiro, para empilhar antes de desenhar. `x1` é a borda direita da folha.
pub fn altura(p: &Painter, f: &Folha, a: &AcaoPlano, x1: f32) -> f32 {
    medir(p, f, a, &Geometria::de(x1)).total()
}

/// O que `empilhar` soma na ação aberta: o painel e o vão até a próxima linha.
pub fn ocupa(p: &Painter, f: &Folha, a: &AcaoPlano, x1: f32) -> f32 {
    altura(p, f, a, x1) + VAO_ABAIXO
}

/// Desenha o painel com o topo em `y`. `x1` é a borda direita da folha. Devolve o que só quem tem `vista` pode fazer: passe a `aplicar`.
pub fn painel(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &AcaoPlano, y: f32, x1: f32) -> Option<Saida> {
    let g = Geometria::de(x1);
    let m = medir(c.p, f, a, &g);
    let ctx = ui.ctx().clone();
    let ms = desde_que_abriu(&ctx, ("acao-painel", &a.referencia));
    // a altura anda junto com as linhas de baixo: o mesmo relógio do `mexer` (350 ms ao abrir); sem `mexer`, vai direto
    let chave_altura = ("acao-altura", &a.referencia);
    if ms < 1.0 {
        anima::deslize(&ctx, chave_altura, &[0.0]);
    }
    let alto = anima::deslize(&ctx, chave_altura, &[m.total()]).0[0].clamp(0.0, m.total());
    let area = c.caixa(X0, y, x1 - X0, alto);
    menu::marcar(ui, area, || linha::alvo(alvo.ws, alvo.bp, a));
    let recorte = c.p.with_clip_rect(area.intersect(c.p.clip_rect()));
    // cada bloco chega na sua vez: sobe 10 px e clareia
    let bloco_n = |i: usize| -> (Painter, f32) {
        let k = cascata(&ctx, ms, i, 0.0);
        (com_alfa(&recorte, k), (1.0 - k) * 10.0)
    };
    let (p0, d0) = bloco_n(0);
    let mut saida = cabecalho::desenhar(ui, &Pincel { p: &p0, o: c.o + vec2(0.0, d0) }, a, &g, y);
    let mut yy = y + m.cabeca;
    let (p1, d1) = bloco_n(1);
    let c1 = Pincel { p: &p1, o: c.o + vec2(0.0, d1) };
    if a.diff.is_some() {
        if diff::desenhar(lapis(&c1), ui, alvo, a, g.x, yy + 2.0, g.w).ver_agentes {
            saida = Some(Saida::Agentes);
        }
        yy += m.diff;
    } else if let Some(s) = planos::planos(ui, &c1, alvo, a, g.x, yy, g.coluna) {
        saida = Some(s);
    }
    // sem a lista de planos (o diff a substitui), os checkpoints ficam na primeira coluna
    let x_checkpoints = if m.planos > 0.0 { g.x_direita } else { g.x };
    let (p2, d2) = bloco_n(2);
    tempo::desenhar(ui, &Pincel { p: &p2, o: c.o + vec2(0.0, d2) }, alvo, a, &f.hoje, (x_checkpoints, yy, g.coluna));
    yy += m.colunas();
    let (p3, d3) = bloco_n(2);
    let c3 = Pincel { p: &p3, o: c.o + vec2(0.0, d3) };
    if m.decisoes > 0.0 {
        yy += VAO_SECAO;
        decisoes::desenhar(ui, &c3, alvo, f, a, g.x, yy, g.w);
        yy += m.decisoes;
    }
    if m.perguntas > 0.0 {
        yy += VAO_SECAO;
        perguntas::desenhar(ui, &c3, alvo, f, a, g.x, yy, g.w);
        yy += m.perguntas;
    }
    let (p4, d4) = bloco_n(3);
    if m.tabelas > 0.0 {
        yy += VAO_SECAO;
        prancha::tabelas(ui, &Pincel { p: &p4, o: c.o + vec2(0.0, d4) }, a, g.x, yy, g.w);
        yy += m.tabelas;
    }
    let (p5, d5) = bloco_n(4);
    if m.detalhes > 0.0 {
        yy += VAO_SECAO;
        prancha::detalhes(ui, &Pincel { p: &p5, o: c.o + vec2(0.0, d5) }, alvo, f, a, g.x, yy, g.w);
    }
    saida
}
