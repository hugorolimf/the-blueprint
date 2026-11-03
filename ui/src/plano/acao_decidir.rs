//! O que se decide numa peça: a decisão, a pergunta ou a tela de um plano, com o estado de cada uma e o que é preciso para decidir. Quem mostra é a ficha do plano, em cima da planta.
//! Pergunta se responde escolhendo; decisão, o agente já escolheu e a pessoa confirma, ajusta ou contesta; tela, a pessoa vê o resultado e aprova, pede ajuste ou reprova.

use super::super::modelo::{Decisao, Detalhe, Pergunta};
use super::blocos::Bloco;
use super::figura::{Marca, Peca};
use crate::i18n::tr;
use crate::tema::{AZUL, MEDIO, VERDE, VERMELHO};
use egui::Color32;

#[derive(Clone, Copy)]
pub enum Item<'a> {
    Decisao(&'a Decisao),
    Pergunta(&'a Pergunta),
    Tela(&'a Detalhe),
}

impl Item<'_> {
    pub fn chave(&self) -> String {
        match self {
            Item::Decisao(d) => format!("d:{}", d.id),
            Item::Pergunta(q) => format!("q:{}", q.id),
            Item::Tela(d) => format!("t:{}", d.id),
        }
    }
    pub fn marca(&self) -> Marca {
        match self {
            Item::Decisao(d) => Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref()),
            Item::Pergunta(q) => if q.resposta.is_empty() { Marca::Espera } else { Marca::Ok },
            Item::Tela(d) => Marca::de_revisao(d.pede_aprovacao, d.revisao.as_ref()),
        }
    }
    pub fn tipo(&self) -> &'static str {
        match self {
            Item::Decisao(_) => tr("Decisão"),
            Item::Pergunta(_) => tr("Pergunta"),
            Item::Tela(_) => tr("Tela"),
        }
    }
    pub fn peca(&self) -> Peca {
        match self {
            Item::Decisao(_) => Peca::Decisao,
            Item::Pergunta(_) => Peca::Pergunta,
            Item::Tela(_) => Peca::Tela,
        }
    }
    pub fn titulo(&self) -> &str {
        match self {
            Item::Decisao(d) => if d.rotulo.is_empty() { &d.texto } else { &d.rotulo },
            Item::Pergunta(q) => &q.texto,
            Item::Tela(d) => if d.titulo.is_empty() { &d.id } else { &d.titulo },
        }
    }
    pub fn estado(&self) -> (&'static str, Color32) {
        match (self.marca(), self) {
            (Marca::Espera, Item::Decisao(_)) => (tr("Falta confirmar"), AZUL),
            (Marca::Espera, Item::Pergunta(_)) => (tr("Falta responder"), AZUL),
            (Marca::Espera, Item::Tela(_)) => (tr("Falta aprovar"), AZUL),
            (Marca::Ok, Item::Pergunta(_)) => (tr("Respondida"), VERDE),
            (Marca::Ok, Item::Decisao(_)) => (tr("Confirmada"), VERDE),
            (Marca::Ok, Item::Tela(_)) => (tr("Aprovada"), VERDE),
            (Marca::Problema, _) => (tr("Rever"), VERMELHO),
            (Marca::Neutra, _) => (tr("Só para saber"), MEDIO),
        }
    }
}

/// O que um bloco pede da pessoa, com as telas que precisam de aprovação.
fn itens<'a>(b: &Bloco<'a>) -> Vec<Item<'a>> {
    b.decisoes.iter().map(|d| Item::Decisao(d)).chain(b.perguntas.iter().map(|q| Item::Pergunta(q))).chain(b.telas.iter().filter(|d| d.pede_aprovacao).map(|d| Item::Tela(d))).collect()
}

/// Tudo o que se pode abrir de um plano, com as telas que só mostram: é o que aparece por dentro da construção.
pub fn todas<'a>(b: &Bloco<'a>) -> Vec<Item<'a>> {
    b.decisoes.iter().map(|d| Item::Decisao(d)).chain(b.perguntas.iter().map(|q| Item::Pergunta(q))).chain(b.telas.iter().map(|d| Item::Tela(d))).collect()
}

/// A primeira peça deste bloco que falta decidir.
pub fn primeira_pendente(b: &Bloco) -> Option<String> {
    itens(b).iter().find(|i| i.marca() == Marca::Espera).map(Item::chave)
}
