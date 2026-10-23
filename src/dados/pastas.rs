//! Vínculo pasta → blueprint: de qual blueprint é a pasta em que o agente abriu a sessão. Sem a linha `Blueprint:` no CLAUDE.md, é o app quem sabe.
//! shortcut: o caminho é comparado em minúsculas e com `/` (certo no Windows; numa pasta Linux que só difere por maiúscula, duas pastas se confundem).
use crate::erro::Resultado;
use rusqlite::Connection;

/// `C:\Dev\Proj\` e `c:/dev/proj` são a mesma pasta.
pub fn normalizar(caminho: &str) -> String {
    caminho.trim().replace('\\', "/").trim_end_matches('/').to_lowercase()
}

pub fn vincular(conn: &Connection, caminho: &str, workspace_id: &str, blueprint_id: &str) -> Resultado<()> {
    conn.execute(
        "INSERT INTO pastas (caminho, workspace_id, blueprint_id) VALUES (?1, ?2, ?3)
         ON CONFLICT (caminho) DO UPDATE SET workspace_id = ?2, blueprint_id = ?3",
        (normalizar(caminho), workspace_id, blueprint_id),
    )?;
    Ok(())
}

/// O blueprint da pasta, ou o da pasta-mãe mais próxima ligada (uma sessão aberta numa subpasta é do mesmo projeto).
pub fn procurar(conn: &Connection, caminho: &str) -> Resultado<Option<(String, String)>> {
    let alvo = normalizar(caminho);
    let mut c = conn.prepare("SELECT caminho, workspace_id, blueprint_id FROM pastas")?;
    let linhas = c.query_map([], |l| Ok((l.get::<_, String>(0)?, l.get::<_, String>(1)?, l.get::<_, String>(2)?)))?;
    let mut melhor: Option<(String, String, String)> = None;
    for linha in linhas {
        let (pasta, ws, bp) = linha?;
        let dentro = alvo == pasta || alvo.strip_prefix(&pasta).is_some_and(|resto| resto.starts_with('/'));
        if dentro && melhor.as_ref().is_none_or(|m| pasta.len() > m.0.len()) {
            melhor = Some((pasta, ws, bp));
        }
    }
    Ok(melhor.map(|(_, ws, bp)| (ws, bp)))
}


/// A pasta ligada a um blueprint (se houver mais de uma, a mais rasa), no formato guardado (minúsculas, com `/`).
pub fn pasta_do_blueprint(conn: &Connection, workspace_id: &str, blueprint_id: &str) -> Resultado<Option<String>> {
    let mut c = conn.prepare("SELECT caminho FROM pastas WHERE workspace_id = ?1 AND blueprint_id = ?2 ORDER BY length(caminho), caminho LIMIT 1")?;
    let achada = c.query_map((workspace_id, blueprint_id), |l| l.get::<_, String>(0))?.next().transpose()?;
    Ok(achada)
}

pub fn listar(conn: &Connection) -> Resultado<serde_json::Value> {
    let mut c = conn.prepare("SELECT caminho, workspace_id, blueprint_id FROM pastas ORDER BY caminho")?;
    let linhas = c.query_map([], |l| Ok(serde_json::json!({"cwd": l.get::<_, String>(0)?, "workspaceId": l.get::<_, String>(1)?, "blueprintId": l.get::<_, String>(2)?})))?;
    Ok(serde_json::Value::Array(linhas.collect::<Result<Vec<_>, _>>()?))
}

pub fn desvincular(conn: &Connection, caminho: &str) -> Resultado<()> {
    conn.execute("DELETE FROM pastas WHERE caminho = ?1", [normalizar(caminho)])?;
    Ok(())
}
