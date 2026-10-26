//! `blueprint --mcp`: o servidor MCP por stdio (JSON-RPC 2.0, uma mensagem por linha) que o agente do usuário registra
//! (`claude mcp add blueprint -- blueprint.exe --mcp`). Porte de mcp-roadmap.mjs: mesmas ferramentas e instruções. Não tem dados próprios: cada ferramenta
//! chama as rotas /api/mcp do servidor local. Tudo aqui é texto para o AGENTE, então é inglês (o agente responde ao usuário no idioma dele); o
//! servidor também responde em inglês porque o cliente manda `Accept-Language: en` (ver `crate::idioma`).
use super::cliente::{codificar, requisitar};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};

const INSTRUCOES: &str = include_str!("instrucoes.txt"); // os primeiros 512 caracteres são autossuficientes: é o que o Codex lê
const FERRAMENTAS: &str = include_str!("ferramentas.json");
const VERSOES: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const GUIA: &str = include_str!("../../skills/blueprint-planejamento/SKILL.md");
const TOPICOS: [(&str, &str); 6] = [
    ("planejar", include_str!("../../skills/blueprint-planejamento/topicos/planejar.md")),
    ("editar", include_str!("../../skills/blueprint-planejamento/topicos/editar.md")),
    ("executar", include_str!("../../skills/blueprint-planejamento/topicos/executar.md")),
    ("prototipos", include_str!("../../skills/blueprint-planejamento/topicos/prototipos.md")),
    ("importar", include_str!("../../skills/blueprint-planejamento/topicos/importar.md")),
    ("apresentar", include_str!("../../skills/blueprint-planejamento/topicos/apresentar.md")),
];
const REGRA: &str = "Keep this blueprint always up to date: plans and plan changes via propor_blueprint; when you start a plan, registrar_progresso; when you finish, marcar_concluido with the evidence; if it did not work, marcar_nao_concluido; if you get stuck, reportar_bloqueio.";

/// só ao criar blueprint: as instruções do servidor já dizem para manter a linha nas sessões seguintes
fn vinculo(w: &str) -> String {
    format!("\nWhen the new blueprint is applied, the app links this session's folder automatically. Workspace: {w}. {REGRA}")
}

type R = Result<String, String>;

pub(crate) fn base() -> String {
    std::env::var("BLUEPRINT_URL").ok().filter(|u| !u.is_empty()).unwrap_or_else(|| "http://127.0.0.1:3000".into()).trim_end_matches('/').to_string()
}

/// Chama uma rota do servidor; se ele recusar, devolve o motivo como o agente precisa lê-lo.
fn servidor(rota: &str, metodo: &str, corpo: Option<&Value>) -> Result<Value, String> {
    let enviado = requisitar(&base(), metodo, rota, corpo.map(Value::to_string).as_deref());
    let (status, texto) = match (enviado, corpo) {
        (Ok(resposta), _) => resposta,
        // app fechado: o registro fica em arquivo e entra no app quando ele abrir (pendentes.rs); só as rotas que não precisam dos dados ao vivo
        (Err(_), Some(corpo)) if metodo == "POST" && super::pendentes::ROTAS.contains(&rota) => return guardar_offline(rota, corpo),
        (Err(e), _) => return Err(e),
    };
    let dados: Value = serde_json::from_str(&texto).unwrap_or_else(|_| json!({}));
    if (200..300).contains(&status) {
        return Ok(dados);
    }
    let lista = |chave: &str| dados.get(chave).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n- ")).unwrap_or_default();
    if dados.get("erros").is_some_and(Value::is_array) {
        let avisos = lista("avisos");
        let quem = if rota == "registro" { "The record" } else { "The proposal" }; // registro não é proposta
        return Err(format!("{quem} has errors; fix them and send again:\n- {}{}", lista("erros"), if avisos.is_empty() { String::new() } else { format!("\nWarnings:\n- {avisos}") }));
    }
    let motivo = dados.get("erro").and_then(Value::as_str).map(String::from).unwrap_or_else(|| texto.chars().take(200).collect());
    Err(format!("The Blueprint refused ({status}): {motivo}"))
}

