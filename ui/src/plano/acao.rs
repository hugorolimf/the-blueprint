//! A ação aberta no lugar, desenhada direto no papel da folha, sem cartão, como a seção aberta da folha original (`folha/acoes.rs`): tudo que se decide do plano se decide aqui. A altura é medida antes de empilhar, e `altura` e `painel` usam a mesma conta.
//! Quem usa: `empilhar` soma `ocupa` na ação aberta; `painel` desenha logo abaixo da linha dela; e `aplicar` faz o que o painel devolver, depois de a folha sair de empréstimo.
//! É uma prancha só: o palco (`acao_obra.rs`), com tudo em cima dele, em figura e clique. O painel abre crescendo (a altura anda junto com as linhas de baixo).

use super::acao_estilo::rotulo_secao;
use super::acao_movimento::{cascata, com_alfa, desde_que_abriu};
use super::diff;
use super::escala::X0;
use super::folha::links;
use super::linha::{self, Pincel};
use super::modelo::{Acao as AcaoPlano, Folha};
use super::revisao::Alvo;
use super::Sub;
use crate::anima;
use crate::menu;
use crate::planta::Texto;
use crate::telas::Acao as Pedido;
use egui::{vec2, Painter, Ui};

#[path = "acao_blocos.rs"]
mod blocos;
#[path = "acao_cabeca.rs"]
mod cabecalho;
#[path = "acao_decisoes.rs"]
mod decisoes;
#[path = "acao_estado.rs"]
mod estado;
#[path = "acao_decidir.rs"]
mod decidir;
#[path = "acao_movimentos.rs"]
mod movimentos;
#[path = "acao_obra_bordo.rs"]
mod obra_bordo;
#[path = "acao_obra_cena.rs"]
mod obra_cena;
#[path = "acao_obra_icones.rs"]
mod obra_icones;
#[path = "acao_obra_mesa.rs"]
mod mesa_acao;
#[path = "acao_obra_tela.rs"]
mod obra_tela;
#[path = "acao_obra_miniatura.rs"]
mod obra_miniatura;
pub use obra_miniatura::{largura_pecas, miniatura, pe, pecas, tem_o_ponto};
#[path = "acao_obra_regra.rs"]
mod obra_regra;
#[path = "acao_obra_dutos.rs"]
mod dutos;
#[path = "acao_obra_formas.rs"]
mod obra_formas;
#[path = "acao_obra_mergulho.rs"]
mod obra_mergulho;
#[path = "acao_obra_ficha.rs"]
mod ficha;
#[path = "acao_textos.rs"]
mod textos;
pub use textos::TEXTOS;
#[path = "acao_obra_pecas.rs"]
mod obra_pecas;
#[path = "acao_obra_letreiro.rs"]
mod obra_letreiro;
#[path = "acao_partes_explodida.rs"]
mod explodida;
#[path = "acao_obra_explosao.rs"]
mod obra_explosao;
#[path = "acao_obra_zoom.rs"]
mod obra_zoom;
#[path = "acao_obra_na_tela.rs"]
mod na_tela;
#[path = "acao_obra_lista.rs"]
mod lista;
#[path = "acao_obra_fora.rs"]
mod obra_fora;
#[path = "acao_obra_sumico.rs"]
mod obra_sumico;
#[path = "acao_obra_rotulos.rs"]
mod obra_rotulos;
#[path = "acao_obra_tempo.rs"]
mod obra_tempo;
#[path = "acao_voo.rs"]
mod voo;
#[path = "acao_mapa_figura.rs"]
mod figura;
#[path = "acao_obra.rs"]
mod obra;
#[path = "acao_obra_figura.rs"]
mod obra_figura;
#[path = "acao_linha_tempo.rs"]
mod tempo;
#[path = "acao_mapa_forma.rs"]
mod forma;
#[path = "acao_planos.rs"]
mod planos;
#[path = "acao_prancha.rs"]
mod prancha;


/// A largura máxima do conteúdo: em janela larga os botões continuam perto do que decidem.
const LARGURA_MAX: f32 = 1800.0;
/// O vão entre as duas colunas, o mesmo da folha original.
const VAO_COLUNAS: f32 = 30.0;
/// A seção «Plano alterado», à direita: larga o bastante para o diff ler bem, sem tirar o lugar das telas.
const FRACAO_DIFF: f32 = 0.34;
const LARGURA_DIFF: (f32, f32) = (400.0, 620.0);
/// A distância do topo da janela em que a seção à direita para ao rolar.
const FOLGA_TOPO: f32 = 12.0;
const VAO_ABAIXO: f32 = 18.0;

/// Gira a obra (o passo `obra-orbita` do roteiro).
#[cfg(not(target_arch = "wasm32"))]
pub fn girar_obra(ctx: &egui::Context, az: f32, el: f32) {
    obra_figura::orbitar(ctx, az, el);
}

