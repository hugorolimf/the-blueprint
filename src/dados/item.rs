//! Um item da árvore por vez (o que `operacoes.rs` precisa): ler e reescrever uma linha, achar o workspace e o blueprint dela, ordenar os irmãos
//! e listar os descendentes. Guiado por `formato.rs`, como a importação e a exportação.
use super::formato::{filhas, Tabela, ACOES, BLUEPRINTS, CHECKPOINTS, COMENTARIOS, LISTA_COMENTARIOS, PLANOS, PROTOTIPOS, RAIZ_ID, WORKSPACES};
use super::linhas::{extrair, inserir, montar, Objeto, AGORA};
use crate::erro::Resultado;
use rusqlite::{params_from_iter, types::Value as Sql, Connection, OptionalExtension};
use serde_json::Value;

/// O nome do tipo nas operações → a tabela.
pub fn tabela(tipo: &str) -> Option<&'static Tabela> {
    Some(match tipo {
        "workspace" => &WORKSPACES,
        "blueprint" => &BLUEPRINTS,
        "acao" => &ACOES,
        "plano" => &PLANOS,
        "checkpoint" => &CHECKPOINTS,
        "prototipo" => &PROTOTIPOS,
        _ => return None,
    })
}

/// O tipo do pai (None para o workspace, que não tem pai).
pub fn tipo_do_pai(tipo: &str) -> Option<&'static str> {
    match tipo {
        "blueprint" => Some("workspace"),
        "acao" => Some("blueprint"),
        "plano" | "checkpoint" | "prototipo" => Some("acao"),
        _ => None,
    }
}

/// As chaves que `alterar` não pode tocar: o id e as listas filhas (que mudam por criar, mover e apagar).
pub fn chave_proibida(t: &Tabela, chave: &str) -> bool {
    chave == "id" || filhas(t).iter().any(|f| f.lista == chave) || (t.nome == WORKSPACES.nome && chave == WORKSPACES.lista)
}

pub struct Lido {
    pub pai: Option<String>,
    pub listas: Vec<String>,
    /// o item sem o id e sem as listas filhas (com `agente.comentarios` no plano)
    pub obj: Objeto,
}

pub fn existe(conn: &Connection, t: &Tabela, id: &str) -> Resultado<bool> {
    Ok(conn.query_row(&format!("SELECT EXISTS (SELECT 1 FROM {} WHERE id = ?1)", t.nome), [id], |l| l.get(0))?)
}

pub fn ler(conn: &Connection, t: &Tabela, id: &str) -> Resultado<Option<Lido>> {
    let pai = t.pai.unwrap_or("NULL");
    let colunas: String = t.colunas.iter().map(|c| format!(", {}", c.coluna)).collect();
    let linha = conn
        .query_row(&format!("SELECT {pai}, listas, extra{colunas} FROM {} WHERE id = ?1", t.nome), [id], |l| {
            let valores = (0..t.colunas.len()).map(|i| l.get::<_, Sql>(3 + i)).collect::<Result<Vec<_>, _>>()?;
            Ok((l.get::<_, Option<String>>(0)?, l.get::<_, String>(1)?, l.get::<_, String>(2)?, valores))
        })
        .optional()?;
    let Some((pai, listas, extra, valores)) = linha else { return Ok(None) };
    let listas: Vec<String> = serde_json::from_str(&listas)?;
    let mut obj = montar(&extra, t.colunas, valores)?;
    if t.nome == PLANOS.nome && listas.iter().any(|x| x == LISTA_COMENTARIOS) {
        let mut c = conn.prepare("SELECT extra, texto, em, tipo, sessao_id FROM comentarios WHERE plano_id = ?1 ORDER BY ordem")?;
        let comentarios = c
            .query_map([id], |l| Ok((l.get::<_, String>(0)?, (1..5).map(|i| l.get::<_, Sql>(i)).collect::<Result<Vec<_>, _>>()?)))?
            .map(|r| -> Resultado<Value> {
                let (extra, valores) = r?;
                Ok(Value::Object(montar(&extra, COMENTARIOS.colunas, valores)?))
            })
            .collect::<Resultado<Vec<_>>>()?;
        if let Some(Value::Object(agente)) = obj.get_mut("agente") {
            agente.insert("comentarios".into(), Value::Array(comentarios));
        }
    }
    Ok(Some(Lido { pai, listas, obj }))
}

/// Reescreve a linha com o objeto dado (sem id e sem listas filhas). No plano, `agente.comentarios` vira as linhas da tabela `comentarios`.
pub fn escrever(conn: &Connection, t: &Tabela, id: &str, mut listas: Vec<String>, mut obj: Objeto) -> Resultado<()> {
    if t.nome == PLANOS.nome {
        conn.execute("DELETE FROM comentarios WHERE plano_id = ?1", [id])?;
        listas.retain(|x| x != LISTA_COMENTARIOS);
        let comentarios = match obj.get_mut("agente") {
            Some(Value::Object(agente)) if matches!(agente.get("comentarios"), Some(Value::Array(_))) => agente.remove("comentarios"),
            _ => None,
        };
        if let Some(Value::Array(itens)) = comentarios {
            listas.push(LISTA_COMENTARIOS.into());
            for (i, c) in itens.into_iter().enumerate() {
                if let Value::Object(c) = c {
                    inserir(conn, &COMENTARIOS, None, Some(id), i as i64, c, &[])?;
                }
            }
        }
    }
    let valores = extrair(&mut obj, t.colunas);
    let sets: String = t.colunas.iter().map(|c| format!(", {} = ?", c.coluna)).collect();
    let mut params = vec![Sql::Text(serde_json::to_string(&listas)?), Sql::Text(Value::Object(obj).to_string())];
    params.extend(valores);
    params.push(Sql::Text(id.to_string()));
    conn.execute(&format!("UPDATE {} SET listas = ?, extra = ?{sets}, revisao = revisao + 1, atualizado_em = {AGORA} WHERE id = ?", t.nome), params_from_iter(params))?;
    Ok(())
}

