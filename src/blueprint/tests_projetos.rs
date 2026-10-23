//! Projetos na proposta: `blueprint.projetos` e `projeto` em cada ação (id ou nome); desconhecido é erro, nada some por omissão.
use super::{contexto_acao, estrutura_blueprint, validar_proposta, Validacao};
use serde_json::{json, Value};

const HOJE: &str = "2026-10-21";

fn blueprint() -> Value {
    json!({"id": "p", "nome": "Portal", "descricao": "d", "obs": "", "projetos": [{"id": "pr1", "nome": "Interface"}, {"id": "pr2", "nome": "Servidor"}], "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2026-11-01", "fim": "2026-11-30", "status": "andamento", "obs": "", "projetoId": "pr1",
         "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "x", "feito": true}], "checkpoints": []},
        {"id": "f2", "nome": "Solta", "inicio": "2026-12-01", "fim": "2026-12-10", "status": "planejado", "obs": "", "planos": [], "checkpoints": []}]})
}

fn acao(extra: Value) -> Value {
    let mut a = json!({"nome": "Ação nova", "inicio": "2026-12-11", "fim": "2026-12-31", "status": "planejado", "observacao": "",
        "planos": [{"texto": "Criar item", "descricao": "Escopo: x. Resultado: y. Validação: node x.test.mjs passa."}], "checkpoints": [{"nome": "Entrega validada", "data": "2026-12-30"}]});
    a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
    a
}

fn valida(entrada: Value, p: Option<&Value>) -> Validacao {
    validar_proposta(&entrada, p, HOJE)
}

fn erro_com(entrada: Value, p: Option<&Value>, trecho: &str) {
    let r = valida(entrada, p);
    assert!(r.erros.iter().any(|e| e.contains(trecho)), "esperava erro com \"{trecho}\"; veio: {:?}", r.erros);
    assert!(r.proposta.is_none());
}

#[test]
fn projeto_por_id_ou_por_nome_vira_id_e_nome() {
    let p = blueprint();
    let r = valida(json!({"acoes": [acao(json!({"projeto": "pr2"})), acao(json!({"projeto": " interface "}))]}), Some(&p));
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let acoes = &r.proposta.unwrap()["acoes"];
    assert_eq!(acoes[0]["projeto"], json!({"id": "pr2", "nome": "Servidor"}));
    assert_eq!(acoes[1]["projeto"], json!({"id": "pr1", "nome": "Interface"}), "o nome casa sem ligar para caixa nem espaços");
}

#[test]
fn sem_o_campo_a_acao_fica_onde_esta_e_geral_ou_vazio_tira_do_projeto() {
    let p = blueprint();
    let r = valida(json!({"acoes": [{"id": "f1", "nome": "Base 2"}, {"id": "f1", "projeto": "Geral"}, {"id": "f1", "projeto": ""}]}), Some(&p));
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let acoes = &r.proposta.unwrap()["acoes"];
    assert!(acoes[0].get("projeto").is_none(), "omitir não muda nada");
    assert_eq!(acoes[1]["projeto"], Value::Null);
    assert_eq!(acoes[2]["projeto"], Value::Null);
}

#[test]
fn projeto_desconhecido_e_erro_claro() {
    let p = blueprint();
    erro_com(json!({"acoes": [acao(json!({"projeto": "Mobile"}))]}), Some(&p), "o projeto \"Mobile\" não existe neste blueprint (projetos: Interface, Servidor;");
    erro_com(json!({"blueprint": {"nome": "N", "descricao": "d"}, "acoes": [acao(json!({"projeto": "Mobile"}))]}), None, "este blueprint não tem projetos");
    erro_com(json!({"acoes": [acao(json!({"projeto": 7}))]}), Some(&p), "`projeto` deve ser o id ou o nome");
}

#[test]
fn projetos_novos_nascem_na_proposta_e_as_acoes_podem_citar_o_nome() {
    let p = blueprint();
    let r = valida(json!({"blueprint": {"projetos": [{"nome": "Dados"}, "Servidor", {"nome": "dados"}, {"id": "pr1"}]}, "acoes": [acao(json!({"projeto": "Dados"}))]}), Some(&p));
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let proposta = r.proposta.unwrap();
    assert_eq!(proposta["projetos"], json!([{"id": "", "nome": "Dados"}]), "o que já existe ou repete não é novo");
    assert_eq!(proposta["acoes"][0]["projeto"], json!({"id": "", "nome": "Dados"}));
}

#[test]
fn blueprint_novo_aceita_projetos() {
    let r = valida(json!({"blueprint": {"nome": "N", "descricao": "d", "projetos": ["A", "B"]}, "acoes": [acao(json!({"projeto": "b"}))]}), None);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let proposta = r.proposta.unwrap();
    assert_eq!(proposta["projetos"].as_array().unwrap().len(), 2);
    assert_eq!(proposta["acoes"][0]["projeto"]["nome"], "B");
}

#[test]
fn proposta_sem_projetos_continua_igual() {
    let r = valida(json!({"acoes": [acao(json!({}))]}), Some(&blueprint()));
    let proposta = r.proposta.unwrap();
    assert!(proposta.get("projetos").is_none() && proposta["acoes"][0].get("projeto").is_none());
}

#[test]
fn nomes_de_projeto_invalidos() {
    let p = blueprint();
    erro_com(json!({"blueprint": {"projetos": ["geral"]}, "acoes": [acao(json!({}))]}), Some(&p), "reservado");
    erro_com(json!({"blueprint": {"projetos": [{"nome": ""}]}, "acoes": [acao(json!({}))]}), Some(&p), "falta o nome do projeto");
    erro_com(json!({"blueprint": {"projetos": [{"id": "inventado"}]}, "acoes": [acao(json!({}))]}), Some(&p), "o id \"inventado\" não existe");
    let muitos: Vec<String> = (0..13).map(|i| format!("P{i}")).collect();
    erro_com(json!({"blueprint": {"projetos": muitos}, "acoes": [acao(json!({}))]}), Some(&p), "Projetos novos demais");
}

#[test]
fn o_agente_le_o_projeto_de_cada_acao() {
    let p = blueprint();
    let e = estrutura_blueprint(&p);
    assert_eq!(e["projetos"], json!([{"id": "pr1", "nome": "Interface"}, {"id": "pr2", "nome": "Servidor"}]));
    assert_eq!(e["acoes"][0]["projeto"], "Interface");
    assert!(e["acoes"][1].get("projeto").is_none(), "ação sem projeto não traz o campo");
    assert_eq!(contexto_acao(&p, "f1").unwrap()["projeto"], "Interface");
    let mut orfa = p.clone();
    orfa["acoes"][1]["projetoId"] = json!("apagado");
    assert!(estrutura_blueprint(&orfa)["acoes"][1].get("projeto").is_none(), "id que não existe mais vale como Geral");
}

#[test]
fn blueprint_sem_projetos_nao_traz_campo_novo() {
    let p = json!({"id": "p", "nome": "P", "descricao": "", "acoes": [{"id": "f", "nome": "A", "inicio": "2026-11-01", "fim": "2026-11-02", "status": "planejado", "planos": [], "checkpoints": []}]});
    let e = estrutura_blueprint(&p);
    assert!(e.get("projetos").is_none() && e["acoes"][0].get("projeto").is_none());
}
