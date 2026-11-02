//! `execucoes`: um agente lançado para um plano, sem FK. Os passos que disputam com outra thread são sempre UPDATE condicional na situação, então quem perde a corrida só recebe `false`.
//! O `segredo` só sai por `execucao/partida` e pela conferência do servidor: `ExecucaoLinha` não serializa e o `Debug` dela o esconde.
use super::hex_aleatorio;
use crate::dados::pastas::normalizar;
use crate::dados::Dados;
use crate::erro::Resultado;
use rusqlite::{params_from_iter, types::Value as Sql, OptionalExtension, Row};
use std::fmt;

/// As situações do índice `execucao_viva`: enquanto uma destas existe, o plano não ganha outra execução. `falhou` conta (ainda pode ser retomada).
pub const SITUACOES_VIVAS: [&str; 6] = ["marcada", "preparando", "rodando", "ajuda", "sumiu", "falhou"];
const SQL_VIVAS: &str = "('marcada','preparando','rodando','ajuda','sumiu','falhou')";

#[derive(Clone, PartialEq, Eq)]
pub struct ExecucaoLinha {
    pub id: String,
    pub workspace_id: String,
    pub blueprint_id: String,
    pub acao_id: String,
    pub plano_id: String,
    pub plano_ref: String,
    pub situacao: String,
    pub motivo: String,
    pub area: String,
    pub tentativa: i64,
    pub segredo: String,
    pub pasta_principal: String,
    pub pasta_trabalho: String,
    pub repositorio: String,
    pub ramo: String,
    pub base: String,
    pub marcada_em: i64,
    pub iniciada_em: Option<i64>,
    pub partida_em: Option<i64>,
    pub ultimo_sinal: Option<i64>,
    pub terminada_em: Option<i64>,
    pub cancelada_em: Option<i64>,
    pub integrada_em: Option<i64>,
    pub parar: bool,
}

impl fmt::Debug for ExecucaoLinha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ExecucaoLinha").field("id", &self.id).field("plano_ref", &self.plano_ref).field("situacao", &self.situacao).field("tentativa", &self.tentativa).finish_non_exhaustive()
    }
}

const COLUNAS: &str = "id, workspace_id, blueprint_id, acao_id, plano_id, plano_ref, situacao, motivo, area, tentativa, segredo, pasta_principal, pasta_trabalho, repositorio, ramo, base, marcada_em, iniciada_em, partida_em, ultimo_sinal, terminada_em, cancelada_em, integrada_em, parar";

fn da_linha(l: &Row) -> rusqlite::Result<ExecucaoLinha> {
    Ok(ExecucaoLinha {
        id: l.get("id")?,
        workspace_id: l.get("workspace_id")?,
        blueprint_id: l.get("blueprint_id")?,
        acao_id: l.get("acao_id")?,
        plano_id: l.get("plano_id")?,
        plano_ref: l.get("plano_ref")?,
        situacao: l.get("situacao")?,
        motivo: l.get("motivo")?,
        area: l.get("area")?,
        tentativa: l.get("tentativa")?,
        segredo: l.get("segredo")?,
        pasta_principal: l.get("pasta_principal")?,
        pasta_trabalho: l.get("pasta_trabalho")?,
        repositorio: l.get("repositorio")?,
        ramo: l.get("ramo")?,
        base: l.get("base")?,
        marcada_em: l.get("marcada_em")?,
        iniciada_em: l.get("iniciada_em")?,
        partida_em: l.get("partida_em")?,
        ultimo_sinal: l.get("ultimo_sinal")?,
        terminada_em: l.get("terminada_em")?,
        cancelada_em: l.get("cancelada_em")?,
        integrada_em: l.get("integrada_em")?,
        parar: l.get::<_, i64>("parar")? != 0,
    })
}

#[derive(Debug, Clone, Default)]
pub struct NovaExecucao<'a> {
    pub workspace_id: &'a str,
    pub blueprint_id: &'a str,
    pub acao_id: &'a str,
    pub plano_id: &'a str,
    pub plano_ref: &'a str,
    pub area: &'a str,
    pub pasta_principal: &'a str,
    pub marcada_em: i64,
}

/// O que `execucao_lancar` grava além de `rodando`. Segredo e tentativa novos são feitos lá.
#[derive(Debug, Clone, Default)]
pub struct Lancamento<'a> {
    pub area: &'a str,
    pub pasta_trabalho: &'a str,
    pub repositorio: &'a str,
    pub ramo: &'a str,
    pub base: &'a str,
    pub agora_ms: i64,
}

/// O que o invólucro recebe, uma vez por tentativa.
#[derive(Clone, PartialEq, Eq)]
pub struct Partida {
    pub tentativa: i64,
    pub segredo: String,
    pub linha: ExecucaoLinha,
}

