//! Lê e valida o corpo de `POST /api/mudancas` (`{base, cliente, ops}`, docs/colaboracao.md seção 6) e o transforma nas operações da camada de
//! dados. Só o formato: existência, permissão e conflito são de `dados::operacoes`. Qualquer operação malformada derruba o lote (400 com o índice).
use crate::dados::Op;
use serde_json::{Map, Value};

pub const MAX_OPS: usize = 500;
const TIPOS: [&str; 6] = ["workspace", "blueprint", "acao", "plano", "checkpoint", "prototipo"];

#[derive(Debug)]
pub struct Lote {
    pub base: Option<u64>,
    pub cliente: String,
    pub ops: Vec<Op>,
    /// cada operação como veio (sem `antes`), para o evento das outras páginas
    pub brutas: Vec<Value>,
}

/// O lote inteiro está errado (`indice` None) ou a operação `indice` está.
#[derive(Debug, PartialEq)]
pub struct Invalida {
    pub indice: Option<usize>,
    pub motivo: String,
}

fn invalida(indice: Option<usize>, motivo: impl Into<String>) -> Invalida {
    Invalida { indice, motivo: motivo.into() }
}

fn id_valido(s: &str) -> bool {
    !s.is_empty() && s.len() <= 80 && !s.chars().any(char::is_control)
}

fn texto(op: &Map<String, Value>, chave: &str, i: usize) -> Result<String, Invalida> {
    match op.get(chave) {
        Some(Value::String(s)) if id_valido(s) => Ok(s.clone()),
        _ => Err(invalida(Some(i), format!("{chave} inválido"))),
    }
}

fn objeto(op: &Map<String, Value>, chave: &str, i: usize) -> Result<Map<String, Value>, Invalida> {
    match op.get(chave) {
        Some(Value::Object(m)) => Ok(m.clone()),
        None if chave == "antes" => Ok(Map::new()),
        _ => Err(invalida(Some(i), format!("{chave} precisa ser um objeto"))),
    }
}

fn pai(op: &Map<String, Value>, tipo: &str, i: usize) -> Result<Option<String>, Invalida> {
    match (tipo, op.get("pai")) {
        ("workspace", None | Some(Value::Null)) => Ok(None),
        ("workspace", _) => Err(invalida(Some(i), "workspace não tem pai")),
        _ => texto(op, "pai", i).map(Some),
    }
}

fn posicao(op: &Map<String, Value>, i: usize) -> Result<usize, Invalida> {
    match op.get("posicao") {
        None => Ok(usize::MAX), // sem posição: vai para o fim
        Some(v) => v.as_u64().map(|n| n.min(1_000_000) as usize).ok_or_else(|| invalida(Some(i), "posicao inválida")),
    }
}

fn uma(valor: &Value, i: usize) -> Result<Op, Invalida> {
    let Value::Object(op) = valor else { return Err(invalida(Some(i), "a operação precisa ser um objeto")) };
    let tipo = match op.get("tipo").and_then(Value::as_str) {
        Some(t) if TIPOS.contains(&t) => t.to_string(),
        _ => return Err(invalida(Some(i), "tipo desconhecido")),
    };
    let id = texto(op, "id", i)?;
    Ok(match op.get("op").and_then(Value::as_str) {
        Some("criar") => Op::Criar { pai: pai(op, &tipo, i)?, posicao: posicao(op, i)?, valor: objeto(op, "valor", i)?, tipo, id },
        Some("alterar") => {
            let campos = objeto(op, "campos", i)?;
            if campos.is_empty() {
                return Err(invalida(Some(i), "alterar sem campos"));
            }
            Op::Alterar { antes: objeto(op, "antes", i)?, campos, tipo, id }
        }
        Some("mover") => Op::Mover { pai: pai(op, &tipo, i)?, posicao: posicao(op, i)?, tipo, id },
        Some("apagar") => Op::Apagar { tipo, id },
        _ => return Err(invalida(Some(i), "op desconhecida")),
    })
}

pub fn ler(corpo: &[u8]) -> Result<Lote, Invalida> {
    let Ok(Value::Object(raiz)) = serde_json::from_slice::<Value>(corpo) else { return Err(invalida(None, "o corpo precisa ser um objeto JSON")) };
    let Some(Value::Array(lista)) = raiz.get("ops") else { return Err(invalida(None, "falta ops[]")) };
    if lista.len() > MAX_OPS {
        return Err(invalida(None, format!("no máximo {MAX_OPS} operações por lote")));
    }
    let cliente = match raiz.get("cliente") {
        Some(Value::String(c)) if c.len() <= 40 && !c.chars().any(char::is_control) => c.clone(),
        None => String::new(),
        _ => return Err(invalida(None, "cliente inválido")),
    };
    let ops = lista.iter().enumerate().map(|(i, v)| uma(v, i)).collect::<Result<Vec<_>, _>>()?;
    let brutas = lista
        .iter()
        .map(|v| {
            let mut v = v.clone();
            if let Some(m) = v.as_object_mut() {
                m.remove("antes");
            }
            v
        })
        .collect();
    Ok(Lote { base: raiz.get("base").and_then(Value::as_u64), cliente, ops, brutas })
}
