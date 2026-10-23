//! Quem pode falar com /api/mcp: só esta máquina (Host e Origin em loopback), salvo MCP_REDE=1.
use super::Mcp;
use crate::http::pedido::Pedido;

const LOOPBACK: [&str; 4] = ["127.0.0.1", "localhost", "::1", "[::1]"];

/// `host:porta` → `host` (o `replace(/:\d+$/, "")` do Node; endereços IPv6 entre colchetes ficam inteiros)
fn sem_porta(host: &str) -> &str {
    match host.rsplit_once(':') {
        Some((h, porta)) if !porta.is_empty() && porta.bytes().all(|c| c.is_ascii_digit()) => h,
        _ => host,
    }
}

pub fn permitido(mcp: &Mcp, pedido: &Pedido) -> bool {
    mcp.rede || so_loopback(mcp, pedido)
}

/// Host e Origin em loopback, com ou sem MCP_REDE: as rotas que abrem processo (iniciar) nunca saem desta máquina.
pub(super) fn so_loopback(mcp: &Mcp, pedido: &Pedido) -> bool {
    let host = pedido.cabecalho("host").unwrap_or("");
    if !LOOPBACK.contains(&sem_porta(host)) || !LOOPBACK.contains(&mcp.host.as_str()) {
        return false;
    }
    pedido.cabecalho("origin").is_none_or(|o| o == format!("http://{host}"))
}