/// Garante que a lista filha apareça no JSON do pai (a exportação só põe as listas marcadas em `listas`).
pub fn marcar_lista(conn: &Connection, t_pai: &Tabela, pai: &str, lista: &str) -> Resultado<()> {
    let listas: Option<String> = conn.query_row(&format!("SELECT listas FROM {} WHERE id = ?1", t_pai.nome), [pai], |l| l.get(0)).optional()?;
    let mut listas: Vec<String> = serde_json::from_str(&listas.unwrap_or_else(|| "[]".into()))?;
    if !listas.iter().any(|x| x == lista) {
        listas.push(lista.into());
        conn.execute(&format!("UPDATE {} SET listas = ?1 WHERE id = ?2", t_pai.nome), (serde_json::to_string(&listas)?, pai))?;
    }
    Ok(())
}

/// O workspace e o blueprint ("" para o próprio workspace) de um item que existe.
pub fn onde(conn: &Connection, tipo: &str, id: &str) -> Resultado<Option<(String, String)>> {
    let sql = match tipo {
        "workspace" => "SELECT id, '' FROM workspaces WHERE id = ?1",
        "blueprint" => "SELECT workspace_id, id FROM blueprints WHERE id = ?1",
        "acao" => "SELECT b.workspace_id, b.id FROM acoes a JOIN blueprints b ON b.id = a.blueprint_id WHERE a.id = ?1",
        "plano" => "SELECT b.workspace_id, b.id FROM planos x JOIN acoes a ON a.id = x.acao_id JOIN blueprints b ON b.id = a.blueprint_id WHERE x.id = ?1",
        "checkpoint" => "SELECT b.workspace_id, b.id FROM checkpoints x JOIN acoes a ON a.id = x.acao_id JOIN blueprints b ON b.id = a.blueprint_id WHERE x.id = ?1",
        "prototipo" => "SELECT b.workspace_id, b.id FROM prototipos x JOIN acoes a ON a.id = x.acao_id JOIN blueprints b ON b.id = a.blueprint_id WHERE x.id = ?1",
        _ => return Ok(None),
    };
    Ok(conn.query_row(sql, [id], |l| Ok((l.get(0)?, l.get(1)?))).optional()?)
}

/// Os ids dos irmãos na ordem (sem o workspace raiz, que é o próprio arquivo).
pub fn irmaos(conn: &Connection, t: &Tabela, pai: Option<&str>) -> Resultado<Vec<String>> {
    let (sql, param) = match (t.pai, pai) {
        (Some(col), Some(p)) => (format!("SELECT id FROM {} WHERE {col} = ?1 ORDER BY ordem, rowid", t.nome), p.to_string()),
        _ => (format!("SELECT id FROM {} WHERE id <> ?1 ORDER BY ordem, rowid", t.nome), RAIZ_ID.to_string()),
    };
    let mut c = conn.prepare(&sql)?;
    let ids = c.query_map([param], |l| l.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// Põe `id` na posição dada entre os irmãos (cortada ao tamanho) e renumera a ordem. Workspaces começam em 1 (o 0 é o raiz).
pub fn posicionar(conn: &Connection, t: &Tabela, pai: Option<&str>, id: &str, posicao: usize) -> Resultado<()> {
    let mut ids = irmaos(conn, t, pai)?;
    ids.retain(|x| x != id);
    ids.insert(posicao.min(ids.len()), id.to_string());
    let base = if t.pai.is_none() { 1 } else { 0 };
    for (i, x) in ids.iter().enumerate() {
        conn.execute(&format!("UPDATE {} SET ordem = ?1 WHERE id = ?2", t.nome), (base + i as i64, x))?;
    }
    Ok(())
}

/// Os descendentes (tipo, id) de um item, para o histórico de apagar.
pub fn descendentes(conn: &Connection, tipo: &str, id: &str) -> Resultado<Vec<(String, String)>> {
    let mut saida = Vec::new();
    let Some(t) = tabela(tipo) else { return Ok(saida) };
    for f in filhas(t) {
        let tipo_filho = match f.nome { "blueprints" => "blueprint", "acoes" => "acao", "planos" => "plano", "checkpoints" => "checkpoint", _ => "prototipo" };
        for filho in irmaos(conn, f, Some(id))? {
            saida.extend(descendentes(conn, tipo_filho, &filho)?);
            saida.push((tipo_filho.to_string(), filho));
        }
    }
    Ok(saida)
}
