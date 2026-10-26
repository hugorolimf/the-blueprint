//! O erro único do backend. As rotas (ação 3.4) traduzem cada variante em um status HTTP.
use std::fmt;

#[derive(Debug)]
pub enum Erro {
    Banco(rusqlite::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    /// dado fora do formato esperado (400)
    Formato(String),
    /// id que não existe (404)
    NaoEncontrado(String),
    /// o contexto do usuário não pode fazer isso (403)
    Proibido(String),
    /// erro de rota já com status e código próprios, levantado dentro de uma transação (`Dados::transacao`)
    Http(Box<crate::http::pedido::ErroHttp>),
}

pub type Resultado<T> = Result<T, Erro>;

impl fmt::Display for Erro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Erro::Banco(e) => write!(f, "banco de dados: {e}"),
            Erro::Json(e) => write!(f, "JSON inválido: {e}"),
            Erro::Io(e) => write!(f, "arquivo: {e}"),
            Erro::Formato(m) | Erro::NaoEncontrado(m) | Erro::Proibido(m) => f.write_str(m),
            Erro::Http(e) => f.write_str(&e.mensagem),
        }
    }
}

impl std::error::Error for Erro {}

impl From<rusqlite::Error> for Erro {
    fn from(e: rusqlite::Error) -> Self {
        Erro::Banco(e)
    }
}
impl From<serde_json::Error> for Erro {
    fn from(e: serde_json::Error) -> Self {
        Erro::Json(e)
    }
}
impl From<std::io::Error> for Erro {
    fn from(e: std::io::Error) -> Self {
        Erro::Io(e)
    }
}
