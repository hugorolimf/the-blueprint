//! `blueprint --mcp`: o servidor MCP por stdio (JSON-RPC 2.0, uma mensagem por linha) que o agente do usuário registra
//! (`claude mcp add blueprint -- blueprint.exe --mcp`). Porte de mcp-roadmap.mjs: mesmas ferramentas e instruções. Não tem dados próprios: cada ferramenta
//! chama as rotas /api/mcp do servidor local. Tudo aqui é texto para o AGENTE, então é inglês (o agente responde ao usuário no idioma dele); o
//! servidor também responde em inglês porque o cliente manda `Accept-Language: en` (ver `crate::idioma`).
use super::argumentos::{alvo, checkpoint, com, com_atividade, estruturado, opcional, plano, plano_se_houver, texto};
use super::cliente::{codificar, requisitar};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};

const INSTRUCOES: &str = include_str!("instrucoes.txt"); // os primeiros 512 caracteres são autossuficientes: é o que o Codex lê
const FERRAMENTAS: &str = include_str!("ferramentas.json");
pub(super) const VERSOES: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const GUIA: &str = include_str!("../../skills/blueprint-planejamento/SKILL.md");
const TOPICOS: [(&str, &str); 10] = [
    ("planejar", include_str!("../../skills/blueprint-planejamento/topicos/planejar.md")),
    ("editar", include_str!("../../skills/blueprint-planejamento/topicos/editar.md")),
    ("executar", include_str!("../../skills/blueprint-planejamento/topicos/executar.md")),
    ("prototipos", include_str!("../../skills/blueprint-planejamento/topicos/prototipos.md")),
    ("importar", include_str!("../../skills/blueprint-planejamento/topicos/importar.md")),
    ("apresentar", include_str!("../../skills/blueprint-planejamento/topicos/apresentar.md")),
    ("apresentacao_ia", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_ia.md")),
    ("apresentacao_conteudo", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_conteudo.md")),
    ("apresentacao_figuras", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_figuras.md")),
    ("apresentacao_ajustar", include_str!("../../skills/blueprint-planejamento/topicos/apresentacao_ajustar.md")),
];
const REGRA: &str = "Keep this blueprint always up to date: plans and plan changes via propor_blueprint; when you start a plan, registrar_progresso; when you finish, marcar_concluido with the evidence; if it did not work, marcar_nao_concluido; if you get stuck, reportar_bloqueio.";

/// só ao criar blueprint: as instruções do servidor já dizem para manter a linha nas sessões seguintes
fn vinculo(w: &str) -> String {
    format!("\nWhen the new blueprint is applied, the app links this session's folder automatically. Workspace: {w}. {REGRA}")
}

type R = Result<String, String>;

/// Por onde as ferramentas falam com o servidor: `None` = o servidor local, por socket (o `--mcp` por stdio); `Some` = direto na rota do próprio processo, que é como a rota
/// `/mcp` por HTTP (src/mcp/http.rs) atende o convidado: recebe (método, rota de /api/mcp, corpo) e devolve (status, corpo).
pub(super) type Via<'a> = Option<&'a dyn Fn(&str, &str, Option<&str>) -> Result<(u16, String), String>>;

pub(crate) fn base() -> String {
    if super::equipe::ligado() {
        // o plugin do convidado: o endereço do `blueprint conectar` (vazio = não conectado; o cliente explica)
        return super::equipe::conexao().map(|c| c.url).unwrap_or_default();
    }
    std::env::var("BLUEPRINT_URL").ok().filter(|u| !u.is_empty()).unwrap_or_else(|| "http://127.0.0.1:3000".into()).trim_end_matches('/').to_string()
}

/// Chama uma rota do servidor; se ele recusar, devolve o motivo como o agente precisa lê-lo.
pub(super) fn servidor(via: Via, rota: &str, metodo: &str, corpo: Option<&Value>) -> Result<Value, String> {
    let enviado = match via {
        Some(direto) => direto(metodo, rota, corpo.map(Value::to_string).as_deref()),
        None => requisitar(&base(), metodo, rota, corpo.map(Value::to_string).as_deref()),
    };
    let (status, texto) = match (enviado, corpo) {
        (Ok(resposta), _) => resposta,
        // app fechado: o registro fica em arquivo e entra no app quando ele abrir (pendentes.rs); só as rotas que não precisam dos dados ao vivo
        // servidor remoto fora do ar: não guarda (o app desta máquina reenviaria como dono, nos dados dele)
        (Err(_), Some(corpo)) if metodo == "POST" && super::pendentes::ROTAS.contains(&rota) && !super::equipe::remoto() => return guardar_offline(rota, corpo),
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

fn evento(via: Via, tipo: &str, texto: String, extra: Map<String, Value>) -> Result<Value, String> {
    servidor(via, "evento", "POST", Some(&Value::Object(com(extra, &[("tipo", json!(tipo)), ("texto", json!(texto))]))))
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

fn chamar(via: Via, nome: &str, a: &Value) -> R {
    match nome {
        "listar_blueprints" => {
            let r = servidor(via, "blueprints", "GET", None)?;
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
            let filtro: String = ["incluir", "projeto"].iter().map(|k| (k, opcional(a, k))).filter(|(_, v)| !v.trim().is_empty()).map(|(k, v)| format!("&{k}={}", codificar(v.trim()))).collect();
            let rota = format!("blueprint?workspace={}&blueprint={}{acao}{filtro}", codificar(&texto(a, "workspace_id")?), codificar(&texto(a, "blueprint_id")?));
            Ok(servidor(via, &rota, "GET", None)?.to_string()) // compacto: cada byte vai para o contexto do agente
        }
        "propor_blueprint" => {
            let mut corpo = alvo(a);
            for (de, para) in [("mensagem", "mensagem"), ("blueprint", "blueprint"), ("acoes", "acoes"), ("remover", "remover"), ("mover", "mover"), ("ajustar", "ajustar"), ("substituir", "substituir")] {
                if let Some(v) = a.get(de) {
                    corpo.insert(para.into(), v.clone());
                }
            }
            if opcional(a, "blueprint_id").is_empty() && via.is_none() { // por HTTP o "diretório atual" seria o do servidor: nunca vai
                if let Ok(pasta) = std::env::current_dir() {
                    corpo.insert("pasta".into(), json!(pasta.display().to_string()));
                }
            }
            let r = servidor(via, "proposta", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Proposal saved: the app is closed. When it opens, the proposal enters the queue for the user to review and apply (ids and format are validated only then).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings (they do not block the submission, but worth fixing):\n- {}", avisos.join("\n- ")) };
            let ligar = if opcional(a, "blueprint_id").is_empty() { vinculo(&opcional(a, "workspace_id")) } else { String::new() };
            Ok(format!("Proposal no. {} is in the app's queue (one at a time; the others in the queue were not touched). The app applies it by itself when automatic application is on (the default in the local app); removals, moved actions and new blueprints still wait for the user's click on the Plan screen. Do not say it entered before ler_blueprint shows it.{aviso}\n{}\nIf the user asks for changes, send `ajustar: {}` with only what changes (the action, plan or checkpoint by the id above); `substituir: {}` replaces the whole proposal.{ligar}", r["numero"], ids_provisorios(&r["ids"]), r["numero"], r["numero"]))
        }
        "registrar_trabalho" => {
            let mut corpo = alvo(a);
            for chave in ["mensagem", "acoes"] {
                if let Some(v) = a.get(chave) {
                    corpo.insert(chave.into(), v.clone());
                }
            }
            texto(a, "blueprint_id")?; // um registro é sobre um blueprint que já existe
            let r = servidor(via, "registro", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Record saved: the app is closed. When it opens, the record enters the queue for the user to decide whether to add it to the blueprint (validation happens only then).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings:\n- {}", avisos.join("\n- ")) };
            Ok(format!("Record no. {} is in the app's queue (it is not a proposal). The app adds it by itself when automatic application is on (the default in the local app); otherwise the user sees what was done, with the checks, and decides on the Plan screen. Check with ler_blueprint before saying it was added, and tell the user it was sent.{aviso}", r["numero"]))
        }
        "vincular_pasta" => {
            // o servidor MCP roda na pasta da sessão do agente: é ela que fica ligada
            if via.is_some() {
                return Err("vincular_pasta needs to run on your own machine: over HTTP, ask the user to link the folder with the Connect my Claude command, or write the line `Blueprint: workspace_id …, blueprint_id …` in CLAUDE.local.md of the project folder.".into());
            }
            let pasta = std::env::current_dir().map_err(|e| format!("Cannot tell which folder this is: {e}"))?;
            if super::equipe::ligado() {
                // o servidor da equipe não conhece as pastas desta máquina: a ligação fica na própria pasta (pasta_local.rs)
                return super::pasta_local::vincular(&pasta, &texto(a, "workspace_id")?, &texto(a, "blueprint_id")?);
            }
            let corpo = com(alvo(a), &[("cwd", json!(pasta.display().to_string()))]);
            servidor(via, "pasta/vincular", "POST", Some(&Value::Object(corpo)))?;
            Ok(format!("Folder {} linked to the blueprint. The next sessions in it start with the blueprint and the current action.", pasta.display()))
        }
        "anexar_prototipo" => {
            let corpo = com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("titulo", json!(texto(a, "titulo")?)), ("html", json!(texto(a, "html")?)), ("prototipoId", json!(opcional(a, "prototipo_id")))]);
            let r = servidor(via, "prototipo", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Prototype saved: the app is closed. It is attached to the action when the app opens.".into());
            }
            let acao = if r["atualizado"] == true { "updated in" } else { "attached to" };
            Ok(format!("Prototype \"{}\" {acao} the action (prototipo_id: {}); the user sees it on the blueprint sheet, in the action's prototype icon. To revise it, call again with this prototipo_id.", r["titulo"].as_str().unwrap_or(""), r["prototipoId"].as_str().unwrap_or("")))
        }
        "anexar_apresentacao" => {
            let corpo = com(alvo(a), &[("conteudo", estruturado(a, "conteudo")), ("figuras", estruturado(a, "figuras")), ("apresentacaoId", json!(opcional(a, "apresentacao_id")))]);
            let r = servidor(via, "apresentacao", "POST", Some(&Value::Object(corpo)))?;
            let avisos: Vec<&str> = r["avisos"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().filter_map(Value::as_str).collect();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings (fix them and send again with this apresentacao_id):\n- {}", avisos.join("\n- ")) };
            let acao = if r["atualizado"] == true { "updated" } else { "saved" };
            Ok(format!("Presentation \"{}\" {acao} (apresentacao_id: {}). The user opens it on the Present screen of the blueprint, at {}. To revise it or regenerate only some figures, call again with this apresentacao_id.{aviso}", r["titulo"].as_str().unwrap_or(""), r["apresentacaoId"].as_str().unwrap_or(""), r["caminho"].as_str().unwrap_or("")))
        }
        "ler_apresentacao" => super::ferr_apresentacao::ler(via, a),
        "editar_apresentacao" => super::ferr_apresentacao::editar(via, a),
        "registrar_progresso" => {
            evento(via, "progresso", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Progress recorded.".into())
        }
        "marcar_concluido" => {
            evento(via, "concluir", texto(a, "evidencia")?, plano(a)?)?;
            Ok("Plan sent to the app as done, with the evidence.".into())
        }
        "marcar_nao_concluido" => {
            evento(via, "nao-concluida", texto(a, "motivo")?, plano(a)?)?;
            Ok("Recorded: the plan stays pending.".into())
        }
        "marcar_checkpoint_feito" => {
            evento(via, "checkpoint-feito", texto(a, "evidencia")?, checkpoint(a)?)?;
            Ok("Checkpoint sent to the app as reached, with the evidence.".into())
        }
        "marcar_checkpoint_nao_feito" => {
            evento(via, "checkpoint-nao-feito", texto(a, "motivo")?, checkpoint(a)?)?;
            Ok("Recorded: the checkpoint stays pending.".into())
        }
        "reportar_bloqueio" => {
            evento(via, "bloqueio", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Block recorded; the user was notified.".into())
        }
        "responder_conversa" => super::conversa::ferramenta(a, &|rota, metodo, corpo| servidor(via, rota, metodo, corpo)),
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
    tratar_via(msg, None)
}

/// `tratar` com a via das ferramentas escolhida (ver `Via`).
pub(super) fn tratar_via(msg: &Value, via: Via) -> Option<Value> {
    tratar_com(msg, via, super::perfil::atual())
}

/// `tratar_via` com o perfil de ferramentas dado (ver `perfil`): `tools/list` só lista as dele e `tools/call` recusa as outras.
pub(super) fn tratar_com(msg: &Value, via: Via, perfil: super::perfil::Perfil) -> Option<Value> {
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
            resposta(id, json!({"result": {"tools": perfil.filtrar(ferramentas)}}))
        }
        "tools/call" => {
            let nome = params["name"].as_str().unwrap_or("");
            let ferramentas: Value = serde_json::from_str(FERRAMENTAS).unwrap_or_else(|_| json!([]));
            if !ferramentas.as_array().is_some_and(|f| f.iter().any(|x| x["name"] == nome)) {
                return Some(erro(-32602, format!("Unknown tool: {nome}")));
            }
            if !perfil.expoe(nome) {
                return Some(erro(-32602, format!("Tool {nome} is not in the {} profile (BLUEPRINT_MCP_PERFIL)", perfil.nome())));
            }
            let argumentos = if params["arguments"].is_object() { params["arguments"].clone() } else { json!({}) };
            GUARDADO_OFFLINE.store(false, Ordering::Relaxed);
            match chamar(via, nome, &argumentos) {
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
    if let Err(e) = super::perfil::iniciar() {
        eprintln!("{e}");
        return 2;
    }
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
