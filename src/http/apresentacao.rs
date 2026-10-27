//! /apresentacao/<id>: a página montada de uma apresentação gerada pela IA (src/apresentacao). O conteúdo e o código das figuras vêm do agente: roda isolado
//! (origem opaca, sem rede, sem acesso ao app), nunca allow-same-origin. `?baixar=1` entrega o mesmo HTML como arquivo (a exportação).
use super::estado::Estado;
use super::pedido::{Corpo, Pedido, Resposta};
use crate::apresentacao::{armazenar, montar::{self, Falha}};
use crate::contexto::{Acao, Contexto};

const CSP: &str = "sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:";

fn texto(status: u16, t: String) -> Option<Resposta> {
    Some(Resposta::texto(status, "text/plain", t))
}

/// None = o caminho não tem a forma de uma apresentação (segue para as outras rotas, que respondem 404).
pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Option<Resposta> {
    let id = pedido.caminho.strip_prefix("/apresentacao/")?;
    if !armazenar::id_valido(id) {
        return None;
    }
    if pedido.metodo != "GET" {
        return texto(405, crate::tr!("method not allowed", "método não permitido"));
    }
    let pasta = estado.mcp.pasta_apresentacoes();
    // fora do app local, só lê quem lê o blueprint dono dela
    if !ctx.local && !armazenar::ler(&pasta, id).is_some_and(|r| { let (ws, bp) = armazenar::alvo(&r); ctx.pode(Acao::Ler, ws, Some(bp)) }) {
        return texto(404, crate::tr!("not found", "não encontrado"));
    }
    let (html, _) = match montar::pagina(&pasta, id) {
        Ok(p) => p,
        Err(Falha::NaoExiste) => return texto(404, crate::tr!("not found", "não encontrado")),
        Err(Falha::SemBiblioteca) => return texto(503, crate::tr!("the presentation library is not part of this build", "a biblioteca de apresentações não faz parte desta compilação")),
    };
    let mut r = Resposta { status: 200, cabecalhos: vec![
        ("content-type".into(), "text/html; charset=utf-8".into()),
        ("cache-control".into(), "no-store".into()),
        ("x-content-type-options".into(), "nosniff".into()),
        ("content-security-policy".into(), CSP.into()),
    ], corpo: Corpo::Bytes(html.into_bytes()) };
    if pedido.parametro("baixar").is_some_and(|v| v == "1") {
        r = r.com("content-disposition", &format!("attachment; filename=\"apresentacao-{id}.html\""));
    }
    Some(r)
}
