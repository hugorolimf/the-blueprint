//! A API local do compartilhamento (`/api/compartilhar`), o Atualizar recusando com ele ligado e o SSE do MCP de quem não é local.
use super::*;

#[test]
fn api_so_desta_maquina_e_com_json() {
    let e = crate::http::tests::estado(false);
    assert_eq!(json_de(&pela_porta_local(&e, &local("GET", "/api/compartilhar", None))), json!({"estado": "desligado"}));
    let mut p = local("GET", "/api/compartilhar", None);
    p.cabecalhos = vec![("Host".into(), "exemplo.com".into())];
    assert_eq!(codigo(&pela_porta_local(&e, &p)), "so-local");
    let mut p = local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "w2"})));
    p.cabecalhos.retain(|(k, _)| k != "Origin");
    p.cabecalhos.push(("Origin".into(), "https://outro-site.exemplo".into()));
    assert_eq!(codigo(&pela_porta_local(&e, &p)), "so-local", "página de outro site não liga o compartilhamento");
    let mut p = local("POST", "/api/compartilhar/ligar", None);
    p.corpo = br#"{"workspaceId":"w2"}"#.to_vec();
    assert_eq!(pela_porta_local(&e, &p).status, 415, "sem application/json");
    for (corpo, cod) in [(json!({}), "falta-workspace"), (json!({"workspaceId": "*"}), "falta-workspace"), (json!({"workspaceId": "nao-existe"}), "workspace-inexistente")] {
        assert_eq!(codigo(&pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(corpo)))), cod);
    }
    assert_eq!(json_de(&pela_porta_local(&e, &local("POST", "/api/compartilhar/desligar", Some(json!({}))))), json!({"estado": "desligado"}));
}

#[test]
fn no_modo_hospedado_a_api_nao_existe() {
    let mut e = crate::http::tests::estado(false);
    e.config.hospedagem = Some(Hospedagem::de("https://equipe.teste", "127.0.0.1", None, false).unwrap());
    let mut p = Pedido::novo("GET", "/api/compartilhar", vec![("Host".into(), "equipe.teste".into()), ("X-Forwarded-Proto".into(), "https".into())], vec![]);
    p.ip = "127.0.0.1".into();
    assert_eq!(codigo(&pela_porta_local(&e, &p)), "so-local");
}

#[test]
fn um_compartilhamento_por_vez() {
    let e = Arc::new(crate::http::tests::estado(false));
    e.compartilhar.definir_estado(Arc::downgrade(&e));
    e.compartilhar.ligado_para_teste(URL, "w2");
    let r = pela_porta_local(&e, &local("POST", "/api/compartilhar/ligar", Some(json!({"workspaceId": "sm"}))));
    assert_eq!((r.status, codigo(&r)), (409, "ja-compartilhando".into()));
    assert_eq!(json_de(&r)["params"]["workspaceId"], "w2");
}

#[test]
fn atualizar_recusa_com_o_compartilhamento_ligado() {
    let e = crate::http::tests::estado(false);
    assert!(crate::http::atualizacao::bloqueio(&e).is_none());
    e.compartilhar.ligado_para_teste(URL, "w2");
    assert!(crate::http::atualizacao::bloqueio(&e).unwrap().contains("compartilh"));
    e.compartilhar.desligar();
    assert!(crate::http::atualizacao::bloqueio(&e).is_none());
}

#[test]
fn sse_do_mcp_de_quem_nao_e_local_nao_leva_texto_do_usuario_nem_outro_workspace() {
    use crate::mcp::permissao::evento_visivel;
    use crate::mcp::Evento;
    let ana = so_do_workspace(&Contexto::membro("m1", "Ana", vec![Acesso::novo("w2", "", Papel::Colaborador)]), "w2");
    let ev = |nome: &'static str, dados: Value| Evento { nome, dados: Some(dados) };
    assert!(!evento_visivel(&ana, &ev("iniciar-acao", json!({"workspaceId": "w2", "blueprintId": "p9", "canalId": "c", "texto": "pedido"}))));
    assert!(!evento_visivel(&ana, &ev("compartilhar", json!({"estado": "ligado"}))));
    assert!(!evento_visivel(&ana, &ev("linha-do-tempo", json!({"workspaceId": "sm", "blueprintId": "p1"}))));
    assert!(evento_visivel(&ana, &ev("linha-do-tempo", json!({"workspaceId": "w2", "blueprintId": "p9"}))));
    assert!(evento_visivel(&ana, &Evento { nome: "fila", dados: None }));
}