impl fmt::Debug for Partida {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Partida").field("tentativa", &self.tentativa).field("linha", &self.linha).finish_non_exhaustive()
    }
}

/// Os campos a mudar; `None` = fica como está. Nos `Option<Option<_>>`, `Some(None)` limpa. `so_de` não vazio: só muda se a situação atual for uma destas.
#[derive(Debug, Clone, Default)]
pub struct Mudanca<'a> {
    pub so_de: &'a [&'a str],
    pub situacao: Option<&'a str>,
    pub motivo: Option<&'a str>,
    pub area: Option<&'a str>,
    pub pasta_trabalho: Option<&'a str>,
    pub repositorio: Option<&'a str>,
    pub ramo: Option<&'a str>,
    pub base: Option<&'a str>,
    pub ultimo_sinal: Option<i64>,
    pub terminada_em: Option<Option<i64>>,
    pub cancelada_em: Option<Option<i64>>,
    pub integrada_em: Option<Option<i64>>,
    pub parar: Option<bool>,
}

fn opcional(v: Option<i64>) -> Sql {
    v.map_or(Sql::Null, Sql::Integer)
}

impl Mudanca<'_> {
    fn sets(&self) -> Vec<(&'static str, Sql)> {
        let mut s: Vec<(&'static str, Sql)> = Vec::new();
        let textos = [("situacao", self.situacao), ("motivo", self.motivo), ("area", self.area), ("pasta_trabalho", self.pasta_trabalho), ("repositorio", self.repositorio), ("ramo", self.ramo), ("base", self.base)];
        for (coluna, valor) in textos {
            if let Some(v) = valor {
                s.push((coluna, Sql::Text(v.to_string())));
            }
        }
        if let Some(v) = self.ultimo_sinal {
            s.push(("ultimo_sinal", Sql::Integer(v)));
        }
        for (coluna, valor) in [("terminada_em", self.terminada_em), ("cancelada_em", self.cancelada_em), ("integrada_em", self.integrada_em)] {
            if let Some(v) = valor {
                s.push((coluna, opcional(v)));
            }
        }
        if let Some(v) = self.parar {
            s.push(("parar", Sql::Integer(v as i64)));
        }
        s
    }
}

impl Dados {
    /// Cria a execução `marcada` e devolve o id. Com outra viva no mesmo plano o índice recusa; só gesto chama (seção 9).
    pub fn execucao_criar(&mut self, n: &NovaExecucao) -> Resultado<String> {
        let id = hex_aleatorio(8);
        self.conn.execute(
            "INSERT INTO execucoes (id, workspace_id, blueprint_id, acao_id, plano_id, plano_ref, situacao, area, pasta_principal, marcada_em) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'marcada', ?7, ?8, ?9)",
            (&id, n.workspace_id, n.blueprint_id, n.acao_id, n.plano_id, n.plano_ref, n.area, n.pasta_principal, n.marcada_em),
        )?;
        Ok(id)
    }

    pub fn execucao_de(&self, id: &str) -> Resultado<Option<ExecucaoLinha>> {
        Ok(self.conn.query_row(&format!("SELECT {COLUNAS} FROM execucoes WHERE id = ?1"), [id], da_linha).optional()?)
    }

    /// Todas do workspace, vivas ou não, da mais antiga à mais nova.
    pub fn execucoes_do_ws(&self, ws: &str) -> Resultado<Vec<ExecucaoLinha>> {
        let mut c = self.conn.prepare(&format!("SELECT {COLUNAS} FROM execucoes WHERE workspace_id = ?1 ORDER BY marcada_em, id"))?;
        let linhas = c.query_map([ws], da_linha)?;
        Ok(linhas.collect::<Result<Vec<_>, _>>()?)
    }

