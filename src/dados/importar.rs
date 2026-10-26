//! data.json → tabelas. Substitui tudo dentro da transação recebida; se algo falhar no meio, quem chama descarta a transação
//! e o banco continua como estava (é isso que torna a gravação atômica).
use super::formato::{filhas, Tabela, COMENTARIOS, LISTA_COMENTARIOS, PLANOS, RAIZ_ID, WORKSPACES};
use super::linhas::{inserir, tirar_lista, Objeto};
use crate::erro::{Erro, Resultado};
use rusqlite::Connection;
use serde_json::Value;

const LIMPAR: &str = "DELETE FROM comentarios; DELETE FROM prototipos; DELETE FROM checkpoints; DELETE FROM planos; DELETE FROM acoes; DELETE FROM blueprints; DELETE FROM workspaces;";

pub fn importar(conn: &Connection, raiz: &Value) -> Resultado<()> {
    let Value::Object(mut obj) = raiz.clone() else {
        return Err(Erro::Formato("a raiz do arquivo de dados precisa ser um objeto".into()));
    };
    conn.execute_batch(LIMPAR)?;
    let workspaces = tirar_lista(&mut obj, WORKSPACES.lista);
    let listas_extra = if workspaces.is_some() { vec![WORKSPACES.lista] } else { vec![] };
    inserir_no(conn, &WORKSPACES, Some(RAIZ_ID), None, 0, Value::Object(obj), &listas_extra)?;
    for (i, ws) in workspaces.unwrap_or_default().into_iter().enumerate() {
        inserir_no(conn, &WORKSPACES, None, None, i as i64 + 1, ws, &[])?;
    }
    Ok(())
}

/// `forcar_id` só para a raiz, que não tem `id` no JSON. `listas_extra` = listas que o chamador já tirou do objeto.
pub(super) fn inserir_no(conn: &Connection, t: &Tabela, forcar_id: Option<&str>, pai: Option<&str>, ordem: i64, valor: Value, listas_extra: &[&str]) -> Resultado<()> {
    let Value::Object(mut obj) = valor else {
        return Err(Erro::Formato(format!("{}: cada item precisa ser um objeto", t.nome)));
    };
    let id: Option<String> = if t.sem_id {
        None
    } else if let Some(f) = forcar_id {
        Some(f.to_string())
    } else {
        match obj.remove("id") {
            Some(Value::String(s)) if !s.is_empty() => Some(s),
            _ => return Err(Erro::Formato(format!("{}: item sem id (texto) na posição {}", t.nome, ordem + 1))),
        }
    };
    if t.nome == WORKSPACES.nome && forcar_id.is_none() && id.as_deref() == Some(RAIZ_ID) {
        return Err(Erro::Formato(format!("o id \"{RAIZ_ID}\" é reservado ao workspace raiz")));
    }
    let mut listas: Vec<&str> = listas_extra.to_vec();
    let mut filhos: Vec<(&'static Tabela, Vec<Value>)> = Vec::new();
    for f in filhas(t) {
        if let Some(v) = tirar_lista(&mut obj, f.lista) {
            listas.push(f.lista);
            filhos.push((f, v));
        }
    }
    let comentarios = if t.nome == PLANOS.nome { tirar_comentarios(&mut obj) } else { None };
    if comentarios.is_some() {
        listas.push(LISTA_COMENTARIOS);
    }
    inserir(conn, t, id.as_deref(), pai, ordem, obj, &listas)?;
    for (f, itens) in filhos {
        for (i, item) in itens.into_iter().enumerate() {
            inserir_no(conn, f, None, id.as_deref(), i as i64, item, &[])?;
        }
    }
    for (i, item) in comentarios.unwrap_or_default().into_iter().enumerate() {
        inserir_no(conn, &COMENTARIOS, None, id.as_deref(), i as i64, item, &[])?;
    }
    Ok(())
}

/// Os comentários do agente moram em `agente.comentarios`: saem do JSON do `agente` e viram linhas da tabela `comentarios`.
fn tirar_comentarios(plano: &mut Objeto) -> Option<Vec<Value>> {
    match plano.get_mut("agente") {
        Some(Value::Object(agente)) if matches!(agente.get("comentarios"), Some(Value::Array(_))) => match agente.remove("comentarios") {
            Some(Value::Array(v)) => Some(v),
            _ => None,
        },
        _ => None,
    }
}
