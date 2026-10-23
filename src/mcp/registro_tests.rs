//! O registro do trabalho já feito na fila do MCP: número próprio (separado das propostas), nada salvo, recusa do que não é registro e ack só do registro.
use super::acoes::proposta;
use super::registro::registro;
use super::Mcp;
use serde_json::{json, Value};

fn mcp() -> Mcp {
    Mcp::novo("127.0.0.1", false, "blueprint.exe", "http://127.0.0.1:3000", std::env::temp_dir())
}

fn dados() -> Value {
    json!({"blueprints": [{"id": "p1", "nome": "Portal", "acoes": [{"id": "f1", "nome": "Base", "inicio": "2030-01-01", "fim": "2030-03-01", "status": "andamento", "obs": "", "planos": [{"id": "t1", "texto": "Criar rota", "descricao": "Escopo x. Validação: roda.", "feito": false}], "checkpoints": []}]}]})
}

fn corpo_registro() -> Value {
    json!({"workspaceId": "sm", "blueprintId": "p1", "mensagem": "Corrigi o menu", "acoes": [{"id": "f1", "planos": [
        {"texto": "Corrigir o menu do botão direito", "descricao": "Escopo: x. Validação: o teste passa.", "feito": true, "evidencia": "node menu.mjs: passa"},
        {"texto": "Cobrir o menu com teste", "descricao": "Escopo: x. Validação: roda."}]}]})
}

fn corpo_proposta() -> Value {
    json!({"workspaceId": "sm", "blueprintId": "p1", "mensagem": "m", "acoes": [{"nome": "Nova ação", "inicio": "2030-04-01", "fim": "2030-04-30", "planos": [
        {"texto": "Criar a tela", "descricao": "Escopo: x. Validação: o teste passa."}, {"texto": "Criar a rota", "descricao": "Escopo: x. Validação: o teste passa."}], "checkpoints": []}]})
}

#[test]
fn o_registro_entra_na_fila_com_numero_proprio_e_sem_apagar_proposta() {
    let m = mcp();
    assert_eq!(proposta(&m, &dados(), &corpo_proposta()).unwrap().1["numero"], 1);
    let (status, r) = registro(&m, &dados(), &corpo_registro()).unwrap();
    assert_eq!((status, r["numero"].clone()), (200, json!(1)), "a contagem dos registros é própria: o primeiro é o nº 1 mesmo com uma proposta na fila");
    assert_eq!(registro(&m, &dados(), &corpo_registro()).unwrap().1["numero"], 2);
    assert_eq!(proposta(&m, &dados(), &corpo_proposta()).unwrap().1["numero"], 2, "e a das propostas segue a dela");
    let itens = m.itens();
    assert_eq!(itens.iter().filter(|i| i["tipo"] == "proposta").count(), 2);
    let reg = itens.iter().find(|i| i["tipo"] == "registro").unwrap();
    assert_eq!(reg["registro"]["registro"], true);
    assert_eq!(reg["registro"]["acoes"][0]["planos"][0]["feito"], true);
    assert_eq!(reg["registro"]["acoes"][0]["planos"][0]["evidencia"], "node menu.mjs: passa");
    assert!(reg.get("proposta").is_none(), "registro não é proposta");
}

#[test]
fn registro_invalido_nao_entra_na_fila() {
    let m = mcp();
    let mut sem_evidencia = corpo_registro();
    sem_evidencia["acoes"][0]["planos"][0].as_object_mut().unwrap().remove("evidencia");
    let (status, r) = registro(&m, &dados(), &sem_evidencia).unwrap();
    assert_eq!(status, 422);
    assert!(r["erros"][0].as_str().unwrap().contains("evidencia"));
    let sem_blueprint = json!({"workspaceId": "sm", "acoes": []});
    assert_eq!(registro(&m, &dados(), &sem_blueprint).unwrap_err().status, 400, "registro precisa de um blueprint que já existe");
    assert!(m.itens().is_empty());
}

#[test]
fn a_pagina_confirma_registro_e_proposta_cada_um_na_sua_chave() {
    let m = mcp();
    proposta(&m, &dados(), &corpo_proposta()).unwrap();
    registro(&m, &dados(), &corpo_registro()).unwrap();
    let itens = m.itens();
    let chave = |tipo: &str| {
        let i = itens.iter().find(|i| i["tipo"] == tipo).unwrap();
        json!({"workspaceId": "sm", "blueprintId": "p1", "numero": i["numero"], "criadaEm": i["criadaEm"]})
    };
    // o número 1 de proposta e o número 1 de registro (e até a mesma hora) não se confundem: cada um sai só pela confirmação do seu tipo
    m.ack_proposta(&chave("proposta"));
    let restam = m.itens();
    assert_eq!((restam.len(), restam[0]["tipo"].clone()), (1, json!("registro")), "confirmar a proposta não tira o registro");
    m.ack_registro(&json!({"workspaceId": "sm", "blueprintId": "outro", "numero": 1, "criadaEm": restam[0]["criadaEm"]}));
    assert_eq!(m.itens().len(), 1, "o alvo e a chave inteira têm de bater");
    m.ack_registro(&chave("registro"));
    assert!(m.itens().is_empty());
}
