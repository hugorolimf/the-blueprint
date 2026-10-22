//! A fila em memória: o servidor guarda, a página busca, aplica e confirma (ack). Cabem no máximo 200 itens.
use serde_json::{Map, Value};
use std::collections::HashSet;

const FILA_MAX: usize = 200;

#[derive(Default)]
pub struct Fila {
    pub itens: Vec<Value>,
    proximo: i64,
}

impl Fila {
    /// `substitui` = (workspaceId, blueprintId) cuja proposta pendente sai antes de entrar a nova.
    pub fn empurra(&mut self, mut item: Map<String, Value>, substitui: Option<(&str, &str)>) -> Value {
        if let Some((ws, proj)) = substitui {
            self.itens.retain(|i| !(i["tipo"] == "proposta" && i["workspaceId"] == ws && i["blueprintId"] == proj));
        }
        self.proximo += 1;
        item.insert("id".into(), Value::from(self.proximo));
        item.insert("em".into(), Value::String(crate::tempo::iso_utc_agora()));
        let novo = Value::Object(item);
        self.itens.push(novo.clone());
        if self.itens.len() > FILA_MAX {
            self.itens.remove(0);
        }
        novo
    }

    pub fn ack(&mut self, ids: &HashSet<i64>) {
        self.itens.retain(|i| !i["id"].as_i64().is_some_and(|id| ids.contains(&id)));
    }
}
