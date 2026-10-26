//! `ler_blueprint` pela camada do MCP: `blueprint` (uso interno) segue completo; `blueprint_lido` aceita `incluir` e `projeto`; com `acao_id` a ação vem inteira.
use super::acoes::{blueprint, blueprint_lido};
use serde_json::{json, Value};

fn dados() -> Value {
    let acoes: Vec<Value> = (0..80)
        .map(|i| json!({"id": format!("f{i}"), "nome": format!("Ação {i} com um nome de tamanho comum"), "inicio": "2030-01-01", "fim": "2030-02-01", "status": if i < 60 { "concluido" } else { "planejado" }, "obs": "instruções",
            "planos": (0..5).map(|k| json!({"id": format!("t{i}-{k}"), "texto": "Implementar o módulo com validação e teste de ponta a ponta, cobrindo os erros", "descricao": "d", "feito": i < 60})).collect::<Vec<_>>(), "checkpoints": []}))
        .collect();
    json!({"blueprints": [{"id": "p1", "nome": "Portal", "descricao": "d", "obs": "", "projetos": [{"id": "pr1", "nome": "Interface"}], "acoes": acoes}]})
}

#[test]
fn o_uso_interno_segue_completo_e_a_leitura_do_agente_vem_resumida() {
    let d = dados();
    let completo = blueprint(&d, "sm", "p1", None).unwrap();
    assert!(completo["blueprint"].get("resumidas").is_none() && completo["blueprint"]["acoes"][0]["planos"].is_array(), "hooks, status e eventos precisam de tudo");
    let lido = blueprint_lido(&d, "sm", "p1", None, "", "").unwrap();
    assert_eq!(lido["blueprint"]["resumidas"]["concluidas"], 60);
    assert_eq!(lido["blueprint"]["acoes"][0]["id"], "f0", "a linha compacta mantém o id");
    assert!(lido.to_string().len() < completo.to_string().len() / 2);
    assert_eq!(blueprint_lido(&d, "sm", "p1", None, "tudo", "").unwrap(), completo);
}

#[test]
fn com_acao_id_a_acao_vem_inteira_mesmo_concluida() {
    let r = blueprint_lido(&dados(), "sm", "p1", Some("f3"), "ativas", "Interface").unwrap();
    assert_eq!(r["acao"]["observacao"], "instruções");
    assert_eq!(r["acao"]["planos"].as_array().unwrap().len(), 5);
}

#[test]
fn parametro_invalido_vira_400_com_a_mensagem() {
    let e = blueprint_lido(&dados(), "sm", "p1", None, "tudoo", "").unwrap_err();
    assert!(e.status == 400 && e.mensagem.contains("incluir must be"));
    let e = blueprint_lido(&dados(), "sm", "p1", None, "tudo", "Fantasma").unwrap_err();
    assert!(e.status == 400 && e.mensagem.contains("Unknown project"));
}
