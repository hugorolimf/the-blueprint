//! Membros e o papel de cada um por escopo (tabelas `membros` e `acessos`, migração 005). Quem chama confere a permissão e abre a transação
//! (`Dados::transacao`); aqui só SQL.
use crate::contexto::{Acesso, Papel};
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};

pub fn criar(conn: &Connection, id: &str, nome: &str) -> Resultado<()> {
    conn.execute("INSERT INTO membros (id, nome) VALUES (?1, ?2)", (id, nome))?;
    Ok(())
}

/// O nome de um membro ainda ativo (não removido).
pub fn nome_ativo(conn: &Connection, id: &str) -> Resultado<Option<String>> {
    Ok(conn.query_row("SELECT nome FROM membros WHERE id = ?1 AND removido_em IS NULL", [id], |l| l.get(0)).optional()?)
}

pub fn acessos(conn: &Connection, membro_id: &str) -> Resultado<Vec<Acesso>> {
    let mut c = conn.prepare("SELECT workspace_id, blueprint_id, papel FROM acessos WHERE membro_id = ?1 ORDER BY workspace_id, blueprint_id")?;
    let linhas = c.query_map([membro_id], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?, l.get::<_, String>(2)?)))?;
    let mut lista = Vec::new();
    for linha in linhas {
        let (ws, bp, papel) = linha?;
        if let Some(p) = Papel::de_texto(&papel) {
            lista.push(Acesso::novo(&ws, &bp, p));
        }
    }
    Ok(lista)
}

/// Soma um acesso ao membro; se ele já tinha um no mesmo escopo, fica o maior papel (entrar por um convite nunca rebaixa).
pub fn somar_acesso(conn: &Connection, membro_id: &str, ws: &str, bp: &str, papel: Papel, convite_id: Option<&str>) -> Resultado<()> {
    let atual: Option<String> = conn.query_row("SELECT papel FROM acessos WHERE membro_id = ?1 AND workspace_id = ?2 AND blueprint_id = ?3", (membro_id, ws, bp), |l| l.get(0)).optional()?;
    if atual.as_deref().and_then(Papel::de_texto).is_some_and(|a| a >= papel) {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO acessos (membro_id, workspace_id, blueprint_id, papel, convite_id) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (membro_id, workspace_id, blueprint_id) DO UPDATE SET papel = ?4, convite_id = ?5",
        (membro_id, ws, bp, papel.texto(), convite_id),
    )?;
    Ok(())
}

/// O papel do membro no workspace inteiro passa a ser `papel` (os acessos por blueprint desse workspace saem: o do workspace cobre todos).
pub fn definir_papel(conn: &Connection, membro_id: &str, ws: &str, papel: Papel) -> Resultado<()> {
    conn.execute("DELETE FROM acessos WHERE membro_id = ?1 AND workspace_id = ?2", (membro_id, ws))?;
    conn.execute("INSERT INTO acessos (membro_id, workspace_id, blueprint_id, papel) VALUES (?1, ?2, '', ?3)", (membro_id, ws, papel.texto()))?;
    Ok(())
}

/// Tira o membro do workspace. Sem nenhum acesso restante, ele é removido: sai das sessões e perde a chave. Devolve se foi removido de vez.
pub fn remover_do_workspace(conn: &Connection, membro_id: &str, ws: &str, agora: &str) -> Resultado<bool> {
    conn.execute("DELETE FROM acessos WHERE membro_id = ?1 AND workspace_id = ?2", (membro_id, ws))?;
    let resta: bool = conn.query_row("SELECT EXISTS (SELECT 1 FROM acessos WHERE membro_id = ?1)", [membro_id], |l| l.get(0))?;
    if resta {
        return Ok(false);
    }
    conn.execute("UPDATE membros SET removido_em = ?2 WHERE id = ?1", (membro_id, agora))?;
    conn.execute("DELETE FROM sessoes WHERE membro_id = ?1", [membro_id])?;
    conn.execute("DELETE FROM chaves_mcp WHERE membro_id = ?1", [membro_id])?;
    Ok(true)
}

/// Quantos membros ativos são donos do workspace inteiro (contando os donos de todos, `'*'`), sem contar `exceto`.
pub fn donos_do_workspace(conn: &Connection, ws: &str, exceto: &str) -> Resultado<i64> {
    Ok(conn.query_row(
        "SELECT count(DISTINCT a.membro_id) FROM acessos a JOIN membros m ON m.id = a.membro_id
         WHERE m.removido_em IS NULL AND a.papel = 'dono' AND a.blueprint_id = '' AND a.workspace_id IN (?1, '*') AND a.membro_id <> ?2",
        (ws, exceto),
        |l| l.get(0),
    )?)
}

/// Existe um dono de todos os workspaces ativo? (sem ele, o servidor imprime o link do dono)
pub fn existe_dono_geral(conn: &Connection) -> Resultado<bool> {
    Ok(donos_do_workspace(conn, "*", "")? > 0)
}

/// As pessoas do workspace (com acesso nele ou em `'*'`), com o maior papel, o blueprint (vazio = o workspace todo) e a última atividade.
pub fn do_workspace(conn: &Connection, ws: &str) -> Resultado<Vec<Value>> {
    let mut c = conn.prepare(
        "SELECT m.id, m.nome, a.papel, a.blueprint_id, (SELECT max(visto_em) FROM sessoes s WHERE s.membro_id = m.id)
         FROM membros m JOIN acessos a ON a.membro_id = m.id
         WHERE m.removido_em IS NULL AND a.workspace_id IN (?1, '*') ORDER BY m.nome, m.id",
    )?;
    let linhas = c.query_map([ws], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?, l.get::<_, String>(2)?, l.get::<_, String>(3)?, l.get::<_, Option<String>>(4)?)))?;
    let mut lista: Vec<Value> = Vec::new();
    for linha in linhas {
        let (id, nome, papel, bp, ultima) = linha?;
        let p = Papel::de_texto(&papel).unwrap_or(Papel::Supervisor);
        let novo = json!({"id": id, "nome": nome, "papel": papel, "blueprintId": bp, "ultimaAtividade": ultima});
        match lista.iter_mut().find(|m| m["id"] == id.as_str()) {
            None => lista.push(novo),
            // mais de um acesso: fica o maior papel; empatados, o do workspace inteiro
            Some(m) => {
                let atual = Papel::de_texto(m["papel"].as_str().unwrap_or("")).unwrap_or(Papel::Supervisor);
                if p > atual || (p == atual && bp.is_empty()) {
                    *m = novo;
                }
            }
        }
    }
    Ok(lista)
}
