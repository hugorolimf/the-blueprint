//! Contagens do banco: o que se compara antes e depois de uma importação para provar que nada se perdeu.
use crate::erro::Resultado;
use rusqlite::Connection;
use std::fmt;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub struct Contagens {
    pub workspaces: i64,
    pub blueprints: i64,
    pub acoes: i64,
    pub planos: i64,
    pub planos_feitos: i64,
    pub checkpoints: i64,
    pub checkpoints_feitos: i64,
    pub comentarios: i64,
    pub prototipos: i64,
}

impl fmt::Display for Contagens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} workspaces, {} blueprints, {} ações, {} planos ({} feitos), {} checkpoints ({} feitos), {} comentários, {} protótipos",
            self.workspaces, self.blueprints, self.acoes, self.planos, self.planos_feitos, self.checkpoints, self.checkpoints_feitos, self.comentarios, self.prototipos
        )
    }
}

fn contar(conn: &Connection, sql: &str) -> Resultado<i64> {
    Ok(conn.query_row(sql, [], |l| l.get(0))?)
}

pub fn contar_tudo(conn: &Connection) -> Resultado<Contagens> {
    Ok(Contagens {
        workspaces: contar(conn, "SELECT COUNT(*) FROM workspaces")?,
        blueprints: contar(conn, "SELECT COUNT(*) FROM blueprints")?,
        acoes: contar(conn, "SELECT COUNT(*) FROM acoes")?,
        planos: contar(conn, "SELECT COUNT(*) FROM planos")?,
        planos_feitos: contar(conn, "SELECT COUNT(*) FROM planos WHERE feito = 1")?,
        checkpoints: contar(conn, "SELECT COUNT(*) FROM checkpoints")?,
        checkpoints_feitos: contar(conn, "SELECT COUNT(*) FROM checkpoints WHERE feito = 1")?,
        comentarios: contar(conn, "SELECT COUNT(*) FROM comentarios")?,
        prototipos: contar(conn, "SELECT COUNT(*) FROM prototipos")?,
    })
}
