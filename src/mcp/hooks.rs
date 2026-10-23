//! O que os hooks do Claude Code usam: ligar uma pasta a um blueprint, dizer ao agente em que blueprint a sessão está e registrar as edições
//! como progresso. Só indicador: nada aqui marca plano como concluído (isso exige evidência) nem escreve nos dados do blueprint.
use super::status::{acao_atual, lista};
use super::{acoes, Mcp};
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};
use std::sync::Mutex;

type R<T> = Result<T, ErroHttp>;

/// O agente que os hooks representam; é o nome padrão do Claude, então a presença se junta à de `registrar_progresso`.
const AGENTE: &str = "Claude";
/// O contexto que a sessão recebe não pode gastar o contexto do agente (o limite do Claude Code é 10.000).
const CONTEXTO_MAX: usize = 3000;

pub(super) fn texto<'a>(v: &'a Value, chave: &str) -> &'a str {
    v.get(chave).and_then(Value::as_str).unwrap_or("")
}

pub(super) fn caminho_valido(c: &str) -> bool {
    !c.trim().is_empty() && c.chars().count() <= 500 && !c.chars().any(char::is_control)
}

pub(super) fn travado<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Consulta o vínculo atual: trocar ou remover uma pasta vale também para sessões já abertas.
pub(super) fn alvo(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Option<(String, String)>> {
    let cwd = texto(corpo, "cwd");
    if caminho_valido(cwd) {
        return Ok(travado(dados).blueprint_da_pasta(&Contexto::local(), cwd)?);
    }
    Ok(mcp.sessao(texto(corpo, "session_id")))
}

/// `pasta/vincular`: liga a pasta ao blueprint (que precisa existir); ligar de novo troca.
pub fn vincular(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (cwd, ws, bp) = (texto(corpo, "cwd"), texto(corpo, "workspaceId"), texto(corpo, "blueprintId"));
    if !caminho_valido(cwd) {
        return Err(falha(400, "cwd inválido: o caminho da pasta, de até 500 caracteres"));
    }
    let mut d = travado(dados);
    acoes::blueprint(d.ler_tudo(&Contexto::local())?, ws, bp, None)?;
    d.vincular_pasta(&Contexto::local(), cwd, ws, bp)?;
    Ok(json!({"ok": true, "workspaceId": ws, "blueprintId": bp}))
}

/// A ação em que o trabalho está (a marcada como em andamento, senão a primeira com plano pendente) e seus planos pendentes.
fn contexto_do_blueprint(estrutura: &Value, ws: &str, bp: &str, n: usize) -> String {
    let p = &estrutura["blueprint"];
    let acoes = lista(&p["acoes"]);
    let atual = acao_atual(acoes).map(|i| (i, &acoes[i]));
    let numeros = numeros_das_acoes(p, n);
    let mut t = format!("The Blueprint: esta pasta é do blueprint «{}» (workspace_id {ws}, blueprint_id {bp}).", texto(p, "nome"));
    match atual {
        Some((i, a)) => {
            // a folha numera as ações como <nº do projeto>.<nº da ação> (2.5; sem projetos, <nº do blueprint>.<nº da ação>): é como o usuário as chama
            t += &format!(" Ação atual: {} «{}» (acao_id {}, de {} a {}). Ao falar com o usuário, chame as ações pelo número da folha.", numeros[i], texto(a, "nome"), texto(a, "id"), texto(a, "inicio"), texto(a, "fim"));
            let pendentes: Vec<String> = lista(&a["planos"]).iter().filter(|x| x["feito"] != true).map(|x| format!("«{}» (plano_id {})", texto(x, "texto"), texto(x, "id"))).collect();
            if !pendentes.is_empty() {
                t += &format!(" Planos pendentes: {}.", pendentes.join("; "));
            }
        }
        None => t += " Todas as ações estão concluídas.",
    }
    t += " Mantenha-o atualizado pelo MCP blueprint: registrar_progresso ao começar cada plano, marcar_concluido só com a evidência. Trabalho fora do plano termina com registrar_trabalho e uma pergunta ao usuário sobre adicioná-lo ao blueprint.";
    t.chars().take(CONTEXTO_MAX).collect()
}

/// `hook/sessao`: o SessionStart pergunta de qual blueprint é a pasta. Devolve o texto para o contexto do agente.
pub fn sessao(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let Some((ws, bp)) = alvo(mcp, dados, corpo)? else {
        let cwd = texto(corpo, "cwd");
        if caminho_valido(cwd) && travado(&mcp.pastas_avisadas).insert(cwd.replace('\\', "/").trim_end_matches('/').to_lowercase()) {
            mcp.empurra(json!({"tipo": "pasta-sem-blueprint", "cwd": cwd}).as_object().unwrap().clone());
        }
        return Ok(json!({"vinculado": false, "contexto": "The Blueprint: esta pasta não está ligada a nenhum blueprint. Se for de um, chame listar_blueprints, confirme com o usuário qual é e chame vincular_pasta; se não for, ignore."}));
    };
    mcp.lembrar_sessao(texto(corpo, "session_id"), &ws, &bp);
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, &ws, &bp);
    let Ok(estrutura) = acoes::blueprint(tudo, &ws, &bp, None) else {
        // o blueprint foi apagado depois de ligado
        return Ok(json!({"vinculado": false, "contexto": "The Blueprint: o blueprint desta pasta não existe mais. Chame listar_blueprints e vincular_pasta com o certo."}));
    };
    Ok(json!({"vinculado": true, "workspaceId": ws, "blueprintId": bp, "contexto": contexto_do_blueprint(&estrutura, &ws, &bp, n)}))
}

