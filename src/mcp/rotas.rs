//! As rotas /api/mcp/* (porte de `rota` do mcp.mjs): só esta máquina, JSON em todo POST, e erros JSON como `{ "erro", "codigo", "params"? }` (`ErroHttp::json`), o texto no idioma do pedido.
use super::acoes::{self, id_prototipo_valido};
use super::hooks;
use super::permissao::{permitido, pode_gravar_item, pode_ler_item, so_loopback};
use super::{FluxoSse, Mcp};
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, Corpo, ErroHttp, Pedido, Resposta};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::Mutex;

fn nao_encontrado() -> Resposta {
    Resposta::texto(404, "text/plain", crate::tr!("not found", "não encontrado"))
}

/// O app local (dono implícito): o que os testes do MCP chamam.
pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido) -> Resposta {
    rota_com(mcp, dados, pedido, &Contexto::local())
}

/// Com o Contexto resolvido no despacho.
pub fn rota_com(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, ctx: &Contexto) -> Resposta {
    let recurso = pedido.caminho.trim_start_matches("/api/mcp/");
    if ctx.local {
        if !permitido(mcp, pedido) {
            return Resposta::texto(403, "text/plain", crate::tr!("The Blueprint MCP only works on this machine (MCP_REDE=1 opens it to the network)", "o MCP do The Blueprint só funciona nesta máquina (MCP_REDE=1 libera a rede)"));
        }
        if (recurso == "iniciar" || recurso.starts_with("conversa")) && !so_loopback(mcp, pedido) {
            return Resposta::texto(403, "text/plain", crate::tr!("starting an action only works on this machine (not even MCP_REDE=1 opens it)", "iniciar uma ação só funciona nesta máquina (nem MCP_REDE=1 libera)"));
        }
    } else if let Err(e) = exige_hospedado(dados, pedido, recurso, ctx) {
        return e.resposta();
    }
    // modo hospedado: cada item que esta chamada puser na fila leva quem mandou
    let autor = (!ctx.local).then(|| json!({"id": ctx.usuario, "nome": ctx.nome}));
    super::fila::com_autor(autor, || executar(mcp, dados, pedido, recurso, ctx)).unwrap_or_else(|e| e.resposta())
}

/// O papel alcança o recurso? (o corpo do POST traz o alvo; JSON torto segue para `executar`, que responde o erro de sempre)
fn exige_hospedado(dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str, ctx: &Contexto) -> Result<(), ErroHttp> {
    let corpo: Value = serde_json::from_slice(&pedido.corpo).unwrap_or(Value::Null);
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner());
    super::permissao::exige_hospedado(ctx, recurso, pedido, &corpo, d.ler_tudo(&Contexto::local())?)
}

/// Reenvia um registro guardado com o app fechado (`pendentes.rs`) à mesma rota POST que o agente usaria ao vivo, com a mesma validação. Não confere Host nem
/// Origin: quem escreveu o arquivo já tinha acesso à pasta de dados do usuário.
pub fn reenviar(mcp: &Mcp, dados: &Mutex<Dados>, recurso: &str, corpo: &Value) -> (u16, Value) {
    let pedido = Pedido::novo("POST", &format!("/api/mcp/{recurso}"), vec![("content-type".into(), "application/json".into())], corpo.to_string().into_bytes());
    let resposta = executar(mcp, dados, &pedido, recurso, &Contexto::local()).unwrap_or_else(|e| e.resposta());
    (resposta.status, serde_json::from_slice(resposta.bytes()).unwrap_or(Value::Null))
}

/// Roda `f` com o data.json em mãos (a cópia em memória da camada de dados), só com o que `ctx` lê (`acesso::filtro`).
fn com_dados<T>(dados: &Mutex<Dados>, ctx: &Contexto, f: impl FnOnce(&Value) -> Result<T, ErroHttp>) -> Result<T, ErroHttp> {
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner());
    f(&crate::acesso::filtro::filtrar(d.ler_tudo(&Contexto::local())?, ctx))
}

