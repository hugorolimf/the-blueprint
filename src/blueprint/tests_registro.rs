//! O registro do trabalho feito fora do plano: aceita `feito` com evidência (e o que ficou pendente), datas passadas sem aviso, e a proposta continua recusando `feito`.
use super::{validar_proposta, validar_registro};
use serde_json::{json, Value};

const HOJE: &str = "2030-06-10";

fn blueprint() -> Value {
    json!({"id": "p", "nome": "Portal", "descricao": "d", "obs": "o", "acoes": [
        {"id": "f1", "nome": "Base", "inicio": "2030-06-01", "fim": "2030-06-30", "status": "andamento", "obs": "obs",
         "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "x", "feito": false}], "checkpoints": []}]})
}

fn plano_feito(texto: &str) -> Value {
    json!({"texto": texto, "descricao": "Escopo: x. Resultado: y. Validação: o teste passa.", "feito": true, "evidencia": "cargo test: 5 passam"})
}

fn registro(extra: Value) -> Value {
    let mut base = json!({"mensagem": "Corrigi o menu", "acoes": [{"id": "f1", "planos": [plano_feito("Corrigir o menu do botão direito"), {"texto": "Cobrir o menu com teste", "descricao": "Escopo: x. Validação: roda."}]}]});
    for (k, v) in extra.as_object().unwrap() {
        base[k] = v.clone();
    }
    base
}

fn erros(entrada: &Value) -> Vec<String> {
    let r = validar_registro(entrada, &blueprint(), HOJE);
    assert!(r.proposta.is_none(), "um registro com erro não vira registro");
    r.erros
}

#[test]
fn registro_com_feito_e_evidencia_leva_os_checks_e_deixa_o_resto_pendente() {
    let r = validar_registro(&registro(json!({})), &blueprint(), HOJE);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let p = r.proposta.unwrap();
    assert_eq!(p["registro"], true);
    let planos = p["acoes"][0]["planos"].as_array().unwrap();
    assert_eq!(planos[0]["feito"], true);
    assert_eq!(planos[0]["evidencia"], "cargo test: 5 passam");
    assert!(planos[1].get("feito").is_none(), "o que não foi marcado fica pendente");
    assert_eq!(planos[2]["id"], "t1", "o plano que já existia e não veio continua como está");
    assert!(planos[2].get("feito").is_none());
}

#[test]
fn checkpoint_feito_tambem_leva_a_evidencia() {
    let entrada = registro(json!({"acoes": [{"id": "f1", "planos": [plano_feito("Corrigir o menu do botão direito")], "checkpoints": [{"nome": "Menu aprovado", "data": "2030-06-10", "feito": true, "evidencia": "usuário viu"}, {"nome": "Menu publicado", "data": "2030-06-20"}]}]}));
    let r = validar_registro(&entrada, &blueprint(), HOJE);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let cps = r.proposta.unwrap()["acoes"][0]["checkpoints"].clone();
    assert_eq!((cps[0]["feito"].clone(), cps[0]["evidencia"].clone()), (json!(true), json!("usuário viu")));
    assert!(cps[1].get("feito").is_none(), "o checkpoint pendente continua pendente");
}

#[test]
fn feito_exige_evidencia_de_tamanho_razoavel() {
    let sem = registro(json!({"acoes": [{"id": "f1", "planos": [{"texto": "Corrigir o menu", "descricao": "Validação: x.", "feito": true}]}]}));
    assert!(erros(&sem).iter().any(|e| e.contains("exige a `evidencia`")));
    let longa = registro(json!({"acoes": [{"id": "f1", "planos": [{"texto": "Corrigir o menu", "descricao": "Validação: x.", "feito": true, "evidencia": "x".repeat(1001)}]}]}));
    assert!(erros(&longa).iter().any(|e| e.contains("o máximo é 1000")));
    let torto = registro(json!({"acoes": [{"id": "f1", "planos": [{"texto": "Corrigir o menu", "descricao": "Validação: x.", "feito": "sim"}]}]}));
    assert!(erros(&torto).iter().any(|e| e.contains("true ou false")));
}

#[test]
fn registro_sem_nada_feito_e_proposta_disfarcada() {
    let nada = registro(json!({"acoes": [{"id": "f1", "planos": [{"texto": "Cobrir o menu com teste", "descricao": "Escopo: x. Validação: roda."}]}]}));
    assert!(erros(&nada).iter().any(|e| e.contains("não tem nada feito")));
}

#[test]
fn registro_nao_remove_nem_move() {
    let com_remocao = registro(json!({"remover": [{"acao_id": "f1"}]}));
    assert!(erros(&com_remocao).iter().any(|e| e.contains("`remover` e `mover` não valem")));
}

#[test]
fn registro_de_acao_nova_no_passado_nao_avisa_data_nem_prazo() {
    let entrada = json!({"acoes": [{"nome": "Ajuste de ontem", "inicio": "2030-06-08", "fim": "2030-06-09", "planos": [plano_feito("Corrigir o menu do botão direito")]}]});
    let r = validar_registro(&entrada, &blueprint(), HOJE);
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    assert!(r.avisos.is_empty(), "o trabalho já aconteceu: sem avisos de data passada, plano único ou prazo; veio {:?}", r.avisos);
    assert_eq!(r.proposta.unwrap()["acoes"][0]["planos"][0]["feito"], true);
}

#[test]
fn proposta_continua_recusando_feito() {
    let entrada = registro(json!({}));
    let r = validar_proposta(&entrada, Some(&blueprint()), HOJE);
    assert!(r.erros.iter().any(|e| e.contains("registrar_trabalho")), "{:?}", r.erros);
    assert!(r.proposta.is_none());
}
