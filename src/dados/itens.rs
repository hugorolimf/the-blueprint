//! Gravações por item (as que o servidor faz sem reescrever o arquivo todo): marcar plano, comentário do agente, protótipo.
//! Cada uma é uma transação e sobe `revisao` e `atualizado_em` da linha tocada.
use super::formato::{COMENTARIOS, LISTA_COMENTARIOS, PROTOTIPOS};
use super::linhas::{inserir, Objeto, AGORA};
use crate::erro::{Erro, Resultado};
use rusqlite::{Connection, OptionalExtension};
use serde_json::Value;

fn objeto(v: &Value, o_que: &str) -> Resultado<Objeto> {
    match v {
        Value::Object(m) => Ok(m.clone()),
        _ => Err(Erro::Formato(format!("{o_que} precisa ser um objeto"))),
    }
}

/// Marca ou desmarca um plano (plano) como feito.
pub fn marcar_plano(conn: &Connection, plano_id: &str, feito: bool) -> Resultado<()> {
    let n = conn.execute(&format!("UPDATE planos SET feito = ?1, revisao = revisao + 1, atualizado_em = {AGORA} WHERE id = ?2"), (feito as i64, plano_id))?;
    if n == 0 {
        return Err(Erro::NaoEncontrado(format!("plano \"{plano_id}\" não existe")));
    }
    Ok(())
}

/// Acrescenta um comentário ao fim de `agente.comentarios` do plano (cria o `agente` se ainda não existir).
pub fn adicionar_comentario(conn: &mut Connection, plano_id: &str, comentario: &Value) -> Resultado<()> {
    let obj = objeto(comentario, "o comentário")?;
    let tx = conn.transaction()?;
    let atual: Option<(String, Option<String>)> = tx.query_row("SELECT listas, agente FROM planos WHERE id = ?1", [plano_id], |l| Ok((l.get(0)?, l.get(1)?))).optional()?;
    let Some((listas, agente)) = atual else {
        return Err(Erro::NaoEncontrado(format!("plano \"{plano_id}\" não existe")));
    };
    let mut listas: Vec<String> = serde_json::from_str(&listas)?;
    if !listas.iter().any(|x| x == LISTA_COMENTARIOS) {
        listas.push(LISTA_COMENTARIOS.to_string());
    }
    let agente = agente.unwrap_or_else(|| "{}".to_string());
    let ordem: i64 = tx.query_row("SELECT COALESCE(MAX(ordem) + 1, 0) FROM comentarios WHERE plano_id = ?1", [plano_id], |l| l.get(0))?;
    inserir(&tx, &COMENTARIOS, None, Some(plano_id), ordem, obj, &[])?;
    tx.execute(&format!("UPDATE planos SET listas = ?1, agente = ?2, revisao = revisao + 1, atualizado_em = {AGORA} WHERE id = ?3"), (serde_json::to_string(&listas)?, agente, plano_id))?;
    tx.commit()?;
    Ok(())
}

/// Cria ou substitui (pelo `id`) um protótipo da ação.
pub fn gravar_prototipo(conn: &mut Connection, acao_id: &str, prototipo: &Value) -> Resultado<()> {
    let mut obj = objeto(prototipo, "o protótipo")?;
    let id = match obj.remove("id") {
        Some(Value::String(s)) if !s.is_empty() => s,
        _ => return Err(Erro::Formato("o protótipo precisa de um id (texto)".into())),
    };
    let tx = conn.transaction()?;
    let listas: Option<String> = tx.query_row("SELECT listas FROM acoes WHERE id = ?1", [acao_id], |l| l.get(0)).optional()?;
    let Some(listas) = listas else {
        return Err(Erro::NaoEncontrado(format!("ação \"{acao_id}\" não existe")));
    };
    let existente: Option<(String, i64)> = tx.query_row("SELECT acao_id, ordem FROM prototipos WHERE id = ?1", [&id], |l| Ok((l.get(0)?, l.get(1)?))).optional()?;
    let ordem = match existente {
        Some((outra, _)) if outra != acao_id => return Err(Erro::Formato(format!("o id de protótipo \"{id}\" já é de outra ação"))),
        Some((_, ordem)) => {
            tx.execute("DELETE FROM prototipos WHERE id = ?1", [&id])?;
            ordem
        }
        None => tx.query_row("SELECT COALESCE(MAX(ordem) + 1, 0) FROM prototipos WHERE acao_id = ?1", [acao_id], |l| l.get(0))?,
    };
    inserir(&tx, &PROTOTIPOS, Some(&id), Some(acao_id), ordem, obj, &[])?;
    let mut listas: Vec<String> = serde_json::from_str(&listas)?;
    if !listas.iter().any(|x| x == PROTOTIPOS.lista) {
        listas.push(PROTOTIPOS.lista.to_string());
    }
    tx.execute(&format!("UPDATE acoes SET listas = ?1, revisao = revisao + 1, atualizado_em = {AGORA} WHERE id = ?2"), (serde_json::to_string(&listas)?, acao_id))?;
    tx.commit()?;
    Ok(())
}

/// Remove um protótipo da ação; devolve se existia.
pub fn remover_prototipo(conn: &Connection, acao_id: &str, prototipo_id: &str) -> Resultado<bool> {
    let n = conn.execute("DELETE FROM prototipos WHERE id = ?1 AND acao_id = ?2", (prototipo_id, acao_id))?;
    Ok(n > 0)
}
