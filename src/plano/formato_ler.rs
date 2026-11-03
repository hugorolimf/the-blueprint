//! A leitura do JSON para as structs de `formato`, juntando os erros: o agente corrige tudo de uma vez. Mensagens em inglês: vão ao agente.
use super::formato::*;
use serde_json::Value;

pub(super) type Erros = Vec<String>;

pub(super) fn opcional(o: &Value, campo: &str, onde: &str, erros: &mut Erros) -> Option<String> {
    match o.get(campo) {
        None | Some(Value::Null) => None,
        Some(Value::String(s)) => Some(s.clone()),
        Some(_) => {
            erros.push(format!("{onde}: {campo} must be a string"));
            None
        }
    }
}

pub(super) fn texto(o: &Value, campo: &str, onde: &str, erros: &mut Erros) -> String {
    opcional(o, campo, onde, erros).unwrap_or_default()
}

pub(super) fn itens<'a>(o: &'a Value, campo: &str, onde: &str, erros: &mut Erros) -> &'a [Value] {
    match o.get(campo) {
        None | Some(Value::Null) => &[],
        Some(Value::Array(l)) => l,
        Some(_) => {
            erros.push(format!("{onde}: {campo} must be a list"));
            &[]
        }
    }
}

pub(super) fn textos(o: &Value, campo: &str, onde: &str, erros: &mut Erros) -> Vec<String> {
    let mut saida = Vec::new();
    for item in itens(o, campo, onde, erros) {
        match item.as_str() {
            Some(s) => saida.push(s.to_string()),
            None => erros.push(format!("{onde}: every item of {campo} must be a string")),
        }
    }
    saida
}

pub(super) fn booleano(o: &Value, campo: &str, onde: &str, erros: &mut Erros) -> bool {
    match o.get(campo) {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => {
            erros.push(format!("{onde}: {campo} must be true or false"));
            false
        }
    }
}

pub(super) fn exige_objeto(v: &Value, onde: &str, erros: &mut Erros) -> bool {
    if !v.is_object() {
        erros.push(format!("{onde}: expected an object"));
    }
    v.is_object()
}

pub(super) fn resultado<T>(valor: T, erros: Erros) -> Result<T, Erros> {
    if erros.is_empty() { Ok(valor) } else { Err(erros) }
}

pub(super) fn ler_plano(acao: &str, i: usize, v: &Value, erros: &mut Erros) -> PlanoItem {
    let gerado = format!("{acao}.{}", i + 1);
    if let Some(s) = v.as_str() {
        return PlanoItem { id: gerado, texto: s.to_string(), id_gerado: true, ..PlanoItem::default() };
    }
    let onde = format!("plan {gerado}");
    if !v.is_object() {
        erros.push(format!("{onde}: expected a string or an object"));
        return PlanoItem::default();
    }
    let id = opcional(v, "id", &onde, erros);
    PlanoItem { id_gerado: id.is_none(), id: id.unwrap_or(gerado), texto: texto(v, "texto", &onde, erros), area: opcional(v, "area", &onde, erros), depende_de: textos(v, "depende_de", &onde, erros), instrucoes: opcional(v, "instrucoes", &onde, erros), forma: opcional(v, "forma", &onde, erros), entrega: opcional(v, "entrega", &onde, erros) }
}

pub(super) fn ler_checkpoint(acao: &str, i: usize, v: &Value, erros: &mut Erros) -> CheckpointDoc {
    let gerado = format!("{acao}.c{}", i + 1);
    let onde = format!("checkpoint {gerado}");
    if !exige_objeto(v, &onde, erros) {
        return CheckpointDoc::default();
    }
    CheckpointDoc { id: opcional(v, "id", &onde, erros).unwrap_or(gerado), texto: texto(v, "texto", &onde, erros), plano: opcional(v, "plano", &onde, erros) }
}

pub(super) fn ler_detalhe(v: &Value, onde: &str, erros: &mut Erros) -> Detalhe {
    if !exige_objeto(v, onde, erros) {
        return Detalhe::default();
    }
    Detalhe { id: texto(v, "id", onde, erros), titulo: opcional(v, "titulo", onde, erros), legenda: opcional(v, "legenda", onde, erros), grupo: opcional(v, "grupo", onde, erros), figura: opcional(v, "figura", onde, erros), plano: opcional(v, "plano", onde, erros), variante: opcional(v, "variante", onde, erros), figura_info: None, pede_aprovacao: booleano(v, "pede_aprovacao", onde, erros) }
}

