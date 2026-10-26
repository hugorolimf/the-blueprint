//! The Blueprint (beta): dados em SQLite, validação da proposta de blueprint e contexto do usuário.
//! O servidor HTTP e o MCP entram na ação 3.4; a janela nativa, na 3.5.
pub mod atualizador;
pub mod cli;
pub mod contexto;
pub mod dados;
pub mod erro;
pub mod hook;
pub mod http;
pub mod idioma;
#[cfg(windows)]
pub mod janela;
pub mod mcp;
pub mod blueprint;
pub mod registrar;
pub mod tempo;
pub mod watch;
