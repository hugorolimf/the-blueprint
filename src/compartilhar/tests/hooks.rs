//! Os hooks do Claude do convidado pela porta dos convidados: com a chave dele, a figura aparece na folha do dono com o nome do convidado;
//! sem a chave ou fora do workspace compartilhado, nada.
use super::*;

fn com_chave(mut p: Pedido, chave: &str) -> Pedido {
    p.cabecalhos.retain(|(k, _)| k != "Origin");
    p.cabecalhos.push(("Authorization".into(), format!("Bearer {chave}")));
    p
}

#[test]
fn o_claude_do_convidado_aparece_na_folha_do_dono() {
    let e = estado_compartilhando("w2");
    let cookie = convidado_entra(&e, "w2", "colaborador", "Bia");
    let r = pela_porta_dos_convidados(&e, &convidado("POST", "/api/chave", Some(&cookie), Some(json!({}))));
    assert_eq!(r.status, 201, "{}", String::from_utf8_lossy(r.bytes()));
    let chave = json_de(&r)["chave"].as_str().unwrap().to_string();
    let corpo = |ws: &str, bp: &str| json!({"session_id": "s1", "cwd": "/proj", "tool_input": {"file_path": "/proj/web/a.js"}, "workspaceId": ws, "blueprintId": bp});
    let r = pela_porta_dos_convidados(&e, &com_chave(convidado("POST", "/api/mcp/hook/edicao", None, Some(corpo("w2", "p9"))), &chave));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    // o dono, pela porta local, vê a figura com o nome da convidada
    let agentes = json_de(&pela_porta_local(&e, &local("GET", "/api/mcp/agentes", None)));
    let bia = agentes["agentes"].as_array().unwrap().iter().find(|a| a["agente"] == "Claude · Bia").cloned().expect("figura da Bia");
    assert_eq!((bia["workspaceId"].as_str(), bia["onde"].clone()), (Some("w2"), json!(["web/a.js"])));
    let sessoes = String::from_utf8_lossy(pela_porta_local(&e, &local("GET", "/api/mcp/sessoes?workspace=w2&blueprint=p9", None)).bytes()).to_string();
    assert!(!sessoes.contains("s1"), "o convidado não entra na lista de sessões (Iniciar é só local): {sessoes}");
    // fora do workspace compartilhado e sem a chave
    assert_eq!(pela_porta_dos_convidados(&e, &com_chave(convidado("POST", "/api/mcp/hook/edicao", None, Some(corpo("sm", "p1"))), &chave)).status, 403);
    assert_eq!(codigo(&pela_porta_dos_convidados(&e, &convidado("POST", "/api/mcp/hook/edicao", Some(&cookie), Some(corpo("w2", "p9"))))), "so-local");
    assert_eq!(codigo(&pela_porta_dos_convidados(&e, &com_chave(convidado("POST", "/api/mcp/hook/status", None, Some(json!({}))), &chave))), "so-local");
}
