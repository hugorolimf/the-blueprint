//! As rotas de acesso (docs/colaboracao.md, seção 5): sessão, entrada pelo convite e chave de MCP aqui; pessoas, convites e atividade em
//! `rotas_pessoas`. No modo local só `GET /api/sessao` existe (`{"modo":"local"}`); o resto responde 404 `so-servidor`.
use super::{chaves, convites, filtro, rotas_pessoas, sessoes};
use crate::contexto::Contexto;
use crate::dados::atividade::{self, Registro};
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use serde_json::{json, Value};
use std::sync::MutexGuard;

pub(super) type R = Result<Resposta, ErroHttp>;

/// É uma rota daqui? (o roteador manda para `rota`)
pub fn eh_rota(caminho: &str) -> bool {
    matches!(caminho, "/api/sessao" | "/api/sessao/sair" | "/api/pessoas" | "/api/convites" | "/api/chave" | "/api/atividade")
        || ["/api/convite/", "/api/pessoas/", "/api/convites/", "/api/chave/"].iter().any(|p| caminho.starts_with(p))
}

pub(super) fn travar(estado: &Estado) -> MutexGuard<'_, Dados> {
    estado.dados.lock().unwrap_or_else(|e| e.into_inner())
}

pub(super) fn corpo(pedido: &Pedido) -> Result<Value, ErroHttp> {
    if !pedido.eh_json() {
        return Err(falha(415, crate::tr!("send application/json", "envie application/json")));
    }
    if pedido.corpo.is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_slice(&pedido.corpo).map_err(|_| falha(400, crate::tr!("invalid JSON", "JSON inválido")).codigo("json-invalido"))
}

pub(super) fn texto<'a>(v: &'a Value, chave: &str) -> &'a str {
    v.get(chave).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn proibido() -> ErroHttp {
    falha(403, crate::tr!("you do not have permission for this", "você não tem permissão para isso"))
}

fn metodo_errado() -> ErroHttp {
    falha(405, crate::tr!("method not allowed", "método não permitido"))
}

pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    let caminho = pedido.caminho.as_str();
    let resultado = match (estado.config.hospedagem.as_ref(), caminho) {
        (None, "/api/sessao") if pedido.metodo == "GET" => Ok(Resposta::json(200, &json!({"modo": "local"}))),
        (None, _) => Err(falha(404, crate::tr!("this route only exists in the hosted server", "esta rota só existe no servidor hospedado")).codigo("so-servidor")),
        (Some(_), "/api/sessao") if pedido.metodo == "GET" => sessao(estado, ctx),
        (Some(_), "/api/sessao/sair") if pedido.metodo == "POST" => sair(estado, pedido, ctx),
        (Some(_), "/api/convite/conferir") if pedido.metodo == "POST" => conferir(estado, pedido),
        (Some(_), "/api/convite/entrar") if pedido.metodo == "POST" => entrar(estado, pedido, ctx),
        (Some(h), "/api/chave" | "/api/chave/revogar") => chave(estado, pedido, ctx, &h.url_publica),
        (Some(h), _) if eh_rota(caminho) && !caminho.starts_with("/api/sessao") && !caminho.starts_with("/api/convite/") => rotas_pessoas::rota(estado, pedido, ctx, &h.url_publica),
        (Some(_), _) if eh_rota(caminho) => Err(metodo_errado()),
        _ => Err(falha(404, crate::tr!("not found", "não encontrado"))),
    };
    resultado.unwrap_or_else(|e| e.resposta())
}

fn sessao(estado: &Estado, ctx: &Contexto) -> R {
    if !ctx.autenticado() {
        return Err(falha(401, crate::tr!("sign in through an invite link", "entre por um link de convite")).codigo("sem-sessao"));
    }
    let _ = estado;
    let acessos: Vec<Value> = ctx.acessos.iter().map(|a| json!({"workspaceId": a.workspace_id, "blueprintId": a.blueprint_id, "papel": a.papel.texto(), "abre": a.abre})).collect();
    let membro = (!ctx.visitante).then(|| json!({"id": ctx.usuario, "nome": ctx.nome}));
    Ok(Resposta::json(200, &json!({"modo": "servidor", "membro": membro, "visitante": ctx.visitante, "acessos": acessos})))
}

