//! Segredos de convite, sessão e chave: 32 bytes do gerador do sistema (`getrandom`; `RandomState` não serve), em base64url sem `=`
//! (43 caracteres). No banco só vai o sha256 em hexadecimal. Nada daqui é escrito em log nem em mensagem de erro.
use sha2::{Digest, Sha256};

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn aleatorio<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    // sem gerador do sistema não há como seguir com segurança
    getrandom::fill(&mut b).expect("o gerador aleatório do sistema falhou");
    b
}

pub fn base64url(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for bloco in bytes.chunks(3) {
        let n = bloco.iter().enumerate().fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..=bloco.len() {
            s.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    s
}

/// 256 bits, 43 caracteres de base64url.
pub fn gerar() -> String {
    base64url(&aleatorio::<32>())
}

/// Id de membro, convite e afins: 16 hexadecimais aleatórios.
pub fn id() -> String {
    aleatorio::<8>().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash(segredo: &str) -> String {
    Sha256::digest(segredo.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Tem a forma de um segredo nosso? (barra lixo antes de tocar o banco; o tamanho é fixo)
pub fn forma_valida(s: &str) -> bool {
    s.len() == 43 && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}
