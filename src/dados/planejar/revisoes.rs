//! `plano_revisoes`: a decisão do usuário por peça (`d:<id>`, `det:<acao>/<id>`, `diff:<acao>`, `q:<id>`). Uma linha por peça; a nova substitui a anterior.
use crate::dados::Dados;
use crate::erro::Resultado;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisaoLinha {
    pub blueprint_id: String,
    pub peca: String,
    pub numero: i64,
    /// `aprovada`, `revisar`, `reprovada` ou `respondida`
    pub estado: String,
    pub observacao: String,
    /// quem compara é a vista: hash diferente do da peça mostrada derruba a revisão
    pub hash_peca: String,
    pub em: i64,
}

impl Dados {
    #[allow(clippy::too_many_arguments)]
    pub fn revisao_gravar(&mut self, bp: &str, peca: &str, numero: i64, estado: &str, observacao: &str, hash_peca: &str, em: i64) -> Resultado<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO plano_revisoes (blueprint_id, peca, numero, estado, observacao, hash_peca, em) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (bp, peca, numero, estado, observacao, hash_peca, em),
        )?;
        Ok(())
    }

    /// Desfazer: a peça volta a não ter revisão.
    pub fn revisao_limpar(&mut self, bp: &str, peca: &str) -> Resultado<()> {
        self.conn.execute("DELETE FROM plano_revisoes WHERE blueprint_id = ?1 AND peca = ?2", (bp, peca))?;
        Ok(())
    }

    pub fn revisoes_do(&self, bp: &str) -> Resultado<Vec<RevisaoLinha>> {
        let mut c = self.conn.prepare("SELECT blueprint_id, peca, numero, estado, observacao, hash_peca, em FROM plano_revisoes WHERE blueprint_id = ?1 ORDER BY peca")?;
        let linhas = c.query_map([bp], |l| {
            Ok(RevisaoLinha { blueprint_id: l.get(0)?, peca: l.get(1)?, numero: l.get(2)?, estado: l.get(3)?, observacao: l.get(4)?, hash_peca: l.get(5)?, em: l.get(6)? })
        })?;
        Ok(linhas.collect::<Result<Vec<_>, _>>()?)
    }
}
