//! `mover`: pedido de levar uma ação inteira de OUTRO blueprint do workspace para o blueprint da proposta. Só valida e descreve;
//! quem move (com planos, checkpoints, "feito" e protótipos) é a página, ao aplicar, e o usuário pode desmarcar.
use super::texto::{campo, lista, str_de, truncar};
use serde_json::{json, Value};

pub fn mover_acoes(pedidos: &[Value], atual: Option<&Value>, blueprints: &[Value], ids_na_proposta: &[String], erros: &mut Vec<String>, avisos: &mut Vec<String>) -> Vec<Value> {
    let mut saida: Vec<Value> = Vec::new();
    if !pedidos.is_empty() && atual.is_none() {
        avisos.push("`mover` é ignorado em blueprint novo (crie o blueprint e proponha o movimento depois).".into());
        return saida;
    }
    let atual_id = atual.map(|p| str_de(p, "id")).unwrap_or_default();
    for (i, r) in pedidos.iter().take(30).enumerate() {
        let (rot, acao_id) = (format!("Movimento {}", i + 1), campo(r, "acao_id"));
        let origem = blueprints.iter().filter(|b| str_de(b, "id") != atual_id).find_map(|b| lista(b.get("acoes")).iter().find(|a| str_de(a, "id") == acao_id).map(|a| (b, a)));
        let Some((bp, acao)) = origem else {
            let aqui = atual.is_some_and(|p| lista(p.get("acoes")).iter().any(|a| str_de(a, "id") == acao_id));
            erros.push(if aqui { format!("{rot}: a ação \"{acao_id}\" já está neste blueprint.") } else { format!("{rot}: o acao_id \"{acao_id}\" não existe em outro blueprint deste workspace (use só ids de ler_blueprint).") });
            continue;
        };
        if saida.iter().any(|s| s["acaoId"] == acao_id.as_str()) {
            erros.push(format!("{rot}: a ação \"{}\" aparece duas vezes em `mover`.", str_de(acao, "nome")));
            continue;
        }
        if ids_na_proposta.contains(&acao_id) {
            erros.push(format!("{rot}: a ação \"{}\" está em `mover` e em `remover`; escolha uma.", str_de(acao, "nome")));
            continue;
        }
        let feitos = lista(acao.get("planos")).iter().filter(|t| t.get("feito") == Some(&Value::Bool(true))).count();
        saida.push(json!({
            "acaoId": acao_id, "acao": str_de(acao, "nome"), "deBlueprintId": str_de(bp, "id"), "deBlueprint": str_de(bp, "nome"),
            "planos": lista(acao.get("planos")).len(), "feitos": feitos, "checkpoints": lista(acao.get("checkpoints")).len(),
            "motivo": truncar(&campo(r, "motivo"), 500),
        }));
    }
    saida
}
