//! As rotas da sincronização: `POST /api/mudancas`, `GET /api/eventos`, `POST /api/presenca` e `GET /api/historico` (docs/colaboracao.md,
//! seções 5 e 6). Recebem o `Contexto` resolvido no despacho; autoria e permissão passam só por `super::{autor, exige_gravar, pode_ler}`.
use super::{autor, exige_gravar, operacoes, pode_ler, Evento, Fluxo, Onde};
use crate::contexto::Contexto;
use crate::dados::Falha;
use crate::http::pedido::{falha, Corpo, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

/// Atende as rotas desta frente; None = o caminho não é dela.
pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Option<Resposta> {
    let r = match pedido.caminho.as_str() {
        "/api/mudancas" => so(pedido, "POST").and_then(|_| mudancas(estado, pedido, ctx)),
        "/api/eventos" => so(pedido, "GET").and_then(|_| eventos(estado, pedido, ctx)),
        "/api/presenca" => so(pedido, "POST").and_then(|_| presenca(estado, pedido, ctx)),
        "/api/historico" => so(pedido, "GET").and_then(|_| historico(estado, pedido, ctx)),
        _ => return None,
    };
    Some(r.unwrap_or_else(|e| e.resposta()))
}

fn so(pedido: &Pedido, metodo: &str) -> Result<(), ErroHttp> {
    if pedido.metodo != metodo {
        return Err(falha(405, crate::tr!("method not allowed", "método não permitido")).codigo("metodo"));
    }
    if metodo == "POST" && !pedido.eh_json() {
        return Err(falha(415, crate::tr!("send application/json", "envie application/json")).codigo("json"));
    }
    Ok(())
}

fn workspace(pedido: &Pedido, ctx: &Contexto) -> Result<String, ErroHttp> {
    let ws = pedido.parametro("workspace").unwrap_or_default();
    if ws.is_empty() {
        return Err(falha(400, crate::tr!("missing workspace", "falta o workspace")).codigo("falta-workspace"));
    }
    if !pode_ler(ctx, &ws, pedido.parametro("blueprint").as_deref().filter(|b| !b.is_empty())) {
        return Err(falha(403, crate::tr!("forbidden", "proibido")));
    }
    Ok(ws)
}

fn invalida(indice: Option<usize>, motivo: &str) -> ErroHttp {
    falha(400, crate::tr!("invalid operation: {motivo}", "operação inválida: {motivo}")).codigo("operacao-invalida").params(json!({"indice": indice, "motivo": motivo}))
}

fn mudancas(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Result<Resposta, ErroHttp> {
    let lote = operacoes::ler(&pedido.corpo).map_err(|e| invalida(e.indice, &e.motivo))?;
    let quem = autor(ctx);
    // a cópia travada serializa as gravações (como o PUT): versão, banco e buffer andam juntos
    let mut guardado = estado.dados_json.lock().unwrap_or_else(|e| e.into_inner());
    let atual = estado.versao_dados.load(Ordering::SeqCst);
    let nova = atual + 1;
    let exige = |adm: bool, ws: &str, bp: Option<&str>| exige_gravar(ctx, adm, ws, bp);
    let saida = estado.dados.lock().unwrap_or_else(|e| e.into_inner()).aplicar_operacoes(&quem, nova, crate::tempo::agora_ms(), &lote.ops, &exige);
    let saida = match saida {
        Ok(s) => s,
        Err(Falha::Invalida { indice, motivo }) => return Err(invalida(Some(indice), &motivo)),
        Err(Falha::Proibida { indice, motivo }) => return Err(falha(403, motivo).params(json!({"indice": indice}))),
        Err(Falha::Erro(e)) => return Err(e.into()),
    };
    let versao = if saida.aplicadas.is_empty() { atual } else { nova };
    if !saida.aplicadas.is_empty() {
        *guardado = None;
        estado.versao_dados.store(nova, Ordering::SeqCst);
        // um evento por blueprint tocado, na ordem em que aparecem no lote
        let mut grupos: Vec<(String, String, Vec<Value>)> = Vec::new();
        for a in &saida.aplicadas {
            let op = lote.brutas[a.indice].clone();
            match grupos.iter_mut().find(|(w, b, _)| *w == a.workspace_id && *b == a.blueprint_id) {
                Some(g) => g.2.push(op),
                None => grupos.push((a.workspace_id.clone(), a.blueprint_id.clone(), vec![op])),
            }
        }
        // um `mover` que caiu em mais de um grupo cruza blueprints: para o visitante, o evento vira `recarregar`
        let cruzam: Vec<usize> = saida.aplicadas.iter().map(|a| a.indice).filter(|i| saida.aplicadas.iter().filter(|b| b.indice == *i).count() > 1).collect();
        let eventos = grupos
            .into_iter()
            .map(|(ws, bp, ops)| {
                let cruza = ops.iter().any(|op| cruzam.iter().any(|i| lote.brutas[*i] == *op));
                let dados = json!({"versao": nova, "workspaceId": ws, "blueprintId": bp, "autor": {"id": quem.id, "nome": quem.nome}, "cliente": lote.cliente, "ops": ops});
                Evento { workspace_id: ws, blueprint_id: bp, dados, cruza }
            })
            .collect();
        estado.sincronia.podar(|c| estado.acesso.sessao_ativa(c)); // quem foi removido não recebe mais nada
        estado.sincronia.publicar(nova, eventos);
    }
    drop(guardado);
    Ok(Resposta::json(200, &json!({"versao": versao, "conflitos": saida.conflitos, "ignoradas": saida.ignoradas})).com("x-dados-versao", &versao.to_string()))
}

fn eventos(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Result<Resposta, ErroHttp> {
    let ws = workspace(pedido, ctx)?;
    // o navegador manda Last-Event-ID sozinho ao reconectar; na primeira conexão a página manda ?desde= (o x-dados-versao do GET)
    let numero = |v: &str| v.trim().parse::<u64>().ok();
    let desde = pedido.cabecalho("last-event-id").and_then(numero).or_else(|| pedido.parametro("desde").as_deref().and_then(numero));
    let cliente = pedido.parametro("cliente").filter(|c| !c.is_empty() && c.len() <= 40);
    estado.sincronia.podar(|c| estado.acesso.sessao_ativa(c));
    let (rx, guarda) = estado.sincronia.assinar(&ws, desde, ctx, cliente.as_deref());
    let cabecalhos = vec![("content-type".into(), "text/event-stream; charset=utf-8".into()), ("cache-control".into(), "no-store".into())];
    Ok(Resposta { status: 200, cabecalhos, corpo: Corpo::Fluxo(Box::new(Fluxo::novo(rx, guarda))) })
}

fn presenca(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Result<Resposta, ErroHttp> {
    let corpo: Value = serde_json::from_slice(&pedido.corpo).map_err(|_| falha(400, crate::tr!("invalid JSON", "JSON inválido")).codigo("json-invalido"))?;
    let texto = |k: &str| corpo[k].as_str().unwrap_or("").chars().take(80).collect::<String>();
    let (ws, bp) = (texto("workspaceId"), texto("blueprintId"));
    if ctx.visitante {
        return Ok(Resposta::json(200, &json!({"ok": true}))); // visitante não aparece na presença
    }
    if !pode_ler(ctx, &ws, None) && (bp.is_empty() || !pode_ler(ctx, &ws, Some(&bp))) {
        return Err(falha(403, crate::tr!("forbidden", "proibido")));
    }
    let editando = match &corpo["editando"] {
        Value::Object(e) if matches!(e.get("tipo"), Some(Value::String(_))) && matches!(e.get("id"), Some(Value::String(_))) => {
            Some(json!({"tipo": texto_de(&e["tipo"]), "id": texto_de(&e["id"])}))
        }
        _ => None,
    };
    estado.sincronia.atualizar_presenca(&autor(ctx), &texto("cliente"), Onde { blueprint_id: bp, acao_id: texto("acaoId"), editando });
    Ok(Resposta::json(200, &json!({"ok": true})))
}

fn texto_de(v: &Value) -> String {
    v.as_str().unwrap_or("").chars().take(80).collect()
}

fn historico(estado: &Estado, pedido: &Pedido, ctx: &Contexto) -> Result<Resposta, ErroHttp> {
    if ctx.visitante {
        return Err(falha(403, crate::tr!("forbidden", "proibido"))); // o histórico tem nomes e valores antigos: visitante nunca lê
    }
    let ws = workspace(pedido, ctx)?;
    let filtro = |k: &str| pedido.parametro(k).filter(|v| !v.is_empty());
    let limite = pedido.parametro("limite").and_then(|v| v.parse::<i64>().ok()).unwrap_or(100).clamp(1, 500);
    let itens = estado.dados.lock().unwrap_or_else(|e| e.into_inner()).historico(ctx, &ws, filtro("blueprint").as_deref(), filtro("item").as_deref(), limite)?;
    Ok(Resposta::json(200, &json!({"itens": itens})))
}
