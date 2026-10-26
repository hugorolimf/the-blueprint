//! `hook/subagentes`: o `subagentStatusLine` do Claude Code (o painel de subagentes) roda `blueprint hook subagentes` a cada atualização do painel e o comando pergunta aqui.
//! Entrada (doc oficial): os campos base do hook, `columns` e `tasks[]` ({id, name?, agentType, status, description, label, tokenCount, ...}).
//! Saída: `{"linhas": [{"id", "content"}]}`, uma linha por subagente que o app conhece; o resto fica sem linha (o painel usa a padrão).
//! O subagente é o MESMO agente que os hooks SubagentStart/Stop criaram (nome = tipo + 4 primeiros caracteres do id, como `hooks::identidade`): aqui nunca nasce agente,
//! só se atualiza o que já existe. Só indicador: nada grava nos dados do blueprint além da linha do tempo, como os hooks de edição.
//! O painel chama a cada tick, então só vira evento o que MUDOU (estado da tarefa ou rótulo); os tokens só são anotados com um salto de `SALTO_TOKENS`.
use super::hooks::{alvo, texto, travado};
use super::status::lista;
use super::{acoes, Mcp};
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::ErroHttp;
use serde_json::{json, Map, Value};
use std::sync::Mutex;

type R<T> = Result<T, ErroHttp>;

const MAX_TAREFAS: usize = 32;
/// Só anota (e avisa a página) de novo os tokens quando mudaram tanto assim: o painel manda a cada tick.
pub(super) const SALTO_TOKENS: u64 = 500;
const COLUNAS_PADRAO: usize = 80;
const ROTULO_MAX: usize = 200;

/// Texto de uma linha só vindo de fora: sem caracteres de controle (a linha é escrita como está no terminal, com escapes ANSI), aparado e curto.
fn limpo(s: &str, max: usize) -> String {
    s.chars().filter(|c| !c.is_control()).take(max).collect::<String>().trim().to_string()
}

/// O nome do agente que os hooks de subagente deram a este subagente (a regra é a de `hooks::nome_do_subagente`). Sem id utilizável: None.
pub(super) fn nome_do(id: &str, tipo: &str) -> Option<String> {
    super::hooks::nome_do_subagente(id, tipo)
}

/// 950 → "950"; 12300 → "12.3k"
pub(super) fn tokens_curtos(n: u64) -> String {
    if n < 1000 { n.to_string() } else { format!("{:.1}k", n as f64 / 1000.0) }
}

/// (id da ação, número na folha) de cada ação do blueprint.
fn numeros_da_folha(dados: &Mutex<Dados>, ws: &str, bp: &str) -> R<Vec<(String, String)>> {
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, ws, bp);
    let Ok(estrutura) = acoes::blueprint(tudo, ws, bp, None) else { return Ok(Vec::new()) };
    let p = &estrutura["blueprint"];
    Ok(lista(&p["acoes"]).iter().zip(numeros_das_acoes(p, n)).map(|(a, numero)| (texto(a, "id").to_string(), numero)).collect())
}

/// O que a tarefa pede para o agente: o estado novo e a mensagem, ou nada se este tick não traz mudança. `anterior_tarefa`/`anterior_rotulo` são o que ficou
/// anotado no tick passado.
fn mudanca(status: &str, nome: &str, rotulo: &str, ag: &Value) -> Option<(&'static str, String)> {
    let anterior = (texto(ag, "tarefa"), texto(ag, "rotulo"));
    let mudou = status != anterior.0;
    match status {
        "running" if !rotulo.is_empty() && rotulo != anterior.1 => {
            // quem já está lendo, escrevendo ou testando continua; os outros estados voltam a "lendo"
            let estado = ["escrevendo", "testando", "aguardando"].into_iter().find(|e| ag["estado"] == *e).unwrap_or("lendo");
            Some((estado, rotulo.to_string()))
        }
        // o SubagentStop já concluiu com a última mensagem dele (melhor que o rótulo): só conclui se ele ainda não concluiu
        "completed" if mudou && ag["estado"] != "concluido" => Some(("concluido", if rotulo.is_empty() { crate::tr!("{nome} finished", "{nome} terminou") } else { rotulo.to_string() })),
        "failed" if mudou => Some(("bloqueado", crate::tr!("subagent failed", "o subagente falhou"))),
        "killed" if mudou => Some(("bloqueado", crate::tr!("subagent was cancelled", "o subagente foi cancelado"))),
        _ => None,
    }
}

