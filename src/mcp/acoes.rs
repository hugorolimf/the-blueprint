//! O que o agente pode fazer pelo MCP (porte das funções de mcp.mjs): listar, ler, propor, avisar o andamento e anexar protótipo.
//! Lê os dados (`dados` é o data.json inteiro) e põe itens na fila; nada aqui altera o banco.
use super::Mcp;
use crate::http::pedido::{falha, ErroHttp};
use crate::blueprint::{contexto_acao, estrutura_lida, mover_acoes, resumo_blueprint, validar_proposta};
use crate::tempo::hoje_local;
use serde_json::{json, Map, Value};
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

type R<T> = Result<T, ErroHttp>;

pub const TIPOS_EVENTO: [&str; 6] = ["progresso", "concluir", "nao-concluida", "bloqueio", "checkpoint-feito", "checkpoint-nao-feito"];
/// Um HTML autocontido (SVG, estilos e script inline) cabe com folga.
pub const PROTOTIPO_MAX: usize = 2 * 1024 * 1024;

pub fn id_prototipo_valido(id: &str) -> bool {
    id.len() == 16 && id.bytes().all(|c| matches!(c, b'0'..=b'9' | b'a'..=b'f'))
}

/// `String(v || "")` do JS, para ids vindos do corpo.
pub(super) fn texto_js(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) if n.as_f64() != Some(0.0) => n.to_string(),
        _ => String::new(),
    }
}

fn str_de(v: &Value, chave: &str) -> String {
    v.get(chave).and_then(Value::as_str).unwrap_or("").to_string()
}

fn lista<'a>(v: Option<&'a Value>) -> &'a [Value] {
    match v {
        Some(Value::Array(a)) => a,
        _ => &[],
    }
}

fn verdade(v: Option<&Value>) -> bool {
    crate::blueprint::verdade(v)
}

/// `corta` do mcp.mjs: passou de `n`, vira os primeiros n-1 caracteres e uma reticência.
fn corta(t: &str, n: usize) -> String {
    if t.chars().count() > n { t.chars().take(n - 1).chain(std::iter::once('…')).collect() } else { t.to_string() }
}

pub(super) struct Workspace<'a> {
    id: String,
    pub(super) nome: String,
    blueprints: &'a [Value],
}

fn ou(a: String, b: String) -> String {
    if a.is_empty() { b } else { a }
}

/// "sm" é o próprio arquivo (o workspace raiz, com o nome em `empresa`); os outros são os workspaces.
fn workspace_dos<'a>(dados: &'a Value, id: &str) -> Option<Workspace<'a>> {
    if id == "sm" {
        let nome = ou(ou(str_de(dados, "nome"), str_de(dados, "empresa")), "Meu workspace".into());
        return Some(Workspace { id: "sm".into(), nome, blueprints: lista(dados.get("blueprints")) });
    }
    let c = lista(dados.get("workspaces")).iter().find(|c| c.get("id").and_then(Value::as_str) == Some(id))?;
    Some(Workspace { id: id.into(), nome: ou(ou(str_de(c, "nome"), str_de(c, "empresa")), "Workspace".into()), blueprints: lista(c.get("blueprints")) })
}

fn workspaces_de(dados: &Value) -> Vec<Workspace<'_>> {
    let workspaces = lista(dados.get("workspaces")).iter().filter_map(|c| c.get("id").and_then(Value::as_str)).filter_map(|id| workspace_dos(dados, id));
    workspace_dos(dados, "sm").into_iter().chain(workspaces).collect()
}

/// `workspace_id` é obrigatório; `blueprint_id` vazio = blueprint que ainda não existe.
pub(super) fn localizar<'a>(dados: &'a Value, workspace_id: &str, blueprint_id: &str) -> R<(Workspace<'a>, Option<&'a Value>)> {
    let ws = workspace_dos(dados, workspace_id).ok_or_else(|| falha(404, crate::tr!("Workspace not found (see listar_blueprints).", "Workspace não encontrado (veja listar_blueprints).")).codigo("workspace-nao-encontrado"))?;
    if blueprint_id.is_empty() {
        return Ok((ws, None));
    }
    let blueprint = ws.blueprints.iter().find(|p| p.get("id").and_then(Value::as_str) == Some(blueprint_id)).ok_or_else(|| falha(404, crate::tr!("Blueprint not found in this workspace (see listar_blueprints).", "Blueprint não encontrado neste workspace (veja listar_blueprints).")).codigo("blueprint-nao-encontrado"))?;
    Ok((ws, Some(blueprint)))
}

