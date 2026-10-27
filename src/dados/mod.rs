//! A camada de dados: o único lugar do programa que conhece o SQLite. As rotas e o MCP só falam com `Dados`, sempre passando o `Contexto`.
//! Leituras saem de uma cópia em memória (refeita só depois de uma gravação); toda gravação é uma transação, então nunca fica pela metade.
pub mod arquivo;
pub mod atividade;
pub mod backups;
pub mod contagens;
pub mod convites;
pub mod membros;
pub mod sessoes;
mod exportar;
pub mod formato;
pub mod historico;
mod importar;
mod item;
mod itens;
mod linha_do_tempo;
mod linhas;
pub mod markdown;
pub mod migracoes;
pub mod operacoes;
mod pastas;

#[cfg(test)]
mod tests;

pub use contagens::Contagens;
pub use linha_do_tempo::{EventoLinha, LIMITE_POR_ACAO};
pub use historico::Autor;
pub use operacoes::{Aplicada, Exige, Falha, Op, Saida};

use crate::contexto::Contexto;
use crate::erro::Resultado;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;

pub struct Dados {
    conn: Connection,
    /// o data.json como a página o lê; None depois de qualquer gravação
    cache: Option<Value>,
    revisao_tempos: u64,
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
        Ok(Dados { conn, cache: None, revisao_tempos: 0 })
    }

    pub fn conexao(&self) -> &Connection {
        &self.conn
    }

    pub fn revisao_tempos(&self) -> u64 { self.revisao_tempos }

    /// Os avisos são duráveis mesmo quando nenhuma janela está aberta.
    pub fn registrar_tempo_plano(&mut self, plano_id: &str, tipo: &str, em: &str) -> Resultado<()> {
        let (inicio, fim) = match tipo {
            "progresso" => (Some(em), None),
            "concluir" => (None, Some(em)),
            _ => return Ok(()),
        };
        let n = self.conn.execute(
            "INSERT INTO tempos_planos (plano_id, inicio_em, fim_em) VALUES (?1, ?2, ?3) ON CONFLICT(plano_id) DO UPDATE SET inicio_em = COALESCE(tempos_planos.inicio_em, excluded.inicio_em), fim_em = COALESCE(tempos_planos.fim_em, excluded.fim_em) WHERE (tempos_planos.inicio_em IS NULL AND tempos_planos.fim_em IS NULL AND excluded.inicio_em IS NOT NULL) OR (tempos_planos.fim_em IS NULL AND excluded.fim_em IS NOT NULL)",
            (plano_id, inicio, fim),
        )?;
        if n > 0 { self.cache = None; self.revisao_tempos += 1; }
        Ok(())
    }

    /// Roda `f` numa transação (as tabelas de acesso: membros, convites, sessões, chaves e atividade). Fora do data.json: não invalida o cache.
    /// Erro em `f` desfaz tudo.
    pub fn transacao<T>(&mut self, f: impl FnOnce(&rusqlite::Transaction) -> Resultado<T>) -> Resultado<T> {
        let tx = self.conn.transaction()?;
        let valor = f(&tx)?;
        tx.commit()?;
        Ok(valor)
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
        // Substituir dados também substitui os tempos, para uma restauração não herdar medições futuras.
        tx.execute("DELETE FROM tempos_planos", [])?;
        {
            let mut planos = tx.prepare("SELECT id, agente FROM planos WHERE agente IS NOT NULL")?;
            let linhas = planos.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?)))?;
            for linha in linhas {
                let (id, agente) = linha?;
                let agente: Value = serde_json::from_str(&agente)?;
                let inicio = agente.get("inicioEm").and_then(Value::as_str).filter(|s| !s.is_empty());
                let fim = agente.get("feitoEm").and_then(Value::as_str).filter(|s| !s.is_empty());
                if inicio.is_some() || fim.is_some() {
                    tx.execute("INSERT INTO tempos_planos (plano_id, inicio_em, fim_em) VALUES (?1, ?2, ?3)", (&id, inicio, fim))?;
                }
            }
        }
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

    /// Aplica um lote de operações por item numa transação (`POST /api/mudancas`); a permissão de cada operação vem de `exige`.
    /// Ver `operacoes.rs` e docs/colaboracao.md, seção 6.
    pub fn aplicar_operacoes(&mut self, autor: &Autor, versao: u64, agora_ms: i64, ops: &[Op], exige: Exige) -> Result<Saida, Falha> {
        let r = operacoes::aplicar(&mut self.conn, autor, versao, agora_ms, ops, exige);
        if matches!(&r, Ok(s) if !s.aplicadas.is_empty()) {
            self.cache = None;
        }
        r
    }

    /// O histórico de campos (`GET /api/historico`), do mais recente ao mais antigo.
    pub fn historico(&self, _ctx: &Contexto, workspace_id: &str, blueprint_id: Option<&str>, item_id: Option<&str>, limite: i64) -> Resultado<Value> {
        historico::listar(&self.conn, workspace_id, blueprint_id, item_id, limite)
    }
}
