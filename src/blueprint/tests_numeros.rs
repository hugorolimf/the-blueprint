//! O número da ação na folha: <projeto>.<posição no projeto> quando o blueprint tem projetos (seções pela data de início da primeira ação, Geral por último),
//! <blueprint>.<posição> quando não tem. Mesma regra de `numeroDaAcao` em web/js/projetos.js (projetos.test.mjs).
use super::numeros_das_acoes;
use serde_json::{json, Value};

fn acao(id: &str, inicio: &str, extra: Value) -> Value {
    let mut a = json!({"id": id, "nome": id, "inicio": inicio, "fim": "2026-12-31"});
    a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    a
}

#[test]
fn sem_projetos_numera_como_sempre_blueprint_ponto_posicao() {
    let p = json!({"acoes": [acao("a", "2026-11-01", json!({})), acao("b", "2026-11-02", json!({}))]});
    assert_eq!(numeros_das_acoes(&p, 3), ["3.1", "3.2"]);
    assert!(numeros_das_acoes(&json!({"acoes": []}), 1).is_empty());
}

#[test]
fn com_projetos_cada_um_e_uma_secao_na_ordem_do_inicio_e_o_geral_por_ultimo() {
    // o projeto criado primeiro (Servidor) começa depois: a seção 1 é a Interface
    let p = json!({"projetos": [{"id": "s", "nome": "Servidor"}, {"id": "i", "nome": "Interface"}, {"id": "v", "nome": "Vazio"}], "acoes": [
        acao("a1", "2026-11-10", json!({"projetoId": "s"})), acao("a2", "2026-11-01", json!({"projetoId": "i"})),
        acao("a3", "2026-11-12", json!({"projetoId": "s"})), acao("solta", "2026-11-20", json!({})), acao("a5", "2026-11-03", json!({"projetoId": "i"}))]});
    assert_eq!(numeros_das_acoes(&p, 1), ["2.1", "1.1", "2.2", "4.1", "1.2"], "Interface 1, Servidor 2, Vazio 3 (sem ações), Geral 4");
}

#[test]
fn a_estrutura_de_ler_blueprint_usa_o_nome_do_projeto_e_o_projeto_que_sumiu_vira_geral() {
    let p = json!({"projetos": [{"id": "g", "nome": "GitHub"}, {"id": "r", "nome": "Rust"}], "acoes": [
        acao("a1", "2026-10-21", json!({"projeto": "GitHub"})), acao("a2", "2026-10-20", json!({"projeto": "rust"})), acao("a3", "2026-10-22", json!({"projeto": "Apagado"}))]});
    assert_eq!(numeros_das_acoes(&p, 1), ["2.1", "1.1", "3.1"], "o nome vale sem diferenciar maiúsculas; o que não existe vai para o Geral");
}