/// O número do blueprint na folha (a posição dele no workspace, a partir de 1): o "2" de "ação 2.5". 0 se não achar.
pub fn numero_do_blueprint(dados: &Value, workspace_id: &str, blueprint_id: &str) -> usize {
    workspace_dos(dados, workspace_id).and_then(|w| w.blueprints.iter().position(|p| p.get("id").and_then(Value::as_str) == Some(blueprint_id))).map_or(0, |i| i + 1)
}

pub fn blueprints(dados: &Value) -> Value {
    let ws: Vec<Value> = workspaces_de(dados).iter().map(|w| json!({"id": w.id, "nome": w.nome, "blueprints": w.blueprints.iter().map(resumo_blueprint).collect::<Vec<_>>()})).collect();
    json!({"hoje": hoje_local(), "workspaces": ws})
}

/// Sem `acao_id`: só a estrutura COMPLETA (quem usa por dentro, como os hooks, precisa de todas as ações); com `acao_id`: aquela ação inteira.
pub fn blueprint(dados: &Value, workspace_id: &str, blueprint_id: &str, acao_id: Option<&str>) -> R<Value> {
    blueprint_lido(dados, workspace_id, blueprint_id, acao_id, "tudo", "")
}

/// O que o agente lê: como `blueprint`, mas a estrutura vem resumida em blueprint grande (`incluir` "ativas") e pode ser filtrada por `projeto` (ver `enxuta`).
pub fn blueprint_lido(dados: &Value, workspace_id: &str, blueprint_id: &str, acao_id: Option<&str>, incluir: &str, projeto: &str) -> R<Value> {
    let (ws, p) = localizar(dados, workspace_id, blueprint_id)?;
    let p = p.ok_or_else(|| falha(400, crate::tr!("Send blueprint_id.", "Informe o blueprint_id.")).codigo("falta-blueprint"))?;
    if let Some(acao_id) = acao_id.filter(|f| !f.is_empty()) {
        let acao = contexto_acao(p, acao_id).ok_or_else(|| falha(404, crate::tr!("Action not found in this blueprint (see ler_blueprint without acao_id).", "Ação não encontrada neste blueprint (veja ler_blueprint sem acao_id).")).codigo("acao-nao-encontrada"))?;
        return Ok(json!({"hoje": hoje_local(), "workspaceId": ws.id, "blueprintId": str_de(p, "id"), "acao": acao}));
    }
    let outros: Vec<Value> = ws.blueprints.iter().filter(|x| x.get("id") != p.get("id")).map(|x| json!({"id": x.get("id"), "nome": x.get("nome")})).collect();
    Ok(json!({"hoje": hoje_local(), "workspaceId": ws.id, "blueprint": estrutura_lida(p, incluir, projeto).map_err(|m| falha(400, m).codigo("parametro-invalido"))?, "outros": outros}))
}

/// Valida e guarda; o usuário revisa e aplica na página. Cada alvo tem uma fila de propostas numeradas: a nova entra no fim e não apaga outra.
/// Com `ajustar: <número>` só o que vier muda naquela proposta; com `substituir: <número>` ela é trocada por inteiro (ver `ajuste`).
pub fn proposta(mcp: &Mcp, dados: &Value, chamada: &Value) -> R<(u16, Value)> {
    let (workspace_id, blueprint_id) = (texto_js(chamada.get("workspaceId")), texto_js(chamada.get("blueprintId")));
    let (ws, p) = localizar(dados, &workspace_id, &blueprint_id)?;
    let (entrada, numero) = super::ajuste::preparar(mcp, chamada, &workspace_id, &blueprint_id)?;
    let corpo = &super::ajuste::corpo_validavel(&entrada, chamada);
    let mut r = validar_proposta(corpo, p, &hoje_local());
    let apagando: Vec<String> = lista(corpo.get("remover")).iter().filter(|x| str_de(x, "plano_id").is_empty() && str_de(x, "checkpoint_id").is_empty()).map(|x| str_de(x, "acao_id")).collect();
    let mover = mover_acoes(lista(corpo.get("mover")), p, ws.blueprints, &apagando, &mut r.erros, &mut r.avisos);
    if !r.erros.is_empty() {
        return Ok((422, json!({"erros": r.erros, "avisos": r.avisos})));
    }
    if let Some(Value::Object(m)) = r.proposta.as_mut() {
        m.insert("mover".into(), Value::Array(mover));
    }
    let mut item = Map::new();
    for (k, v) in [("tipo", json!("proposta")), ("workspaceId", json!(workspace_id)), ("blueprintId", json!(blueprint_id)), ("proposta", r.proposta.unwrap_or(Value::Null)), ("avisos", json!(r.avisos))] {
        item.insert(k.into(), v);
    }
    if blueprint_id.is_empty() {
        if let Some(pasta) = corpo["pasta"].as_str().filter(|p| super::hooks::caminho_valido(p)) {
            item.insert("pasta".into(), json!(pasta));
        }
    }
    let ids = super::ajuste::ids(&entrada);
    let guardada = mcp.empurra_proposta(item, entrada, numero);
    Ok((200, json!({"ok": true, "avisos": r.avisos, "numero": guardada["numero"], "ids": ids})))
}

