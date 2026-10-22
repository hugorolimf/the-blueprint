//! Conversão entre um objeto JSON e uma linha de tabela, igual para todas as tabelas (guiada por `formato.rs`).
use super::formato::{Coluna, Tabela, Tipo};
use crate::erro::Resultado;
use rusqlite::{params_from_iter, types::Value as Sql, Connection};
use serde_json::{Map, Value};

pub type Objeto = Map<String, Value>;

/// Expressão SQL da hora atual (ISO 8601, UTC) usada em `atualizado_em`.
pub const AGORA: &str = "strftime('%Y-%m-%dT%H:%M:%fZ', 'now')";

/// Tira do objeto as chaves que têm coluna (quando o tipo confere) e devolve os valores na ordem das colunas.
/// Chave ausente ou de tipo inesperado fica em `extra` e a coluna recebe NULL.
pub fn extrair(obj: &mut Objeto, colunas: &[Coluna]) -> Vec<Sql> {
    colunas
        .iter()
        .map(|c| {
            let confere = match (c.tipo, obj.get(c.chave)) {
                (Tipo::Texto, Some(Value::String(_))) | (Tipo::Bool, Some(Value::Bool(_))) | (Tipo::Numero, Some(Value::Number(_))) | (Tipo::Json, Some(_)) => true,
                _ => false,
            };
            if !confere {
                return Sql::Null;
            }
            match (c.tipo, obj.remove(c.chave).unwrap_or(Value::Null)) {
                (Tipo::Texto, Value::String(s)) => Sql::Text(s),
                (Tipo::Bool, Value::Bool(b)) => Sql::Integer(b as i64),
                // números grandes demais para i64 viram REAL (perdem precisão; o app não os usa)
                (Tipo::Numero, Value::Number(n)) => n.as_i64().map(Sql::Integer).unwrap_or_else(|| Sql::Real(n.as_f64().unwrap_or(0.0))),
                (_, outro) => Sql::Text(outro.to_string()),
            }
        })
        .collect()
}

fn numero(f: f64) -> Value {
    // 1.0 e 1 são o mesmo número no JSON do app: devolve inteiro quando não há parte fracionária
    if f.fract() == 0.0 && f.abs() < 9e15 { Value::from(f as i64) } else { serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null) }
}

/// O contrário de `extrair`: parte do `extra` e devolve as colunas ao objeto (NULL = a chave não existia).
pub fn montar(extra: &str, colunas: &[Coluna], valores: Vec<Sql>) -> Resultado<Objeto> {
    let mut obj = match serde_json::from_str::<Value>(extra)? {
        Value::Object(m) => m,
        _ => Map::new(),
    };
    for (c, v) in colunas.iter().zip(valores) {
        let valor = match (c.tipo, v) {
            (Tipo::Texto, Sql::Text(s)) => Value::String(s),
            (Tipo::Bool, Sql::Integer(i)) => Value::Bool(i != 0),
            (Tipo::Numero, Sql::Integer(i)) => Value::from(i),
            (Tipo::Numero, Sql::Real(f)) => numero(f),
            (Tipo::Json, Sql::Text(s)) => serde_json::from_str(&s)?,
            _ => continue,
        };
        obj.insert(c.chave.to_string(), valor);
    }
    Ok(obj)
}

/// Insere uma linha. `id` é None nas tabelas sem id (o banco numera). `listas` = listas filhas que existiam no JSON.
pub fn inserir(conn: &Connection, t: &Tabela, id: Option<&str>, pai: Option<&str>, ordem: i64, mut obj: Objeto, listas: &[&str]) -> Resultado<()> {
    let valores = extrair(&mut obj, t.colunas);
    let mut nomes: Vec<&str> = Vec::new();
    let mut params: Vec<Sql> = Vec::new();
    if let Some(id) = id {
        nomes.push("id");
        params.push(Sql::Text(id.to_string()));
    }
    if let (Some(col), Some(p)) = (t.pai, pai) {
        nomes.push(col);
        params.push(Sql::Text(p.to_string()));
    }
    nomes.extend(["ordem", "listas", "extra"]);
    params.extend([Sql::Integer(ordem), Sql::Text(serde_json::to_string(listas)?), Sql::Text(Value::Object(obj).to_string())]);
    nomes.extend(t.colunas.iter().map(|c| c.coluna));
    params.extend(valores);
    let sql = format!("INSERT INTO {} ({}) VALUES ({})", t.nome, nomes.join(", "), vec!["?"; nomes.len()].join(", "));
    conn.execute(&sql, params_from_iter(params))?;
    Ok(())
}

/// Tira de `obj` a lista `chave`, se existir como lista (senão fica em `extra`, como qualquer campo desconhecido).
pub fn tirar_lista(obj: &mut Objeto, chave: &str) -> Option<Vec<Value>> {
    if matches!(obj.get(chave), Some(Value::Array(_))) {
        if let Some(Value::Array(v)) = obj.remove(chave) {
            return Some(v);
        }
    }
    None
}