    /// Todas as vivas, de qualquer workspace (o motor anda por elas).
    pub fn execucoes_vivas(&self) -> Resultado<Vec<ExecucaoLinha>> {
        let mut c = self.conn.prepare(&format!("SELECT {COLUNAS} FROM execucoes WHERE situacao IN {SQL_VIVAS} ORDER BY marcada_em, id"))?;
        let linhas = c.query_map([], da_linha)?;
        Ok(linhas.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn execucao_viva(&self, bp: &str, plano: &str) -> Resultado<Option<ExecucaoLinha>> {
        let sql = format!("SELECT {COLUNAS} FROM execucoes WHERE blueprint_id = ?1 AND plano_id = ?2 AND situacao IN {SQL_VIVAS}");
        Ok(self.conn.query_row(&sql, (bp, plano), da_linha).optional()?)
    }

    /// A execução viva, com ramo, cuja cópia contém `cwd` (D9). Sem ramo não há cópia própria: a sessão comum do usuário naquela pasta não vira o agente.
    pub fn execucao_por_pasta(&self, cwd: &str) -> Resultado<Option<ExecucaoLinha>> {
        let alvo = normalizar(cwd);
        let mut c = self.conn.prepare(&format!("SELECT {COLUNAS} FROM execucoes WHERE situacao IN {SQL_VIVAS} AND ramo <> '' AND pasta_trabalho <> ''"))?;
        let linhas = c.query_map([], da_linha)?;
        let mut melhor: Option<(usize, ExecucaoLinha)> = None;
        for linha in linhas {
            let linha = linha?;
            let pasta = normalizar(&linha.pasta_trabalho);
            let dentro = alvo == pasta || alvo.strip_prefix(&pasta).is_some_and(|resto| resto.starts_with('/'));
            if dentro && melhor.as_ref().is_none_or(|m| pasta.len() > m.0) {
                melhor = Some((pasta.len(), linha));
            }
        }
        Ok(melhor.map(|(_, linha)| linha))
    }

    /// Aplica os campos dados. `false` = nada mudou (não existe ou a situação não está em `so_de`).
    pub fn execucao_mudar(&mut self, id: &str, m: &Mudanca) -> Resultado<bool> {
        let (colunas, mut valores): (Vec<_>, Vec<_>) = m.sets().into_iter().unzip();
        if colunas.is_empty() {
            return Ok(false);
        }
        let sets = colunas.iter().map(|c| format!("{c} = ?")).collect::<Vec<_>>().join(", ");
        let mut sql = format!("UPDATE execucoes SET {sets} WHERE id = ?");
        valores.push(Sql::Text(id.to_string()));
        if !m.so_de.is_empty() {
            sql += &format!(" AND situacao IN ({})", vec!["?"; m.so_de.len()].join(", "));
            valores.extend(m.so_de.iter().map(|s| Sql::Text(s.to_string())));
        }
        Ok(self.conn.execute(&sql, params_from_iter(valores))? > 0)
    }

    /// marcada → preparando. `false`: alguém já pegou ou cancelou, e o motor desiste.
    pub fn execucao_reivindicar(&mut self, id: &str) -> Resultado<bool> {
        Ok(self.conn.execute("UPDATE execucoes SET situacao = 'preparando' WHERE id = ?1 AND situacao = 'marcada'", [id])? > 0)
    }

    /// preparando → rodando, com segredo e tentativa novos. `false`: cancelou durante o git, e o motor não lança.
    pub fn execucao_lancar(&mut self, id: &str, l: &Lancamento) -> Resultado<bool> {
        let n = self.conn.execute(
            "UPDATE execucoes SET situacao = 'rodando', motivo = '', segredo = ?1, tentativa = tentativa + 1, partida_em = NULL, ultimo_sinal = NULL, terminada_em = NULL,
             iniciada_em = ?2, area = ?3, pasta_trabalho = ?4, repositorio = ?5, ramo = ?6, base = ?7 WHERE id = ?8 AND situacao = 'preparando'",
            (hex_aleatorio(16), l.agora_ms, l.area, l.pasta_trabalho, l.repositorio, l.ramo, l.base, id),
        )?;
        Ok(n > 0)
    }

    /// O invólucro pega o segredo uma vez por tentativa, só com `rodando`. `None` = já partiu ou não é hora.
    pub fn execucao_partir(&mut self, id: &str, agora_ms: i64) -> Resultado<Option<Partida>> {
        let n = self.conn.execute("UPDATE execucoes SET partida_em = ?1 WHERE id = ?2 AND situacao = 'rodando' AND partida_em IS NULL", (agora_ms, id))?;
        if n == 0 {
            return Ok(None);
        }
        Ok(self.execucao_de(id)?.map(|linha| Partida { tentativa: linha.tentativa, segredo: linha.segredo.clone(), linha }))
    }

    /// Qualquer viva vira `cancelada` e o invólucro encerra o filho no próximo sinal. `marcada` não tem processo: já termina aqui. Ramo e cópia ficam.
    pub fn execucao_cancelar(&mut self, id: &str, agora_ms: i64) -> Resultado<bool> {
        let n = self.conn.execute(
            &format!("UPDATE execucoes SET terminada_em = CASE WHEN situacao = 'marcada' THEN ?1 ELSE terminada_em END, situacao = 'cancelada', cancelada_em = ?1, parar = 1 WHERE id = ?2 AND situacao IN {SQL_VIVAS}"),
            (agora_ms, id),
        )?;
        Ok(n > 0)
    }
}
