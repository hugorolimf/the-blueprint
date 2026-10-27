//! Cópias de segurança do banco sem o usuário pedir: ao abrir o app, no máximo uma por dia em `backups/automaticos` (as 7 mais recentes),
//! e a volta a uma cópia (`blueprint restaurar`, só com o app fechado). As cópias de antes de atualizar ficam em `backups/` (`atualizador::backup`).
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const PREFIXO: &str = "auto-";
const MANTER: usize = 7;
const DIA_MS: i64 = 24 * 60 * 60 * 1000;

/// A pasta `backups` ao lado do banco (a mesma das cópias de antes de atualizar).
pub fn pasta_backups(banco: &Path) -> PathBuf {
    banco.parent().map_or_else(|| "backups".into(), |p| p.join("backups"))
}

/// Copia o banco para `pasta` se a cópia automática mais recente tiver mais de 24 h. Devolve a cópia feita (None: ainda não era hora ou não há banco).
pub fn backup_diario(banco: &Path, pasta: &Path, agora_ms: i64) -> Result<Option<PathBuf>, String> {
    if !banco.is_file() {
        return Ok(None);
    }
    if automaticas(pasta).last().is_some_and(|(em, _)| agora_ms - em < DIA_MS) {
        return Ok(None);
    }
    std::fs::create_dir_all(pasta).map_err(|e| format!("não consegui criar a pasta de backups: {e}"))?;
    let alvo = pasta.join(format!("{PREFIXO}{agora_ms}.db"));
    // conexão própria: o WAL deixa ler enquanto o servidor grava, e o VACUUM INTO tira uma cópia consistente
    let conn = Connection::open(banco).map_err(|e| format!("não consegui abrir o banco: {e}"))?;
    conn.execute("VACUUM INTO ?1", [alvo.to_string_lossy().as_ref()]).map_err(|e| format!("não consegui copiar o banco: {e}"))?;
    let todas = automaticas(pasta);
    for (_, velha) in todas.iter().take(todas.len().saturating_sub(MANTER)) {
        let _ = std::fs::remove_file(velha);
    }
    Ok(Some(alvo))
}

/// As cópias automáticas, da mais antiga para a mais nova (o nome traz o instante da cópia).
fn automaticas(pasta: &Path) -> Vec<(i64, PathBuf)> {
    let Ok(lista) = std::fs::read_dir(pasta) else { return Vec::new() };
    let mut copias: Vec<(i64, PathBuf)> = lista
        .flatten()
        .filter_map(|e| Some((e.file_name().to_str()?.strip_prefix(PREFIXO)?.strip_suffix(".db")?.parse().ok()?, e.path())))
        .collect();
    copias.sort();
    copias
}

/// Todas as cópias que dá para restaurar (automáticas e de antes de atualizar), da mais nova para a mais antiga.
pub fn listar(banco: &Path) -> Vec<PathBuf> {
    let base = pasta_backups(banco);
    let mut copias: Vec<(std::time::SystemTime, PathBuf)> = [base.join("automaticos"), base]
        .iter()
        .filter_map(|p| std::fs::read_dir(p).ok())
        .flat_map(|l| l.flatten())
        .filter(|e| e.path().extension().is_some_and(|x| x == "db") && e.path().is_file())
        .map(|e| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path()))
        .collect();
    copias.sort_by(|a, b| b.0.cmp(&a.0));
    copias.into_iter().map(|(_, p)| p).collect()
}

/// Troca o banco pela `copia`, guardando antes o banco atual em `backups/`. Só com o app fechado (quem chama confere).
/// Devolve onde ficou o banco de antes (None se não havia banco).
pub fn restaurar(banco: &Path, copia: &Path, agora_ms: i64) -> Result<Option<PathBuf>, String> {
    let conferida = Connection::open_with_flags(copia, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .and_then(|c| c.query_row("PRAGMA quick_check", [], |l| l.get::<_, String>(0)))
        .map_err(|e| format!("{} não é um banco válido: {e}", copia.display()))?;
    if conferida != "ok" {
        return Err(format!("{} está corrompido: {conferida}", copia.display()));
    }
    let guardado = if banco.is_file() {
        let pasta = pasta_backups(banco);
        std::fs::create_dir_all(&pasta).map_err(|e| format!("não consegui criar a pasta de backups: {e}"))?;
        let alvo = pasta.join(format!("blueprint-antes-de-restaurar-{agora_ms}.db"));
        let conn = Connection::open(banco).map_err(|e| format!("não consegui abrir o banco atual: {e}"))?;
        conn.execute("VACUUM INTO ?1", [alvo.to_string_lossy().as_ref()]).map_err(|e| format!("não consegui guardar o banco atual: {e}"))?;
        drop(conn);
        Some(alvo)
    } else {
        None
    };
    // o WAL e o índice compartilhado do banco antigo não valem para a cópia
    for sufixo in ["-wal", "-shm"] {
        let mut p = banco.as_os_str().to_owned();
        p.push(sufixo);
        let _ = std::fs::remove_file(PathBuf::from(p));
    }
    std::fs::copy(copia, banco).map_err(|e| format!("não consegui copiar {} para {}: {e}", copia.display(), banco.display()))?;
    Ok(guardado)
}
