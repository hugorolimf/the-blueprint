//! `remover`: só monta o pedido de apagar (com os nomes, o motivo e avisos). Quem apaga é o usuário, ao marcar e aplicar na tela.
use super::texto::{campo, lista, str_de, truncar};
use serde_json::{json, Value};
use std::collections::HashSet;

pub fn remocoes(pedidos: &[Value], acoes: &[Value], movidas: &HashSet<String>, ids_na_proposta: &[String], erros: &mut Vec<String>, avisos: &mut Vec<String>) -> Vec<Value> {
    let mut saida = Vec::new();
    for (i, r) in pedidos.iter().take(60).enumerate() {
        let rot = crate::tr!("Removal {}", "Remoção {}", i + 1);
        let (acao_id, tid, cid) = (campo(r, "acao_id"), campo(r, "plano_id"), campo(r, "checkpoint_id"));
        let Some(acao) = acoes.iter().find(|x| str_de(x, "id") == acao_id) else {
            erros.push(crate::tr!("{rot}: acao_id \"{acao_id}\" does not exist in this blueprint (use only ids from ler_blueprint).", "{rot}: o acao_id \"{acao_id}\" não existe neste blueprint (use só ids de ler_blueprint)."));
            continue;
        };
        let nome_acao = str_de(acao, "nome");
        if !tid.is_empty() && !cid.is_empty() {
            erros.push(crate::tr!("{rot}: send plano_id or checkpoint_id, not both (with neither, the whole action is removed).", "{rot}: informe plano_id ou checkpoint_id, não os dois (sem nenhum dos dois, remove a ação inteira)."));
            continue;
        }
        let item = if !tid.is_empty() {
            lista(acao.get("planos")).iter().find(|x| str_de(x, "id") == tid)
        } else if !cid.is_empty() {
            lista(acao.get("checkpoints")).iter().find(|x| str_de(x, "id") == cid)
        } else {
            None
        };
        let alvo = if tid.is_empty() { &cid } else { &tid };
        if !alvo.is_empty() && item.is_none() {
            erros.push(crate::tr!("{rot}: id \"{alvo}\" does not exist in action \"{nome_acao}\".", "{rot}: o id \"{alvo}\" não existe na ação \"{nome_acao}\"."));
            continue;
        }
        if movidas.contains(alvo) {
            let nome_item = item.map(|x| ou_nome(x)).unwrap_or_default();
            erros.push(crate::tr!("{rot}: \"{nome_item}\" is already moved in this proposal; it cannot be removed as well.", "{rot}: \"{nome_item}\" já é movido nesta proposta; não pode ser removido também."));
            continue;
        }
        if item.is_none() && ids_na_proposta.iter().any(|id| *id == str_de(acao, "id")) {
            erros.push(crate::tr!("{rot}: action \"{nome_acao}\" is in both `acoes` and `remover`; choose one.", "{rot}: a ação \"{nome_acao}\" está em `acoes` e em `remover`; escolha uma."));
            continue;
        }
        let feito = |x: &Value| x.get("feito") == Some(&Value::Bool(true));
        if item.is_some_and(feito) || (item.is_none() && lista(acao.get("planos")).iter().any(feito)) {
            avisos.push(crate::tr!("{rot}: deletes work already marked as done in \"{nome_acao}\"; make sure that is intended.", "{rot}: apaga trabalho já marcado como feito em \"{nome_acao}\"; confirme que é isso mesmo."));
        }
        saida.push(json!({
            "acaoId": str_de(acao, "id"), "acao": nome_acao,
            "planoId": tid, "plano": if tid.is_empty() { String::new() } else { item.map(|x| str_de(x, "texto")).unwrap_or_default() },
            "checkpointId": cid, "checkpoint": if cid.is_empty() { String::new() } else { item.map(|x| str_de(x, "nome")).unwrap_or_default() },
            "motivo": truncar(&campo(r, "motivo"), 500),
        }));
    }
    saida
}

/// `item.texto || item.nome`
fn ou_nome(x: &Value) -> String {
    let t = str_de(x, "texto");
    if t.is_empty() { str_de(x, "nome") } else { t }
}
