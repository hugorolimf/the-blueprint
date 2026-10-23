//! A integridade do instalador: o SHA-256 do arquivo comparado com a linha dele no `SHA256SUMS` da release (formato do `sha256sum`: `<hash>  <nome>`).
//! shortcut: o hash vem da mesma release que o instalador, então prova que o download não corrompeu, mas não quem o publicou; assinatura (ed25519 ou
//! Authenticode) é a melhoria para quando houver uma chave.
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_do_arquivo(caminho: &Path) -> std::io::Result<String> {
    let mut f = std::fs::File::open(caminho)?;
    let mut h = Sha256::new();
    let mut bloco = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut bloco)?;
        if n == 0 {
            break;
        }
        h.update(&bloco[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

/// O hash que o `SHA256SUMS` dá para o arquivo `nome` (minúsculo); None se a linha não existe ou não parece um SHA-256.
pub fn procurar(sums: &str, nome: &str) -> Option<String> {
    sums.lines().find_map(|linha| {
        let (hash, resto) = linha.trim().split_once(char::is_whitespace)?;
        let arquivo = resto.trim().trim_start_matches('*');
        (arquivo == nome && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())).then(|| hash.to_ascii_lowercase())
    })
}

/// O arquivo bate com o hash esperado?
pub fn confere(caminho: &Path, esperado: &str) -> bool {
    sha256_do_arquivo(caminho).is_ok_and(|h| h.eq_ignore_ascii_case(esperado))
}
