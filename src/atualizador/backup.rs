//! A cópia do banco feita antes de instalar uma versão nova (ela pode migrar o banco; a cópia é o caminho de volta). Fica em `backups/` ao lado do banco
//! e só as 3 mais recentes são mantidas.
use rusqlite::Connection;
use std::path::{Path, PathBuf};

const PREFIXO: &str = "blueprint-antes-de-";
const MANTER: usize = 3;

/// Copia o banco aberto (com o WAL já consolidado, por `VACUUM INTO`) para `pasta`. Devolve o caminho da cópia.
pub fn copiar_banco(conn: &Connection, pasta: &Path, versao: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(pasta).map_err(|e| format!("não consegui criar a pasta de backups: {e}"))?;
    let seguro: String = versao.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')).collect();
    let alvo = pasta.join(format!("{PREFIXO}{seguro}-{}.db", crate::tempo::agora_ms()));
    conn.execute("VACUUM INTO ?1", [alvo.to_string_lossy().as_ref()]).map_err(|e| format!("não consegui copiar o banco: {e}"))?;
    podar(pasta);
    Ok(alvo)
}

/// Apaga os backups mais antigos, deixando os `MANTER` mais recentes (o nome termina com o instante da cópia).
fn podar(pasta: &Path) {
    let Ok(lista) = std::fs::read_dir(pasta) else { return };
    let mut copias: Vec<(i64, PathBuf)> = lista
        .flatten()
        .filter_map(|e| {
            let nome = e.file_name().to_string_lossy().into_owned();
            let instante = nome.strip_prefix(PREFIXO)?.strip_suffix(".db")?.rsplit('-').next()?.parse().ok()?;
            Some((instante, e.path()))
        })
        .collect();
    copias.sort();
    let sobra = copias.len().saturating_sub(MANTER);
    for (_, caminho) in copias.into_iter().take(sobra) {
        let _ = std::fs::remove_file(caminho);
    }
}
