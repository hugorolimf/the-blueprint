//! As ferramentas `ler_apresentacao` e `editar_apresentacao` (contrato em docs/interno/apresentacao-ia.md, "Agent edits"): o agente lê uma apresentação que já existe e muda SÓ o que o
//! usuário pediu, em operações pequenas, em vez de regerá-la. Tudo aqui é texto para o AGENTE, então é inglês. Valem também pelo `/mcp` HTTP do convidado (a mesma `servidor`).
use super::argumentos::{alvo, com, estruturado, texto};
use super::cliente::codificar;
use super::stdio::{servidor, Via};
use serde_json::{json, Value};

fn tamanho(v: &Value) -> usize {
    v.as_str().map_or(0, |s| s.chars().count())
}

/// `figuras_codigo` pode vir como lista ou como o JSON dela em texto (alguns clientes mandam assim).
fn pedidas(a: &Value) -> Vec<String> {
    match estruturado(a, "figuras_codigo") {
        Value::Array(l) => l.iter().filter_map(Value::as_str).map(String::from).collect(),
        _ => Vec::new(),
    }
}

pub(super) fn ler(via: Via, a: &Value) -> Result<String, String> {
    let id = texto(a, "apresentacao_id")?;
    let r = servidor(via, &format!("apresentacao?id={}", codificar(&id)), "GET", None)?;
    let (quer, figuras) = (pedidas(a), r["figuras"].as_array().map(Vec::as_slice).unwrap_or(&[]));
    let editada = match (r["editadoEm"].as_str(), r["editadoPor"].as_str()) {
        (Some(em), Some(por)) => format!(", last edited {em} by {por}"),
        (Some(em), None) => format!(", last edited {em}"),
        _ => String::new(),
    };
    let mut saida = format!("Presentation \"{}\" (apresentacao_id: {id}): audience {}, level {}, generated {}{editada}.\n\nconteudo (JSON):\n{}\n\nFigures ({}); the codigo is shown only for the ids you list in figuras_codigo (sizes in characters):\n", r["titulo"].as_str().unwrap_or(""), r["publico"].as_str().unwrap_or(""), r["nivel"].as_str().unwrap_or(""), r["em"].as_str().unwrap_or(""), r["conteudo"], figuras.len());
    for f in figuras {
        let fid = f["id"].as_str().unwrap_or("");
        saida += &format!("- id {fid:?}, secao {}, titulo {}, impressao {}, codigo {} chars\n", f["secao"], f["titulo"], f["impressao"], tamanho(&f["codigo"]));
    }
    for fid in &quer {
        match figuras.iter().find(|f| f["id"].as_str() == Some(fid)) {
            Some(f) => saida += &format!("\ncodigo of figure {fid:?}:\n{}\n", f["codigo"].as_str().unwrap_or("")),
            None => saida += &format!("\n(figure {fid:?} does not exist in this presentation)\n"),
        }
    }
    Ok(saida)
}

pub(super) fn editar(via: Via, a: &Value) -> Result<String, String> {
    let id = texto(a, "apresentacao_id")?;
    let operacoes = estruturado(a, "operacoes");
    let n = operacoes.as_array().map_or(0, Vec::len);
    let corpo = com(alvo(a), &[("apresentacaoId", json!(id)), ("operacoes", operacoes), ("autor", json!("agente"))]);
    let r = servidor(via, "apresentacao/editar", "POST", Some(&Value::Object(corpo)))?;
    let avisos: Vec<&str> = r["avisos"].as_array().map(Vec::as_slice).unwrap_or(&[]).iter().filter_map(Value::as_str).collect();
    let aviso = if avisos.is_empty() { String::new() } else { format!("\nWarnings (fix them with another editar_apresentacao):\n- {}", avisos.join("\n- ")) };
    Ok(format!("Presentation {id} edited: {n} operation(s) applied together (edited at {}). Tell the user in one sentence what changed; do not regenerate it.{aviso}", r["editadoEm"].as_str().unwrap_or("")))
}
