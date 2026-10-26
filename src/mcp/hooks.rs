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

/// Consulta o vínculo atual: trocar ou remover uma pasta vale também para sessões já abertas. O `cwd` de uma worktree do git
/// cai na pasta principal do repositório quando ele mesmo não tem vínculo (ver `worktree.rs`).
pub(super) fn alvo(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Option<(String, String)>> {
    alvo_de(mcp, dados, corpo, &Contexto::local())
}

/// O membro (modo hospedado, convidado) diz o blueprint no corpo; a tabela de pastas e as sessões são só do usuário local.
fn alvo_de(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value, ctx: &Contexto) -> R<Option<(String, String)>> {
    if !ctx.local {
        return Ok(super::membro::alvo(corpo, ctx));
    }
    let cwd = texto(corpo, "cwd");
    if caminho_valido(cwd) {
        return super::worktree::blueprint_da_pasta(dados, cwd);
    }
    Ok(mcp.sessao(texto(corpo, "session_id")))
}

/// `pasta/vincular`: liga a pasta ao blueprint (que precisa existir); ligar de novo troca. Numa worktree do git o vínculo é gravado na pasta principal
/// do repositório, para valer em todas as worktrees dele.
pub fn vincular(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let (cwd, ws, bp) = (texto(corpo, "cwd"), texto(corpo, "workspaceId"), texto(corpo, "blueprintId"));
    if !caminho_valido(cwd) {
        return Err(falha(400, crate::tr!("invalid cwd: the folder path, up to 500 characters", "cwd inválido: o caminho da pasta, de até 500 caracteres")).codigo("cwd-invalido"));
    }
    let pasta = super::worktree::principal_ou(cwd);
    let mut d = travado(dados);
    acoes::blueprint(d.ler_tudo(&Contexto::local())?, ws, bp, None)?;
    d.vincular_pasta(&Contexto::local(), &pasta, ws, bp)?;
    Ok(json!({"ok": true, "workspaceId": ws, "blueprintId": bp}))
}

/// A ação em que o trabalho está (a marcada como em andamento, senão a primeira com plano pendente) e seus planos pendentes.
fn contexto_do_blueprint(estrutura: &Value, ws: &str, bp: &str, n: usize, subagente: bool) -> String {
    let p = &estrutura["blueprint"];
    let acoes = lista(&p["acoes"]);
    let atual = acao_atual(acoes).map(|i| (i, &acoes[i]));
    let numeros = numeros_das_acoes(p, n);
    let nome_bp = texto(p, "nome");
    let mut t = crate::tr!("The Blueprint: this folder belongs to blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}).", "The Blueprint: esta pasta é do blueprint «{nome_bp}» (workspace_id {ws}, blueprint_id {bp}).");
    match atual {
        Some((i, a)) => {
            // a folha numera as ações como <nº do projeto>.<nº da ação> (2.5; sem projetos, <nº do blueprint>.<nº da ação>): é como o usuário as chama
            t += &crate::tr!(" Current action: {} «{}» (acao_id {}, from {} to {}). When talking to the user, call actions by their sheet number.", " Ação atual: {} «{}» (acao_id {}, de {} a {}). Ao falar com o usuário, chame as ações pelo número da folha.", numeros[i], texto(a, "nome"), texto(a, "id"), texto(a, "inicio"), texto(a, "fim"));
            let pendentes: Vec<String> = lista(&a["planos"]).iter().filter(|x| x["feito"] != true).map(|x| format!("«{}» (plano_id {})", texto(x, "texto"), texto(x, "id"))).collect();
            if !pendentes.is_empty() {
                t += &crate::tr!(" Pending plans: {}.", " Planos pendentes: {}.", pendentes.join("; "));
            }
        }
        None => t += crate::idioma::ou(" All actions are done.", " Todas as ações estão concluídas."),
    }
    t += if subagente {
        // quem lê é o subagente (sempre em inglês): ele não marca nada, o agente principal é quem fecha os planos com evidência
        " You are a subagent: do not mark plans done; report with registrar_progresso only if asked."
    } else {
        crate::idioma::ou(
            " Keep it up to date through the blueprint MCP: registrar_progresso when starting each plan, marcar_concluido only with the evidence. Work outside the plan ends with registrar_trabalho and a question to the user about adding it to the blueprint.",
            " Mantenha-o atualizado pelo MCP blueprint: registrar_progresso ao começar cada plano, marcar_concluido só com a evidência. Trabalho fora do plano termina com registrar_trabalho e uma pergunta ao usuário sobre adicioná-lo ao blueprint.",
        )
    };
    t.chars().take(CONTEXTO_MAX).collect()
}

/// O contexto do SubagentStart: o mesmo blueprint e ação atual da sessão, em inglês (é o subagente quem lê), com a regra de não marcar plano.
/// `None` se o blueprint não existe mais.
fn contexto_do_subagente(dados: &Mutex<Dados>, ws: &str, bp: &str) -> R<Option<String>> {
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, ws, bp);
    let Ok(estrutura) = acoes::blueprint(tudo, ws, bp, None) else { return Ok(None) };
    Ok(Some(crate::idioma::com(crate::idioma::Idioma::En, || contexto_do_blueprint(&estrutura, ws, bp, n, true))))
}

