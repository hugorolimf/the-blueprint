//! A peça da planta da ação, em traço isométrico (Anatomy): o plano é uma laje dividida em quatro faixas fixas, uma para cada tipo do que há nele.
//! Atrás as telas, de pé; depois as decisões, em caixas; as perguntas, em pinos; na frente os checkpoints, deitados. A faixa é sempre a mesma, então duas lajes se comparam de relance.
//! A cor diz o estado de cada peça e a laje tracejada é o plano que ainda não foi aprovado; o que a cor diz também vem escrito na legenda e na dica.

use super::super::acao_estilo::{alfa, mix};
use super::super::modelo::{Plano, Revisao, Situacao};
use super::super::revisao::{self, EstadoPeca};
use super::super::vazio_iso::{Iso, Tons, COS};
use super::blocos::Bloco;
use crate::tema::{AZUL, BAIXO, CAMPO, CARTAO, MEDIO, QUADRA, TINTA, VERDE, VERMELHO};
use egui::{Color32, Stroke};

/// O lado da laje, em unidades do mundo.
pub const LADO: f32 = 4.0;
const ESPESSURA: f32 = 0.3;
/// Quanto as telas de pé sobem acima do canto de trás da laje.
const ACIMA: f32 = 1.1;
/// As divisões entre as faixas.
const DIVISOES: [f32; 3] = [1.02, 2.02, 3.02];
/// Onde cada faixa fica, da de trás para a da frente: é a ordem em que se desenha.
const FAIXAS: [f32; 4] = [0.22, 1.28, 2.46, 3.28];
const X_INICIO: f32 = 0.3;
const PASSO: f32 = 0.74;
/// O que cabe numa faixa; a conta exata fica escrita na dica.
const MAX_POR_FAIXA: usize = 5;

/// A largura da laje na tela, com `k` pixels por unidade.
pub fn largura(k: f32) -> f32 {
    2.0 * LADO * COS * k
}

/// A altura da figura na tela, das telas de pé até o canto da frente.
pub fn altura(k: f32) -> f32 {
    (ACIMA + LADO + ESPESSURA) * k
}

/// O centro da laje em relação à esquerda e ao topo da figura: passe a `Iso::o`.
pub fn origem(k: f32) -> egui::Vec2 {
    egui::vec2(largura(k) / 2.0, ACIMA * k)
}

/// A ponta da direita e a da esquerda da laje, onde as ligações chegam.
pub fn pontas(iso: &Iso) -> (egui::Pos2, egui::Pos2) {
    (iso.ponto((LADO, 0.0, 0.0)), iso.ponto((0.0, LADO, 0.0)))
}

/// O que a peça pede de quem olha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Marca {
    /// falta decidir
    Espera,
    Ok,
    /// não pede decisão
    Neutra,
    /// mandou revisar ou reprovou
    Problema,
}

impl Marca {
    pub fn de_revisao(pede: bool, r: Option<&Revisao>) -> Marca {
        if !pede {
            return Marca::Neutra;
        }
        match revisao::estado_de(r) {
            EstadoPeca::Pendente => Marca::Espera,
            EstadoPeca::Aprovada => Marca::Ok,
            _ => Marca::Problema,
        }
    }

    fn tons(self) -> Tons {
        let cor = match self {
            Marca::Espera => AZUL,
            Marca::Ok => VERDE,
            Marca::Neutra => MEDIO,
            Marca::Problema => VERMELHO,
        };
        let forca = if self == Marca::Neutra { 0.32 } else { 0.8 };
        Tons { topo: mix(CAMPO, cor, forca), esq: mix(CAMPO, cor, forca * 0.45), dir: mix(CAMPO, cor, forca * 0.25), traco: Stroke::new(0.9, cor) }
    }
}

/// O plano já está pronto?
pub fn feito(q: &Plano) -> bool {
    q.feito || q.situacao == Situacao::Pronto
}

/// O tipo da peça: cada um tem a sua forma e a sua faixa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Peca {
    Tela,
    Decisao,
    Pergunta,
    Checkpoint,
}

/// O que há em cima de uma laje, faixa por faixa.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conteudo {
    pub telas: Vec<Marca>,
    pub decisoes: Vec<Marca>,
    pub perguntas: Vec<Marca>,
    pub checkpoints: Vec<Marca>,
}