/// A última ferramenta foi guardada em arquivo porque o app estava fechado (a resposta ao agente avisa; o stdio atende uma ferramenta por vez).
static GUARDADO_OFFLINE: AtomicBool = AtomicBool::new(false);

fn guardar_offline(rota: &str, corpo: &Value) -> Result<Value, String> {
    let pasta = super::pendentes::pasta_do_banco(&crate::http::Config::do_app().banco);
    let pendentes = super::pendentes::guardar(&pasta, rota, corpo).map_err(|e| format!("The Blueprint is not open and the record could not be saved: {e}"))?;
    GUARDADO_OFFLINE.store(true, Ordering::Relaxed);
    Ok(json!({"offline": true, "pendentes": pendentes}))
}

const AVISO_OFFLINE: &str = "\n(The Blueprint is closed: the record was saved and enters the app when it opens, without you repeating it. Validation (ids, format) only happens then; if something is refused, the file stays in pendentes/recusados, next to the database.)";

fn texto(a: &Value, chave: &str) -> Result<String, String> {
    match a.get(chave).and_then(Value::as_str).map(str::trim) {
        Some(t) if !t.is_empty() => Ok(t.to_string()),
        _ => Err(format!("{chave} is required.")),
    }
}

fn opcional(a: &Value, chave: &str) -> String {
    a.get(chave).and_then(Value::as_str).unwrap_or("").to_string()
}

fn alvo(a: &Value) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("workspaceId".into(), json!(opcional(a, "workspace_id")));
    m.insert("blueprintId".into(), json!(opcional(a, "blueprint_id")));
    m
}

fn com(mut m: Map<String, Value>, extra: &[(&str, Value)]) -> Map<String, Value> {
    for (k, v) in extra {
        m.insert((*k).into(), v.clone());
    }
    m
}

fn plano(a: &Value) -> Result<Map<String, Value>, String> {
    Ok(com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("planoId", json!(texto(a, "plano_id")?))]))
}

fn checkpoint(a: &Value) -> Result<Map<String, Value>, String> {
    Ok(com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("checkpointId", json!(texto(a, "checkpoint_id")?))]))
}

/// Os campos opcionais de atividade (painel de agentes) que a ferramenta recebeu; o servidor valida e recusa o que estiver fora do contrato.
fn com_atividade(mut m: Map<String, Value>, a: &Value) -> Map<String, Value> {
    for campo in ["agente", "papel", "estado", "onde", "percentual"] {
        if let Some(v) = a.get(campo) {
            m.insert(campo.into(), v.clone());
        }
    }
    m
}

fn plano_se_houver(a: &Value) -> Result<Map<String, Value>, String> {
    if a.get("plano_id").is_some_and(|v| crate::blueprint::verdade(Some(v))) { plano(a) } else { Ok(alvo(a)) }
}

fn evento(tipo: &str, texto: String, extra: Map<String, Value>) -> Result<Value, String> {
    servidor("evento", "POST", Some(&Value::Object(com(extra, &[("tipo", json!(tipo)), ("texto", json!(texto))]))))
}

/// tira o bloco `---…---` do começo de um arquivo de guia
fn sem_cabecalho(t: &str) -> String {
    match t.strip_prefix("---").and_then(|r| r.split_once("---")) {
        Some((_, resto)) => resto.trim_start().to_string(),
        None => t.to_string(),
    }
}

