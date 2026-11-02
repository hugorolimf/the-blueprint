//! `plano_versoes`: o que chegou do agente e ainda não entrou na árvore (D1). Escrever aqui não sobe a versão dos dados (D4).
//! Um blueprint tem no máximo uma escrita aberta e uma versão pendente (índices da migração). Guarda as 20 mais novas, e só apaga as já resolvidas.
use super::{hex_aleatorio, lista_de};
use crate::dados::Dados;
use crate::erro::{Erro, Resultado};
use rusqlite::{Connection, OptionalExtension, Row};
use serde_json::Value;

const RETER: i64 = 20;
const COLUNAS: &str = "blueprint_id, numero, workspace_id, escrita_id, origem, situacao, agente, plano, aguardando, tocadas, removidas, atualizada_em, criada_em, fechada_em";

#[derive(Debug, Clone, PartialEq)]
pub struct VersaoLinha {
    pub blueprint_id: String,
    pub numero: i64,
    pub workspace_id: String,
    pub escrita_id: String,
    /// `escrita` ou `salvar`
    pub origem: String,
    /// `escrevendo`, `fechada` ou `resolvida`
    pub situacao: String,
    pub agente: String,
    pub plano: Value,
    pub aguardando: Vec<String>,
    pub tocadas: Vec<String>,
    pub removidas: Vec<String>,
    pub atualizada_em: i64,
    pub criada_em: i64,
    pub fechada_em: Option<i64>,
}

fn da_linha(l: &Row) -> rusqlite::Result<VersaoLinha> {
    let plano: String = l.get("plano")?;
    Ok(VersaoLinha {
        blueprint_id: l.get("blueprint_id")?,
        numero: l.get("numero")?,
        workspace_id: l.get("workspace_id")?,
        escrita_id: l.get("escrita_id")?,
        origem: l.get("origem")?,
        situacao: l.get("situacao")?,
        agente: l.get("agente")?,
        plano: serde_json::from_str(&plano).unwrap_or(Value::Null),
        aguardando: lista_de(&l.get::<_, String>("aguardando")?),
        tocadas: lista_de(&l.get::<_, String>("tocadas")?),
        removidas: lista_de(&l.get::<_, String>("removidas")?),
        atualizada_em: l.get("atualizada_em")?,
        criada_em: l.get("criada_em")?,
        fechada_em: l.get("fechada_em")?,
    })
}

fn uma(conn: &Connection, filtro: &str, valor: &str) -> Resultado<Option<VersaoLinha>> {
    let sql = format!("SELECT {COLUNAS} FROM plano_versoes WHERE {filtro}");
    Ok(conn.query_row(&sql, [valor], da_linha).optional()?)
}

/// Apaga as resolvidas além das 20 mais novas, e as figuras de versão que não existe mais. Os arquivos em disco são por conteúdo e ficam.
fn reter(conn: &Connection, bp: &str) -> Resultado<()> {
    conn.execute(
        "DELETE FROM plano_versoes WHERE blueprint_id = ?1 AND situacao = 'resolvida'
         AND numero NOT IN (SELECT numero FROM plano_versoes WHERE blueprint_id = ?1 ORDER BY numero DESC LIMIT ?2)",
        (bp, RETER),
    )?;
    limpar_figuras_orfas(conn, bp)
}

fn limpar_figuras_orfas(conn: &Connection, bp: &str) -> Resultado<()> {
    conn.execute("DELETE FROM plano_figuras WHERE blueprint_id = ?1 AND numero NOT IN (SELECT numero FROM plano_versoes WHERE blueprint_id = ?1)", [bp])?;
    Ok(())
}

impl Dados {
    /// Só cria: com outra escrita aberta no blueprint, o índice recusa. Devolver a aberta é de quem chama, conferindo a origem. Devolve `(numero, escrita_id)`.
    #[allow(clippy::too_many_arguments)]
    pub fn plano_abrir(&mut self, ws: &str, bp: &str, origem: &str, agente: &str, doc: &Value, aguardando: &[String], agora_ms: i64) -> Resultado<(i64, String)> {
        let tx = self.conn.transaction()?;
        let numero: i64 = tx.query_row("SELECT COALESCE(MAX(numero), 0) + 1 FROM plano_versoes WHERE blueprint_id = ?1", [bp], |l| l.get(0))?;
        let escrita_id = hex_aleatorio(16);
        tx.execute(
            "INSERT INTO plano_versoes (blueprint_id, numero, workspace_id, escrita_id, origem, situacao, agente, plano, aguardando, atualizada_em, criada_em)
             VALUES (?1, ?2, ?3, ?4, ?5, 'escrevendo', ?6, ?7, ?8, ?9, ?9)",
            (bp, numero, ws, &escrita_id, origem, agente, doc.to_string(), serde_json::to_string(aguardando)?, agora_ms),
        )?;
        reter(&tx, bp)?;
        tx.commit()?;
        Ok((numero, escrita_id))
    }

