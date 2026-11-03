//! A linha do tempo da execução por ação: cada evento que passa por `acoes::evento` e tem ação é anotado no banco (`dados::linha_do_tempo`) e avisado por SSE
//! (`linha-do-tempo`, só ids). Aqui fica o filtro do que entra: a evidência das marcações nunca entra (já vira comentário do plano), nem conteúdo de arquivo
//! (o servidor nem o recebe) ou comando; o texto do progresso e do bloqueio vai cortado. Regra completa: docs/claude-na-folha.md.
use super::acoes;
use super::Mcp;
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, Mudanca};
use crate::dados::{Dados, EventoLinha, LIMITE_POR_ACAO};
use crate::http::pedido::{falha, ErroHttp, Pedido};
use crate::plano::ids;
use serde_json::{json, Value};
use std::sync::Mutex;

const TEXTO_MAX: usize = 200;
const ONDE_MAX: usize = 120;

/// O evento da linha do tempo para um item da fila já validado; None se ele não for de uma ação.
pub(super) fn da_fila(item: &Value) -> Option<EventoLinha> {
    let s = |k: &str| item.get(k).and_then(Value::as_str).filter(|v| !v.is_empty()).map(str::to_string);
    let (tipo, acao_id) = (s("tipo")?, s("acaoId")?);
    let com_texto = tipo == "progresso" || tipo == "bloqueio";
    Some(EventoLinha {
        workspace_id: s("workspaceId").unwrap_or_default(),
        blueprint_id: s("blueprintId").unwrap_or_default(),
        acao_id,
        alvo: if tipo.starts_with("checkpoint") { s("checkpointId") } else { s("planoId") },
        agente: s("agente"),
        estado: s("estado"),
        texto: if com_texto { item["texto"].as_str().unwrap_or("").chars().take(TEXTO_MAX).collect() } else { String::new() },
        onde: item["onde"].as_array().and_then(|l| l.first()).and_then(Value::as_str).map(|o| o.chars().take(ONDE_MAX).collect()),
        tipo,
    })
}

/// As situações em que há um agente por trás da execução: só nelas o segredo vale para concluir, bloquear ou mudar de situação.
const COM_AGENTE: [&str; 3] = ["rodando", "ajuda", "sumiu"];

/// Quem atende o evento: a fila antiga ou a execução que grava no servidor (D8).
enum Caso {
    Fila,
    Plano(ExecucaoLinha),
    Checkpoint,
    Desfazer(ExecucaoLinha),
}

fn campo<'a>(corpo: &'a Value, chave: &str) -> &'a str {
    corpo.get(chave).and_then(Value::as_str).unwrap_or("")
}

fn segredo_certo(x: &ExecucaoLinha, segredo: &str) -> bool {
    !segredo.is_empty() && x.segredo == segredo
}

fn tem_agente(x: &ExecucaoLinha) -> bool {
    COM_AGENTE.contains(&x.situacao.as_str())
}

/// `acaoId`, `planoId` e `checkpointId` podem vir no `ref` do plano; daqui para a frente todos são o id real. Quem manda só o segredo da execução fica no plano dela.
fn traduzido(d: &mut Dados, corpo: &Value) -> Result<Value, ErroHttp> {
    let mut novo = corpo.clone();
    let (ws, bp) = (campo(corpo, "workspaceId"), campo(corpo, "blueprintId"));
    if campo(corpo, "planoId").is_empty() && !campo(corpo, "execucaoId").is_empty() {
        if let Some(x) = d.execucao_de(campo(corpo, "execucaoId"))?.filter(|x| x.blueprint_id == bp && segredo_certo(x, campo(corpo, "execucaoSegredo"))) {
            (novo["acaoId"], novo["planoId"]) = (json!(x.acao_id), json!(x.plano_id));
        }
    }
    if let Ok((_, Some(blueprint))) = acoes::localizar(d.ler_tudo(&Contexto::local())?, ws, bp) {
        for (chave, tipo) in [("acaoId", "acao"), ("planoId", "plano"), ("checkpointId", "checkpoint")] {
            if let Some(real) = novo.get(chave).and_then(Value::as_str).and_then(|r| ids::traduzir(blueprint, r, tipo)) {
                novo[chave] = json!(real);
            }
        }
    }
    Ok(novo)
}

/// Decide antes da validação, que já avisa a presença: quem não é dono do plano não põe figura de concluído na folha.
fn caso_de(d: &Dados, corpo: &Value) -> Result<Caso, ErroHttp> {
    let (tipo, segredo) = (campo(corpo, "tipo"), campo(corpo, "execucaoSegredo"));
    let (ws, bp, acao, plano) = (campo(corpo, "workspaceId"), campo(corpo, "blueprintId"), campo(corpo, "acaoId"), campo(corpo, "planoId"));
    if tipo.starts_with("checkpoint") {
        let dono = d.execucoes_do_ws(ws)?.into_iter().any(|x| x.blueprint_id == bp && x.acao_id == acao && tem_agente(&x) && segredo_certo(&x, segredo));
        return Ok(if dono { Caso::Checkpoint } else { Caso::Fila });
    }
    if plano.is_empty() {
        return Ok(Caso::Fila);
    }
    let viva = d.execucao_viva(bp, plano)?;
    let certa = viva.as_ref().is_some_and(|x| tem_agente(x) && segredo_certo(x, segredo));
    match (tipo, viva) {
        ("progresso", Some(x)) if certa => Ok(Caso::Plano(x)),
        ("concluir" | "bloqueio" | "nao-concluida", Some(x)) if certa => Ok(Caso::Plano(x)),
        ("concluir" | "bloqueio" | "nao-concluida", Some(_)) => Err(falha(403, crate::tr!("This plan belongs to another agent: send your execucao_segredo.", "Este plano é de outro agente: mande o seu execucao_segredo.")).codigo("nao-e-seu-plano")),
        ("nao-concluida", None) => {
            let pronta = d.execucoes_do_ws(ws)?.into_iter().find(|x| x.blueprint_id == bp && x.plano_id == plano && x.situacao == "pronta" && x.integrada_em.is_none() && segredo_certo(x, segredo));
            Ok(pronta.map_or(Caso::Fila, Caso::Desfazer))
        }
        _ => Ok(Caso::Fila),
    }
}

