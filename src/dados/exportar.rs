//! Tabelas → data.json, no mesmo formato que a página lê e grava. Constrói de baixo para cima: comentários, planos, ações, blueprints, workspaces.
use super::formato::{Tabela, ACOES, BLUEPRINTS, CHECKPOINTS, COMENTARIOS, LISTA_COMENTARIOS, PLANOS, PROTOTIPOS, RAIZ_ID, WORKSPACES};
use super::linhas::{montar, Objeto};
use crate::erro::{Erro, Resultado};
use rusqlite::{types::Value as Sql, Connection};
use serde_json::Value;
use std::collections::HashMap;

/// itens já montados, agrupados pelo id do pai e na ordem dos irmãos
type Grupos = HashMap<String, Vec<Value>>;

struct Linha {
    id: Option<String>,
    pai: Option<String>,
    listas: Vec<String>,
    obj: Objeto,
}

fn carregar(conn: &Connection, t: &Tabela) -> Resultado<Vec<Linha>> {
    let id = if t.sem_id { "NULL" } else { "id" };
    let (pai, ordena) = match t.pai {
        Some(p) => (p, format!("{p}, ordem")),
        None => ("NULL", "ordem".to_string()),
    };
    let colunas: String = t.colunas.iter().map(|c| format!(", {}", c.coluna)).collect();
    let sql = format!("SELECT {id}, {pai}, listas, extra{colunas} FROM {} ORDER BY {ordena}", t.nome);
    let mut c = conn.prepare(&sql)?;
    let mut linhas = c.query([])?;
    let mut saida = Vec::new();
    while let Some(l) = linhas.next()? {
        let id: Option<String> = if t.sem_id { None } else { l.get(0)? };
        let valores = (0..t.colunas.len()).map(|i| l.get::<_, Sql>(4 + i)).collect::<Result<Vec<_>, _>>()?;
        let listas: Vec<String> = serde_json::from_str(&l.get::<_, String>(2)?)?;
        saida.push(Linha { id, pai: l.get(1)?, listas, obj: montar(&l.get::<_, String>(3)?, t.colunas, valores)? });
    }
    Ok(saida)
}

/// Põe o `id` no objeto, encaixa as listas filhas que existiam no JSON e entrega agrupado pelo pai.
fn agrupar(linhas: Vec<Linha>, filhas: &mut [(&str, &mut Grupos)]) -> Grupos {
    let mut saida = Grupos::new();
    for l in linhas {
        let mut obj = l.obj;
        if let Some(id) = &l.id {
            obj.insert("id".into(), Value::String(id.clone()));
        }
        for (lista, grupos) in filhas.iter_mut() {
            if l.listas.iter().any(|x| x == *lista) {
                let itens = l.id.as_ref().and_then(|id| grupos.remove(id)).unwrap_or_default();
                obj.insert((*lista).to_string(), Value::Array(itens));
            }
        }
        saida.entry(l.pai.unwrap_or_default()).or_default().push(Value::Object(obj));
    }
    saida
}

pub fn exportar(conn: &Connection) -> Resultado<Value> {
    let mut comentarios = agrupar(carregar(conn, &COMENTARIOS)?, &mut []);
    let mut planos = carregar(conn, &PLANOS)?;
    let mut consulta_tempos = conn.prepare("SELECT plano_id, inicio_em, fim_em FROM tempos_planos")?;
    let tempos: HashMap<String, (Option<String>, Option<String>)> = consulta_tempos.query_map([], |l| Ok((l.get(0)?, (l.get(1)?, l.get(2)?))))?.collect::<Result<_, _>>()?;
    for p in &mut planos {
        if let Some((inicio, fim)) = p.id.as_ref().and_then(|id| tempos.get(id)) {
            let agente = p.obj.entry("agente".to_string()).or_insert_with(|| serde_json::json!({}));
            if let Some(a) = agente.as_object_mut() {
                for (chave, valor) in [("inicioEm", inicio), ("feitoEm", fim)] {
                    if let Some(valor) = valor { a.entry(chave.to_string()).or_insert_with(|| Value::String(valor.clone())); }
                }
            }
        }
    }
    for p in planos.iter_mut().filter(|p| p.listas.iter().any(|x| x == LISTA_COMENTARIOS)) {
        let itens = p.id.as_ref().and_then(|id| comentarios.remove(id)).unwrap_or_default();
        if let Some(Value::Object(agente)) = p.obj.get_mut("agente") {
            agente.insert("comentarios".into(), Value::Array(itens));
        }
    }
    let mut planos = agrupar(planos, &mut []);
    let mut checkpoints = agrupar(carregar(conn, &CHECKPOINTS)?, &mut []);
    let mut prototipos = agrupar(carregar(conn, &PROTOTIPOS)?, &mut []);
    let mut acoes = agrupar(
        carregar(conn, &ACOES)?,
        &mut [(PLANOS.lista, &mut planos), (CHECKPOINTS.lista, &mut checkpoints), (PROTOTIPOS.lista, &mut prototipos)],
    );
    let mut blueprints = agrupar(carregar(conn, &BLUEPRINTS)?, &mut [(ACOES.lista, &mut acoes)]);

    let mut raiz: Option<Objeto> = None;
    let mut raiz_listas: Vec<String> = Vec::new();
    let mut workspaces: Vec<Value> = Vec::new();
    for l in carregar(conn, &WORKSPACES)? {
        let eh_raiz = l.id.as_deref() == Some(RAIZ_ID);
        let mut obj = l.obj;
        if !eh_raiz {
            if let Some(id) = &l.id {
                obj.insert("id".into(), Value::String(id.clone()));
            }
        }
        if l.listas.iter().any(|x| x == BLUEPRINTS.lista) {
            let itens = l.id.as_ref().and_then(|id| blueprints.remove(id)).unwrap_or_default();
            obj.insert(BLUEPRINTS.lista.into(), Value::Array(itens));
        }
        if eh_raiz {
            raiz_listas = l.listas;
            raiz = Some(obj);
        } else {
            workspaces.push(Value::Object(obj));
        }
    }
    let mut raiz = raiz.ok_or_else(|| Erro::NaoEncontrado("o banco está vazio: importe um data.json antes".into()))?;
    if raiz_listas.iter().any(|x| x == WORKSPACES.lista) {
        raiz.insert(WORKSPACES.lista.into(), Value::Array(workspaces));
    }
    Ok(Value::Object(raiz))
}
