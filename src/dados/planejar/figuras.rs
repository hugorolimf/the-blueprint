//! `plano_figuras`: as figuras recebidas por versão, com hash e medidas. O arquivo mora em `<dados>/figuras/<hash>.<ext>` (D13); aqui só o registro.
use crate::dados::Dados;
use crate::erro::Resultado;
use rusqlite::{OptionalExtension, Row};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiguraLinha {
    pub blueprint_id: String,
    pub numero: i64,
    /// o caminho relativo citado em `Detalhe.figura`
    pub caminho: String,
    pub hash: String,
    /// `svg`, `png`, `jpg` ou `webp`
    pub tipo: String,
    pub tamanho: i64,
    pub largura: Option<u32>,
    pub altura: Option<u32>,
}

const COLUNAS: &str = "blueprint_id, numero, caminho, hash, tipo, tamanho, largura, altura";

fn da_linha(l: &Row) -> rusqlite::Result<FiguraLinha> {
    Ok(FiguraLinha {
        blueprint_id: l.get(0)?,
        numero: l.get(1)?,
        caminho: l.get(2)?,
        hash: l.get(3)?,
        tipo: l.get(4)?,
        tamanho: l.get(5)?,
        largura: l.get(6)?,
        altura: l.get(7)?,
    })
}

impl Dados {
    /// Reenvio do mesmo caminho na mesma versão substitui (retomar uma escrita é seguro).
    pub fn figura_registrar(&mut self, f: &FiguraLinha) -> Resultado<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO plano_figuras (blueprint_id, numero, caminho, hash, tipo, tamanho, largura, altura) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (&f.blueprint_id, f.numero, &f.caminho, &f.hash, &f.tipo, f.tamanho, f.largura, f.altura),
        )?;
        Ok(())
    }

    pub fn figura_de(&self, bp: &str, numero: i64, caminho: &str) -> Resultado<Option<FiguraLinha>> {
        let sql = format!("SELECT {COLUNAS} FROM plano_figuras WHERE blueprint_id = ?1 AND numero = ?2 AND caminho = ?3");
        Ok(self.conn.query_row(&sql, (bp, numero, caminho), da_linha).optional()?)
    }

    pub fn figuras_da_versao(&self, bp: &str, numero: i64) -> Resultado<usize> {
        let n: i64 = self.conn.query_row("SELECT COUNT(*) FROM plano_figuras WHERE blueprint_id = ?1 AND numero = ?2", (bp, numero), |l| l.get(0))?;
        Ok(n as usize)
    }

    /// Uma linha qualquer com este conteúdo, para servir a figura com o tipo certo (`planejar/figura?hash=`).
    pub fn figura_por_hash(&self, hash: &str) -> Resultado<Option<FiguraLinha>> {
        let sql = format!("SELECT {COLUNAS} FROM plano_figuras WHERE hash = ?1 LIMIT 1");
        Ok(self.conn.query_row(&sql, [hash], da_linha).optional()?)
    }
}
