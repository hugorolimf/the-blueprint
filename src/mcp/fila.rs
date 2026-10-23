//! A fila em memória: o servidor guarda, a página busca, aplica e confirma (ack). Cabem no máximo 200 itens.
//! As propostas formam uma fila por alvo (workspace + blueprint): cada uma tem um número crescente, entra no fim e só sai quando a
//! página confirma (ack) que o usuário a aplicou ou descartou. Nada disso sobrevive ao reinício do app (é só memória).
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};

const FILA_MAX: usize = 200;

#[derive(Default)]
pub struct Fila {
    pub itens: Vec<Value>,
    proximo: i64,
    /// o último número dado a uma proposta de cada alvo (workspace, blueprint); nunca volta atrás
    numeros: HashMap<(String, String), i64>,
    /// o que o agente enviou em cada proposta pendente (id do item → entrada, com os ids provisórios), para `ajustar`
    entradas: HashMap<i64, Value>,
}

fn alvo_de(i: &Value) -> (&str, &str) {
    (i["workspaceId"].as_str().unwrap_or(""), i["blueprintId"].as_str().unwrap_or(""))
}

impl Fila {
    /// Põe o item no fim, com id e hora. Devolve o item como ficou.
    pub fn empurra(&mut self, mut item: Map<String, Value>) -> Value {
        self.proximo += 1;
        item.insert("id".into(), Value::from(self.proximo));
        item.insert("em".into(), Value::String(crate::tempo::iso_utc_agora()));
        let novo = Value::Object(item);
        self.itens.push(novo.clone());
        if self.itens.len() > FILA_MAX {
            let velho = self.itens.remove(0);
            self.entradas.remove(&velho["id"].as_i64().unwrap_or(0));
        }
        novo
    }

    /// Proposta nova entra no fim com o próximo número do alvo e NÃO apaga nenhuma outra. Com `numero` (ajuste ou substituição), a proposta
    /// desse número sai de onde estava e esta entra no fim com o mesmo número e a mesma `criadaEm`.
    pub fn empurra_proposta(&mut self, mut item: Map<String, Value>, entrada: Value, numero: Option<i64>) -> Value {
        let alvo = (item.get("workspaceId").and_then(Value::as_str).unwrap_or("").to_string(), item.get("blueprintId").and_then(Value::as_str).unwrap_or("").to_string());
        let antiga = numero.and_then(|n| self.posicao(&alvo.0, &alvo.1, n));
        let (numero, criada) = match antiga.map(|i| self.itens.remove(i)) {
            Some(velha) => {
                self.entradas.remove(&velha["id"].as_i64().unwrap_or(0));
                (numero.unwrap_or(0), velha["criadaEm"].clone())
            }
            None => {
                let atual = self.numeros.entry(alvo).or_insert(0);
                let n = numero.unwrap_or(*atual + 1);
                *atual = (*atual).max(n);
                (n, Value::Null)
            }
        };
        item.insert("numero".into(), Value::from(numero));
        let novo = self.empurra(item);
        let criada = if criada.is_null() { novo["em"].clone() } else { criada };
        let i = self.itens.len() - 1;
        self.itens[i]["criadaEm"] = criada;
        self.entradas.insert(novo["id"].as_i64().unwrap_or(0), entrada);
        self.itens[i].clone()
    }

    fn posicao(&self, ws: &str, bp: &str, numero: i64) -> Option<usize> {
        self.itens.iter().position(|i| i["tipo"] == "proposta" && alvo_de(i) == (ws, bp) && i["numero"] == numero)
    }

    /// A entrada (o que o agente mandou, com ids provisórios) da proposta pendente `numero` do alvo.
    pub fn entrada(&self, ws: &str, bp: &str, numero: i64) -> Option<Value> {
        self.posicao(ws, bp, numero).and_then(|i| self.entradas.get(&self.itens[i]["id"].as_i64()?)).cloned()
    }

    /// Os números das propostas pendentes do alvo, do menor ao maior.
    pub fn pendentes(&self, ws: &str, bp: &str) -> Vec<i64> {
        let mut n: Vec<i64> = self.itens.iter().filter(|i| i["tipo"] == "proposta" && alvo_de(i) == (ws, bp)).filter_map(|i| i["numero"].as_i64()).collect();
        n.sort_unstable();
        n
    }

    pub fn ack(&mut self, ids: &HashSet<i64>) {
        self.itens.retain(|i| !i["id"].as_i64().is_some_and(|id| ids.contains(&id)));
        self.entradas.retain(|id, _| !ids.contains(id));
    }

    /// A página aplicou ou descartou a proposta (workspace, blueprint, número, criadaEm). Pela chave inteira, e não pelo id do item, uma proposta
    /// antiga de antes de o app reiniciar nunca tira uma nova que ficou com o mesmo id.
    pub fn ack_proposta(&mut self, ws: &str, bp: &str, numero: i64, criada_em: &str) {
        let ids: HashSet<i64> = self.itens.iter().filter(|i| i["tipo"] == "proposta" && alvo_de(i) == (ws, bp) && i["numero"] == numero && i["criadaEm"] == criada_em).filter_map(|i| i["id"].as_i64()).collect();
        self.ack(&ids);
    }
}