fn por_id<'a>(itens: &'a [Value], id: &Value) -> Option<&'a Value> {
    itens.iter().find(|x| x.get("id") == Some(id))
}

pub const PAPEIS: [&str; 3] = ["explorador", "construtor", "revisor"];
pub const ESTADOS_AGENTE: [&str; 6] = ["lendo", "escrevendo", "testando", "aguardando", "bloqueado", "concluido"];

/// texto de uma linha só: não vazio, até `max` caracteres e sem caracteres de controle
fn limpo(s: &str, max: usize) -> bool {
    !s.is_empty() && s.chars().count() <= max && !s.chars().any(char::is_control)
}

/// Os campos opcionais de atividade de registrar_progresso e reportar_bloqueio: agente (nome estável), papel, estado, onde e percentual.
/// Ausentes valem como antes (só o texto); presentes e fora do contrato são recusados. Devolve o nome do agente e os demais campos já limpos.
fn declaracao(corpo: &Value) -> R<(Option<String>, Map<String, Value>)> {
    let mut extras = Map::new();
    let agente = match corpo.get("agente") {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) if s.trim().is_empty() => None,
        Some(Value::String(s)) if limpo(s.trim(), 40) && !s.chars().any(char::is_control) => Some(s.trim().to_string()),
        Some(_) => return Err(falha(400, crate::tr!("invalid agente: use a name of up to 40 characters, without control characters", "agente inválido: use um nome de até 40 caracteres, sem caracteres de controle")).codigo("agente-invalido")),
    };
    for (campo, permitidos, dica) in [("papel", &PAPEIS[..], crate::tr!("explorador, construtor or revisor", "explorador, construtor ou revisor")), ("estado", &ESTADOS_AGENTE[..], crate::tr!("lendo, escrevendo, testando, aguardando, bloqueado or concluido", "lendo, escrevendo, testando, aguardando, bloqueado ou concluido"))] {
        match corpo.get(campo) {
            None | Some(Value::Null) => {}
            Some(Value::String(s)) if permitidos.contains(&s.as_str()) => {
                extras.insert(campo.into(), json!(s));
            }
            Some(_) => return Err(falha(400, crate::tr!("invalid {campo}: use {dica}", "{campo} inválido: use {dica}")).codigo("campo-invalido").params(json!({"campo": campo}))),
        }
    }
    let onde_ruim = || falha(400, crate::tr!("invalid onde: up to 5 paths of up to 120 characters, without control characters", "onde inválido: até 5 caminhos de até 120 caracteres, sem caracteres de controle")).codigo("onde-invalido");
    match corpo.get("onde") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if limpo(s, 120) => {
            extras.insert("onde".into(), json!([s]));
        }
        Some(Value::Array(a)) if a.len() <= 5 && a.iter().all(|x| x.as_str().is_some_and(|s| limpo(s, 120))) => {
            extras.insert("onde".into(), json!(a));
        }
        Some(_) => return Err(onde_ruim()),
    }
    match corpo.get("percentual") {
        None | Some(Value::Null) => {}
        Some(Value::Number(n)) if n.as_f64().is_some_and(|p| (0.0..=100.0).contains(&p)) => {
            extras.insert("percentual".into(), json!(n.as_f64().unwrap_or(0.0).round() as i64));
        }
        Some(_) => return Err(falha(400, crate::tr!("invalid percentual: a number from 0 to 100", "percentual inválido: um número de 0 a 100")).codigo("percentual-invalido")),
    }
    Ok((agente, extras))
}