/// O arquivo editado, relativo à pasta da sessão quando der, e no máximo 120 caracteres (o contrato de `onde`).
fn onde(arquivo: &str, cwd: &str) -> Option<String> {
    let a = arquivo.replace('\\', "/");
    let base = cwd.replace('\\', "/");
    let rel = a.strip_prefix(base.trim_end_matches('/')).map(|r| r.trim_start_matches('/')).filter(|r| !r.is_empty()).unwrap_or(&a);
    let n = rel.chars().count();
    let curto: String = rel.chars().skip(n.saturating_sub(120)).collect();
    (!curto.is_empty() && !curto.chars().any(char::is_control)).then_some(curto)
}

/// Quem fez a chamada do hook: dentro de um subagente o Claude Code manda `agent_id` e `agent_type` (a sessão principal não manda). O subagente vira um agente
/// próprio, "tipo + 4 primeiros caracteres do id" (até 40 caracteres), com o papel pelo tipo (Explore = explorador, *review* = revisor, o resto = construtor);
/// a sessão principal continua sendo "Claude". Devolve (nome, papel, é subagente).
fn identidade(corpo: &Value) -> (String, &'static str, bool) {
    let id: String = texto(corpo, "agent_id").chars().filter(|c| c.is_ascii_alphanumeric()).take(4).collect();
    if id.is_empty() {
        return (AGENTE.to_string(), "construtor", false);
    }
    let tipo: String = texto(corpo, "agent_type").chars().filter(|c| !c.is_control() && *c != ';').take(30).collect();
    let tipo = if tipo.trim().is_empty() { "subagente".to_string() } else { tipo.trim().to_string() };
    let minusculo = tipo.to_lowercase();
    let papel = if minusculo.contains("explore") { "explorador" } else if minusculo.contains("review") { "revisor" } else { "construtor" };
    (format!("{tipo} {id}"), papel, true)
}

/// O evento de progresso de `nome`. Um aviso sem plano limpa o plano do agente, então herda a ação e o plano do último aviso dele neste blueprint (senão a figura
/// sai do plano a cada edição). Um subagente novo não tem aviso anterior: fica no cabeçalho do blueprint.
fn evento_de(mcp: &Mcp, nome: &str, papel: &str, ws: &str, bp: &str, texto: &str, estado: &str) -> Value {
    let mut evento = json!({"tipo": "progresso", "texto": texto, "workspaceId": ws, "blueprintId": bp, "agente": nome, "papel": papel, "estado": estado});
    let agentes = mcp.agentes();
    let anterior = agentes["agentes"].as_array().and_then(|l| l.iter().find(|a| a["agente"] == nome && a["blueprintId"] == bp));
    for campo in ["acaoId", "planoId"] {
        if let Some(v) = anterior.map(|a| &a[campo]).filter(|v| v.is_string()) {
            evento[campo] = v.clone();
        }
    }
    evento
}

/// `hook/edicao`: o PostToolUse de Edit e Write vira `registrar_progresso` (o robô acompanha o arquivo), em nome do subagente se a edição veio de um.
/// Sessão sem blueprint: não faz nada.
pub fn edicao(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let Some((ws, bp)) = alvo(mcp, dados, corpo)? else { return Ok(json!({})) };
    let Some(arquivo) = corpo["tool_input"]["file_path"].as_str().and_then(|f| onde(f, texto(corpo, "cwd"))) else { return Ok(json!({})) };
    let (nome, papel, _) = identidade(corpo);
    let mut evento = evento_de(mcp, &nome, papel, &ws, &bp, &format!("Editando {arquivo}"), "escrevendo");
    evento["onde"] = json!([arquivo]);
    if evento.get("planoId").is_none() {
        super::lembrete::editou_sem_plano(mcp, texto(corpo, "session_id"), &ws, &bp);
    }
    let mut d = travado(dados);
    acoes::evento(mcp, d.ler_tudo(&Contexto::local())?, &evento)?;
    Ok(json!({}))
}

/// `hook/subagente`: SubagentStart põe o subagente na folha (lendo, no cabeçalho do blueprint) e SubagentStop o marca concluído com a última mensagem dele
/// (`last_assistant_message`, até 2000 caracteres) como a mensagem de conclusão que o usuário lê ao clicar. Só indicador; sessão sem blueprint, ou evento
/// que não veio de um subagente: não faz nada.
pub fn subagente(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (nome, papel, sub) = identidade(corpo);
    let fim = match texto(corpo, "hook_event_name") {
        "SubagentStart" => false,
        "SubagentStop" => true,
        _ => return Ok(json!({})),
    };
    let Some((ws, bp)) = alvo(mcp, dados, corpo)?.filter(|_| sub) else { return Ok(json!({})) };
    let mensagem = texto(corpo, "last_assistant_message").trim();
    let (frase, estado) = if !fim {
        (format!("{nome} começou"), "lendo")
    } else {
        (if mensagem.is_empty() { format!("{nome} terminou") } else { mensagem.to_string() }, "concluido")
    };
    let evento = evento_de(mcp, &nome, papel, &ws, &bp, &frase, estado);
    let mut d = travado(dados);
    acoes::evento(mcp, d.ler_tudo(&Contexto::local())?, &evento)?;
    Ok(json!({}))
}


pub fn desvincular(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let cwd = texto(corpo, "cwd");
    if !caminho_valido(cwd) { return Err(falha(400, "cwd inválido")); }
    travado(dados).desvincular_pasta(&Contexto::local(), cwd)?;
    Ok(json!({"ok": true}))
}
