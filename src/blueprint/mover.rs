//! Mover: um id de plano ou checkpoint que está em OUTRA ação do blueprint vai para a ação onde foi citado (mantém o id e o "feito").
use super::texto::{campo, lista, str_de};
use serde_json::Value;
use std::collections::HashSet;

/// O item com este id numa ação diferente de `acao_id`, copiado e marcado com `de` = a ação de origem.
pub fn noutra(acoes: &[Value], lista_nome: &str, id: &str, acao_id: Option<&str>) -> Option<Value> {
    for o in acoes {
        if Some(str_de(o, "id").as_str()) == acao_id {
            continue;
        }
        if let Some(x) = lista(o.get(lista_nome)).iter().find(|y| y.get("id").and_then(Value::as_str) == Some(id)) {
            let mut copia = x.clone();
            if let Some(m) = copia.as_object_mut() {
                m.insert("de".into(), Value::String(str_de(o, "id")));
            }
            return Some(copia);
        }
    }
    None
}

/// Ids citados numa ação que pertencem a outra ação: são movimentos. A ação de origem não os "mantém" de volta.
pub fn movidas(acoes: &[Value], enviadas: &[Value]) -> HashSet<String> {
    let mut saida = HashSet::new();
    for f in enviadas {
        for (lista_nome, campo_envio) in [("planos", "planos"), ("checkpoints", "checkpoints")] {
            for t in lista(f.get(campo_envio)) {
                let (id, fid) = (campo(t, "id"), campo(f, "id"));
                let na_propria = acoes.iter().find(|x| str_de(x, "id") == fid).is_some_and(|x| lista(x.get(lista_nome)).iter().any(|y| str_de(y, "id") == id));
                if !id.is_empty() && !na_propria && noutra(acoes, lista_nome, &id, Some(&fid)).is_some() {
                    saida.insert(id);
                }
            }
        }
    }
    saida
}
