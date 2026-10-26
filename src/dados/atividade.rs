//! A atividade de acesso (tabela `atividade`, migração 005): quem entrou, convites, papéis, remoções, chaves e o link do dono. Nunca segredo,
//! chave nem conteúdo. As mudanças de item ficam em `historico_campos` (frente SINCRONIA).
use crate::erro::Resultado;
use rusqlite::Connection;
use serde_json::{json, Value};

/// Retenção, aplicada a cada gravação.
pub const DIAS: i64 = 90;
pub const MAX_LINHAS: i64 = 20_000;

pub struct Registro<'a> {
    pub membro_id: Option<&'a str>,
    pub nome: &'a str,
    pub tipo: &'a str,
    pub workspace_id: &'a str,
    pub blueprint_id: &'a str,
    pub alvo: &'a str,
}

pub fn gravar(conn: &Connection, agora_ms: i64, r: &Registro) -> Resultado<()> {
    let em = crate::tempo::iso_utc(agora_ms);
    conn.execute(
        "INSERT INTO atividade (em, membro_id, nome, tipo, workspace_id, blueprint_id, alvo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (&em, r.membro_id, r.nome, r.tipo, r.workspace_id, r.blueprint_id, r.alvo),
    )?;
    conn.execute("DELETE FROM atividade WHERE em < ?1", [crate::tempo::iso_utc(agora_ms - DIAS * 86_400_000)])?;
    conn.execute("DELETE FROM atividade WHERE id <= (SELECT max(id) FROM atividade) - ?1", [MAX_LINHAS])?;
    Ok(())
}

/// Os eventos do workspace, do mais recente ao mais antigo.
pub fn listar(conn: &Connection, ws: &str, limite: i64) -> Resultado<Vec<Value>> {
    let mut c = conn.prepare("SELECT em, membro_id, nome, tipo, blueprint_id, alvo FROM atividade WHERE workspace_id = ?1 ORDER BY id DESC LIMIT ?2")?;
    let linhas = c.query_map((ws, limite), |l| {
        let membro: Option<String> = l.get(1)?;
        let nome: String = l.get(2)?;
        Ok(json!({"em": l.get::<_, String>(0)?, "membro": membro.map(|id| json!({"id": id, "nome": nome})), "tipo": l.get::<_, String>(3)?,
                  "blueprintId": l.get::<_, String>(4)?, "alvo": l.get::<_, String>(5)?}))
    })?;
    Ok(linhas.collect::<Result<Vec<_>, _>>()?)
}