/// O evento de progresso do agente (a ação e o plano são os que ele já tinha) pelo mesmo caminho dos hooks: fila, presença e linha do tempo.
/// Falhar aqui (plano apagado, por exemplo) não derruba a linha do painel.
fn emitir(mcp: &Mcp, dados: &Mutex<Dados>, ag: &Value, nome: &str, ws: &str, bp: &str, estado: &str, mensagem: &str) {
    let mut ev = json!({"tipo": "progresso", "texto": mensagem, "workspaceId": ws, "blueprintId": bp, "agente": nome, "estado": estado});
    if let Some(papel) = ag["papel"].as_str().filter(|p| acoes::PAPEIS.contains(p)) {
        ev["papel"] = json!(papel);
    }
    for campo in ["acaoId", "planoId"] {
        if let Some(v) = ag[campo].as_str().filter(|v| !v.is_empty()) {
            ev[campo] = json!(v);
        }
    }
    if let Err(e) = super::linha_do_tempo::evento(mcp, dados, &ev) {
        eprintln!("subagentes: evento de {nome} recusado: {}", e.status);
    }
}

/// A linha do painel: `nome · estado · ação N.M · rótulo · 12.3k tokens`, no máximo `colunas` caracteres (o rótulo é o que encolhe).
pub(super) fn linha(titulo: &str, status: &str, numero: Option<&str>, rotulo: &str, tokens: Option<u64>, colunas: usize) -> String {
    let fim = match status {
        "completed" => Some(crate::tr!("done", "concluído")),
        "failed" => Some(crate::tr!("failed", "falhou")),
        "killed" => Some(crate::tr!("cancelled", "cancelado")),
        _ => None,
    };
    let antes: Vec<String> = [Some(titulo.to_string()), fim, numero.map(|n| crate::tr!("action {n}", "ação {n}"))].into_iter().flatten().collect();
    let depois = tokens.map(|n| format!("{} tokens", tokens_curtos(n)));
    let fixo: usize = antes.iter().chain(depois.iter()).map(|p| p.chars().count() + 3).sum();
    let espaco = colunas.saturating_sub(fixo).max(10);
    let rotulo: String = if rotulo.chars().count() > espaco { rotulo.chars().take(espaco - 1).chain(std::iter::once('…')).collect() } else { rotulo.to_string() };
    let mut partes = antes;
    partes.extend((!rotulo.is_empty()).then_some(rotulo));
    partes.extend(depois);
    partes.join(" · ").chars().take(colunas).collect()
}

pub fn subagentes(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let vazio = json!({"linhas": []});
    let tarefas = lista(&corpo["tasks"]);
    let Some((ws, bp)) = alvo(mcp, dados, corpo)?.filter(|_| !tarefas.is_empty()) else { return Ok(vazio) };
    let colunas = corpo["columns"].as_u64().map_or(COLUNAS_PADRAO, |c| c.clamp(20, 400) as usize);
    let numeros = numeros_da_folha(dados, &ws, &bp)?;
    let agentes = mcp.agentes();
    let mut linhas = Vec::new();
    for t in tarefas.iter().take(MAX_TAREFAS) {
        let (Some(nome), Some(id)) = (nome_do(texto(t, "id"), texto(t, "agentType")), t["id"].as_str().filter(|i| !i.is_empty())) else { continue };
        let Some(ag) = lista(&agentes["agentes"]).iter().find(|a| a["agente"] == nome && a["blueprintId"] == bp && a["workspaceId"] == ws) else { continue }; // só agente que os hooks já criaram
        let status = texto(t, "status");
        let rotulo = limpo(if texto(t, "label").is_empty() { texto(t, "description") } else { texto(t, "label") }, ROTULO_MAX);
        let tokens = t["tokenCount"].as_f64().filter(|n| n.is_finite() && *n >= 0.0).map(|n| n as u64);
        let mudanca = mudanca(status, &nome, &rotulo, ag);
        if let Some((estado, mensagem)) = &mudanca {
            emitir(mcp, dados, ag, &nome, &ws, &bp, estado, mensagem);
        }
        let mut anotar = Map::new();
        if status != texto(ag, "tarefa") && !status.is_empty() {
            anotar.insert("tarefa".into(), json!(status));
        }
        if !rotulo.is_empty() && rotulo != texto(ag, "rotulo") {
            anotar.insert("rotulo".into(), json!(rotulo));
        }
        if let Some(n) = tokens.filter(|n| ag["tokens"].as_u64().is_none_or(|antes| n.abs_diff(antes) >= SALTO_TOKENS) || mudanca.is_some()) {
            anotar.insert("tokens".into(), json!(n));
        }
        if !anotar.is_empty() {
            mcp.anotar_atividade(&nome, anotar);
        }
        let titulo = Some(limpo(texto(t, "name"), 30)).filter(|n| !n.is_empty()).unwrap_or(nome.clone());
        let numero = ag["acaoId"].as_str().and_then(|a| numeros.iter().find(|(id, _)| id == a)).map(|(_, n)| n.as_str());
        linhas.push(json!({"id": id, "content": linha(&titulo, status, numero, &rotulo, tokens, colunas)}));
    }
    Ok(json!({"linhas": linhas}))
}
