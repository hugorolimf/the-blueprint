//! Há quanto tempo esta janela vê uma coisa como está: o agente trabalhando, a versão que chegou. O servidor não manda a hora, então é «visto há»:
//! conta desde o primeiro quadro em que a janela viu. Os textos ficam em `agentes::TEXTOS`.

use crate::i18n::{tr, tr_com};
use egui::{Context, Id};
use std::fmt::Debug;
use std::hash::Hash;

/// Segundos desde que esta janela viu `chave` pela primeira vez. Mudar a chave (o plano mudou de situação) recomeça a conta.
pub fn visto_ha(ctx: &Context, chave: impl Hash + Debug) -> u64 {
    let id = Id::unique(("tempo-visto", chave));
    let agora = ctx.input(|i| i.time);
    let desde = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with::<f64>(id, || agora));
    (agora - desde).max(0.0) as u64
}

/// «agora», «3 min», «1 h 05 min», «2 d».
pub fn escrito(segundos: u64) -> String {
    match segundos {
        0..=44 => tr("agora").to_string(),
        45..=3569 => tr_com("{n} min", &[("n", &((segundos + 30) / 60).to_string())]),
        3570..=86_399 => tr_com("{h} h {m} min", &[("h", &(segundos / 3600).to_string()), ("m", &format!("{:02}", segundos % 3600 / 60))]),
        _ => tr_com("{n} d", &[("n", &(segundos / 86_400).to_string())]),
    }
}
