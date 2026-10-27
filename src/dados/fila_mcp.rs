//! A cópia durável da fila do MCP (tabelas `fila_mcp` e `fila_mcp_contadores`, migração 008). Quem decide o que entra e sai é `mcp::fila`, que continua
//! lendo da memória; aqui só grava cada mudança numa transação e relê tudo ao abrir o app. Conexão própria no mesmo arquivo: a fila é usada sem a trava
//! de `Dados`, e o WAL deixa as duas conviverem no mesmo processo (nunca dois processos no mesmo banco).
use crate::erro::Resultado;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

pub struct FilaBanco {
    conn: Connection,
}

/// O que estava guardado: os itens na ordem (com a entrada da proposta, se houver) e os contadores `(tipo, workspace, blueprint, último)`.
#[derive(Debug, Default)]
pub struct FilaGuardada {
    pub itens: Vec<(Value, Option<Value>)>,
    pub contadores: Vec<(String, String, String, i64)>,
}

impl FilaBanco {
    /// Abre o banco (o mesmo de `Dados`) e aplica as migrações que faltarem (normalmente nenhuma: `Dados::abrir` veio antes).
    pub fn abrir(caminho: &Path) -> Resultado<FilaBanco> {
        let mut conn = Connection::open(caminho)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA synchronous = NORMAL;")?;
        super::migracoes::aplicar(&mut conn)?;
        Ok(FilaBanco { conn })
    }

    /// Tudo o que está guardado. Linha com JSON estragado é pulada com aviso: uma linha ruim não impede o app de abrir.
    pub fn ler(&self) -> Resultado<FilaGuardada> {
        let mut c = self.conn.prepare("SELECT id, item, entrada FROM fila_mcp ORDER BY id")?;
        let linhas = c.query_map([], |l| Ok((l.get::<_, i64>(0)?, l.get::<_, String>(1)?, l.get::<_, Option<String>>(2)?)))?;
        let mut g = FilaGuardada::default();
        for linha in linhas {
            let (id, item, entrada) = linha?;
            match serde_json::from_str::<Value>(&item) {
                Ok(item) => g.itens.push((item, entrada.and_then(|e| serde_json::from_str(&e).ok()))),
                Err(e) => eprintln!("fila do MCP: item {id} guardado com JSON inválido foi ignorado: {e}"),
            }
        }
        let mut c = self.conn.prepare("SELECT tipo, workspace_id, blueprint_id, ultimo FROM fila_mcp_contadores")?;
        g.contadores = c.query_map([], |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?, l.get(3)?)))?.collect::<Result<_, _>>()?;
        Ok(g)
    }

    /// Uma mudança da fila, numa transação: tira os ids de `sai`, grava o item que `entra` (id, item, entrada) e sobe os contadores (nunca descem).
    pub fn gravar(&mut self, sai: &[i64], entra: Option<(i64, &Value, Option<&Value>)>, contadores: &[(&str, &str, &str, i64)]) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        for id in sai {
            tx.execute("DELETE FROM fila_mcp WHERE id = ?1", [id])?;
        }
        if let Some((id, item, entrada)) = entra {
            let entrada = entrada.map(serde_json::to_string).transpose()?;
            tx.execute("INSERT OR REPLACE INTO fila_mcp (id, item, entrada) VALUES (?1, ?2, ?3)", (id, serde_json::to_string(item)?, entrada))?;
        }
        for (tipo, ws, bp, ultimo) in contadores {
            tx.execute(
                "INSERT INTO fila_mcp_contadores (tipo, workspace_id, blueprint_id, ultimo) VALUES (?1, ?2, ?3, ?4) ON CONFLICT (tipo, workspace_id, blueprint_id) DO UPDATE SET ultimo = max(ultimo, excluded.ultimo)",
                (tipo, ws, bp, ultimo),
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
