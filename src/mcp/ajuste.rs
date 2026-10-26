//! Ajustar uma proposta que está na fila (`propor_blueprint` com `ajustar: <número>`): mescla só os campos enviados na entrada que o agente
//! mandou antes, sem apagar nada por omissão (a mesma regra das ações existentes). Quem chama revalida a proposta INTEIRA depois.
//! Os itens novos (sem `id`) ganham um id provisório, começado por "+", que vale só enquanto a proposta está na fila: "+a1" (ação),
//! "+a1.p2" (plano), "+a1.c1" (checkpoint); numa ação existente, "+<id da ação>.p1". Ficam no campo `prov` da entrada, que o validador ignora.
use super::Mcp;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Map, Value};

type R<T> = Result<T, ErroHttp>;

/// O que identifica o alvo ou o pedido de ajuste, não o conteúdo da proposta.
const META: [&str; 5] = ["workspaceId", "blueprintId", "pasta", "ajustar", "substituir"];

fn chave<'a>(x: &'a Value, k: &str) -> &'a str {
    x.get(k).and_then(Value::as_str).unwrap_or("")
}

fn numero_de(v: Option<&Value>, campo: &str) -> R<Option<i64>> {
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) if n.as_i64().is_some_and(|n| n > 0) => Ok(n.as_i64()),
        Some(Value::String(s)) if s.trim().parse::<i64>().is_ok_and(|n| n > 0) => Ok(s.trim().parse().ok()),
        Some(_) => Err(falha(400, crate::tr!("invalid {campo}: use the proposal number that `propor_blueprint` returned (an integer from 1).", "{campo} inválido: use o número da proposta que `propor_blueprint` devolveu (um inteiro a partir de 1).")).codigo("numero-invalido").params(json!({"campo": campo}))),
    }
}