/// O que a execução grava, depois da mesma validação da fila: aqui nada vai para ela.
fn pela_execucao(mcp: &Mcp, d: &mut Dados, caso: &Caso, corpo: &Value) -> Result<Value, ErroHttp> {
    let tudo = d.ler_tudo(&Contexto::local())?;
    let mut item = match caso {
        Caso::Fila => unreachable!("a fila não passa por aqui"),
        Caso::Plano(_) => acoes::evento_sem_fila(mcp, tudo, corpo)?,
        Caso::Checkpoint | Caso::Desfazer(_) => acoes::evento_desfazendo(mcp, tudo, corpo)?,
    };
    let em = crate::tempo::iso_utc_agora();
    item.insert("em".into(), json!(em));
    let (tipo, texto) = (campo(corpo, "tipo"), item["texto"].as_str().unwrap_or("").to_string());
    let motivo: String = texto.chars().take(TEXTO_MAX).collect();
    let agora = crate::tempo::agora_ms();
    let mudar = |d: &mut Dados, x: &ExecucaoLinha, so_de: &[&str], situacao: &str, motivo: &str| -> Result<(), ErroHttp> {
        d.execucao_mudar(&x.id, &Mudanca { so_de, situacao: Some(situacao), motivo: Some(motivo), ultimo_sinal: Some(agora), ..Mudanca::default() })?;
        Ok(())
    };
    match caso {
        Caso::Fila => {}
        Caso::Checkpoint => d.checkpoint_por_execucao(item.get("checkpointId").and_then(Value::as_str).unwrap_or(""), tipo == "checkpoint-feito", &texto, &em)?,
        Caso::Desfazer(x) => {
            d.desfazer_por_execucao(&x.plano_id, &texto, &em)?;
            mudar(d, x, &["pronta"], "falhou", &motivo)?;
            d.orq_pedir(&x.workspace_id, &format!("falhou:{}", x.plano_ref), false)?;
        }
        Caso::Plano(x) => match tipo {
            "concluir" => {
                d.concluir_por_execucao(&x.plano_id, &texto, &em)?;
                mudar(d, x, &COM_AGENTE, "pronta", "")?;
                d.orq_pedir(&x.workspace_id, &format!("pronta:{}", x.plano_ref), false)?;
            }
            "bloqueio" => {
                mudar(d, x, &COM_AGENTE, "ajuda", &motivo)?;
                d.orq_pedir(&x.workspace_id, &format!("ajuda:{}", x.plano_ref), false)?;
            }
            "nao-concluida" => {
                mudar(d, x, &COM_AGENTE, "falhou", &motivo)?;
                d.orq_pedir(&x.workspace_id, &format!("falhou:{}", x.plano_ref), false)?;
            }
            _ => mudar(d, x, &COM_AGENTE, "rodando", "")?, // progresso: de `ajuda` e de `sumiu` volta a trabalhar
        },
    }
    mcp.planejar_mudou();
    Ok(Value::Object(item))
}

/// `acoes::evento` (ou, com execução viva no plano, o que ela grava) e, se ele aceitou e o evento é de uma ação, a anotação na linha do tempo. Falhar ao anotar não recusa o evento
/// (ele já está na fila; recusar faria o agente reenviar e duplicar).
pub(super) fn evento(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<Value, ErroHttp> {
    let ctx = Contexto::local();
    let mut d = dados.lock().unwrap_or_else(|e| e.into_inner());
    let corpo = &traduzido(&mut d, corpo)?;
    let caso = caso_de(&d, corpo)?;
    let item = match caso {
        Caso::Fila => acoes::evento(mcp, d.ler_tudo(&ctx)?, corpo)?,
        _ => pela_execucao(mcp, &mut d, &caso, corpo)?,
    };
    if let Some(e) = da_fila(&item) {
        match d.gravar_linha_do_tempo(&ctx, crate::tempo::agora_ms(), &e) {
            Ok(()) => mcp.notificar("linha-do-tempo", Some(json!({"workspaceId": e.workspace_id, "blueprintId": e.blueprint_id, "acaoId": e.acao_id}))),
            Err(erro) => eprintln!("linha do tempo: não foi possível anotar o evento: {erro}"),
        }
    }
    drop(d);
    if let (Caso::Plano(x), "concluir") = (&caso, campo(corpo, "tipo")) {
        super::planejar::motor::aviso_da_copia(dados, mcp, x); // o git fica fora do Mutex<Dados>
    }
    Ok(item)
}

/// `GET /api/mcp/linha-do-tempo?workspace=&blueprint=&acao=`: os eventos da ação, do mais recente ao mais antigo.
pub(super) fn listar(dados: &Mutex<Dados>, pedido: &Pedido) -> Result<Value, ErroHttp> {
    let p = |n: &str| pedido.parametro(n).unwrap_or_default();
    let eventos = dados.lock().unwrap_or_else(|e| e.into_inner()).linha_do_tempo(&Contexto::local(), &p("workspace"), &p("blueprint"), &p("acao"))?;
    Ok(json!({"eventos": eventos, "limite": LIMITE_POR_ACAO}))
}
