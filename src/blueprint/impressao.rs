//! Impressão de um projeto: um hash curto do texto, das ações e das datas dele. O agente a guarda como `impressao` da figura
//! que explica o projeto (apresentações com IA) e, ao gerar de novo, só refaz as figuras cuja impressão mudou.
use super::texto::{lista, str_de};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Só o que a apresentação mostra: nomes, descrição, datas, situação e os títulos de planos e checkpoints.
/// `feito` dos planos e as instruções internas ficam de fora, para marcar um plano não refazer a figura.
pub fn impressao_do_projeto(p: &Value, projeto: &Value) -> String {
    let id = str_de(projeto, "id");
    let acoes: Vec<Value> = lista(p.get("acoes"))
        .iter()
        .filter(|a| str_de(a, "projetoId") == id)
        .map(|a| {
            let planos: Vec<String> = lista(a.get("planos")).iter().map(|x| str_de(x, "texto")).collect();
            let checkpoints: Vec<Value> = lista(a.get("checkpoints")).iter().map(|c| json!([str_de(c, "nome"), str_de(c, "data")])).collect();
            json!([str_de(a, "nome"), str_de(a, "inicio"), str_de(a, "fim"), str_de(a, "status"), planos, checkpoints])
        })
        .collect();
    let dados = json!([str_de(projeto, "nome"), str_de(projeto, "descricao"), acoes]); // listas, não objetos: a ordem não depende do mapa
    Sha256::digest(dados.to_string().as_bytes()).iter().take(8).map(|b| format!("{b:02x}")).collect()
}
