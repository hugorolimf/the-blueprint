//! A linha do tempo da execução do agente por ação: o que ele avisou (começou, mudou de arquivo, bloqueou, concluiu), com hora, guardado fora do data.json.
//! Retenção na mesma transação de cada gravação (nada roda em repouso): os `LIMITE_POR_ACAO` mais recentes por ação e nada com mais de `DIAS`.
//! O que entra e o que nunca entra está em docs/claude-na-folha.md ("Linha do tempo da execução por ação"); quem filtra é `mcp::linha_do_tempo`.
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};

pub const LIMITE_POR_ACAO: i64 = 50;
pub const DIAS: i64 = 30;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct EventoLinha {
    pub workspace_id: String,
    pub blueprint_id: String,
    pub acao_id: String,
    /// id do plano ou do checkpoint, conforme o tipo
    pub alvo: Option<String>,
    pub agente: Option<String>,
    pub tipo: String,
    pub estado: Option<String>,
    pub texto: String,
    pub onde: Option<String>,
}

const DA_ACAO: &str = "workspace_id = ?1 AND blueprint_id = ?2 AND acao_id = ?3";

/// Grava o evento; se for igual ao último da ação (mesmo agente, tipo, alvo, texto, estado e arquivo), só atualiza a hora dele.
pub fn gravar(conn: &mut Connection, agora_ms: i64, e: &EventoLinha) -> Resultado<()> {
    let em = crate::tempo::iso_utc(agora_ms);
    let tx = conn.transaction()?;
    let repetido: Option<i64> = tx
        .query_row(
            &format!("SELECT id FROM linha_do_tempo WHERE id = (SELECT max(id) FROM linha_do_tempo WHERE {DA_ACAO}) AND alvo IS ?4 AND agente IS ?5 AND tipo = ?6 AND estado IS ?7 AND texto = ?8 AND onde IS ?9"),
            (&e.workspace_id, &e.blueprint_id, &e.acao_id, &e.alvo, &e.agente, &e.tipo, &e.estado, &e.texto, &e.onde),
            |l| l.get(0),
        )
        .optional()?;
    match repetido {
        Some(id) => {
            tx.execute("UPDATE linha_do_tempo SET em = ?1 WHERE id = ?2", (&em, id))?;
        }
        None => {
            tx.execute(
                "INSERT INTO linha_do_tempo (workspace_id, blueprint_id, acao_id, alvo, em, agente, tipo, estado, texto, onde) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                (&e.workspace_id, &e.blueprint_id, &e.acao_id, &e.alvo, &em, &e.agente, &e.tipo, &e.estado, &e.texto, &e.onde),
            )?;
        }
    }
    tx.execute(
        &format!("DELETE FROM linha_do_tempo WHERE {DA_ACAO} AND id NOT IN (SELECT id FROM linha_do_tempo WHERE {DA_ACAO} ORDER BY id DESC LIMIT ?4)"),
        (&e.workspace_id, &e.blueprint_id, &e.acao_id, LIMITE_POR_ACAO),
    )?;
    tx.execute("DELETE FROM linha_do_tempo WHERE em < ?1", [crate::tempo::iso_utc(agora_ms - DIAS * 86_400_000)])?;
    tx.commit()?;
    Ok(())
}

/// Os eventos da ação, do mais recente ao mais antigo.
pub fn listar(conn: &Connection, workspace_id: &str, blueprint_id: &str, acao_id: &str) -> Resultado<Value> {
    let mut c = conn.prepare(&format!("SELECT em, agente, tipo, estado, texto, onde, alvo FROM linha_do_tempo WHERE {DA_ACAO} ORDER BY id DESC LIMIT ?4"))?;
    let linhas = c.query_map((workspace_id, blueprint_id, acao_id, LIMITE_POR_ACAO), |l| {
        Ok(json!({"em": l.get::<_, String>(0)?, "agente": l.get::<_, Option<String>>(1)?, "tipo": l.get::<_, String>(2)?, "estado": l.get::<_, Option<String>>(3)?, "texto": l.get::<_, String>(4)?, "onde": l.get::<_, Option<String>>(5)?, "alvo": l.get::<_, Option<String>>(6)?}))
    })?;
    Ok(Value::Array(linhas.collect::<Result<Vec<_>, _>>()?))
}
