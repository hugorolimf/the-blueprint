//! O que o papel lê do data.json (docs/colaboracao.md, seção 3): só os workspaces e blueprints que o contexto alcança; o visitante não lê
//! os comentários do agente, `chats` nem `usuario`. O app local e o dono de todos leem tudo, sem cópia.
use crate::contexto::{Acao, Contexto};
use serde_json::{Map, Value};
use std::borrow::Cow;

/// Lê tudo? (sem cópia nem filtro)
pub fn le_tudo(ctx: &Contexto) -> bool {
    ctx.local || (ctx.pode(Acao::Administrar, "*", None) && !ctx.visitante)
}

fn blueprints_visiveis(ctx: &Contexto, ws: &str, lista: Option<&Value>) -> Vec<Value> {
    let lista = lista.and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    lista.iter().filter(|b| ctx.pode(Acao::Ler, ws, b.get("id").and_then(Value::as_str))).cloned().collect()
}

/// O data.json como `ctx` pode vê-lo. O workspace raiz (`sm`) é o próprio arquivo: sem acesso a ele, a raiz fica só com o nome e sem blueprints.
pub fn filtrar<'a>(dados: &'a Value, ctx: &Contexto) -> Cow<'a, Value> {
    if le_tudo(ctx) {
        return Cow::Borrowed(dados);
    }
    let mut raiz = Map::new();
    let le_raiz = ctx.pode(Acao::Ler, "sm", None);
    for (k, v) in dados.as_object().into_iter().flatten() {
        match k.as_str() {
            "blueprints" | "workspaces" => {}
            "chats" | "usuario" if ctx.visitante => {}
            // da raiz sem acesso a ela, só o que dá nome ao workspace
            _ if !le_raiz && !matches!(k.as_str(), "empresa" | "nome" | "titulo") => {}
            _ => {
                raiz.insert(k.clone(), v.clone());
            }
        }
    }
    raiz.insert("blueprints".into(), Value::Array(blueprints_visiveis(ctx, "sm", dados.get("blueprints"))));
    let mut workspaces = Vec::new();
    for w in dados.get("workspaces").and_then(Value::as_array).into_iter().flatten() {
        let id = w.get("id").and_then(Value::as_str).unwrap_or("");
        let visiveis = blueprints_visiveis(ctx, id, w.get("blueprints"));
        if visiveis.is_empty() && !ctx.pode(Acao::Ler, id, None) {
            continue;
        }
        let mut copia = w.clone();
        copia["blueprints"] = Value::Array(visiveis);
        workspaces.push(copia);
    }
    if dados.get("workspaces").is_some() {
        raiz.insert("workspaces".into(), Value::Array(workspaces));
    }
    let mut saida = Value::Object(raiz);
    if ctx.visitante {
        para_visitante(&mut saida);
    }
    Cow::Owned(saida)
}

fn lista<'a>(v: &'a Value, chave: &str) -> &'a [Value] {
    v.get(chave).and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn texto<'a>(v: &'a Value, chave: &str) -> &'a str {
    v.get(chave).and_then(Value::as_str).unwrap_or("")
}

/// Os blueprints do workspace (`sm` é o próprio arquivo); None = o workspace não existe.
pub fn blueprints_do<'a>(dados: &'a Value, ws: &str) -> Option<&'a [Value]> {
    if ws == "sm" {
        return Some(lista(dados, "blueprints"));
    }
    lista(dados, "workspaces").iter().find(|w| texto(w, "id") == ws).map(|w| lista(w, "blueprints"))
}

pub fn nome_do_workspace(dados: &Value, ws: &str) -> Option<String> {
    let w = if ws == "sm" { dados } else { lista(dados, "workspaces").iter().find(|w| texto(w, "id") == ws)? };
    Some([texto(w, "nome"), texto(w, "empresa")].into_iter().find(|n| !n.is_empty()).unwrap_or("").to_string())
}

pub fn nome_do_blueprint(dados: &Value, ws: &str, bp: &str) -> Option<String> {
    blueprints_do(dados, ws)?.iter().find(|b| texto(b, "id") == bp).map(|b| texto(b, "nome").to_string())
}

/// Em que (workspace, blueprint) está o protótipo `id` (lista `prototipos` de alguma ação).
pub fn blueprint_do_prototipo(dados: &Value, id: &str) -> Option<(String, String)> {
    let workspaces = std::iter::once(("sm", lista(dados, "blueprints"))).chain(lista(dados, "workspaces").iter().map(|w| (texto(w, "id"), lista(w, "blueprints"))));
    for (ws, blueprints) in workspaces {
        for b in blueprints {
            if lista(b, "acoes").iter().flat_map(|a| lista(a, "prototipos")).any(|p| texto(p, "id") == id) {
                return Some((ws.to_string(), texto(b, "id").to_string()));
            }
        }
    }
    None
}

/// Tira o que o visitante não lê de qualquer pedaço da árvore (blueprint, ação, plano ou o arquivo): os comentários do agente.
/// A SINCRONIA usa nos eventos que vão a um visitante.
pub fn para_visitante(v: &mut Value) {
    match v {
        Value::Object(m) => {
            if let Some(Value::Object(agente)) = m.get_mut("agente") {
                agente.remove("comentarios");
            }
            m.remove("chats");
            m.values_mut().for_each(para_visitante);
        }
        Value::Array(a) => a.iter_mut().for_each(para_visitante),
        _ => {}
    }
}
