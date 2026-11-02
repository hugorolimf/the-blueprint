//! `execucao/*`: o que o invólucro do agente fala com o app (contrato, 5.4). A partida entrega o segredo uma vez por tentativa; sinal e fim com segredo de outra tentativa não mudam nada.
//! Nenhuma destas rotas inicia coisa alguma: só o motor lança, e só o gesto marca.
use super::prompt;
use crate::contexto::Contexto;
use crate::dados::planejar::{ExecucaoLinha, Mudanca};
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::mcp::hooks::{texto, travado};
use crate::mcp::Mcp;
use crate::plano::ids::nome_agente;
use serde_json::{json, Value};
use std::sync::Mutex;

/// O comando quando o usuário não configurou outro: é o caminho com plugin e hooks.
const COMANDO_PADRAO: &str = "claude";
const COM_SINAL: [&str; 3] = ["rodando", "ajuda", "sumiu"];

fn sem_execucao() -> ErroHttp {
    falha(404, crate::tr!("execution not found", "execução não encontrada")).codigo("execucao-nao-encontrada")
}

/// `comando_agente` é uma lista JSON de argumentos, nunca uma linha de shell.
fn comando_do_agente(d: &Dados) -> Result<Vec<String>, ErroHttp> {
    let guardado = d.config_ler("comando_agente")?.and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok()).filter(|c| !c.is_empty());
    Ok(guardado.unwrap_or_else(|| vec![COMANDO_PADRAO.to_string()]))
}

fn blueprint_de<'a>(tudo: &'a Value, x: &ExecucaoLinha) -> Option<&'a Value> {
    let lista_de = |v: &'a Value| v.get("blueprints").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let ws = if x.workspace_id == "sm" { Some(tudo) } else { tudo.get("workspaces")?.as_array()?.iter().find(|w| w.get("id").and_then(Value::as_str) == Some(x.workspace_id.as_str())) };
    lista_de(ws?).iter().find(|b| b.get("id").and_then(Value::as_str) == Some(x.blueprint_id.as_str()))
}

fn partida(mcp: &Mcp, dados: &Mutex<Dados>, id: &str, agora: i64) -> Result<Value, ErroHttp> {
    let mut d = travado(dados);
    let Some(p) = d.execucao_partir(id, agora)? else {
        let x = d.execucao_de(id)?.ok_or_else(sem_execucao)?;
        return Err(if x.situacao == "rodando" {
            falha(409, crate::tr!("this attempt has already started", "esta tentativa já partiu")).codigo("ja-partiu")
        } else {
            falha(409, crate::tr!("this execution is not running", "esta execução não está rodando")).codigo("nao-esta-rodando")
        });
    };
    let revisoes = d.revisoes_do(&p.linha.blueprint_id)?;
    let comando = comando_do_agente(&d)?;
    let tudo = d.ler_tudo(&Contexto::local())?;
    let bp = blueprint_de(tudo, &p.linha).ok_or_else(sem_execucao)?;
    let texto_prompt = prompt::montar(bp, &p.linha, &revisoes);
    drop(d);
    mcp.planejar_mudou();
    let claude = comando.len() == 1 && comando[0] == COMANDO_PADRAO;
    Ok(json!({"tentativa": p.tentativa, "segredo": p.segredo, "prompt": texto_prompt, "comando": comando, "claude": claude, "nomeAgente": nome_agente(&p.linha.plano_ref), "pasta": p.linha.pasta_trabalho}))
}

/// `parar` também para quem tem o segredo de outra tentativa, sem mudar nada: uma execução retomada não pode ficar com dois agentes na mesma cópia.
fn sinal(mcp: &Mcp, dados: &Mutex<Dados>, id: &str, segredo: &str, agora: i64) -> Result<Value, ErroHttp> {
    let mut d = travado(dados);
    let Some(x) = d.execucao_de(id)? else { return Ok(json!({"parar": true})) };
    if x.parar || !COM_SINAL.contains(&x.situacao.as_str()) || segredo.is_empty() || x.segredo != segredo {
        return Ok(json!({"parar": true}));
    }
    let situacao = (x.situacao == "sumiu").then_some("rodando");
    d.execucao_mudar(id, &Mudanca { so_de: &COM_SINAL, situacao, ultimo_sinal: Some(agora), ..Mudanca::default() })?;
    drop(d);
    mcp.planejar_mudou();
    Ok(json!({"parar": false}))
}

/// Só quem ainda devia estar trabalhando vira `falhou`; `cancelada` e `pronta` ficam, ganhando só a hora do fim.
fn fim(mcp: &Mcp, dados: &Mutex<Dados>, id: &str, segredo: &str, agora: i64) -> Result<Value, ErroHttp> {
    let mut d = travado(dados);
    let Some(x) = d.execucao_de(id)? else { return Err(sem_execucao()) };
    if segredo.is_empty() || x.segredo != segredo {
        return Ok(json!({"ok": true, "ignorado": true}));
    }
    let falhou = d.execucao_mudar(id, &Mudanca { so_de: &COM_SINAL, situacao: Some("falhou"), motivo: Some("terminou-sem-concluir"), terminada_em: Some(Some(agora)), ..Mudanca::default() })?;
    if falhou {
        d.orq_pedir(&x.workspace_id, &format!("falhou:{}", x.plano_ref), false)?;
    } else {
        d.execucao_mudar(id, &Mudanca { terminada_em: Some(Some(agora)), ..Mudanca::default() })?;
    }
    drop(d);
    mcp.planejar_mudou();
    Ok(json!({"ok": true}))
}

pub fn rota(mcp: &Mcp, dados: &Mutex<Dados>, _pedido: &Pedido, recurso: &str, corpo: &Value) -> Result<Resposta, ErroHttp> {
    let (id, segredo, agora) = (texto(corpo, "id"), texto(corpo, "segredo"), crate::tempo::agora_ms());
    let resposta = match recurso {
        "execucao/partida" => partida(mcp, dados, id, agora)?,
        "execucao/sinal" => sinal(mcp, dados, id, segredo, agora)?,
        "execucao/fim" => fim(mcp, dados, id, segredo, agora)?,
        _ => return Err(super::nao_encontrado()),
    };
    Ok(Resposta::json(200, &resposta))
}