    /// Regrava o conteúdo da escrita aberta. Escrita que não existe ou já fechou: `NaoEncontrado`.
    pub fn plano_gravar(&mut self, escrita_id: &str, doc: &Value, aguardando: &[String], tocadas: &[String], removidas: &[String], agora_ms: i64) -> Resultado<()> {
        let n = self.conn.execute(
            "UPDATE plano_versoes SET plano = ?1, aguardando = ?2, tocadas = ?3, removidas = ?4, atualizada_em = ?5 WHERE escrita_id = ?6 AND situacao = 'escrevendo'",
            (doc.to_string(), serde_json::to_string(aguardando)?, serde_json::to_string(tocadas)?, serde_json::to_string(removidas)?, agora_ms, escrita_id),
        )?;
        if n == 0 {
            return Err(sem_escrita(escrita_id));
        }
        Ok(())
    }

    /// A versão desta escrita, em qualquer situação.
    pub fn plano_escrita(&self, escrita_id: &str) -> Resultado<Option<VersaoLinha>> {
        uma(&self.conn, "escrita_id = ?1", escrita_id)
    }

    pub fn plano_aberta(&self, bp: &str) -> Resultado<Option<VersaoLinha>> {
        uma(&self.conn, "blueprint_id = ?1 AND situacao = 'escrevendo'", bp)
    }

    pub fn plano_pendente(&self, bp: &str) -> Resultado<Option<VersaoLinha>> {
        uma(&self.conn, "blueprint_id = ?1 AND situacao = 'fechada'", bp)
    }

    /// Fecha a escrita aberta; a pendente anterior vira resolvida (a nova já foi feita em cima dela). Devolve o número.
    pub fn plano_fechar(&mut self, escrita_id: &str, agora_ms: i64) -> Resultado<i64> {
        let tx = self.conn.transaction()?;
        let achada: Option<(String, i64)> = tx
            .query_row("SELECT blueprint_id, numero FROM plano_versoes WHERE escrita_id = ?1 AND situacao = 'escrevendo'", [escrita_id], |l| Ok((l.get(0)?, l.get(1)?)))
            .optional()?;
        let Some((bp, numero)) = achada else { return Err(sem_escrita(escrita_id)) };
        tx.execute("UPDATE plano_versoes SET situacao = 'resolvida' WHERE blueprint_id = ?1 AND situacao = 'fechada'", [&bp])?;
        tx.execute("UPDATE plano_versoes SET situacao = 'fechada', fechada_em = ?1, atualizada_em = ?1 WHERE escrita_id = ?2", (agora_ms, escrita_id))?;
        tx.commit()?;
        Ok(numero)
    }

    /// Nada mais falta na versão pendente. Em versão que não está pendente, não faz nada.
    pub fn plano_resolver(&mut self, bp: &str, numero: i64) -> Resultado<()> {
        self.conn.execute("UPDATE plano_versoes SET situacao = 'resolvida' WHERE blueprint_id = ?1 AND numero = ?2 AND situacao = 'fechada'", (bp, numero))?;
        Ok(())
    }

    /// Regrava o conteúdo da versão pendente: o "Editar" da revisão levou outro texto à árvore, e o diff precisa dar igual.
    pub fn plano_regravar_fechada(&mut self, bp: &str, numero: i64, doc: &Value) -> Resultado<()> {
        self.conn.execute("UPDATE plano_versoes SET plano = ?1 WHERE blueprint_id = ?2 AND numero = ?3 AND situacao = 'fechada'", (doc.to_string(), bp, numero))?;
        Ok(())
    }

    /// Os blueprints com alguma linha de versão, figura ou revisão: o motor compara com a árvore para achar os que sumiram.
    pub fn planejar_blueprints_com_linhas(&self) -> Resultado<Vec<String>> {
        let mut c = self.conn.prepare_cached("SELECT blueprint_id FROM plano_versoes UNION SELECT blueprint_id FROM plano_figuras UNION SELECT blueprint_id FROM plano_revisoes")?;
        let linhas = c.query_map([], |l| l.get(0))?.collect::<Result<_, _>>()?;
        Ok(linhas)
    }

    /// Apaga a escrita aberta e as figuras dela. A árvore não muda.
    pub fn plano_descartar(&mut self, escrita_id: &str) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        let bp: Option<String> = tx.query_row("SELECT blueprint_id FROM plano_versoes WHERE escrita_id = ?1 AND situacao = 'escrevendo'", [escrita_id], |l| l.get(0)).optional()?;
        let Some(bp) = bp else { return Err(sem_escrita(escrita_id)) };
        tx.execute("DELETE FROM plano_versoes WHERE escrita_id = ?1", [escrita_id])?;
        limpar_figuras_orfas(&tx, &bp)?;
        tx.commit()?;
        Ok(())
    }

    /// O blueprint sumiu da árvore: apaga versões, figuras e revisões dele. O motor chama depois de cancelar as execuções, que ficam como histórico.
    pub fn planejar_limpar(&mut self, bp: &str) -> Resultado<()> {
        let tx = self.conn.transaction()?;
        for tabela in ["plano_versoes", "plano_figuras", "plano_revisoes"] {
            tx.execute(&format!("DELETE FROM {tabela} WHERE blueprint_id = ?1"), [bp])?;
        }
        tx.commit()?;
        Ok(())
    }
}

fn sem_escrita(escrita_id: &str) -> Erro {
    Erro::NaoEncontrado(format!("não há escrita aberta \"{escrita_id}\""))
}