fn executar(mcp: &Mcp, dados: &Mutex<Dados>, pedido: &Pedido, recurso: &str, ctx: &Contexto) -> Result<Resposta, ErroHttp> {
    if pedido.metodo == "GET" {
        return Ok(match recurso {
            "blueprints" => Resposta::json(200, &com_dados(dados, ctx, |d| Ok(acoes::blueprints(d)))?),
            "blueprint" => {
                let p = |nome: &str| pedido.parametro(nome).unwrap_or_default();
                Resposta::json(200, &com_dados(dados, ctx, |d| acoes::blueprint(d, &p("workspace"), &p("blueprint"), pedido.parametro("acao").as_deref()))?)
            }
            "pastas" => Resposta::json(200, &json!({"pastas": hooks::travado(dados).listar_pastas(&Contexto::local())?})),
            "fila" => Resposta::json(200, &json!({"itens": mcp.itens().into_iter().filter(|i| pode_ler_item(ctx, i)).collect::<Vec<_>>()})),
            "sessoes" => Resposta::json(200, &mcp.sessoes_do_blueprint(&pedido.parametro("workspace").unwrap_or_default(), &pedido.parametro("blueprint").unwrap_or_default())), // as sessões do Claude abertas no blueprint
            "conversa" => Resposta::json(200, &mcp.conversa(&pedido.parametro("workspace").unwrap_or_default(), &pedido.parametro("blueprint").unwrap_or_default(), &pedido.parametro("acao").unwrap_or_default())), // a conversa da ação (só memória)
            "agentes" => {
                // quem está trabalhando agora (presença, só memória); no modo hospedado, só nos blueprints que o papel lê
                let mut a = mcp.agentes();
                if let Some(lista) = a["agentes"].as_array_mut().filter(|_| !ctx.local) {
                    lista.retain(|ag| pode_ler_item(ctx, ag));
                }
                Resposta::json(200, &a)
            }
            "linha-do-tempo" => Resposta::json(200, &super::linha_do_tempo::listar(dados, pedido)?), // o que o agente fez na ação, do mais recente ao mais antigo
            // a página mostra como registrar o servidor MCP no agente (no modo hospedado, sem o caminho do servidor: a tela da chave dá os comandos)
            "info" => Resposta::json(200, &json!({"arquivo": if ctx.local { mcp.arquivo.as_str() } else { "blueprint" }, "url": mcp.url, "exe": true})),
            "eventos" => {
                let mut fluxo = FluxoSse::novo(mcp.assinar());
                // `?canal=&cwd=`: a conexão é de uma sessão com o canal ligado e ela é destinatária enquanto durar
                if let (Some(id), Some(cwd)) = (pedido.parametro("canal"), pedido.parametro("cwd")) {
                    if let Some(guarda) = mcp.abrir_canal(&id, &cwd) {
                        fluxo = fluxo.com_guarda(guarda);
                    }
                }
                Resposta { status: 200, cabecalhos: vec![("content-type".into(), "text/event-stream; charset=utf-8".into()), ("cache-control".into(), "no-store".into())], corpo: Corpo::Fluxo(Box::new(fluxo)) }
            }
            _ => nao_encontrado(),
        });
    }
    if pedido.metodo != "POST" {
        return Ok(Resposta::texto(405, "text/plain", crate::tr!("method not allowed", "método não permitido")));
    }
    if !pedido.eh_json() {
        return Ok(Resposta::texto(415, "text/plain", crate::tr!("send application/json", "envie application/json")));
    }
    let corpo: Value = if pedido.corpo.is_empty() { json!({}) } else { serde_json::from_slice(&pedido.corpo).map_err(|e| falha(400, crate::tr!("invalid JSON: {e}", "JSON inválido: {e}")).codigo("json-invalido"))? };
    if recurso.starts_with("hook/") {
        mcp.sinal_sessao(hooks::texto(&corpo, "session_id")); // qualquer hook da sessão prova que ela está viva (expiração em sessoes.rs)
    }
    Ok(match recurso {
        "proposta" => {
            let (status, resposta) = com_dados(dados, ctx, |d| acoes::proposta(mcp, d, &corpo))?;
            Resposta::json(status, &resposta)
        }
        "registro" => {
            let (status, resposta) = com_dados(dados, ctx, |d| super::registro::registro(mcp, d, &corpo))?;
            if status == 200 {
                super::lembrete::registrado(mcp, &hooks::texto(&corpo, "workspaceId"), &hooks::texto(&corpo, "blueprintId"));
            }
            Resposta::json(status, &resposta)
        }
        "evento" => {
            let r = super::linha_do_tempo::evento(mcp, dados, &corpo)?; // acoes::evento e, se for de uma ação, a linha do tempo
            // um progresso com plano também conta como registro: some o lembrete da sessão
            if corpo.get("planoId").and_then(Value::as_str).is_some_and(|p| !p.is_empty()) {
                super::lembrete::registrado(mcp, &hooks::texto(&corpo, "workspaceId"), &hooks::texto(&corpo, "blueprintId"));
            }
            Resposta::json(200, &r)
        }
        "hook/parada" => Resposta::json(200, &super::lembrete::parada(mcp, &corpo)), // o Stop do turno (ver lembrete.rs)
        "aviso" => Resposta::json(200, &com_dados(dados, ctx, |d| super::eventos::aviso(mcp, d, &corpo))?),
        "prototipo" => Resposta::json(200, &com_dados(dados, ctx, |d| acoes::prototipo(mcp, d, &corpo))?),
        // os hooks do Claude Code e a ligação pasta → blueprint (ver hooks.rs)
        "hook/sessao" => Resposta::json(200, &hooks::sessao(mcp, dados, &corpo)?),
        "hook/fim" => Resposta::json(200, &super::sessoes::fim(mcp, &corpo)), // o SessionEnd tira a sessão da lista
        "hook/edicao" => Resposta::json(200, &hooks::edicao(mcp, dados, &corpo)?),
        "hook/subagente" => Resposta::json(200, &hooks::subagente(mcp, dados, &corpo)?),
        "hook/status" => Resposta::json(200, &super::status::status(mcp, dados, &corpo)?),
        "pasta/desvincular" => Resposta::json(200, &hooks::desvincular(dados, &corpo)?),
        "pasta/vincular" => Resposta::json(200, &hooks::vincular(dados, &corpo)?),
        "iniciar" => {
            let (status, resposta) = if corpo.get("sessaoId").is_some() {
                super::iniciar::em_sessao(mcp, dados, &corpo)? // entrega o pedido a uma sessão já aberta, pelo canal dela
            } else {
                super::iniciar::iniciar(dados, &corpo, &super::iniciar::abrir_terminal)? // abre o terminal de verdade; os testes usam um lançador falso
            };
            Resposta::json(status, &resposta)
        }
        // comentário do plano e conversa com o plano (ver conversa.rs): ida pelo canal da sessão escolhida, volta pela ferramenta responder_conversa
        "conversa" => { let (status, r) = super::conversa::enviar(mcp, dados, &corpo)?; Resposta::json(status, &r) }
        "conversa/resposta" => { let (status, r) = super::conversa::responder(mcp, dados, &corpo)?; Resposta::json(status, &r) }
        "prototipo/remover" => {
            // chamado pela página ao excluir; só mexe em arquivos com id gerado por aqui
            let id = corpo.get("prototipoId").and_then(Value::as_str).filter(|i| id_prototipo_valido(i)).ok_or_else(|| falha(400, crate::tr!("invalid prototipoId", "prototipoId inválido")))?;
            let _ = std::fs::remove_file(mcp.prototipos.join(format!("{id}.html")));
            Resposta::json(200, &json!({"ok": true}))
        }
        "fila/ack" => {
            let mut ids: HashSet<i64> = corpo.get("ids").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default();
            if !ctx.local {
                // aplicar ou descartar exige gravar no alvo de cada item
                let gravaveis: HashSet<i64> = mcp.itens().iter().filter(|i| pode_gravar_item(ctx, i)).filter_map(|i| i["id"].as_i64()).collect();
                ids.retain(|id| gravaveis.contains(id));
            }
            mcp.ack(&ids);
            let do_ctx = |v: &&Value| ctx.local || pode_gravar_item(ctx, v);
            corpo.get("propostas").and_then(Value::as_array).into_iter().flatten().filter(do_ctx).for_each(|p| mcp.ack_proposta(p)); // propostas pela chave: ver `Fila::ack_proposta`
            corpo.get("registros").and_then(Value::as_array).into_iter().flatten().filter(do_ctx).for_each(|r| mcp.ack_registro(r)); // registros: a contagem e a chave são próprias
            Resposta::json(200, &json!({"ok": true}))
        }
        _ => nao_encontrado(),
    })
}
