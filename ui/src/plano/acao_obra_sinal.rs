//! O sinal de cada construção, no rótulo dela: pronta, em obra, livre para iniciar, travada por um plano que nem começou, na fila ou parada. A cor diz o estado, e a dica escreve.

use super::super::super::caixinha;
use super::super::super::modelo::{Folha, Plano, Situacao};
use super::super::estado;
use super::super::figura::feito;
use super::super::obra_fora as fora;
use super::super::obra_icones::Icone;
use super::super::obra_regra as regra;
use crate::i18n::{tr, tr_com};
use crate::tema::{AMBAR, AZUL, MEDIO, TINTA, VERDE, VERMELHO};
use egui::Color32;

/// O estado da construção, em figura.
#[derive(Clone, Debug, PartialEq)]
pub enum Sinal {
    Pronto,
    EmObra,
    Livre,
    /// os planos anteriores que nem começaram
    Travado(Vec<String>),
    /// já pedido, esperando a vez
    NaFila(Vec<String>),
    Parou,
    Nenhum,
}

pub fn sinal_de(f: &Folha, q: &Plano) -> Sinal {
    if feito(q) {
        return Sinal::Pronto;
    }
    match q.situacao {
        Situacao::Trabalhando => Sinal::EmObra,
        Situacao::Parado | Situacao::Bloqueado => Sinal::Parou,
        Situacao::Fila | Situacao::Dependencia | Situacao::Area => Sinal::NaFila(q.depende_de.iter().filter(|d| !fora::pronto(f, d)).cloned().collect()),
        Situacao::Nao if caixinha::clique_de(q).is_some() => {
            let faltam = regra::sem_iniciar(f, q);
            if faltam.is_empty() { Sinal::Livre } else { Sinal::Travado(faltam.iter().map(|r| r.to_string()).collect()) }
        }
        _ => Sinal::Nenhum,
    }
}

impl Sinal {
    pub fn icone(&self) -> Option<(Icone, Color32)> {
        Some(match self {
            Sinal::Pronto => (Icone::Visto, VERDE),
            Sinal::EmObra => (Icone::Obra, AZUL),
            Sinal::Livre => (Icone::Iniciar, TINTA),
            Sinal::Travado(_) => (Icone::Cadeado, AMBAR),
            Sinal::NaFila(_) => (Icone::Fila, AMBAR),
            Sinal::Parou => (Icone::Parou, VERMELHO),
            Sinal::Nenhum => return None,
        })
    }

    /// O número de quem falta, ao lado do sinal.
    pub fn quem(&self) -> String {
        match self {
            Sinal::Travado(l) | Sinal::NaFila(l) => l.join(", "),
            _ => String::new(),
        }
    }

    pub fn dica(&self, q: &Plano, pede: usize) -> String {
        match self {
            Sinal::Pronto => tr("pronto").to_string(),
            Sinal::EmObra => tr("em obra").to_string(),
            Sinal::Livre if pede > 0 => tr_com("Antes de iniciar, decida {n}", &[("n", &pede.to_string())]),
            Sinal::Livre if q.depende_de.is_empty() => tr_com("Iniciar {ref}", &[("ref", &q.referencia)]),
            Sinal::Livre => tr_com("Inicia quando {lista} ficar pronto", &[("lista", &q.depende_de.join(", "))]),
            Sinal::Travado(l) => tr_com("Inicie {lista} antes: este plano depende dele.", &[("lista", &l.join(", "))]),
            Sinal::NaFila(l) if l.is_empty() => tr("Na fila").to_string(),
            Sinal::NaFila(l) => tr_com("Na fila, esperando {lista}", &[("lista", &l.join(", "))]),
            Sinal::Parou => if q.motivo.is_empty() { estado::situacao_de(q).map_or(String::new(), |(s, _)| s.to_string()) } else { q.motivo.clone() },
            Sinal::Nenhum => String::new(),
        }
    }
}

/// A frase curta de cada construção: pronta, em obra, livre, ou de quem ela espera.
pub fn fala(q: &Plano, pronto: impl Fn(&str) -> bool) -> (String, Color32) {
    if feito(q) {
        return (tr("pronto").to_string(), VERDE);
    }
    match q.situacao {
        Situacao::Trabalhando => return (tr("em obra").to_string(), AZUL),
        Situacao::Parado | Situacao::Bloqueado => return estado::situacao_de(q).map_or((String::new(), VERMELHO), |(s, c)| (s.to_string(), c)),
        _ => {}
    }
    let falta: Vec<&str> = q.depende_de.iter().map(String::as_str).filter(|d| !pronto(d)).collect();
    if falta.is_empty() {
        (tr("livre para subir").to_string(), MEDIO)
    } else {
        (tr_com("espera {lista}", &[("lista", &falta.join(", "))]), AMBAR)
    }
}

