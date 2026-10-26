//! O servidor HTTP: a interface e o kit (estáticos), /api/dados, /api/mcp e os protótipos. Síncrono (tiny_http), uma thread por pedido,
//! sem runtime assíncrono. As rotas são funções de `Pedido` para `Resposta`, então testam sem abrir socket.
mod api_dados;
pub mod atualizacao;
pub mod config;
pub mod estaticos;
mod estado;
pub mod pedido;
mod prototipos;
pub(crate) mod rotas;
mod servidor;

#[cfg(test)]
mod tests;

pub use config::Config;
pub use estado::Estado;
pub use servidor::{iniciar, Servidor};
