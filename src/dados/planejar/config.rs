//! `planejar_config`: chave e valor, fora de `/api/dados` (chaves: `limite:<ws>`, `comando_agente`, `comando_orquestrador`). O comando só muda por gesto.
use crate::dados::Dados;
use crate::erro::Resultado;
use rusqlite::OptionalExtension;

impl Dados {
    pub fn config_ler(&self, chave: &str) -> Resultado<Option<String>> {
        Ok(self.conn.query_row("SELECT valor FROM planejar_config WHERE chave = ?1", [chave], |l| l.get(0)).optional()?)
    }

    pub fn config_gravar(&mut self, chave: &str, valor: &str) -> Resultado<()> {
        self.conn.execute("INSERT OR REPLACE INTO planejar_config (chave, valor) VALUES (?1, ?2)", (chave, valor))?;
        Ok(())
    }
}