/// `hook/sessao`: o SessionStart pergunta de qual blueprint é a pasta. Devolve o texto para o contexto do agente.
#[cfg(test)]
pub fn sessao(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    sessao_de(mcp, dados, corpo, &Contexto::local())
}

/// `hook/sessao` com o contexto de quem chamou. Do membro: sem aviso de pasta solta e sem entrar na lista de sessões (Iniciar é só local).
pub fn sessao_de(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value, ctx: &Contexto) -> R<Value> {
    let Some((ws, bp)) = alvo_de(mcp, dados, corpo, ctx)? else {
        if !ctx.local {
            return Ok(json!({"vinculado": false}));
        }
        let cwd = texto(corpo, "cwd");
        if caminho_valido(cwd) {
            let pasta = super::worktree::principal_ou(cwd); // numa worktree, é a pasta principal que a pessoa precisa ligar
            if travado(&mcp.pastas_avisadas).insert(pasta.replace('\\', "/").trim_end_matches('/').to_lowercase()) {
                mcp.empurra(json!({"tipo": "pasta-sem-blueprint", "cwd": pasta}).as_object().unwrap().clone());
            }
        }
        return Ok(json!({"vinculado": false, "contexto": crate::idioma::ou("The Blueprint: this folder is not linked to any blueprint. If it belongs to one, call listar_blueprints, confirm with the user which one and call vincular_pasta; if not, ignore this.", "The Blueprint: esta pasta não está ligada a nenhum blueprint. Se for de um, chame listar_blueprints, confirme com o usuário qual é e chame vincular_pasta; se não for, ignore.")}));
    };
    if ctx.local {
        mcp.lembrar_sessao(texto(corpo, "session_id"), &ws, &bp);
        mcp.abrir_sessao(texto(corpo, "session_id"), texto(corpo, "cwd"), &ws, &bp);
    }
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, &ws, &bp);
    let Ok(estrutura) = acoes::blueprint(tudo, &ws, &bp, None) else {
        // o blueprint foi apagado depois de ligado
        return Ok(json!({"vinculado": false, "contexto": crate::idioma::ou("The Blueprint: this folder's blueprint no longer exists. Call listar_blueprints and vincular_pasta with the right one.", "The Blueprint: o blueprint desta pasta não existe mais. Chame listar_blueprints e vincular_pasta com o certo.")}));
    };
    Ok(json!({"vinculado": true, "workspaceId": ws, "blueprintId": bp, "contexto": contexto_do_blueprint(&estrutura, &ws, &bp, n, false)}))
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
/// Os agentes do plugin chegam com o prefixo dele (`blueprint:blueprint-explorer`, `blueprint:blueprint-reviewer`, `blueprint:blueprint-builder`);
/// o papel sai do nome, então eles viram explorador, revisor e construtor sem mapa à parte.
pub(super) fn identidade(corpo: &Value) -> (String, &'static str, bool) {
    let tipo = tipo_limpo(texto(corpo, "agent_type"));
    match nome_do_subagente(texto(corpo, "agent_id"), texto(corpo, "agent_type")) {
        Some(nome) => (nome, papel_do_tipo(&tipo), true),
        None => (AGENTE.to_string(), "construtor", false),
    }
}

