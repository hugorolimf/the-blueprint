//! Sessões do navegador (cookie `bp_sessao`) e chaves do MCP (tabelas `sessoes` e `chaves_mcp`, migração 005): só o sha256 de cada segredo.
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension};

/// De quem é a sessão: um membro, ou um visitante preso ao convite.
#[derive(Clone, Debug, PartialEq)]
pub struct Sessao {
    pub membro_id: Option<String>,
    pub convite_id: Option<String>,
    pub expira_em: String,
}

pub fn criar(conn: &Connection, hash: &str, s: &Sessao, agora: &str) -> Resultado<()> {
    conn.execute(
        "INSERT INTO sessoes (hash, membro_id, convite_id, expira_em, visto_em) VALUES (?1, ?2, ?3, ?4, ?5)",
        (hash, &s.membro_id, &s.convite_id, &s.expira_em, agora),
    )?;
    Ok(())
}

/// A sessão ainda válida com esse hash (as vencidas são apagadas aqui mesmo, sem trabalho em repouso).
pub fn buscar(conn: &Connection, hash: &str, agora: &str) -> Resultado<Option<Sessao>> {
    let achada = conn
        .query_row("SELECT membro_id, convite_id, expira_em FROM sessoes WHERE hash = ?1", [hash], |l| Ok(Sessao { membro_id: l.get(0)?, convite_id: l.get(1)?, expira_em: l.get(2)? }))
        .optional()?;
    match achada {
        Some(s) if s.expira_em.as_str() <= agora => {
            conn.execute("DELETE FROM sessoes WHERE hash = ?1", [hash])?;
            Ok(None)
        }
        outra => Ok(outra),
    }
}

/// Anota que a sessão foi vista (quem chama limita a no máximo uma vez a cada 10 min).
pub fn ver(conn: &Connection, hash: &str, agora: &str) -> Resultado<()> {
    conn.execute("UPDATE sessoes SET visto_em = ?2 WHERE hash = ?1", (hash, agora))?;
    Ok(())
}

pub fn apagar(conn: &Connection, hash: &str) -> Resultado<()> {
    conn.execute("DELETE FROM sessoes WHERE hash = ?1", [hash])?;
    Ok(())
}

/// Guarda a chave do membro, trocando a anterior (uma por membro).
pub fn gravar_chave(conn: &Connection, hash: &str, membro_id: &str, prefixo: &str) -> Resultado<()> {
    conn.execute("DELETE FROM chaves_mcp WHERE membro_id = ?1", [membro_id])?;
    conn.execute("INSERT INTO chaves_mcp (hash, membro_id, prefixo) VALUES (?1, ?2, ?3)", (hash, membro_id, prefixo))?;
    Ok(())
}

/// O membro dono da chave, se a chave existe.
pub fn membro_da_chave(conn: &Connection, hash: &str) -> Resultado<Option<String>> {
    Ok(conn.query_row("SELECT membro_id FROM chaves_mcp WHERE hash = ?1", [hash], |l| l.get(0)).optional()?)
}

pub fn chave_usada(conn: &Connection, hash: &str, agora: &str) -> Resultado<()> {
    conn.execute("UPDATE chaves_mcp SET usada_em = ?2 WHERE hash = ?1", (hash, agora))?;
    Ok(())
}

/// (prefixo, criada_em, usada_em) da chave do membro.
pub fn chave_do_membro(conn: &Connection, membro_id: &str) -> Resultado<Option<(String, String, Option<String>)>> {
    Ok(conn.query_row("SELECT prefixo, criada_em, usada_em FROM chaves_mcp WHERE membro_id = ?1", [membro_id], |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?))).optional()?)
}

pub fn revogar_chave(conn: &Connection, membro_id: &str) -> Resultado<bool> {
    Ok(conn.execute("DELETE FROM chaves_mcp WHERE membro_id = ?1", [membro_id])? > 0)
}
