//! Migrações numeradas (migracoes/NNN_*.sql), aplicadas uma vez cada, em ordem, cada uma numa transação.
use crate::erro::Resultado;
use rusqlite::Connection;
use serde_json::Value;

/// Para criar uma migração: um arquivo novo em migracoes/ e uma linha aqui. Nunca edite uma já aplicada.
const MIGRACOES: &[(&str, &str)] = &[
    ("001_inicial", include_str!("migracoes/001_inicial.sql")),
    ("002_vocabulario", include_str!("migracoes/002_vocabulario.sql")),
    ("003_pastas", include_str!("migracoes/003_pastas.sql")),
    ("004_linha_do_tempo", include_str!("migracoes/004_linha_do_tempo.sql")),
    ("005_acesso", include_str!("migracoes/005_acesso.sql")),
    ("006_historico", include_str!("migracoes/006_historico.sql")),
    ("007_tempos_planos", include_str!("migracoes/007_tempos_planos.sql")),
    ("008_fila_mcp", include_str!("migracoes/008_fila_mcp.sql")),
    ("009_planejar", include_str!("migracoes/009_planejar.sql")),
    ("010_sem_datas", include_str!("migracoes/010_sem_datas.sql")),
];

/// Aplica as que faltam e devolve os nomes aplicados agora.
pub fn aplicar(conn: &mut Connection) -> Resultado<Vec<&'static str>> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS migracoes (nome TEXT PRIMARY KEY, aplicada_em TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')))")?;
    let mut novas = Vec::new();
    for (nome, sql) in MIGRACOES {
        let ja: bool = conn.query_row("SELECT EXISTS (SELECT 1 FROM migracoes WHERE nome = ?1)", [nome], |l| l.get(0))?;
        if ja {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        if *nome == "010_sem_datas" {
            limpar_json_aninhado(&tx)?;
        }
        tx.execute("INSERT INTO migracoes (nome) VALUES (?1)", [nome])?;
        tx.commit()?;
        novas.push(*nome);
    }
    Ok(novas)
}

/// O que o SQL de `010_sem_datas` não alcança: as datas dentro de listas no JSON das versões do plano, da fila do MCP e das decisões do blueprint.
fn limpar_json_aninhado(tx: &Connection) -> Resultado<()> {
    for (tabela, chave, coluna) in [("plano_versoes", "rowid", "plano"), ("fila_mcp", "id", "item"), ("fila_mcp", "id", "entrada"), ("blueprints", "id", "extra")] {
        let mut leitura = tx.prepare(&format!("SELECT {chave}, {coluna} FROM {tabela} WHERE {coluna} IS NOT NULL AND json_valid({coluna})"))?;
        let linhas = leitura.query_map([], |l| Ok((l.get::<_, rusqlite::types::Value>(0)?, l.get::<_, String>(1)?)))?.collect::<Result<Vec<_>, _>>()?;
        for (id, texto) in linhas {
            let mut valor: Value = serde_json::from_str(&texto)?;
            sem_datas(&mut valor);
            tx.execute(&format!("UPDATE {tabela} SET {coluna} = ?1 WHERE {chave} = ?2"), (valor.to_string(), id))?;
        }
    }
    Ok(())
}

/// Em qualquer nível: ações sem início e fim (`de`/`ate` no plano novo), planos sem horas, checkpoints e decisões sem data.
fn sem_datas(v: &mut Value) {
    let tirar = |lista: Option<&mut Value>, chaves: &[&str]| {
        for item in lista.and_then(Value::as_array_mut).into_iter().flatten().filter_map(Value::as_object_mut) {
            chaves.iter().for_each(|c| drop(item.remove(*c)));
        }
    };
    match v {
        Value::Object(o) => {
            tirar(o.get_mut("acoes"), &["inicio", "fim", "semData", "de", "ate"]);
            tirar(o.get_mut("planos"), &["horas", "feitoEm", "inicioEm"]);
            tirar(o.get_mut("checkpoints"), &["data", "feitoEm"]);
            tirar(o.get_mut("decisoes"), &["data"]);
            o.values_mut().for_each(sem_datas);
        }
        Value::Array(itens) => itens.iter_mut().for_each(sem_datas),
        _ => {}
    }
}

/// Nomes das tabelas do usuário (sem as internas do SQLite), em ordem alfabética.
pub fn tabelas(conn: &Connection) -> Resultado<Vec<String>> {
    let mut c = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?;
    let nomes = c.query_map([], |l| l.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(nomes)
}