impl Conteudo {
    pub fn de(b: &Bloco) -> Conteudo {
        Conteudo {
            telas: b.telas.iter().map(|d| Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref())).collect(),
            decisoes: b.decisoes.iter().map(|d| Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref())).collect(),
            perguntas: b.perguntas.iter().map(|q| if q.resposta.is_empty() { Marca::Espera } else { Marca::Ok }).collect(),
            checkpoints: b.checkpoints.iter().map(|k| if k.feito { Marca::Ok } else { Marca::Neutra }).collect(),
        }
    }
}

/// Como a laje aparece: `realce` é o mouse em cima ou a peça escolhida, de 0 a 1.
#[derive(Clone, Copy, Debug)]
pub struct Aparencia {
    pub traco: Color32,
    pub proposta: bool,
    pub realce: f32,
}

impl Aparencia {
    /// A borda nunca é âmbar: quem espera fica no traço normal, e a espera vem escrita embaixo.
    pub fn de(q: &Plano, realce: f32) -> Aparencia {
        let traco = match q.situacao {
            Situacao::Pronto => VERDE,
            Situacao::Trabalhando => AZUL,
            Situacao::Parado | Situacao::Bloqueado => VERMELHO,
            _ if q.feito => VERDE,
            _ => MEDIO,
        };
        Aparencia { traco, proposta: q.situacao == Situacao::Revisao, realce }
    }
}

/// O centro da peça `i` de um tipo, em relação ao canto da laje. `None` para o que não coube na faixa.
pub fn centro_peca(tipo: Peca, i: usize) -> Option<(f32, f32, f32)> {
    if i >= MAX_POR_FAIXA {
        return None;
    }
    let x = X_INICIO + i as f32 * PASSO;
    Some(match tipo {
        Peca::Tela => (x + 0.29, FAIXAS[0], 0.41),
        Peca::Decisao => (x + 0.23, FAIXAS[1] + 0.23, 0.17),
        Peca::Pergunta => (x, FAIXAS[2], 0.5),
        Peca::Checkpoint => (x + 0.23, FAIXAS[3] + 0.23, 0.02),
    })
}

/// Uma peça solta, com o canto de trás em `(x, y)`.
pub fn peca(iso: &Iso, tipo: Peca, x: f32, y: f32, m: Marca) {
    let t = m.tons();
    match tipo {
        Peca::Tela => iso.caixa((x, y, 0.0), (0.58, 0.08, 0.82), t),
        Peca::Decisao => iso.caixa((x, y, 0.0), (0.46, 0.46, 0.34), t),
        Peca::Pergunta => {
            iso.linha((x, y, 0.0), (x, y, 0.62), t.traco);
            let topo = iso.ponto((x, y, 0.7));
            iso.p.circle(topo, 0.17 * iso.k, t.topo, t.traco);
        }
        Peca::Checkpoint => iso.face(&[(x, y, 0.02), (x + 0.46, y, 0.02), (x + 0.46, y + 0.46, 0.02), (x, y + 0.46, 0.02)], t.topo, t.traco),
    }
}

fn faixa(iso: &Iso, tipo: Peca, y: f32, lista: &[Marca]) {
    for (i, m) in lista.iter().take(MAX_POR_FAIXA).enumerate() {
        peca(iso, tipo, X_INICIO + i as f32 * PASSO, y, *m);
    }
}

/// A laje e tudo o que há nela.
pub fn laje(iso: &Iso, c: &Conteudo, a: &Aparencia) {
    let traco = Stroke::new(1.0 + 0.4 * a.realce, mix(a.traco, TINTA, a.realce * 0.6));
    let tons = Tons { topo: QUADRA.gamma_multiply(0.3 + 0.45 * a.realce), esq: CARTAO, dir: CAMPO, traco };
    let (lugar, tamanho) = ((0.0, 0.0, -ESPESSURA), (LADO, LADO, ESPESSURA));
    if a.proposta {
        iso.caixa(lugar, tamanho, Tons { traco: Stroke::NONE, ..tons });
        iso.caixa_tracejada(lugar, tamanho, traco, 0.0);
    } else {
        iso.caixa(lugar, tamanho, tons);
    }
    for y in DIVISOES {
        iso.linha((0.0, y, 0.0), (LADO, y, 0.0), Stroke::new(0.6, alfa(BAIXO, 0.4)));
    }
    faixa(iso, Peca::Tela, FAIXAS[0], &c.telas);
    faixa(iso, Peca::Decisao, FAIXAS[1], &c.decisoes);
    faixa(iso, Peca::Pergunta, FAIXAS[2], &c.perguntas);
    faixa(iso, Peca::Checkpoint, FAIXAS[3], &c.checkpoints);
}
