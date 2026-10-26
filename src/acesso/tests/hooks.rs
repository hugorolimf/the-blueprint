//! Os hooks do Claude de um membro no modo hospedado: só com a chave, só no blueprint em que o papel grava, com o nome do membro na figura
//! e a sessão separada por membro.
use super::chaves::{do_agente, gerar};
use super::*;

fn chave(e: &Estado, papel: Papel, nome: &str) -> String {
    let cookie = pessoa(e, papel, "w2", "", nome);
    gerar(e, &cookie)["chave"].as_str().unwrap().to_string()
}

fn edicao(ws: &str, bp: &str) -> Value {
    json!({"session_id": "s1", "cwd": "C:/proj", "tool_input": {"file_path": "C:/proj/src/a.rs"}, "workspaceId": ws, "blueprintId": bp})
}

#[test]
fn o_hook_do_membro_com_a_chave_vira_a_figura_dele_e_o_lembrete_e_so_dele() {
    let e = estado();
    let (colab, outra) = (chave(&e, Papel::Colaborador, "Colab"), chave(&e, Papel::Colaborador, "Outra"));
    let r = despachar(&e, &do_agente("POST", "/api/mcp/hook/edicao", &colab, Some(edicao("w2", "p9"))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let agentes = json_de(&despachar(&e, &do_agente("GET", "/api/mcp/agentes", &colab, None)));
    let figura = agentes["agentes"].as_array().unwrap().iter().find(|a| a["agente"] == "Claude · Colab").cloned().expect("figura do Claude do membro");
    assert_eq!((figura["blueprintId"].as_str(), figura["onde"].clone()), (Some("p9"), json!(["src/a.rs"])));
    // a mesma sessão s1 em outro membro não é a dele: o lembrete do Stop só volta para quem editou
    assert_eq!(json_de(&despachar(&e, &do_agente("POST", "/api/mcp/hook/parada", &outra, Some(json!({"session_id": "s1"}))))), json!({}));
    let lembrete = json_de(&despachar(&e, &do_agente("POST", "/api/mcp/hook/parada", &colab, Some(json!({"session_id": "s1"})))));
    assert!(lembrete["hookSpecificOutput"]["additionalContext"].as_str().unwrap().contains("registrar_trabalho"), "{lembrete}");
    let sessao = json_de(&despachar(&e, &do_agente("POST", "/api/mcp/hook/sessao", &colab, Some(edicao("w2", "p9")))));
    assert_eq!(sessao["vinculado"], true);
    assert!(sessao["contexto"].as_str().unwrap().contains("blueprint_id p9"));
    assert_eq!(json_de(&despachar(&e, &do_agente("POST", "/api/mcp/hook/sessao", &colab, Some(json!({"cwd": "C:/proj"}))))), json!({"vinculado": false}), "sem a linha Blueprint: nada");
    assert_eq!(despachar(&e, &do_agente("POST", "/api/mcp/hook/fim", &colab, Some(json!({"session_id": "s1"})))).status, 200);
}

#[test]
fn o_progresso_do_agente_do_membro_leva_o_nome_dele() {
    let e = estado();
    let colab = chave(&e, Papel::Colaborador, "Colab");
    let r = despachar(&e, &do_agente("POST", "/api/mcp/evento", &colab, Some(json!({"tipo": "progresso", "workspaceId": "w2", "blueprintId": "p9", "texto": "lendo", "agente": "Claude"}))));
    assert_eq!(r.status, 200);
    let fila = json_de(&despachar(&e, &do_agente("GET", "/api/mcp/fila", &colab, None)));
    assert_eq!(fila["itens"][0]["agente"], "Claude · Colab", "não se junta ao Claude do dono");
}

#[test]
fn hook_sem_chave_fora_do_workspace_ou_de_quem_so_le_e_recusado() {
    let e = estado();
    let colab = chave(&e, Papel::Colaborador, "Colab");
    let sup = chave(&e, Papel::Supervisor, "Sup");
    let cookie = pessoa(&e, Papel::Colaborador, "w2", "", "Pelo navegador");
    for p in [pedido("POST", "/api/mcp/hook/edicao", Some(&cookie), Some(edicao("w2", "p9"))), pedido("POST", "/api/mcp/hook/edicao", None, Some(edicao("w2", "p9")))] {
        let r = despachar(&e, &p);
        assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (404, Some("so-local")), "sem a chave o hook continua só local");
    }
    assert_eq!(despachar(&e, &do_agente("POST", "/api/mcp/hook/edicao", &colab, Some(edicao("sm", "p1")))).status, 403, "fora do workspace");
    assert_eq!(despachar(&e, &do_agente("POST", "/api/mcp/hook/edicao", &sup, Some(edicao("w2", "p9")))).status, 403, "supervisor só lê");
    assert_eq!(despachar(&e, &do_agente("POST", "/api/mcp/hook/edicao", "bpk_inventada", Some(edicao("w2", "p9")))).status, 401);
    for rota in ["/api/mcp/hook/status", "/api/mcp/sessoes", "/api/mcp/iniciar"] {
        assert_eq!(despachar(&e, &do_agente("POST", rota, &colab, Some(json!({})))).status, 404, "{rota} continua só local, mesmo com a chave");
    }
    let agentes = json_de(&despachar(&e, &do_agente("GET", "/api/mcp/agentes", &colab, None)));
    assert_eq!(agentes["agentes"], json!([]), "nada recusado virou figura");
}