/// O mesmo, com o nome do membro quando o hook vem de um convidado (`membro.rs`).
fn identidade_de(corpo: &Value, ctx: &Contexto) -> (String, &'static str, bool) {
    let (nome, papel, sub) = identidade(corpo);
    (super::membro::com_membro(&nome, ctx), papel, sub)
}

/// O tipo do subagente como nome: sem controle nem ';' (o Windows Terminal o lê como separador), até 30 caracteres; vazio vira "subagente".
fn tipo_limpo(tipo: &str) -> String {
    let t: String = tipo.chars().filter(|c| !c.is_control() && *c != ';').take(30).collect();
    if t.trim().is_empty() { "subagente".to_string() } else { t.trim().to_string() }
}

/// O nome do subagente: tipo + 4 primeiros caracteres alfanuméricos do id, sem o prefixo `agent-` com que a documentação do Claude Code mostra o id no
/// SubagentStart (senão todos virariam «Explore agen»). Sem id utilizável: None. É a única regra; o painel de subagentes (`subagentes_status`) a chama.
pub(super) fn nome_do_subagente(id: &str, tipo: &str) -> Option<String> {
    let id: String = id.strip_prefix("agent-").unwrap_or(id).chars().filter(|c| c.is_ascii_alphanumeric()).take(4).collect();
    (!id.is_empty()).then(|| format!("{} {id}", tipo_limpo(tipo)))
}

/// Explore e `*explorer` = explorador, `*review*` = revisor, o resto (general-purpose, `*builder`, agentes do usuário) = construtor.
fn papel_do_tipo(tipo: &str) -> &'static str {
    let t = tipo.to_lowercase();
    if t.contains("explor") { "explorador" } else if t.contains("review") { "revisor" } else { "construtor" }
}

/// O evento de progresso de `nome`. Um aviso sem plano limpa o plano do agente, então herda a ação e o plano do último aviso dele neste blueprint (senão a figura
/// sai do plano a cada edição). Um subagente novo não tem aviso anterior: herda o plano do Claude principal se o aviso dele for recente e não concluído
/// (assim a figura já nasce ao lado do plano); sem isso, fica no cabeçalho do blueprint.
pub(super) fn evento_de(mcp: &Mcp, nome: &str, papel: &str, ws: &str, bp: &str, texto: &str, estado: &str) -> Value {
    evento_de_membro(mcp, &Contexto::local(), nome, papel, ws, bp, texto, estado)
}

/// O mesmo, com o Claude principal do membro quando o hook vem de um convidado.
#[allow(clippy::too_many_arguments)]
fn evento_de_membro(mcp: &Mcp, ctx: &Contexto, nome: &str, papel: &str, ws: &str, bp: &str, texto: &str, estado: &str) -> Value {
    let principal = super::membro::com_membro(AGENTE, ctx);
    let mut evento = json!({"tipo": "progresso", "texto": texto, "workspaceId": ws, "blueprintId": bp, "agente": nome, "papel": papel, "estado": estado});
    let agentes = mcp.agentes();
    let anterior = agentes["agentes"].as_array().and_then(|l| l.iter().find(|a| a["agente"] == nome && a["blueprintId"] == bp));
    for campo in ["acaoId", "planoId"] {
        if let Some(v) = anterior.map(|a| &a[campo]).filter(|v| v.is_string()) {
            evento[campo] = v.clone();
        }
    }
    if anterior.is_none() && nome != principal {
        if let Some((acao, plano)) = mcp.plano_de(&principal, bp) {
            (evento["acaoId"], evento["planoId"]) = (json!(acao), json!(plano));
        }
    }
    evento
}

