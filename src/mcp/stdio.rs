//! `blueprint --mcp`: o servidor MCP por stdio (JSON-RPC 2.0, uma mensagem por linha) que o agente do usuário registra
//! (`claude mcp add blueprint -- blueprint.exe --mcp`). Porte de mcp-roadmap.mjs: mesmas ferramentas, mesmas instruções e mesmos textos.
//! Não tem dados próprios: cada ferramenta chama as rotas /api/mcp do servidor local.
use super::cliente::{codificar, requisitar};
use serde_json::{json, Map, Value};
use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};

const INSTRUCOES: &str = include_str!("instrucoes.txt"); // os primeiros 512 caracteres são autossuficientes: é o que o Codex lê
const FERRAMENTAS: &str = include_str!("ferramentas.json");
const VERSOES: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const GUIA: &str = include_str!("../../skills/blueprint-planejamento/SKILL.md");
const TOPICOS: [(&str, &str); 4] = [
    ("planejar", include_str!("../../skills/blueprint-planejamento/topicos/planejar.md")),
    ("editar", include_str!("../../skills/blueprint-planejamento/topicos/editar.md")),
    ("executar", include_str!("../../skills/blueprint-planejamento/topicos/executar.md")),
    ("prototipos", include_str!("../../skills/blueprint-planejamento/topicos/prototipos.md")),
];
const REGRA: &str = "Mantenha este blueprint sempre atualizado: planos e mudanças de plano via propor_blueprint; ao começar um plano, registrar_progresso; ao terminar, marcar_concluido com a evidência; se não deu, marcar_nao_concluido; se travar, reportar_bloqueio.";

/// só ao criar blueprint: as instruções do servidor já dizem para manter a linha nas sessões seguintes
fn vinculo(w: &str) -> String {
    format!("\nAo aplicar o blueprint novo, o app liga a pasta desta sessão automaticamente. Workspace: {w}. {REGRA}")
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
        let quem = if rota == "registro" { "O registro" } else { "A proposta" }; // registro não é proposta
        return Err(format!("{quem} tem erros; corrija e envie de novo:\n- {}{}", lista("erros"), if avisos.is_empty() { String::new() } else { format!("\nAvisos:\n- {avisos}") }));
    }
    let motivo = dados.get("erro").and_then(Value::as_str).map(String::from).unwrap_or_else(|| texto.chars().take(200).collect());
    Err(format!("O The Blueprint recusou ({status}): {motivo}"))
}

/// A última ferramenta foi guardada em arquivo porque o app estava fechado (a resposta ao agente avisa; o stdio atende uma ferramenta por vez).
static GUARDADO_OFFLINE: AtomicBool = AtomicBool::new(false);

fn guardar_offline(rota: &str, corpo: &Value) -> Result<Value, String> {
    let pasta = super::pendentes::pasta_do_banco(&crate::http::Config::do_app().banco);
    let pendentes = super::pendentes::guardar(&pasta, rota, corpo).map_err(|e| format!("O The Blueprint não está aberto e não consegui guardar o registro: {e}"))?;
    GUARDADO_OFFLINE.store(true, Ordering::Relaxed);
    Ok(json!({"offline": true, "pendentes": pendentes}))
}

const AVISO_OFFLINE: &str = "\n(O The Blueprint está fechado: o registro ficou guardado e entra no app quando ele abrir, sem você precisar repetir. A validação (ids, formato) só acontece então; se algo for recusado, o arquivo fica em pendentes/recusados, ao lado do banco.)";

fn texto(a: &Value, chave: &str) -> Result<String, String> {
    match a.get(chave).and_then(Value::as_str).map(str::trim) {
        Some(t) if !t.is_empty() => Ok(t.to_string()),
        _ => Err(format!("{chave} é obrigatório.")),
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
        format!("- ação {}{}{}", item(a, "nome"), if planos.is_empty() { String::new() } else { format!("; planos: {planos}") }, if cps.is_empty() { String::new() } else { format!("; checkpoints: {cps}") })
    }).collect();
    format!("Ids provisórios (valem enquanto a proposta está na fila):\n{}", linhas.join("\n"))
}

