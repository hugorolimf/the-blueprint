//! O MCP por HTTP (`POST /mcp`, src/mcp/http.rs) pela porta dos convidados: o convidado com a chave dele lê e registra com o papel dele, sem o app instalado;
//! sem chave só a descrição do servidor; chave inválida, outro workspace, navegador e métodos errados são recusados; e no app local a rota não existe.
use super::*;

fn chave_de(e: &Estado, papel: &str, nome: &str) -> String {
    let cookie = convidado_entra(e, "w2", papel, nome);
    let r = pela_porta_dos_convidados(e, &convidado("POST", "/api/chave", Some(&cookie), Some(json!({}))));
    assert_eq!(r.status, 201, "{}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)["chave"].as_str().unwrap().to_string()
}

/// Um cliente MCP: sem Origin, com a chave (se houver) no Authorization.
fn mcp(metodo: &str, chave: Option<&str>, corpo: Option<Value>) -> Pedido {
    let mut p = convidado(metodo, "/mcp", None, corpo);
    p.cabecalhos.retain(|(k, _)| k != "Origin");
    if let Some(c) = chave {
        p.cabecalhos.push(("Authorization".into(), format!("Bearer {c}")));
    }
    p
}

fn rpc(id: u32, metodo: &str, params: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "method": metodo, "params": params})
}

fn chamar(e: &Estado, chave: Option<&str>, ferramenta: &str, args: Value) -> Resposta {
    pela_porta_dos_convidados(e, &mcp("POST", chave, Some(rpc(7, "tools/call", json!({"name": ferramenta, "arguments": args})))))
}

fn texto_da_ferramenta(r: &Resposta) -> (bool, String) {
    let v = json_de(r);
    (v["result"]["isError"] == true, v["result"]["content"][0]["text"].as_str().unwrap_or("").to_string())
}

#[test]
fn o_convidado_com_a_chave_le_o_blueprint_e_registra_progresso_pelo_http() {
    let e = estado_compartilhando("w2");
    let chave = chave_de(&e, "colaborador", "Bia");
    let r = pela_porta_dos_convidados(&e, &mcp("POST", Some(&chave), Some(rpc(1, "initialize", json!({"protocolVersion": "2025-06-18"})))));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(json_de(&r)["result"]["serverInfo"]["name"], "blueprint");
    assert!(json_de(&r)["result"]["instructions"].as_str().unwrap().contains("propor_blueprint"));
    let lista = json_de(&pela_porta_dos_convidados(&e, &mcp("POST", Some(&chave), Some(rpc(2, "tools/list", json!({}))))));
    assert!(lista["result"]["tools"].as_array().unwrap().iter().any(|t| t["name"] == "registrar_progresso"));
    // lê o blueprint do workspace compartilhado
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&chave), "ler_blueprint", json!({"workspace_id": "w2", "blueprint_id": "p9"})));
    assert!(!erro && texto.contains("\"p9\""), "{texto}");
    let lido: Value = serde_json::from_str(&texto).unwrap();
    assert_eq!(lido["voce"]["nome"], "Bia", "o agente sabe quem é o membro da chave (planos com responsavel.id igual ao dele são dele)");
    assert!(lido["voce"]["id"].as_str().is_some_and(|i| !i.is_empty()));
    let do_dono = json_de(&pela_porta_local(&e, &local("GET", "/api/mcp/blueprint?workspace=w2&blueprint=p9", None)));
    assert!(do_dono.get("voce").is_none(), "no app local não há membro");
    // registra progresso: o Claude do convidado aparece na folha do dono com o nome dele
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&chave), "registrar_progresso", json!({"workspace_id": "w2", "blueprint_id": "p9", "texto": "Lendo o blueprint", "estado": "lendo"})));
    assert!(!erro, "{texto}");
    let agentes = json_de(&pela_porta_local(&e, &local("GET", "/api/mcp/agentes", None)));
    assert!(agentes["agentes"].as_array().unwrap().iter().any(|a| a["agente"] == "Claude · Bia" && a["workspaceId"] == "w2"), "{agentes}");
}

#[test]
fn outro_workspace_e_ferramentas_da_maquina_da_pessoa_sao_recusados() {
    let e = estado_compartilhando("w2");
    let chave = chave_de(&e, "colaborador", "Bia");
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&chave), "ler_blueprint", json!({"workspace_id": "sm", "blueprint_id": "p1"})));
    assert!(erro, "o workspace raiz não é do convidado: {texto}");
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&chave), "registrar_progresso", json!({"workspace_id": "sm", "blueprint_id": "p1", "texto": "x"})));
    assert!(erro, "{texto}");
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&chave), "vincular_pasta", json!({"workspace_id": "w2", "blueprint_id": "p9"})));
    assert!(erro && texto.contains("CLAUDE.local.md"), "a pasta do servidor nunca é ligada: {texto}");
}

