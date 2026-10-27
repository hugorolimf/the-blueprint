//! O roteador: a mesma ordem do servidor Node (página, estáticos, protótipos, /api/mcp, /api/dados, 404).
use super::pedido::{Pedido, Resposta};
use super::{api_dados, apresentacao, prototipos, Estado};
use crate::contexto::Contexto;

const LIMITE_PADRAO: usize = 256 * 1024;
const LIMITE_DADOS: usize = 10 * 1024 * 1024;
const LIMITE_MUDANCAS: usize = 2 * 1024 * 1024;
const LIMITE_PROTOTIPO: usize = crate::mcp::acoes::PROTOTIPO_MAX + 64 * 1024;
/// O conteúdo e as figuras (até 2 MB juntos, contrato das apresentações) mais o que o JSON escapa.
const LIMITE_APRESENTACAO: usize = crate::apresentacao::validar::TOTAL_MAX + 256 * 1024;

/// Quanto corpo cada rota aceita (o Node decide isto ao ler o corpo; aqui se decide antes de ler).
pub fn limite_corpo(metodo: &str, caminho: &str) -> usize {
    match (metodo, caminho) {
        ("PUT", "/api/dados") => LIMITE_DADOS,
        ("POST", "/api/mudancas") => LIMITE_MUDANCAS,
        (_, "/api/mcp/prototipo") => LIMITE_PROTOTIPO,
        (_, "/api/mcp/apresentacao" | "/mcp") => LIMITE_APRESENTACAO, // /mcp também leva anexar_prototipo e anexar_apresentacao (o maior dos dois)
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
/// No modo hospedado, `crate::acesso::despachar` confere HTTPS, Host e Origin e resolve o contexto pela sessão ou pela chave.
fn despachar_no_idioma(estado: &Estado, pedido: &Pedido) -> Resposta {
    match &estado.config.hospedagem {
        Some(h) => crate::acesso::despachar(estado, pedido, h, |ctx| rotear(estado, pedido, ctx)),
        None => rotear(estado, pedido, &Contexto::local()),
    }
}

/// A porta dos convidados (src/compartilhar): todo pedido é convidado, nunca `Contexto::local()`. Passa pelo mesmo despacho do modo
/// hospedado (HTTPS pelo túnel, Host e Origin da URL do túnel, sessão ou chave, rotas só locais em 404) e o contexto só alcança o
/// workspace compartilhado. O SSE termina a cada evento, porque o quick tunnel só entrega a resposta inteira (`compartilhar::fluxo`).
pub fn despachar_convidado(estado: &Estado, pedido: &Pedido) -> Resposta {
    crate::idioma::com(crate::idioma::do_pedido(pedido), || {
        let Some((h, ws)) = estado.compartilhar.porta_aberta() else {
            return Resposta::texto(503, "text/plain", crate::tr!("sharing is off", "o compartilhamento está desligado"));
        };
        let r = crate::acesso::despachar(estado, pedido, &h, |ctx| {
            // os dados inteiros (importação) são só do app local: pela porta dos convidados nem o dono do workspace os troca
            if pedido.metodo == "PUT" && pedido.caminho == "/api/dados" {
                return super::pedido::falha(404, crate::tr!("this route only exists in the local app", "esta rota só existe no app local")).codigo("so-local").resposta();
            }
            rotear(estado, pedido, &crate::compartilhar::so_do_workspace(ctx, &ws))
        });
        crate::compartilhar::fluxo::curto(r)
    })
}

fn rotear(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    let caminho = pedido.caminho.as_str();
    // `/convite` (modo hospedado) é a própria página: ela lê o segredo do fragmento, que nunca chega aqui
    let pagina = if caminho == "/" || (caminho == "/convite" && !ctx.local) { "/blueprint.html" } else { caminho };
    if let Some(r) = estado.assets.responder(pagina, pedido) {
        return r;
    }
    if !ctx.local && caminho.starts_with("/prototipos/") && !prototipo_visivel(estado, caminho, ctx) {
        return Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado"));
    }
    if let Some(r) = prototipos::rota(estado, caminho) {
        return r;
    }
    if let Some(r) = apresentacao::rota(estado, pedido, ctx) {
        return r;
    }
    if crate::acesso::rotas::eh_rota(caminho) {
        return crate::acesso::rotas::rota(estado, pedido, ctx);
    }
    if caminho == "/mcp" {
        return crate::mcp::http::rota(estado, pedido, ctx); // o MCP por HTTP: só para convidado e modo hospedado
    }
    if caminho.starts_with("/api/mcp/") {
        return crate::mcp::rotas::rota_estado(estado, pedido, ctx);
    }
    if caminho.starts_with("/api/terminal/") {
        return crate::terminal::rotas::rota(estado, pedido);
    }
    if caminho.starts_with("/api/repositorio/") {
        return crate::repositorio::rotas::rota(estado, pedido);
    }
    if caminho.starts_with("/api/atualizacao") {
        return super::atualizacao::rota(estado, pedido);
    }
    if caminho == "/api/equipes" || caminho.starts_with("/api/equipes/") {
        return super::equipes::rota(estado, pedido);
    }
    if caminho == "/api/compartilhar" || caminho.starts_with("/api/compartilhar/") {
        return crate::compartilhar::rotas::rota(estado, pedido);
    }
    if caminho == "/api/app/mostrar" {
        return mostrar_janela(estado, pedido);
    }
    if caminho == "/api/dados" {
        return api_dados::rota(estado, pedido, ctx);
    }
    // frente SINCRONIA: mudanças por item, eventos, presença e histórico (o Contexto do pedido chega pronto à rota)
    if let Some(r) = crate::sincronia::rotas::rota(estado, pedido, ctx) {
        return r;
    }
    Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado"))
}

/// Modo hospedado: o protótipo só sai para quem lê o blueprint da ação que o anexou.
fn prototipo_visivel(estado: &Estado, caminho: &str, ctx: &Contexto) -> bool {
    let id = caminho.trim_start_matches("/prototipos/").trim_end_matches(".html");
    let mut dados = estado.dados.lock().unwrap_or_else(|e| e.into_inner());
    let Ok(arquivo) = dados.ler_tudo(&Contexto::local()) else { return false };
    crate::acesso::filtro::blueprint_do_prototipo(arquivo, id).is_some_and(|(ws, bp)| ctx.pode(crate::contexto::Acao::Ler, &ws, Some(&bp)))
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
            f(serde_json::from_slice::<serde_json::Value>(&pedido.corpo).ok().and_then(|v| v["url"].as_str().map(String::from)));
            Resposta::json(200, &serde_json::json!({"ok": true}))
        }
        None => Resposta::texto(404, "text/plain", "sem janela"),
    }
}