/// Os ids provisórios da proposta, em poucas linhas: ação (id real ou "+aN") e os planos e checkpoints novos dela.
pub(super) fn ids_provisorios(ids: &Value) -> String {
    let item = |x: &Value, k: &str| format!("{} {:?}", x["id"].as_str().unwrap_or(""), x[k].as_str().unwrap_or(""));
    let lista = |a: &Value, chave: &str, nome: &str| -> String { a[chave].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().map(|x| item(x, nome)).collect::<Vec<_>>().join(", ") };
    let linhas: Vec<String> = ids.as_array().map(Vec::as_slice).unwrap_or(&[]).iter().map(|a| {
        let (planos, cps) = (lista(a, "planos", "texto"), lista(a, "checkpoints", "nome"));
        format!("- action {}{}{}", item(a, "nome"), if planos.is_empty() { String::new() } else { format!("; plans: {planos}") }, if cps.is_empty() { String::new() } else { format!("; checkpoints: {cps}") })
    }).collect();
    format!("Provisional ids (valid while the proposal is in the queue):\n{}", linhas.join("\n"))
}

fn chamar(nome: &str, a: &Value) -> R {
    match nome {
        "listar_blueprints" => {
            let r = servidor("blueprints", "GET", None)?;
            let linhas: Vec<String> = r["workspaces"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().map(|w| {
                let blueprints = w["blueprints"].as_array().map(Vec::as_slice).unwrap_or(&[]);
                let corpo = if blueprints.is_empty() { "  (no blueprints)".to_string() } else { blueprints.iter().map(|p| format!("  - {} (blueprint_id: {}) · {} action(s)", p["nome"].as_str().unwrap_or(""), p["id"].as_str().unwrap_or(""), p["acoes"])).collect::<Vec<_>>().join("\n") };
                format!("Workspace {} (workspace_id: {})\n{corpo}", w["nome"].as_str().unwrap_or(""), w["id"].as_str().unwrap_or(""))
            }).collect();
            Ok(format!("Today: {}\n{}", r["hoje"].as_str().unwrap_or(""), linhas.join("\n")))
        }
        "guia_de_planejamento" => {
            let t = opcional(a, "topico");
            let t = t.trim();
            if t.is_empty() {
                return Ok(sem_cabecalho(GUIA));
            }
            match TOPICOS.iter().find(|(n, _)| *n == t) {
                Some((_, conteudo)) => Ok(sem_cabecalho(conteudo)),
                None => Err(format!("Unknown topic: {t}. Use one of: {}.", TOPICOS.map(|(n, _)| n).join(", "))),
            }
        }
        "ler_blueprint" => {
            let acao = opcional(a, "acao_id");
            let acao = if acao.trim().is_empty() { String::new() } else { format!("&acao={}", codificar(acao.trim())) };
            let rota = format!("blueprint?workspace={}&blueprint={}{acao}", codificar(&texto(a, "workspace_id")?), codificar(&texto(a, "blueprint_id")?));
            Ok(servidor(&rota, "GET", None)?.to_string()) // compacto: cada byte vai para o contexto do agente
        }
        "propor_blueprint" => {
            let mut corpo = alvo(a);
            for (de, para) in [("mensagem", "mensagem"), ("blueprint", "blueprint"), ("acoes", "acoes"), ("remover", "remover"), ("mover", "mover"), ("ajustar", "ajustar"), ("substituir", "substituir")] {
                if let Some(v) = a.get(de) {
                    corpo.insert(para.into(), v.clone());
                }
            }
            if opcional(a, "blueprint_id").is_empty() {
                if let Ok(pasta) = std::env::current_dir() {
                    corpo.insert("pasta".into(), json!(pasta.display().to_string()));
                }
            }
            let r = servidor("proposta", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Proposal saved: the app is closed. When it opens, the proposal enters the queue for the user to review and apply (ids and format are validated only then).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings (they do not block the submission, but worth fixing):\n- {}", avisos.join("\n- ")) };
            let ligar = if opcional(a, "blueprint_id").is_empty() { vinculo(&opcional(a, "workspace_id")) } else { String::new() };
            Ok(format!("Proposal no. {} is in the app's queue for the user to review and apply (one at a time; the others in the queue were not touched).{aviso}\n{}\nIf the user asks for changes, send `ajustar: {}` with only what changes (the action, plan or checkpoint by the id above); `substituir: {}` replaces the whole proposal.{ligar}", r["numero"], ids_provisorios(&r["ids"]), r["numero"], r["numero"]))
        }
        "registrar_trabalho" => {
            let mut corpo = alvo(a);
            for chave in ["mensagem", "acoes"] {
                if let Some(v) = a.get(chave) {
                    corpo.insert(chave.into(), v.clone());
                }
            }
            texto(a, "blueprint_id")?; // um registro é sobre um blueprint que já existe
            let r = servidor("registro", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Record saved: the app is closed. When it opens, the record enters the queue for the user to decide whether to add it to the blueprint (validation happens only then).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings:\n- {}", avisos.join("\n- ")) };
            Ok(format!("Record no. {} is in the app's queue (it is not a proposal). Nothing was saved: the user sees what was done, with the checks, and decides whether to add it to the blueprint. Tell the user now and ask whether to add it.{aviso}", r["numero"]))
        }
        "vincular_pasta" => {
            // o servidor MCP roda na pasta da sessão do agente: é ela que fica ligada
            let pasta = std::env::current_dir().map_err(|e| format!("Cannot tell which folder this is: {e}"))?;
            let corpo = com(alvo(a), &[("cwd", json!(pasta.display().to_string()))]);
            servidor("pasta/vincular", "POST", Some(&Value::Object(corpo)))?;
            Ok(format!("Folder {} linked to the blueprint. The next sessions in it start with the blueprint and the current action.", pasta.display()))
        }
        "anexar_prototipo" => {
            let corpo = com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("titulo", json!(texto(a, "titulo")?)), ("html", json!(texto(a, "html")?)), ("prototipoId", json!(opcional(a, "prototipo_id")))]);
            let r = servidor("prototipo", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Prototype saved: the app is closed. It is attached to the action when the app opens.".into());
            }
            let acao = if r["atualizado"] == true { "updated in" } else { "attached to" };
            Ok(format!("Prototype \"{}\" {acao} the action (prototipo_id: {}); the user sees it on the blueprint sheet, in the action's prototype icon. To revise it, call again with this prototipo_id.", r["titulo"].as_str().unwrap_or(""), r["prototipoId"].as_str().unwrap_or("")))
        }
        "registrar_progresso" => {
            evento("progresso", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Progress recorded.".into())
        }
        "marcar_concluido" => {
            evento("concluir", texto(a, "evidencia")?, plano(a)?)?;
            Ok("Plan sent to the app as done, with the evidence.".into())
        }
        "marcar_nao_concluido" => {
            evento("nao-concluida", texto(a, "motivo")?, plano(a)?)?;
            Ok("Recorded: the plan stays pending.".into())
        }
        "marcar_checkpoint_feito" => {
            evento("checkpoint-feito", texto(a, "evidencia")?, checkpoint(a)?)?;
            Ok("Checkpoint sent to the app as reached, with the evidence.".into())
        }
        "marcar_checkpoint_nao_feito" => {
            evento("checkpoint-nao-feito", texto(a, "motivo")?, checkpoint(a)?)?;
            Ok("Recorded: the checkpoint stays pending.".into())
        }
        "reportar_bloqueio" => {
            evento("bloqueio", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Block recorded; the user was notified.".into())
        }
        "responder_conversa" => super::conversa::ferramenta(a, &servidor),
        _ => Err(format!("Unknown tool: {nome}")),
    }
}

fn resposta(id: &Value, corpo: Value) -> Value {
    let mut m = Map::new();
    m.insert("jsonrpc".into(), json!("2.0"));
    m.insert("id".into(), id.clone());
    if let Value::Object(extra) = corpo {
        m.extend(extra);
    }
    Value::Object(m)
}

/// Responde a uma mensagem JSON-RPC; None para notificações (sem id), que não têm resposta.
pub fn tratar(msg: &Value) -> Option<Value> {
    let id = msg.get("id")?;
    let (metodo, params) = (msg["method"].as_str().unwrap_or(""), &msg["params"]);
    let erro = |codigo: i32, mensagem: String| resposta(id, json!({"error": {"code": codigo, "message": mensagem}}));
    Some(match metodo {
        "initialize" => {
            let pedida = params["protocolVersion"].as_str().unwrap_or("");
            let versao = VERSOES.iter().find(|v| **v == pedida).unwrap_or(&VERSOES[0]);
            resposta(id, json!({"result": {"protocolVersion": versao, "capabilities": {"tools": {}, "experimental": {"claude/channel": {}}}, "serverInfo": {"name": "blueprint", "version": "2.0.0"}, "instructions": INSTRUCOES}}))
        }
        "ping" => resposta(id, json!({"result": {}})),
        "tools/list" => {
            let ferramentas: Value = serde_json::from_str(FERRAMENTAS).unwrap_or_else(|_| json!([]));
            resposta(id, json!({"result": {"tools": ferramentas}}))
        }
        "tools/call" => {
            let nome = params["name"].as_str().unwrap_or("");
            let ferramentas: Value = serde_json::from_str(FERRAMENTAS).unwrap_or_else(|_| json!([]));
            if !ferramentas.as_array().is_some_and(|f| f.iter().any(|x| x["name"] == nome)) {
                return Some(erro(-32602, format!("Unknown tool: {nome}")));
            }
            let argumentos = if params["arguments"].is_object() { params["arguments"].clone() } else { json!({}) };
            GUARDADO_OFFLINE.store(false, Ordering::Relaxed);
            match chamar(nome, &argumentos) {
                Ok(mut texto) => {
                    if GUARDADO_OFFLINE.swap(false, Ordering::Relaxed) {
                        texto.push_str(AVISO_OFFLINE);
                    }
                    resposta(id, json!({"result": {"content": [{"type": "text", "text": texto}]}}))
                }
                Err(texto) => resposta(id, json!({"result": {"isError": true, "content": [{"type": "text", "text": texto}]}})), // o erro volta ao modelo, que pode corrigir
            }
        }
        outro => erro(-32601, format!("Method not supported: {outro}")),
    })
}

/// O id desta sessão com o canal ligado (16 hex): o app o usa para endereçar o pedido de iniciar uma ação. Não é segredo (o app só escuta em loopback);
/// só precisa ser diferente entre processos, então basta o pid com a hora.
fn id_do_canal() -> String {
    use sha2::{Digest, Sha256};
    let semente = format!("{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
    Sha256::digest(semente).iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// Lê mensagens de stdin (uma por linha) até o agente fechar a conversa.
pub fn executar() -> i32 {
    crate::idioma::fixar(crate::idioma::Idioma::En); // texto para o agente: o servidor responde em inglês (Accept-Language do cliente)
    let saida = std::io::stdout();
    let mut canal_iniciado = false;
    for linha in std::io::stdin().lock().lines().map_while(Result::ok) {
        if linha.trim().is_empty() {
            continue;
        }
        let r = match serde_json::from_str::<Value>(&linha) {
            Ok(msg) => {
                if !canal_iniciado && msg["method"] == "notifications/initialized" && std::env::var("BLUEPRINT_CHANNEL").as_deref() == Ok("1") {
                    canal_iniciado = true;
                    let id = id_do_canal();
                    std::thread::spawn(|| crate::watch::observar(Some(id), |texto| {
                        let mut saida = std::io::stdout().lock();
                        writeln!(saida, "{}", notificacao_canal(texto))?;
                        saida.flush()
                    }));
                }
                tratar(&msg)
            },
            Err(_) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "invalid JSON"}})),
        };
        if let Some(r) = r {
            let mut s = saida.lock();
            if writeln!(s, "{r}").and_then(|_| s.flush()).is_err() {
                break;
            }
        }
    }
    0
}

pub(super) fn notificacao_canal(texto: &str) -> Value {
    json!({"jsonrpc":"2.0", "method":"notifications/claude/channel", "params":{"content":texto}})
}