#[test]
fn o_supervisor_so_le_e_o_visitante_nao_usa_o_mcp() {
    let e = estado_compartilhando("w2");
    let sup = chave_de(&e, "supervisor", "Sup");
    let (erro, _) = texto_da_ferramenta(&chamar(&e, Some(&sup), "ler_blueprint", json!({"workspace_id": "w2", "blueprint_id": "p9"})));
    assert!(!erro, "supervisor lê");
    let (erro, texto) = texto_da_ferramenta(&chamar(&e, Some(&sup), "registrar_progresso", json!({"workspace_id": "w2", "blueprint_id": "p9", "texto": "x"})));
    assert!(erro && texto.contains("403"), "supervisor não grava: {texto}");
}

#[test]
fn sem_chave_so_a_descricao_do_servidor_e_chave_ruim_e_401() {
    let e = estado_compartilhando("w2");
    // o pré-teste de conexão do Claude Code não manda o cabeçalho: initialize, ping e tools/list respondem
    for (id, metodo) in [(1, "initialize"), (2, "ping"), (3, "tools/list")] {
        let r = pela_porta_dos_convidados(&e, &mcp("POST", None, Some(rpc(id, metodo, json!({"protocolVersion": "2025-06-18"})))));
        assert_eq!(r.status, 200, "{metodo}: {}", String::from_utf8_lossy(r.bytes()));
    }
    // mas nenhuma ferramenta roda sem chave, e a resposta não anuncia OAuth
    let r = chamar(&e, None, "ler_blueprint", json!({"workspace_id": "w2", "blueprint_id": "p9"}));
    assert_eq!(r.status, 401);
    assert!(r.cabecalho("www-authenticate").is_none(), "sem WWW-Authenticate: o cliente usa o cabeçalho, não OAuth");
    assert_eq!(json_de(&r)["error"]["code"], -32001);
    // chave de formato torto ou revogada: 401 antes de chegar à rota
    for ruim in ["bpk_naoexiste", "lixo", "bpk_"] {
        let r = pela_porta_dos_convidados(&e, &mcp("POST", Some(ruim), Some(rpc(1, "tools/list", json!({})))));
        assert_eq!((r.status, codigo(&r)), (401, "chave-invalida".to_string()), "{ruim}");
    }
}

#[test]
fn notificacao_vira_202_sem_corpo_e_o_resto_do_protocolo_e_validado() {
    let e = estado_compartilhando("w2");
    let chave = chave_de(&e, "colaborador", "Bia");
    let r = pela_porta_dos_convidados(&e, &mcp("POST", Some(&chave), Some(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}))));
    assert_eq!((r.status, r.bytes().len()), (202, 0));
    assert_eq!(pela_porta_dos_convidados(&e, &mcp("POST", Some(&chave), Some(json!([rpc(1, "ping", json!({}))])))).status, 400, "sem lote");
    let mut torto = mcp("POST", Some(&chave), None);
    torto.corpo = b"{".to_vec();
    let r = pela_porta_dos_convidados(&e, &torto);
    assert_eq!((r.status, json_de(&r)["error"]["code"].clone()), (400, json!(-32700)));
    let mut versao = mcp("POST", Some(&chave), Some(rpc(1, "ping", json!({}))));
    versao.cabecalhos.push(("MCP-Protocol-Version".into(), "1999-01-01".into()));
    assert_eq!(pela_porta_dos_convidados(&e, &versao).status, 400);
    let mut versao_boa = mcp("POST", Some(&chave), Some(rpc(1, "ping", json!({}))));
    versao_boa.cabecalhos.push(("MCP-Protocol-Version".into(), "2025-06-18".into()));
    assert_eq!(pela_porta_dos_convidados(&e, &versao_boa).status, 200);
    let r = pela_porta_dos_convidados(&e, &mcp("POST", Some(&chave), Some(rpc(1, "metodo/que-nao-existe", json!({})))));
    assert_eq!((r.status, json_de(&r)["error"]["code"].clone()), (200, json!(-32601)), "erro de protocolo volta como JSON-RPC");
}

#[test]
fn get_delete_e_navegador_sao_recusados_e_no_app_local_a_rota_nao_existe() {
    let e = estado_compartilhando("w2");
    let chave = chave_de(&e, "colaborador", "Bia");
    for m in ["GET", "DELETE", "PUT"] {
        let r = pela_porta_dos_convidados(&e, &mcp(m, Some(&chave), None));
        assert_eq!((r.status, r.cabecalho("allow")), (405, Some("POST")), "{m}");
    }
    // um navegador manda Origin, mesmo com a chave: nunca
    let mut navegador = mcp("POST", Some(&chave), Some(rpc(1, "ping", json!({}))));
    navegador.cabecalhos.push(("Origin".into(), "https://site-malvado.example".into()));
    assert_eq!(pela_porta_dos_convidados(&e, &navegador).status, 403);
    // o app local (porta 3000) não serve o /mcp
    let r = pela_porta_local(&e, &local("POST", "/mcp", Some(rpc(1, "ping", json!({})))));
    assert_eq!(r.status, 404);
}