/// Os campos de conteúdo do corpo (tudo menos o alvo e o pedido de ajuste).
fn conteudo(corpo: &Value) -> Map<String, Value> {
    corpo.as_object().map(|m| m.iter().filter(|(k, _)| !META.contains(&k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default()
}

/// Junta `novos` a `lista`: o item com um `id` (real ou provisório) que já está na lista recebe só os campos enviados; o resto entra no fim.
/// `aninhadas` são as listas dentro de cada item (planos e checkpoints de uma ação), mescladas do mesmo jeito.
fn mesclar_itens(lista: &mut Vec<Value>, novos: &[Value], aninhadas: &[&str]) -> R<()> {
    for novo in novos {
        let id = chave(novo, "id");
        let existente = if id.is_empty() { None } else { lista.iter_mut().find(|x| chave(x, "id") == id || chave(x, "prov") == id) };
        let Some(alvo) = existente else {
            if id.starts_with('+') {
                return Err(falha(400, crate::tr!("provisional id \"{id}\" does not exist in this proposal (use the ids returned by propor_blueprint).", "id provisório \"{id}\" não existe nesta proposta (use os ids devolvidos por propor_blueprint).")).codigo("id-provisorio").params(json!({"id": id})));
            }
            lista.push(novo.clone());
            continue;
        };
        let Some(campos) = novo.as_object().filter(|_| alvo.is_object()) else { continue };
        for (k, v) in campos.iter().filter(|(k, _)| !matches!(k.as_str(), "id" | "prov")) {
            match v {
                Value::Array(itens) if aninhadas.contains(&k.as_str()) => {
                    let destino = alvo[k.as_str()].take();
                    let mut destino = if let Value::Array(a) = destino { a } else { Vec::new() };
                    mesclar_itens(&mut destino, itens, &[])?;
                    alvo[k.as_str()] = Value::Array(destino);
                }
                _ => alvo[k.as_str()] = v.clone(),
            }
        }
    }
    Ok(())
}

fn mesclar(antiga: &Value, ajuste: &Value) -> R<Value> {
    let mut saida = antiga.as_object().cloned().unwrap_or_default();
    for (k, v) in conteudo(ajuste) {
        match (k.as_str(), &v) {
            ("acoes", Value::Array(novas)) => {
                let mut a = saida.remove("acoes").and_then(|x| if let Value::Array(a) = x { Some(a) } else { None }).unwrap_or_default();
                mesclar_itens(&mut a, novas, &["planos", "checkpoints"])?;
                saida.insert(k, Value::Array(a));
            }
            ("remover" | "mover", Value::Array(novos)) => {
                // pedidos de apagar e de mover só se acumulam; o que já estava na proposta não se repete
                let a = saida.entry(k).or_insert_with(|| json!([]));
                if let Value::Array(a) = a {
                    novos.iter().filter(|n| !a.contains(n)).cloned().collect::<Vec<_>>().into_iter().for_each(|n| a.push(n));
                }
            }
            ("blueprint", Value::Object(novos)) if saida.get("blueprint").is_some_and(Value::is_object) => {
                if let Some(Value::Object(b)) = saida.get_mut("blueprint") {
                    b.extend(novos.clone());
                }
            }
            _ => {
                saida.insert(k, v);
            }
        }
    }
    Ok(Value::Object(saida))
}

/// Dá o id provisório (`prov`) aos itens novos que ainda não têm. Os que já têm mantêm o seu, mesmo depois de outros ajustes.
fn rotular(entrada: &mut Value) {
    let Some(acoes) = entrada.get_mut("acoes").and_then(Value::as_array_mut) else { return };
    let ultimo = |prefixo: &str, ids: Vec<&str>| ids.iter().filter_map(|p| p.strip_prefix(prefixo)).filter_map(|n| n.parse::<u32>().ok()).max().unwrap_or(0);
    let mut n_acao = ultimo("+a", acoes.iter().map(|a| chave(a, "prov")).collect());
    for a in acoes.iter_mut().filter(|a| a.is_object()) {
        if chave(a, "id").is_empty() && chave(a, "prov").is_empty() {
            n_acao += 1;
            a["prov"] = json!(format!("+a{n_acao}"));
        }
        let dono = if chave(a, "id").is_empty() { chave(a, "prov").to_string() } else { format!("+{}", chave(a, "id")) };
        for (lista, letra) in [("planos", 'p'), ("checkpoints", 'c')] {
            let Some(itens) = a.get_mut(lista).and_then(Value::as_array_mut) else { continue };
            let prefixo = format!("{dono}.{letra}");
            let mut n = ultimo(&prefixo, itens.iter().map(|x| chave(x, "prov")).collect());
            for x in itens.iter_mut().filter(|x| x.is_object() && chave(x, "id").is_empty() && chave(x, "prov").is_empty()) {
                n += 1;
                x["prov"] = json!(format!("{prefixo}{n}"));
            }
        }
    }
}

/// Decide o que validar: a entrada nova, ou a entrada guardada com o ajuste mesclado. Devolve a entrada (só conteúdo, com ids provisórios)
/// e o número da proposta que ela ocupa (`None` = proposta nova, que entra no fim da fila).
pub fn preparar(mcp: &Mcp, corpo: &Value, workspace_id: &str, blueprint_id: &str) -> R<(Value, Option<i64>)> {
    let (ajustar, substituir) = (numero_de(corpo.get("ajustar"), "ajustar")?, numero_de(corpo.get("substituir"), "substituir")?);
    let mut entrada = match (ajustar, substituir) {
        (Some(_), Some(_)) => return Err(falha(400, crate::tr!("Use ajustar OR substituir, not both.", "Use ajustar OU substituir, não os dois.")).codigo("ajustar-e-substituir")),
        (None, None) => Value::Object(conteudo(corpo)),
        (Some(n), None) | (None, Some(n)) => {
            let antiga = mcp.entrada_da_proposta(workspace_id, blueprint_id, n).map_err(|pendentes| {
                let lista = pendentes.iter().map(|n| crate::tr!("no. {n}", "nº {n}")).collect::<Vec<_>>().join(", ");
                let fila = if pendentes.is_empty() {
                    crate::tr!("This blueprint's proposal queue is empty (the proposal may already have been applied or discarded); send a new proposal.", "A fila de propostas deste blueprint está vazia (a proposta pode já ter sido aplicada ou descartada); envie uma proposta nova.")
                } else {
                    crate::tr!("This blueprint's queue holds proposals {lista}; send one of them or a new proposal.", "Na fila deste blueprint há as propostas {lista}; envie uma delas ou uma proposta nova.")
                };
                falha(404, crate::tr!("There is no proposal no. {n} in the queue. {fila}", "Não existe a proposta nº {n} na fila. {fila}")).codigo("proposta-nao-encontrada").params(json!({"numero": n, "pendentes": pendentes}))
            })?;
            if ajustar.is_some() { mesclar(&antiga, corpo)? } else { Value::Object(conteudo(corpo)) }
        }
    };
    rotular(&mut entrada);
    Ok((entrada, ajustar.or(substituir)))
}

/// O corpo que o validador lê: o conteúdo da proposta mais o alvo e a pasta da chamada atual.
pub fn corpo_validavel(entrada: &Value, chamada: &Value) -> Value {
    let mut c = entrada.clone();
    for k in ["workspaceId", "blueprintId", "pasta"] {
        if let Some(v) = chamada.get(k) {
            c[k] = v.clone();
        }
    }
    c
}

/// Os ids da proposta para o agente ajustar: cada ação (id real ou provisório) e, nela, só os planos e checkpoints novos (com id provisório).
pub fn ids(entrada: &Value) -> Value {
    let curto = |x: &Value, k: &str| -> String { chave(x, k).chars().take(40).collect() };
    let itens = |a: &Value, lista: &str, nome: &str| -> Vec<Value> { a.get(lista).and_then(Value::as_array).into_iter().flatten().filter(|x| !chave(x, "prov").is_empty()).map(|x| json!({"id": chave(x, "prov"), nome: curto(x, nome)})).collect() };
    let acoes = entrada.get("acoes").and_then(Value::as_array).into_iter().flatten().filter(|a| a.is_object()).map(|a| {
        let id = if chave(a, "id").is_empty() { chave(a, "prov") } else { chave(a, "id") };
        json!({"id": id, "nome": curto(a, "nome"), "planos": itens(a, "planos", "texto"), "checkpoints": itens(a, "checkpoints", "nome")})
    });
    Value::Array(acoes.collect())
}