pub(super) fn ler_tabela(v: &Value, onde: &str, erros: &mut Erros) -> Tabela {
    if !exige_objeto(v, onde, erros) {
        return Tabela::default();
    }
    let mut larguras = Vec::new();
    for l in itens(v, "larguras", onde, erros) {
        match l.as_f64() {
            Some(n) => larguras.push(n),
            None => erros.push(format!("{onde}: every item of larguras must be a number")),
        }
    }
    let mut linhas = Vec::new();
    for linha in itens(v, "linhas", onde, erros) {
        let Some(celulas) = linha.as_array() else {
            erros.push(format!("{onde}: every item of linhas must be a list"));
            continue;
        };
        let mut lida = Vec::with_capacity(celulas.len());
        for c in celulas {
            match c {
                Value::String(s) => lida.push(s.clone()),
                Value::Number(_) | Value::Bool(_) => lida.push(c.to_string()),
                _ => erros.push(format!("{onde}: every cell must be a string")),
            }
        }
        linhas.push(lida);
    }
    Tabela { plano: opcional(v, "plano", onde, erros), titulo: opcional(v, "titulo", onde, erros), colunas: textos(v, "colunas", onde, erros), larguras, linhas }
}

pub(super) fn ler_acao(v: &Value, onde: &str, erros: &mut Erros) -> AcaoDoc {
    if !exige_objeto(v, onde, erros) {
        return AcaoDoc::default();
    }
    let id = texto(v, "id", onde, erros);
    let onde = if id.is_empty() { onde.to_string() } else { format!("action {id}") };
    let planos = itens(v, "planos", &onde, erros).iter().enumerate().map(|(i, p)| ler_plano(&id, i, p, erros)).collect();
    let checkpoints = itens(v, "checkpoints", &onde, erros).iter().enumerate().map(|(i, c)| ler_checkpoint(&id, i, c, erros)).collect();
    let detalhes = itens(v, "detalhes", &onde, erros).iter().map(|d| ler_detalhe(d, &format!("{onde}, detail"), erros)).collect();
    let tabelas = itens(v, "tabelas", &onde, erros).iter().map(|t| ler_tabela(t, &format!("{onde}, table"), erros)).collect();
    AcaoDoc { nome: texto(v, "nome", &onde, erros), texto: opcional(v, "texto", &onde, erros), id, planos, checkpoints, detalhes, tabelas }
}

pub(super) fn ler_decisao(v: &Value, erros: &mut Erros) -> Decisao {
    if !exige_objeto(v, "decision", erros) {
        return Decisao::default();
    }
    let id = texto(v, "id", "decision", erros);
    let onde = format!("decision {id}");
    Decisao { rotulo: opcional(v, "rotulo", &onde, erros), texto: texto(v, "texto", &onde, erros), acoes: textos(v, "acoes", &onde, erros), plano: opcional(v, "plano", &onde, erros), pede_aprovacao: booleano(v, "pede_aprovacao", &onde, erros), id }
}

pub(super) fn ler_recomendada(v: &Value, opcoes: &[String], onde: &str, erros: &mut Erros) -> Option<String> {
    match v.get("recomendada") {
        Some(Value::Number(n)) => {
            let opcao = n.as_u64().and_then(|i| opcoes.get(i as usize)).cloned();
            if opcao.is_none() {
                erros.push(format!("{onde}: recomendada {n} is not one of the opcoes"));
            }
            opcao
        }
        _ => opcional(v, "recomendada", onde, erros),
    }
}

pub(super) fn ler_pergunta(v: &Value, erros: &mut Erros) -> Pergunta {
    if !exige_objeto(v, "question", erros) {
        return Pergunta::default();
    }
    let id = texto(v, "id", "question", erros);
    let onde = format!("question {id}");
    let opcoes = textos(v, "opcoes", &onde, erros);
    Pergunta { texto: texto(v, "texto", &onde, erros), recomendada: ler_recomendada(v, &opcoes, &onde, erros), acoes: textos(v, "acoes", &onde, erros), plano: opcional(v, "plano", &onde, erros), opcoes, id }
}
