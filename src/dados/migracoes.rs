//! Migrações numeradas (migracoes/NNN_*.sql), aplicadas uma vez cada, em ordem, cada uma numa transação.
use crate::erro::Resultado;
use rusqlite::Connection;

/// Para criar uma migração: um arquivo novo em migracoes/ e uma linha aqui. Nunca edite uma já aplicada.
const MIGRACOES: &[(&str, &str)] = &[
    ("001_inicial", include_str!("migracoes/001_inicial.sql")),
    ("002_vocabulario", include_str!("migracoes/002_vocabulario.sql")),
    ("003_pastas", include_str!("migracoes/003_pastas.sql")),
    ("004_linha_do_tempo", include_str!("migracoes/004_linha_do_tempo.sql")),
    ("005_acesso", include_str!("migracoes/005_acesso.sql")),
    ("006_historico", include_str!("migracoes/006_historico.sql")),
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
        tx.execute("INSERT INTO migracoes (nome) VALUES (?1)", [nome])?;
        tx.commit()?;
        novas.push(*nome);
    }
    Ok(novas)
}

/// Nomes das tabelas do usuário (sem as internas do SQLite), em ordem alfabética.
pub fn tabelas(conn: &Connection) -> Resultado<Vec<String>> {
    let mut c = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?;
    let nomes = c.query_map([], |l| l.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    Ok(nomes)
}