/// O que o painel pede e só quem tem `vista` pode fazer. Marcar e revisar já foram ao servidor.
#[derive(Debug)]
pub enum Saida {
    Recolher,
    Agentes,
    Pedido(Pedido),
    /// abre outra ação e rola até ela
    IrPara(String),
}

/// Faz o que o painel pediu; o que for de `app` volta como pedido.
pub fn aplicar(vista: &mut crate::telas::Vista, _a: &AcaoPlano, s: Saida) -> Option<Pedido> {
    match s {
        Saida::Recolher => vista.plano.aberta = None,
        Saida::Agentes => vista.plano.tela = Sub::Agentes,
        Saida::Pedido(p) => return Some(p),
        Saida::IrPara(id) => vista.ir_para = Some(id),
    }
    None
}

/// As alturas de cada parte: medir e desenhar saem desta mesma conta.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Partes {
    pub cabeca: f32,
    /// a seção à direita; zero sem diff
    pub diff: f32,
    /// o palco, ou a vista expandida
    pub corpo: f32,
}

impl Partes {
    pub fn total(&self) -> f32 {
        (self.cabeca + self.corpo).max(self.diff) + 6.0
    }
}

/// As medidas do painel nesta largura da folha.
#[derive(Clone, Copy)]
pub(super) struct Geometria {
    pub x: f32,
    /// a largura do resumo e dos planos: a parte da esquerda, quando há diff
    pub w: f32,
    /// a seção «Plano alterado»; zero sem diff
    pub x_diff: f32,
    pub w_diff: f32,
}

impl Geometria {
    fn de(x1: f32, com_diff: bool) -> Geometria {
        let (x, total) = (X0, (x1 - X0).min(LARGURA_MAX));
        if com_diff {
            let w_diff = (total * FRACAO_DIFF).clamp(LARGURA_DIFF.0, LARGURA_DIFF.1);
            let w = total - w_diff - VAO_COLUNAS;
            return Geometria { x, w, x_diff: x + w + VAO_COLUNAS, w_diff };
        }
        Geometria { x, w: total, x_diff: x, w_diff: 0.0 }
    }
}

/// O bloco de texto, sempre inteiro: mesma trilha do resto do painel, e altura de linha fixa para a conta fechar.
pub fn bloco(s: &str, tam: f32, max: f32, passo: f32) -> Texto<'_> {
    Texto::novo(s, tam).trilha(0.03).caixa(max.max(1.0)).lh(passo)
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
    let montagem = blocos::montar(f, a);
    Partes {
        cabeca: cabecalho::altura(p, a, g),
        diff: if a.diff.is_some() { diff::altura(p, a, g.w_diff) } else { 0.0 },
        corpo: obra::altura(p, f, a, &montagem, g.w),
    }
}

/// A altura do painel inteiro, para empilhar antes de desenhar. `x1` é a borda direita da folha.
pub fn altura(p: &Painter, f: &Folha, a: &AcaoPlano, x1: f32) -> f32 {
    medir(p, f, a, &Geometria::de(x1, a.diff.is_some())).total()
}

/// O que `empilhar` soma na ação aberta: o painel e o vão até a próxima linha.
pub fn ocupa(p: &Painter, f: &Folha, a: &AcaoPlano, x1: f32) -> f32 {
    altura(p, f, a, x1) + VAO_ABAIXO
}

/// Desenha o painel com o topo em `y`. `x1` é a borda direita da folha. Devolve o que só quem tem `vista` pode fazer: passe a `aplicar`.
pub fn painel(ui: &mut Ui, c: &Pincel, alvo: Alvo, f: &Folha, a: &AcaoPlano, y: f32, x1: f32) -> Option<Saida> {
    let g = Geometria::de(x1, a.diff.is_some());
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
    let montagem = blocos::montar(f, a);
    let mut saida = cabecalho::desenhar(ui, &Pincel { p: &p0, o: c.o + vec2(0.0, d0) }, a, &g, y);
    let (p1, d1) = bloco_n(1);
    let c1 = Pincel { p: &p1, o: c.o + vec2(0.0, d1) };
    if a.diff.is_some() {
        // a seção à direita acompanha a rolagem enquanto couber na altura do painel
        let topo_visivel = c.p.clip_rect().top() - c.o.y + FOLGA_TOPO;
        let y_diff = topo_visivel.min(y + (m.total() - m.diff).max(0.0)).max(y);
        diff::desenhar(lapis(&c1), ui, alvo, a, g.x_diff, y_diff, g.w_diff);
    }
    if let Some(s) = obra::desenhar(ui, &c1, alvo, f, a, &montagem, g.x, y + m.cabeca, g.w) {
        saida = Some(s);
    }
    saida
}