fn chamar(nome: &str, a: &Value) -> R {
    match nome {
        "listar_blueprints" => {
            let r = servidor("blueprints", "GET", None)?;
            let linhas: Vec<String> = r["workspaces"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().map(|w| {
                let blueprints = w["blueprints"].as_array().map(Vec::as_slice).unwrap_or(&[]);
                let corpo = if blueprints.is_empty() { "  (sem blueprints)".to_string() } else { blueprints.iter().map(|p| format!("  - {} (blueprint_id: {}) · {} ação(s)", p["nome"].as_str().unwrap_or(""), p["id"].as_str().unwrap_or(""), p["acoes"])).collect::<Vec<_>>().join("\n") };
                format!("Workspace {} (workspace_id: {})\n{corpo}", w["nome"].as_str().unwrap_or(""), w["id"].as_str().unwrap_or(""))
            }).collect();
            Ok(format!("Hoje: {}\n{}", r["hoje"].as_str().unwrap_or(""), linhas.join("\n")))
        }
        "guia_de_planejamento" => {
            let t = opcional(a, "topico");
            let t = t.trim();
            if t.is_empty() {
                return Ok(sem_cabecalho(GUIA));
            }
            match TOPICOS.iter().find(|(n, _)| *n == t) {
                Some((_, conteudo)) => Ok(sem_cabecalho(conteudo)),
                None => Err(format!("Tópico desconhecido: {t}. Use um destes: {}.", TOPICOS.map(|(n, _)| n).join(", "))),
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
                return Ok("Proposta guardada: o app está fechado. Quando ele abrir, ela entra na fila para o usuário revisar e aplicar (ids e formato são validados só então).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("\nAvisos (não impedem o envio, mas vale ajustar):\n- {}", avisos.join("\n- ")) };
            let ligar = if opcional(a, "blueprint_id").is_empty() { vinculo(&opcional(a, "workspace_id")) } else { String::new() };
            Ok(format!("Proposta nº {} na fila do app, para o usuário revisar e aplicar (uma de cada vez; as outras da fila não foram mexidas).{aviso}\n{}\nSe ele pedir mudanças, envie `ajustar: {}` só com o que muda (a ação, o plano ou o checkpoint pelo id acima); `substituir: {}` troca a proposta inteira.{ligar}", r["numero"], ids_provisorios(&r["ids"]), r["numero"], r["numero"]))
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
                return Ok("Registro guardado: o app está fechado. Quando ele abrir, entra na fila para o usuário decidir se adiciona ao blueprint (a validação é feita só então).".into());
            }
            let avisos: Vec<&str> = r["avisos"].as_array().map(|l| l.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            let aviso = if avisos.is_empty() { String::new() } else { format!("
Avisos:
- {}", avisos.join("
- ")) };
            Ok(format!("Registro nº {} na fila do app (não é proposta). Nada foi salvo: o usuário vê o que foi feito, com os checks, e decide se adiciona ao blueprint. Avise o usuário agora e pergunte se quer adicioná-lo.{aviso}", r["numero"]))
        }
        "vincular_pasta" => {
            // o servidor MCP roda na pasta da sessão do agente: é ela que fica ligada
            let pasta = std::env::current_dir().map_err(|e| format!("Não sei em que pasta estou: {e}"))?;
            let corpo = com(alvo(a), &[("cwd", json!(pasta.display().to_string()))]);
            servidor("pasta/vincular", "POST", Some(&Value::Object(corpo)))?;
            Ok(format!("Pasta {} ligada ao blueprint. As próximas sessões nela já começam com o blueprint e a ação atual.", pasta.display()))
        }
        "anexar_prototipo" => {
            let corpo = com(alvo(a), &[("acaoId", json!(texto(a, "acao_id")?)), ("titulo", json!(texto(a, "titulo")?)), ("html", json!(texto(a, "html")?)), ("prototipoId", json!(opcional(a, "prototipo_id")))]);
            let r = servidor("prototipo", "POST", Some(&Value::Object(corpo)))?;
            if r["offline"] == true {
                return Ok("Protótipo guardado: o app está fechado. Ele é anexado à ação quando o app abrir.".into());
            }
            let acao = if r["atualizado"] == true { "atualizado" } else { "anexado" };
            Ok(format!("Protótipo \"{}\" {acao} à ação (prototipo_id: {}); o usuário o vê na folha do blueprint, no ícone de protótipo da ação. Para revisar, chame de novo com este prototipo_id.", r["titulo"].as_str().unwrap_or(""), r["prototipoId"].as_str().unwrap_or("")))
        }
        "registrar_progresso" => {
            evento("progresso", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Progresso registrado.".into())
        }
        "marcar_concluido" => {
            evento("concluir", texto(a, "evidencia")?, plano(a)?)?;
            Ok("Plano enviado ao app como concluído, com a evidência.".into())
        }
        "marcar_nao_concluido" => {
            evento("nao-concluida", texto(a, "motivo")?, plano(a)?)?;
            Ok("Registrado: o plano continua pendente.".into())
        }
        "marcar_checkpoint_feito" => {
            evento("checkpoint-feito", texto(a, "evidencia")?, checkpoint(a)?)?;
            Ok("Checkpoint enviado ao app como atingido, com a evidência.".into())
        }
        "marcar_checkpoint_nao_feito" => {
            evento("checkpoint-nao-feito", texto(a, "motivo")?, checkpoint(a)?)?;
            Ok("Registrado: o checkpoint continua pendente.".into())
        }
        "reportar_bloqueio" => {
            evento("bloqueio", texto(a, "texto")?, com_atividade(plano_se_houver(a)?, a))?;
            Ok("Bloqueio registrado; o usuário foi avisado.".into())
        }
        _ => Err(format!("Ferramenta desconhecida: {nome}")),
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
                return Some(erro(-32602, format!("Ferramenta desconhecida: {nome}")));
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
        outro => erro(-32601, format!("Método não suportado: {outro}")),
    })
}

/// Lê mensagens de stdin (uma por linha) até o agente fechar a conversa.
pub fn executar() -> i32 {
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
                    std::thread::spawn(|| crate::watch::observar(|texto| {
                        let mut saida = std::io::stdout().lock();
                        writeln!(saida, "{}", notificacao_canal(texto))?;
                        saida.flush()
                    }));
                }
                tratar(&msg)
            },
            Err(_) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "JSON inválido"}})),
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
