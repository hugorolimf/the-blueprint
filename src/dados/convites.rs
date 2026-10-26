//! Convites (tabela `convites`, migração 005): só o sha256 do segredo fica aqui. Quem chama confere a permissão e abre a transação.
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension, Row};
use serde_json::{json, Value};

#[derive(Clone, Debug, PartialEq)]
pub struct Convite {
    pub id: String,
    pub papel: String,
    pub workspace_id: String,
    pub blueprint_id: String,
    pub abre: String,
    pub criado_por: Option<String>,
    pub criado_em: String,
    pub expira_em: String,
    /// None = sem limite (link público)
    pub usos_max: Option<i64>,
    pub usos: i64,
    pub revogado_em: Option<String>,
}

impl Convite {
    /// Ainda entra alguém por ele? (não revogado, não vencido, não esgotado)
    pub fn valido(&self, agora: &str) -> bool {
        self.revogado_em.is_none() && self.expira_em.as_str() > agora && self.usos_max.is_none_or(|m| self.usos < m)
    }

    /// Como a tela lista (sem o segredo, que nunca é guardado). `criador` = (id, nome) de quem criou.
    pub fn json(&self, criador: Option<(String, String)>) -> Value {
        json!({"id": self.id, "papel": self.papel, "workspaceId": self.workspace_id, "blueprintId": self.blueprint_id, "abre": self.abre,
               "criadoPor": criador.map(|(id, nome)| json!({"id": id, "nome": nome})), "criadoEm": self.criado_em, "expiraEm": self.expira_em,
               "usosMax": self.usos_max, "usos": self.usos, "revogado": self.revogado_em.is_some()})
    }
}

const COLUNAS: &str = "id, papel, workspace_id, blueprint_id, abre, criado_por, criado_em, expira_em, usos_max, usos, revogado_em";

fn de_linha(l: &Row) -> rusqlite::Result<Convite> {
    Ok(Convite {
        id: l.get(0)?,
        papel: l.get(1)?,
        workspace_id: l.get(2)?,
        blueprint_id: l.get(3)?,
        abre: l.get(4)?,
        criado_por: l.get(5)?,
        criado_em: l.get(6)?,
        expira_em: l.get(7)?,
        usos_max: l.get(8)?,
        usos: l.get(9)?,
        revogado_em: l.get(10)?,
    })
}

/// Grava o convite (o `id` e o `hash` vêm de quem gerou o segredo). `c.usos`, `c.criado_em` e `c.revogado_em` são ignorados.
pub fn criar(conn: &Connection, c: &Convite, hash: &str) -> Resultado<()> {
    conn.execute(
        "INSERT INTO convites (id, hash, papel, workspace_id, blueprint_id, abre, criado_por, expira_em, usos_max) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![c.id, hash, c.papel, c.workspace_id, c.blueprint_id, c.abre, c.criado_por, c.expira_em, c.usos_max],
    )?;
    Ok(())
}

pub fn por_hash(conn: &Connection, hash: &str) -> Resultado<Option<Convite>> {
    Ok(conn.query_row(&format!("SELECT {COLUNAS} FROM convites WHERE hash = ?1"), [hash], de_linha).optional()?)
}

pub fn por_id(conn: &Connection, id: &str) -> Resultado<Option<Convite>> {
    Ok(conn.query_row(&format!("SELECT {COLUNAS} FROM convites WHERE id = ?1"), [id], de_linha).optional()?)
}

pub fn usar(conn: &Connection, id: &str) -> Resultado<()> {
    conn.execute("UPDATE convites SET usos = usos + 1 WHERE id = ?1", [id])?;
    Ok(())
}

/// Revoga e derruba as sessões de visitante que vieram dele. Devolve se havia o que revogar.
pub fn revogar(conn: &Connection, id: &str, agora: &str) -> Resultado<bool> {
    let n = conn.execute("UPDATE convites SET revogado_em = ?2 WHERE id = ?1 AND revogado_em IS NULL", (id, agora))?;
    conn.execute("DELETE FROM sessoes WHERE convite_id = ?1 AND membro_id IS NULL", [id])?;
    Ok(n > 0)
}

/// Os convites do workspace, do mais novo ao mais antigo, com o nome de quem criou.
pub fn do_workspace(conn: &Connection, ws: &str) -> Resultado<Vec<Value>> {
    let sql = format!("SELECT {COLUNAS}, (SELECT nome FROM membros m WHERE m.id = c.criado_por) FROM convites c WHERE workspace_id = ?1 ORDER BY criado_em DESC, id");
    let mut c = conn.prepare(&sql)?;
    let linhas = c.query_map([ws], |l| Ok((de_linha(l)?, l.get::<_, Option<String>>(11)?)))?;
    let mut lista = Vec::new();
    for linha in linhas {
        let (convite, nome) = linha?;
        let criador = convite.criado_por.clone().map(|id| (id, nome.unwrap_or_default()));
        lista.push(convite.json(criador));
    }
    Ok(lista)
}

/// O convite de dono de todos ainda pendente (o link impresso pelo servidor que ninguém usou).
pub fn pendente_do_dono(conn: &Connection, agora: &str) -> Resultado<Option<Convite>> {
    let sql = format!("SELECT {COLUNAS} FROM convites WHERE papel = 'dono' AND workspace_id = '*' AND revogado_em IS NULL ORDER BY criado_em DESC");
    let mut c = conn.prepare(&sql)?;
    let todos = c.query_map([], de_linha)?.collect::<Result<Vec<_>, _>>()?;
    Ok(todos.into_iter().find(|c| c.valido(agora)))
}