fn sair(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> R {
    if let Some(valor) = pedido.cookie(sessoes::COOKIE).filter(|v| super::segredo::forma_valida(v)) {
        let agora = crate::tempo::agora_ms();
        travar(estado).transacao(|t| {
            crate::dados::sessoes::apagar(t, &super::segredo::hash(valor))?;
            if ctx.autenticado() && !ctx.visitante {
                atividade::gravar(t, agora, &Registro { membro_id: Some(&ctx.usuario), nome: &ctx.nome, tipo: "saiu", workspace_id: "", blueprint_id: "", alvo: "" })?;
            }
            Ok(())
        })?;
    }
    Ok(Resposta::json(200, &json!({"ok": true})).com("set-cookie", &sessoes::cookie_vencido()))
}

/// O mesmo erro para segredo inexistente, vencido, revogado ou esgotado; cada falha conta para o limite de tentativas.
fn invalido(estado: &Estado, ip: &str, agora: i64) -> ErroHttp {
    estado.acesso.limite.lock().unwrap_or_else(|e| e.into_inner()).falhou(ip, agora);
    falha(404, crate::tr!("invalid, expired or revoked invite", "convite inválido, vencido ou revogado")).codigo("convite-invalido")
}

/// 429 `muitas-tentativas` antes de tocar o banco, se o IP (ou o total) passou do limite.
fn limitar(estado: &Estado, pedido: &Pedido, agora: i64) -> Result<String, ErroHttp> {
    let ip = estado.config.hospedagem.as_ref().map_or_else(|| pedido.ip.clone(), |h| super::cabecalhos::ip_cliente(h, pedido));
    if let Some(segundos) = estado.acesso.limite.lock().unwrap_or_else(|e| e.into_inner()).bloqueado(&ip, agora) {
        return Err(falha(429, crate::tr!("too many attempts; try again in {segundos} s", "tentativas demais; tente de novo em {segundos} s")).codigo("muitas-tentativas").params(json!({"segundos": segundos})));
    }
    Ok(ip)
}

fn conferir(estado: &Estado, pedido: &Pedido) -> R {
    let agora = crate::tempo::agora_ms();
    let ip = limitar(estado, pedido, agora)?;
    let c = corpo(pedido)?;
    let mut dados = travar(estado);
    let Some(convite) = convites::conferir(&dados, texto(&c, "segredo"), agora)? else {
        drop(dados);
        return Err(invalido(estado, &ip, agora));
    };
    let arquivo = dados.ler_tudo(&Contexto::local())?;
    let (ws, bp) = (convite.workspace_id.as_str(), convite.blueprint_id.as_str());
    let workspace = json!({"id": ws, "nome": filtro::nome_do_workspace(arquivo, ws).unwrap_or_default()});
    let blueprint = (!bp.is_empty()).then(|| json!({"id": bp, "nome": filtro::nome_do_blueprint(arquivo, ws, bp).unwrap_or_default()}));
    let pede_nome = convite.papel != "visitante";
    Ok(Resposta::json(200, &json!({"papel": convite.papel, "workspace": workspace, "blueprint": blueprint, "abre": convite.abre, "pedeNome": pede_nome})))
}

fn entrar(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> R {
    let agora = crate::tempo::agora_ms();
    let ip = limitar(estado, pedido, agora)?;
    let c = corpo(pedido)?;
    let entrada = convites::entrar(&mut travar(estado), ctx, texto(&c, "segredo"), c.get("nome").and_then(Value::as_str), agora)?;
    let Some(e) = entrada else { return Err(invalido(estado, &ip, agora)) };
    Ok(Resposta::json(200, &json!({"ok": true, "destino": e.destino})).com("set-cookie", &sessoes::cookie(&e.cookie, e.max_age)))
}

fn chave(estado: &Estado, pedido: &Pedido, ctx: &Contexto, url_publica: &str) -> R {
    if ctx.visitante {
        return Err(proibido());
    }
    let agora = crate::tempo::agora_ms();
    let registro = |tipo: &'static str| Registro { membro_id: Some(&ctx.usuario), nome: &ctx.nome, tipo, workspace_id: "", blueprint_id: "", alvo: &ctx.usuario };
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/chave") => {
            let k = crate::dados::sessoes::chave_do_membro(travar(estado).conexao(), &ctx.usuario)?;
            Ok(Resposta::json(200, &json!({"chave": k.map(|(prefixo, criada, usada)| json!({"prefixo": prefixo, "criadaEm": criada, "usadaEm": usada}))})))
        }
        ("POST", "/api/chave") => {
            corpo(pedido)?;
            let (chave, prefixo) = travar(estado).transacao(|t| {
                let gerada = chaves::gerar(t, &ctx.usuario)?;
                atividade::gravar(t, agora, &registro("chave-criada"))?;
                Ok(gerada)
            })?;
            Ok(Resposta::json(201, &json!({"chave": chave, "prefixo": prefixo, "comandos": chaves::comandos(url_publica, &chave)})))
        }
        ("POST", "/api/chave/revogar") => {
            corpo(pedido)?;
            travar(estado).transacao(|t| {
                if crate::dados::sessoes::revogar_chave(t, &ctx.usuario)? {
                    atividade::gravar(t, agora, &registro("chave-revogada"))?;
                }
                Ok(())
            })?;
            Ok(Resposta::json(200, &json!({"ok": true})))
        }
        _ => Err(metodo_errado()),
    }
}
