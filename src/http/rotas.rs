//! O roteador: a mesma ordem do servidor Node (página, estáticos, protótipos, /api/mcp, /api/dados, 404).
use super::pedido::{Pedido, Resposta};
use super::{api_dados, prototipos, Estado};
use crate::contexto::Contexto;

const LIMITE_PADRAO: usize = 256 * 1024;
const LIMITE_DADOS: usize = 10 * 1024 * 1024;
const LIMITE_PROTOTIPO: usize = crate::mcp::acoes::PROTOTIPO_MAX + 64 * 1024;

/// Quanto corpo cada rota aceita (o Node decide isto ao ler o corpo; aqui se decide antes de ler).
pub fn limite_corpo(metodo: &str, caminho: &str) -> usize {
    match (metodo, caminho) {
        ("PUT", "/api/dados") => LIMITE_DADOS,
        (_, "/api/mcp/prototipo") => LIMITE_PROTOTIPO,
        _ => LIMITE_PADRAO,
    }
}

pub fn corpo_grande_demais(caminho: &str) -> Resposta {
    let texto = crate::tr!("too large", "grande demais");
    if caminho.starts_with("/api/mcp/") { Resposta::json(413, &serde_json::json!({"erro": texto, "codigo": "grande-demais"})) } else { Resposta::texto(413, "text/plain", texto) }
}

/// Atende o pedido no idioma dele (`Accept-Language`, senão o do usuário): ver `crate::idioma`.
pub fn despachar(estado: &Estado, pedido: &Pedido) -> Resposta {
    crate::idioma::com(crate::idioma::do_pedido(pedido), || despachar_no_idioma(estado, pedido))
}

/// O Contexto é resolvido uma vez aqui e entregue às rotas; toda rota nova tem a forma `fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta`.
fn despachar_no_idioma(estado: &Estado, pedido: &Pedido) -> Resposta {
    let ctx = Contexto::local();
    rotear(estado, pedido, &ctx)
}

fn rotear(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    let caminho = pedido.caminho.as_str();
    let pagina = if caminho == "/" { "/blueprint.html" } else { caminho };
    if let Some(r) = estado.assets.responder(pagina, pedido) {
        return r;
    }
    if let Some(r) = prototipos::rota(estado, caminho) {
        return r;
    }
    if caminho.starts_with("/api/mcp/") {
        return crate::mcp::rotas::rota_com(&estado.mcp, &estado.dados, pedido, ctx);
    }
    if caminho.starts_with("/api/terminal/") {
        return crate::terminal::rotas::rota(estado, pedido);
    }
    if caminho.starts_with("/api/atualizacao") {
        return super::atualizacao::rota(estado, pedido);
    }
    if caminho == "/api/app/mostrar" {
        return mostrar_janela(estado, pedido);
    }
    if caminho == "/api/dados" {
        return api_dados::rota(estado, pedido, ctx);
    }
    Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado"))
}

/// A segunda execução do app pede à primeira que traga a janela à frente. Só POST vindo desta máquina e sem Origin (página de outro
/// site não consegue: o navegador sempre manda Origin em POST entre origens).
fn mostrar_janela(estado: &Estado, pedido: &Pedido) -> Resposta {
    let host = pedido.cabecalho("host").unwrap_or("");
    let host = host.rsplit_once(':').filter(|(_, p)| p.bytes().all(|c| c.is_ascii_digit())).map_or(host, |(h, _)| h);
    if pedido.metodo != "POST" || pedido.cabecalho("origin").is_some() || !matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]") {
        return Resposta::texto(403, "text/plain", "só esta máquina");
    }
    match estado.ao_mostrar.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
        Some(f) => {
            f();
            Resposta::json(200, &serde_json::json!({"ok": true}))
        }
        None => Resposta::texto(404, "text/plain", "sem janela"),
    }
}
