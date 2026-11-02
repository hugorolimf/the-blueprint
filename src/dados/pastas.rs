//! Vínculo pasta → blueprint: de qual blueprint é a pasta em que o agente abriu a sessão. Sem a linha `Blueprint:` no CLAUDE.md, é o app quem sabe.
//! shortcut: o caminho é comparado em minúsculas e com `/` (certo no Windows; numa pasta Linux que só difere por maiúscula, duas pastas se confundem).
use crate::erro::Resultado;
use rusqlite::{Connection, OptionalExtension};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};

/// `C:\Dev\Proj\` e `c:/dev/proj` são a mesma pasta.
pub fn normalizar(caminho: &str) -> String {
    caminho.trim().replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// O caminho como o usuário o deu, sem espaço nas pontas nem barra no fim: é o que existe no disco quando o sistema distingue maiúsculas.
fn caminho_real(caminho: &str) -> String {
    let limpo = caminho.trim();
    let sem_barra = if cfg!(windows) { limpo.trim_end_matches(['/', '\\']) } else { limpo.trim_end_matches('/') };
    if sem_barra.is_empty() { limpo.to_string() } else { sem_barra.to_string() }
}

pub fn vincular(conn: &Connection, caminho: &str, workspace_id: &str, blueprint_id: &str) -> Resultado<()> {
    conn.execute(
        "INSERT INTO pastas (caminho, workspace_id, blueprint_id, caminho_real) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT (caminho) DO UPDATE SET workspace_id = ?2, blueprint_id = ?3, caminho_real = ?4",
        (normalizar(caminho), workspace_id, blueprint_id, caminho_real(caminho)),
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

/// A pasta mais rasa do blueprint como ela é no disco; `None` se não é mais pasta. Vínculo antigo (sem `caminho_real`) é resolvido no disco uma vez e fica gravado.
/// Nada vem do `cwd` de hook: subpasta e cópia de agente gravariam o caminho errado, pois `procurar` casa por prefixo.
pub fn pasta_real_do_blueprint(conn: &Connection, workspace_id: &str, blueprint_id: &str) -> Resultado<Option<String>> {
    let achada: Option<(String, Option<String>)> = conn
        .query_row("SELECT caminho, caminho_real FROM pastas WHERE workspace_id = ?1 AND blueprint_id = ?2 ORDER BY length(caminho), caminho LIMIT 1", (workspace_id, blueprint_id), |l| Ok((l.get(0)?, l.get(1)?)))
        .optional()?;
    let Some((caminho, real)) = achada else { return Ok(None) };
    let real = match real {
        Some(real) => real,
        None => {
            let Some(resolvido) = resolver_caixa(&caminho) else { return Ok(None) };
            conn.execute("UPDATE pastas SET caminho_real = ?1 WHERE caminho = ?2", (&resolvido, &caminho))?;
            resolvido
        }
    };
    Ok(Path::new(&real).is_dir().then_some(real))
}

/// Anda no disco segmento a segmento procurando as pastas sem olhar a caixa. Igual exato vence; dois candidatos não dá para decidir e dá `None`.
fn resolver_caixa(caminho: &str) -> Option<String> {
    let alvo = Path::new(caminho);
    if !alvo.is_absolute() {
        return None;
    }
    let mut atual = PathBuf::new();
    for parte in alvo.components() {
        match parte {
            Component::Prefix(_) | Component::RootDir => atual.push(parte),
            Component::Normal(segmento) => atual = proximo_segmento(atual, segmento)?,
            Component::CurDir | Component::ParentDir => return None,
        }
    }
    atual.into_os_string().into_string().ok()
}

fn proximo_segmento(pasta: PathBuf, segmento: &OsStr) -> Option<PathBuf> {
    let exato = pasta.join(segmento);
    if exato.is_dir() {
        return Some(exato);
    }
    let procurado = segmento.to_str()?.to_lowercase();
    let mut candidatos = fs::read_dir(&pasta).ok()?.filter_map(Result::ok).filter(|e| e.file_name().to_str().is_some_and(|n| n.to_lowercase() == procurado)).map(|e| e.path()).filter(|p| p.is_dir());
    let unico = candidatos.next()?;
    candidatos.next().is_none().then_some(unico)
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
