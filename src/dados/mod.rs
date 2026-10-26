//! A camada de dados: o único lugar do programa que conhece o SQLite. As rotas e o MCP só falam com `Dados`, sempre passando o `Contexto`.
//! Leituras saem de uma cópia em memória (refeita só depois de uma gravação); toda gravação é uma transação, então nunca fica pela metade.
pub mod arquivo;
pub mod contagens;
mod exportar;
pub mod formato;
mod importar;
mod itens;
mod linha_do_tempo;
mod linhas;
pub mod markdown;
pub mod migracoes;
mod pastas;

#[cfg(test)]
mod tests;

pub use contagens::Contagens;
pub use linha_do_tempo::{EventoLinha, LIMITE_POR_ACAO};

use crate::contexto::Contexto;
use crate::erro::Resultado;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

pub struct Dados {
    conn: Connection,
    /// o data.json como a página o lê; None depois de qualquer gravação
    cache: Option<Value>,
}

impl Dados {
    /// Abre (ou cria) o banco em arquivo, em modo WAL, e aplica as migrações.
    pub fn abrir(caminho: &Path) -> Resultado<Dados> {
        let conn = Connection::open(caminho)?;
        conn.query_row("PRAGMA journal_mode = WAL", [], |l| l.get::<_, String>(0))?;
        Dados::preparar(conn)
    }

    /// Banco só na memória (testes).
    pub fn em_memoria() -> Resultado<Dados> {
        Dados::preparar(Connection::open_in_memory()?)
    }

    fn preparar(mut conn: Connection) -> Resultado<Dados> {
        conn.execute_batch("PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;")?;
        migracoes::aplicar(&mut conn)?;
        Ok(Dados { conn, cache: None })
    }

    pub fn conexao(&self) -> &Connection {
        &self.conn
    }

    /// O data.json inteiro (o que `GET /api/dados` devolve).
    pub fn ler_tudo(&mut self, _ctx: &Contexto) -> Resultado<&Value> {
        if self.cache.is_none() {
            self.cache = Some(exportar::exportar(&self.conn)?);
        }
        Ok(self.cache.as_ref().expect("cache acabou de ser preenchido"))
    }

    /// Substitui tudo pelo data.json recebido (o que `PUT /api/dados` faz), numa transação: se algo for inválido, o banco fica como estava.
    pub fn gravar_tudo(&mut self, ctx: &Contexto, dados: &Value) -> Resultado<Contagens> {
        ctx.exige_editar()?;
        let tx = self.conn.transaction()?;
        importar::importar(&tx, dados)?;
        tx.commit()?;
        self.cache = None;
        self.contagens(ctx)
    }

    pub fn contagens(&self, _ctx: &Contexto) -> Resultado<Contagens> {
        contagens::contar_tudo(&self.conn)
    }

    pub fn marcar_plano(&mut self, ctx: &Contexto, plano_id: &str, feito: bool) -> Resultado<()> {
        ctx.exige_editar()?;
        itens::marcar_plano(&self.conn, plano_id, feito)?;
        self.cache = None;
        Ok(())
    }

    /// Liga a pasta a um blueprint (quem chama já conferiu que ele existe). Fora do data.json: não invalida o cache.
    pub fn vincular_pasta(&mut self, ctx: &Contexto, caminho: &str, workspace_id: &str, blueprint_id: &str) -> Resultado<()> {
        ctx.exige_editar()?;
        pastas::vincular(&self.conn, caminho, workspace_id, blueprint_id)
    }

    pub fn blueprint_da_pasta(&self, _ctx: &Contexto, caminho: &str) -> Resultado<Option<(String, String)>> {
        pastas::procurar(&self.conn, caminho)
    }

    pub fn pasta_do_blueprint(&self, _ctx: &Contexto, workspace_id: &str, blueprint_id: &str) -> Resultado<Option<String>> {
        pastas::pasta_do_blueprint(&self.conn, workspace_id, blueprint_id)
    }

    pub fn listar_pastas(&self, _ctx: &Contexto) -> Resultado<Value> {
        pastas::listar(&self.conn)
    }

    pub fn desvincular_pasta(&mut self, ctx: &Contexto, caminho: &str) -> Resultado<()> {
        ctx.exige_editar()?;
        pastas::desvincular(&self.conn, caminho)
    }

    /// Anota um evento do agente na linha do tempo da ação. Fora do data.json: não invalida o cache.
    pub fn gravar_linha_do_tempo(&mut self, ctx: &Contexto, agora_ms: i64, evento: &EventoLinha) -> Resultado<()> {
        ctx.exige_editar()?;
        linha_do_tempo::gravar(&mut self.conn, agora_ms, evento)
    }

    pub fn linha_do_tempo(&self, _ctx: &Contexto, workspace_id: &str, blueprint_id: &str, acao_id: &str) -> Resultado<Value> {
        linha_do_tempo::listar(&self.conn, workspace_id, blueprint_id, acao_id)
    }

    pub fn adicionar_comentario(&mut self, ctx: &Contexto, plano_id: &str, comentario: &Value) -> Resultado<()> {
        ctx.exige_editar()?;
        itens::adicionar_comentario(&mut self.conn, plano_id, comentario)?;
        self.cache = None;
        Ok(())
    }

    pub fn gravar_prototipo(&mut self, ctx: &Contexto, acao_id: &str, prototipo: &Value) -> Resultado<()> {
        ctx.exige_editar()?;
        itens::gravar_prototipo(&mut self.conn, acao_id, prototipo)?;
        self.cache = None;
        Ok(())
    }

    pub fn remover_prototipo(&mut self, ctx: &Contexto, acao_id: &str, prototipo_id: &str) -> Resultado<bool> {
        ctx.exige_editar()?;
        let removeu = itens::remover_prototipo(&self.conn, acao_id, prototipo_id)?;
        self.cache = None;
        Ok(removeu)
    }
}