/// `hook/edicao`: o PostToolUse de Edit e Write vira `registrar_progresso` (o robô acompanha o arquivo), em nome do subagente se a edição veio de um.
/// Sessão sem blueprint: não faz nada.
#[cfg(test)]
pub fn edicao(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    edicao_de(mcp, dados, corpo, &Contexto::local())
}

pub fn edicao_de(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value, ctx: &Contexto) -> R<Value> {
    let Some((ws, bp)) = alvo_de(mcp, dados, corpo, ctx)? else { return Ok(json!({})) };
    let Some(arquivo) = corpo["tool_input"]["file_path"].as_str().and_then(|f| onde(f, texto(corpo, "cwd"))) else { return Ok(json!({})) };
    let (nome, papel, _) = identidade_de(corpo, ctx);
    let mut evento = evento_de_membro(mcp, ctx, &nome, papel, &ws, &bp, &crate::tr!("Editing {arquivo}", "Editando {arquivo}"), "escrevendo");
    evento["onde"] = json!([arquivo]);
    if evento.get("planoId").is_none() {
        super::lembrete::editou_sem_plano(mcp, texto(corpo, "session_id"), &ws, &bp);
    }
    super::linha_do_tempo::evento(mcp, dados, &evento)?;
    Ok(json!({}))
}

/// `hook/subagente`: SubagentStart põe o subagente na folha (lendo, no cabeçalho do blueprint) e SubagentStop o marca concluído com a última mensagem dele
/// (`last_assistant_message`, até 2000 caracteres) como a mensagem de conclusão que o usuário lê ao clicar. Só indicador; sessão sem blueprint, ou evento
/// que não veio de um subagente: não faz nada. O START ainda devolve `hookSpecificOutput.additionalContext` com o blueprint da pasta e a regra do subagente
/// (ver `contexto_do_subagente`); o STOP nunca devolve `decision`, para nunca segurar o subagente.
#[cfg(test)]
pub fn subagente(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    subagente_de(mcp, dados, corpo, &Contexto::local())
}

pub fn subagente_de(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value, ctx: &Contexto) -> R<Value> {
    let (nome, papel, sub) = identidade_de(corpo, ctx);
    let fim = match texto(corpo, "hook_event_name") {
        "SubagentStart" => false,
        "SubagentStop" => true,
        _ => return Ok(json!({})),
    };
    let Some((ws, bp)) = alvo_de(mcp, dados, corpo, ctx)?.filter(|_| sub) else { return Ok(json!({})) };
    let mensagem = texto(corpo, "last_assistant_message").trim();
    let (frase, estado) = if !fim {
        (format!("{nome} começou"), "lendo")
    } else {
        (if mensagem.is_empty() { crate::tr!("{nome} finished", "{nome} terminou") } else { mensagem.to_string() }, "concluido")
    };
    let evento = evento_de_membro(mcp, ctx, &nome, papel, &ws, &bp, &frase, estado);
    super::linha_do_tempo::evento(mcp, dados, &evento)?;
    if fim {
        return Ok(json!({}));
    }
    Ok(match contexto_do_subagente(dados, &ws, &bp)? {
        Some(c) => json!({"hookSpecificOutput": {"hookEventName": "SubagentStart", "additionalContext": c}}),
        None => json!({}),
    })
}

pub fn desvincular(dados: &Mutex<Dados>, corpo: &Value) -> R<Value> {
    let cwd = texto(corpo, "cwd");
    if !caminho_valido(cwd) { return Err(falha(400, crate::tr!("invalid cwd", "cwd inválido")).codigo("cwd-invalido")); }
    let principal = super::worktree::principal_ou(cwd);
    let mut d = travado(dados);
    d.desvincular_pasta(&Contexto::local(), cwd)?;
    if principal != cwd {
        d.desvincular_pasta(&Contexto::local(), &principal)?; // numa worktree, o vínculo que vale é o da pasta principal
    }
    Ok(json!({"ok": true}))
}