pub fn evento(mcp: &Mcp, dados: &Value, corpo: &Value) -> R<Value> {
    let tipo = corpo.get("tipo").and_then(Value::as_str).filter(|t| TIPOS_EVENTO.contains(t)).ok_or_else(|| falha(400, crate::tr!("invalid event type", "tipo de evento inválido")).codigo("tipo-invalido"))?;
    let texto = corta(&texto_js(corpo.get("texto")), 2000).trim().to_string();
    if texto.is_empty() {
        return Err(falha(400, crate::tr!("texto is required", "texto é obrigatório")).codigo("falta-texto"));
    }
    let (workspace_id, blueprint_id) = (texto_js(corpo.get("workspaceId")), texto_js(corpo.get("blueprintId")));
    let mut item = Map::new();
    for (k, v) in [("tipo", json!(tipo)), ("texto", json!(texto)), ("workspaceId", json!(workspace_id)), ("blueprintId", json!(blueprint_id))] {
        item.insert(k.into(), v);
    }
    let marca = tipo == "concluir" || tipo == "nao-concluida";
    let declarado = if tipo == "progresso" || tipo == "bloqueio" { Some(declaracao(corpo)?) } else { None };
    let acao_de = |p: Option<&'_ Value>| -> Option<Value> {
        let id = corpo.get("acaoId")?;
        por_id(lista(p?.get("acoes")), id).cloned()
    };
    if tipo.starts_with("checkpoint") {
        // checkpoint da ação: feito só com evidência, e só se ainda estiver pendente
        let (_, p) = localizar(dados, &workspace_id, &blueprint_id)?;
        let acao = acao_de(p);
        let checkpoint = acao.as_ref().zip(corpo.get("checkpointId")).and_then(|(f, id)| por_id(lista(f.get("checkpoints")), id));
        let (Some(acao), Some(checkpoint)) = (acao.as_ref(), checkpoint) else { return Err(falha(404, crate::tr!("Action or checkpoint not found (see ler_blueprint).", "Ação ou checkpoint não encontrado (veja ler_blueprint).")).codigo("checkpoint-nao-encontrado")) };
        if verdade(checkpoint.get("feito")) {
            return Err(falha(409, crate::tr!("This checkpoint is already done.", "Este checkpoint já está concluído.")).codigo("checkpoint-ja-feito"));
        }
        for (k, v) in [("acaoId", acao["id"].clone()), ("checkpointId", checkpoint["id"].clone()), ("checkpoint", json!(str_de(checkpoint, "nome")))] {
            item.insert(k.into(), v);
        }
    } else if marca || verdade(corpo.get("planoId")) {
        // progresso e bloqueio podem dizer em que plano o agente está (o robô da folha vai até ela)
        let (_, p) = localizar(dados, &workspace_id, &blueprint_id)?;
        let acao = acao_de(p);
        let plano = acao.as_ref().zip(corpo.get("planoId")).and_then(|(f, id)| por_id(lista(f.get("planos")), id));
        let (Some(acao), Some(plano)) = (acao.as_ref(), plano) else { return Err(falha(404, crate::tr!("Action or plan not found (see ler_blueprint).", "Ação ou plano não encontrada (veja ler_blueprint).")).codigo("plano-nao-encontrado")) };
        if marca && verdade(plano.get("feito")) {
            return Err(falha(409, crate::tr!("This plan is already done.", "Este plano já está concluído.")).codigo("plano-ja-feito"));
        }
        for (k, v) in [("acaoId", acao["id"].clone()), ("planoId", plano["id"].clone()), ("plano", json!(str_de(plano, "texto")))] {
            item.insert(k.into(), v);
        }
    }
    if let Some((agente, extras)) = declarado {
        // o item da fila leva os campos (a página os usa no robô) e a presença do agente fica no registro de atividade
        item.extend(extras.clone());
        if let Some(a) = &agente {
            item.insert("agente".into(), json!(a));
        }
        let mut atividade = extras;
        if tipo == "bloqueio" {
            atividade.entry("estado").or_insert(json!("bloqueado"));
        }
        for c in ["workspaceId", "blueprintId", "acaoId", "planoId", "plano"] {
            if let Some(v) = item.get(c) {
                atividade.insert(c.into(), v.clone());
            }
        }
        atividade.insert("tipo".into(), json!(tipo));
        atividade.insert("mensagem".into(), item["texto"].clone());
        mcp.registrar_atividade(agente.as_deref().unwrap_or("agente"), atividade);
    } else if tipo == "concluir" {
        mcp.concluir_plano(&blueprint_id, item.get("planoId").and_then(Value::as_str).unwrap_or(""), &item["texto"]);
    }
    Ok(mcp.empurra(item))
}

