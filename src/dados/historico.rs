//! O histórico de campos (migração 006): o que `aplicar_operacoes` mudou, campo a campo, com quem e a versão. Escrito na mesma transação do
//! lote; a retenção também (100 linhas por item tocado e nada com mais de `DIAS`), então nada roda em repouso. Ver docs/colaboracao.md, seção 2.
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{json, Value};

pub const POR_ITEM: i64 = 100;
pub const DIAS: i64 = 180;
/// `antes` e `depois` maiores que isto viram `{"cortado":true}` (o histórico não guarda subárvores inteiras)
pub const CORTE: usize = 4096;

/// Quem fez a mudança: o id do membro ("local" no modo local) e o nome dele na hora.
#[derive(Clone, Debug, PartialEq)]
pub struct Autor {
    pub id: String,
    pub nome: String,
}

/// Uma linha do histórico. `campo` é `*` para criar, mover e apagar.
pub struct Linha<'a> {
    pub versao: u64,
    pub em: &'a str,
    pub autor: &'a Autor,
    pub workspace_id: &'a str,
    pub blueprint_id: &'a str,
    pub tipo: &'a str,
    pub item_id: &'a str,
    pub campo: &'a str,
    pub operacao: &'a str,
    pub antes: Option<&'a Value>,
    pub depois: Option<&'a Value>,
}

fn cortar(v: Option<&Value>) -> Option<String> {
    let texto = v?.to_string();
    Some(if texto.len() > CORTE { json!({"cortado": true}).to_string() } else { texto })
}

pub fn gravar(conn: &Connection, l: &Linha) -> Resultado<()> {
    conn.execute(
        "INSERT INTO historico_campos (versao, em, membro_id, nome, workspace_id, blueprint_id, tipo, item_id, campo, operacao, antes, depois) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        rusqlite::params![l.versao as i64, l.em, l.autor.id, l.autor.nome, l.workspace_id, l.blueprint_id, l.tipo, l.item_id, l.campo, l.operacao, cortar(l.antes), cortar(l.depois)],
    )?;
    Ok(())
}

/// Quem mudou por último: `{id, nome}` e a hora. `campo = None` procura a linha de apagar do item.
pub fn ultima(conn: &Connection, tipo: &str, item_id: &str, campo: Option<&str>) -> Resultado<Option<(Value, String)>> {
    let (sql, campo) = match campo {
        Some(c) => ("SELECT membro_id, nome, em FROM historico_campos WHERE tipo = ?1 AND item_id = ?2 AND campo = ?3 ORDER BY id DESC LIMIT 1", c),
        None => ("SELECT membro_id, nome, em FROM historico_campos WHERE tipo = ?1 AND item_id = ?2 AND campo = ?3 AND operacao = 'apagar' ORDER BY id DESC LIMIT 1", "*"),
    };
    let linha = conn.query_row(sql, (tipo, item_id, campo), |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?, l.get::<_, String>(2)?))).optional()?;
    Ok(linha.map(|(id, nome, em)| (json!({"id": id, "nome": nome}), em)))
}

/// Retenção dos itens tocados pelo lote e das linhas velhas.
pub fn reter(conn: &Connection, agora_ms: i64, tocados: &[(String, String)]) -> Resultado<()> {
    for (tipo, item) in tocados {
        conn.execute(
            "DELETE FROM historico_campos WHERE tipo = ?1 AND item_id = ?2 AND id NOT IN (SELECT id FROM historico_campos WHERE tipo = ?1 AND item_id = ?2 ORDER BY id DESC LIMIT ?3)",
            (tipo, item, POR_ITEM),
        )?;
    }
    conn.execute("DELETE FROM historico_campos WHERE em < ?1", [crate::tempo::iso_utc(agora_ms - DIAS * 86_400_000)])?;
    Ok(())
}

/// As linhas do workspace (e, se vierem, do blueprint e do item), da mais recente à mais antiga, no formato de `GET /api/historico`.
pub fn listar(conn: &Connection, workspace_id: &str, blueprint_id: Option<&str>, item_id: Option<&str>, limite: i64) -> Resultado<Value> {
    let mut c = conn.prepare(
        "SELECT versao, em, membro_id, nome, tipo, item_id, campo, operacao, antes, depois FROM historico_campos
         WHERE workspace_id = ?1 AND (?2 IS NULL OR blueprint_id = ?2) AND (?3 IS NULL OR item_id = ?3) ORDER BY id DESC LIMIT ?4",
    )?;
    let json_ou_null = |t: Option<String>| t.and_then(|t| serde_json::from_str::<Value>(&t).ok()).unwrap_or(Value::Null);
    let linhas = c.query_map((workspace_id, blueprint_id, item_id, limite), |l| {
        Ok(json!({
            "versao": l.get::<_, i64>(0)?, "em": l.get::<_, String>(1)?, "membro": {"id": l.get::<_, String>(2)?, "nome": l.get::<_, String>(3)?},
            "tipo": l.get::<_, String>(4)?, "itemId": l.get::<_, String>(5)?, "campo": l.get::<_, String>(6)?, "operacao": l.get::<_, String>(7)?,
            "antes": json_ou_null(l.get(8)?), "depois": json_ou_null(l.get(9)?),
        }))
    })?;
    Ok(Value::Array(linhas.collect::<Result<Vec<_>, _>>()?))
}
