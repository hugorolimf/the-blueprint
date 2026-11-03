//! O número da ação na folha: <projeto>.<posição no projeto> quando o blueprint tem projetos (seções na ordem em que os projetos foram criados, Geral por último),
//! <blueprint>.<posição> quando não tem. Mesma regra de `numeroDaAcao` em web/js/projetos.js (projetos.test.mjs).
use super::numeros_das_acoes;
use serde_json::{json, Value};

fn acao(id: &str, extra: Value) -> Value {
    let mut a = json!({"id": id, "nome": id});
    a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    a
}

#[test]
fn sem_projetos_numera_como_sempre_blueprint_ponto_posicao() {
    let p = json!({"acoes": [acao("a", json!({})), acao("b", json!({}))]});
    assert_eq!(numeros_das_acoes(&p, 3), ["3.1", "3.2"]);
    assert!(numeros_das_acoes(&json!({"acoes": []}), 1).is_empty());
}

#[test]
fn com_projetos_cada_um_e_uma_secao_na_ordem_de_criacao_e_o_geral_por_ultimo() {
    let p = json!({"projetos": [{"id": "s", "nome": "Servidor"}, {"id": "i", "nome": "Interface"}, {"id": "v", "nome": "Vazio"}], "acoes": [
        acao("a1", json!({"projetoId": "s"})), acao("a2", json!({"projetoId": "i"})),
        acao("a3", json!({"projetoId": "s"})), acao("solta", json!({})), acao("a5", json!({"projetoId": "i"}))]});
    assert_eq!(numeros_das_acoes(&p, 1), ["1.1", "2.1", "1.2", "4.1", "2.2"], "Servidor 1, Interface 2, Vazio 3 (sem ações), Geral 4");
}

#[test]
fn a_estrutura_de_ler_blueprint_usa_o_nome_do_projeto_e_o_projeto_que_sumiu_vira_geral() {
    let p = json!({"projetos": [{"id": "g", "nome": "GitHub"}, {"id": "r", "nome": "Rust"}], "acoes": [
        acao("a1", json!({"projeto": "GitHub"})), acao("a2", json!({"projeto": "rust"})), acao("a3", json!({"projeto": "Apagado"}))]});
    assert_eq!(numeros_das_acoes(&p, 1), ["1.1", "2.1", "3.1"], "o nome vale sem diferenciar maiúsculas; o que não existe vai para o Geral");
}
