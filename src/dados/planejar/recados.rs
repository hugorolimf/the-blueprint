//! `recados`: conversa entre usuário, orquestrador e agentes. Ler não tem efeito: vários leitores não se atrapalham. Guarda os 200 mais novos por workspace.
use crate::dados::Dados;
use crate::erro::Resultado;
use rusqlite::Row;

const RETER: i64 = 200;
const SELECIONA: &str = "SELECT id, workspace_id, blueprint_id, acao_id, execucao_id, de, texto, em FROM recados";

/// `blueprint_id`, `acao_id` e `execucao_id` vazios = não é de nenhum em particular.
#[derive(Debug, Clone, Default)]
pub struct NovoRecado<'a> {
    pub workspace_id: &'a str,
    pub blueprint_id: &'a str,
    pub acao_id: &'a str,
    pub execucao_id: &'a str,
    /// `usuario`, `orquestrador` ou `agente`
    pub de: &'a str,
    pub texto: &'a str,
    pub em: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecadoLinha {
    pub id: i64,
    pub workspace_id: String,
    pub blueprint_id: String,
    pub acao_id: String,
    pub execucao_id: String,
    pub de: String,
    pub texto: String,
    pub em: i64,
}

fn da_linha(l: &Row) -> rusqlite::Result<RecadoLinha> {
    Ok(RecadoLinha { id: l.get(0)?, workspace_id: l.get(1)?, blueprint_id: l.get(2)?, acao_id: l.get(3)?, execucao_id: l.get(4)?, de: l.get(5)?, texto: l.get(6)?, em: l.get(7)? })
}

impl Dados {
    pub fn recado_gravar(&mut self, r: &NovoRecado) -> Resultado<i64> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO recados (workspace_id, blueprint_id, acao_id, execucao_id, de, texto, em) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (r.workspace_id, r.blueprint_id, r.acao_id, r.execucao_id, r.de, r.texto, r.em),
        )?;
        let id = tx.last_insert_rowid();
        tx.execute("DELETE FROM recados WHERE workspace_id = ?1 AND id NOT IN (SELECT id FROM recados WHERE workspace_id = ?1 ORDER BY id DESC LIMIT ?2)", (r.workspace_id, RETER))?;
        tx.commit()?;
        Ok(id)
    }

    /// Os últimos `limite` recados do workspace, do mais antigo ao mais novo.
    pub fn recados_do(&self, ws: &str, limite: usize) -> Resultado<Vec<RecadoLinha>> {
        let mut c = self.conn.prepare(&format!("{SELECIONA} WHERE workspace_id = ?1 ORDER BY id DESC LIMIT ?2"))?;
        let mut linhas = c.query_map((ws, limite as i64), da_linha)?.collect::<Result<Vec<_>, _>>()?;
        linhas.reverse();
        Ok(linhas)
    }

    /// Os recados deixados para esta execução, do mais antigo ao mais novo.
    pub fn recados_para(&self, execucao: &str) -> Resultado<Vec<RecadoLinha>> {
        let mut c = self.conn.prepare(&format!("{SELECIONA} WHERE execucao_id = ?1 ORDER BY id"))?;
        let linhas = c.query_map([execucao], da_linha)?;
        Ok(linhas.collect::<Result<Vec<_>, _>>()?)
    }
}