/// 16 hexadecimais aleatórios (o `randomBytes(8)` do Node). Não é segredo: só evita colisão de nome de arquivo.
fn novo_id_prototipo() -> String {
    let (a, b) = (RandomState::new().build_hasher().finish(), RandomState::new().build_hasher().finish());
    format!("{:016x}", a ^ b.rotate_left(32) ^ u64::from(std::process::id()))
}

/// Protótipo de uma ação: HTML autocontido (por exemplo, feito com a skill anatomy). Fica em arquivo; a página guarda só a referência.
pub fn prototipo(mcp: &Mcp, dados: &Value, corpo: &Value) -> R<Value> {
    let (workspace_id, blueprint_id) = (texto_js(corpo.get("workspaceId")), texto_js(corpo.get("blueprintId")));
    let (_, p) = localizar(dados, &workspace_id, &blueprint_id)?;
    let acao = corpo.get("acaoId").zip(p).and_then(|(id, p)| por_id(lista(p.get("acoes")), id)).ok_or_else(|| falha(404, crate::tr!("Action not found (see ler_blueprint).", "Ação não encontrada (veja ler_blueprint).")).codigo("acao-nao-encontrada"))?;
    let titulo = corta(&texto_js(corpo.get("titulo")), 120).trim().to_string();
    let html = corpo.get("html").and_then(Value::as_str).unwrap_or("");
    if titulo.is_empty() {
        return Err(falha(400, crate::tr!("titulo is required", "titulo é obrigatório")).codigo("falta-titulo"));
    }
    let tem_tag = html.as_bytes().windows(2).any(|w| w[0] == b'<' && (w[1].is_ascii_alphabetic() || w[1] == b'!'));
    if !tem_tag {
        return Err(falha(400, crate::tr!("html is required: a self-contained HTML document", "html é obrigatório: um documento HTML autocontido")).codigo("falta-html"));
    }
    if html.len() > PROTOTIPO_MAX {
        let mb = PROTOTIPO_MAX / 1024 / 1024;
        return Err(falha(413, crate::tr!("the prototype is over {mb} MB; make it smaller (no large images or inline libraries)", "o protótipo passa de {mb} MB; reduza (sem imagens grandes nem bibliotecas inline)")).codigo("prototipo-grande").params(json!({"mb": mb})));
    }
    let atualiza = texto_js(corpo.get("prototipoId"));
    if !atualiza.is_empty() && !lista(acao.get("prototipos")).iter().any(|x| x.get("id").and_then(Value::as_str) == Some(atualiza.as_str())) {
        return Err(falha(404, crate::tr!("prototipo_id does not exist in this action (see ler_blueprint)", "prototipo_id não existe nesta ação (veja ler_blueprint)")).codigo("prototipo-nao-encontrado"));
    }
    let prototipo_id = if atualiza.is_empty() { novo_id_prototipo() } else { atualiza.clone() };
    if !id_prototipo_valido(&prototipo_id) {
        return Err(falha(400, crate::tr!("invalid prototipo_id", "prototipo_id inválido")).codigo("prototipo-invalido"));
    }
    std::fs::create_dir_all(&mcp.prototipos).and_then(|_| std::fs::write(mcp.prototipos.join(format!("{prototipo_id}.html")), html)).map_err(|e| falha(500, crate::tr!("could not save the prototype: {e}", "não consegui gravar o protótipo: {e}")))?;
    let mut item = Map::new();
    for (k, v) in [
        ("tipo", json!("prototipo")), ("workspaceId", json!(workspace_id)), ("blueprintId", json!(blueprint_id)), ("acaoId", acao["id"].clone()),
        ("prototipoId", json!(prototipo_id)), ("titulo", json!(titulo)), ("atualizado", json!(!atualiza.is_empty())), ("plano", json!("")), ("texto", json!(titulo)),
    ] {
        item.insert(k.into(), v);
    }
    Ok(mcp.empurra(item))
}
