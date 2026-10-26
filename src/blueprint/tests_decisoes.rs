//! Decisões na proposta: `blueprint.decisoes`, com id atualiza e completa, sem id cria; só a proposta de decisões é válida sem ações.
use super::{estrutura_blueprint, validar_proposta, Validacao};
use serde_json::{json, Value};

fn blueprint() -> Value {
    json!({"id": "p", "nome": "Portal", "descricao": "d", "obs": "", "decisoes": [
        {"id": "d1", "titulo": "Servidor", "escolha": "Kotlin com Ktor", "motivo": "regras num lugar só", "estado": "decidida", "em": "2026-10-26"}], "acoes": []})
}

fn valida(entrada: Value, p: &Value) -> Validacao {
    validar_proposta(&entrada, Some(p), "2026-10-26")
}

fn erro_com(entrada: Value, trecho: &str) {
    let r = valida(entrada, &blueprint());
    assert!(r.erros.iter().any(|e| e.contains(trecho)), "esperava erro com \"{trecho}\"; veio: {:?}", r.erros);
    assert!(r.proposta.is_none());
}

#[test]
fn so_decisoes_basta_e_o_estado_vem_da_escolha() {
    let r = valida(json!({"acoes": [], "blueprint": {"decisoes": [{"titulo": "Banco", "escolha": "PostgreSQL"}, {"titulo": "Hospedagem"}]}}), &blueprint());
    assert!(r.erros.is_empty(), "{:?}", r.erros);
    let d = &r.proposta.unwrap()["decisoes"];
    assert_eq!((d[0]["id"].as_str(), d[0]["estado"].as_str()), (Some(""), Some("decidida")));
    assert_eq!(d[1]["estado"], "aberta", "sem escolha ainda não foi decidida");
}

#[test]
fn id_existente_completa_o_que_faltou() {
    let r = valida(json!({"acoes": [], "blueprint": {"decisoes": [{"id": "d1", "motivo": "menos código duplicado"}]}}), &blueprint());
    let d = &r.proposta.unwrap()["decisoes"][0];
    assert_eq!((d["titulo"].as_str(), d["escolha"].as_str(), d["estado"].as_str()), (Some("Servidor"), Some("Kotlin com Ktor"), Some("decidida")));
    assert_eq!(d["motivo"], "menos código duplicado");
}

#[test]
fn recusa_id_desconhecido_titulo_ausente_estado_invalido_e_decidida_sem_escolha() {
    erro_com(json!({"acoes": [], "blueprint": {"decisoes": [{"id": "zzz", "titulo": "x"}]}}), "não existe");
    erro_com(json!({"acoes": [], "blueprint": {"decisoes": [{"escolha": "x"}]}}), "falta o título");
    erro_com(json!({"acoes": [], "blueprint": {"decisoes": [{"titulo": "x", "estado": "talvez"}]}}), "`estado`");
    erro_com(json!({"acoes": [], "blueprint": {"decisoes": [{"titulo": "x", "estado": "decidida"}]}}), "precisa de `escolha`");
}

#[test]
fn ler_blueprint_lista_as_decisoes() {
    let e = estrutura_blueprint(&blueprint());
    assert_eq!(e["decisoes"], json!([{"id": "d1", "titulo": "Servidor", "estado": "decidida", "escolha": "Kotlin com Ktor"}]));
    assert!(estrutura_blueprint(&json!({"id": "q", "nome": "Q", "acoes": []})).get("decisoes").is_none());
}
