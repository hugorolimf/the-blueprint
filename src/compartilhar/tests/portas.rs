//! As rotas pelas duas portas: pela dos convidados nada vira o dono local, as rotas só locais não existem e só o workspace compartilhado
//! aparece; pela local tudo continua como antes, e os convites saem na URL do túnel.
use super::*;

/// Toda rota que só existe no app local (abre processo, fala com a janela, com os hooks, com o atualizador ou liga o compartilhamento).
const SO_LOCAIS: &[(&str, &str)] = &[
    ("GET", "/api/terminal/sessoes"),
    ("POST", "/api/terminal/iniciar"),
    ("GET", "/api/terminal/fluxo?id=x"),
    ("GET", "/api/atualizacao"),
    ("POST", "/api/atualizacao/instalar"),
    ("POST", "/api/app/mostrar"),
    ("POST", "/api/mcp/iniciar"),
    ("GET", "/api/mcp/sessoes?workspace=w2&blueprint=p9"),
    ("GET", "/api/mcp/pastas"),
    ("POST", "/api/mcp/pasta/vincular"),
    ("GET", "/api/mcp/conversa?workspace=w2"),
    ("POST", "/api/mcp/hook/sessao"),
    ("POST", "/api/mcp/hook/edicao"),
    ("POST", "/api/mcp/hook/parada"),
    ("GET", "/api/compartilhar"),
    ("POST", "/api/compartilhar/ligar"),
    ("POST", "/api/compartilhar/desligar"),
    ("PUT", "/api/dados"),
];

#[test]
fn cada_rota_so_local_da_404_pela_porta_dos_convidados_mesmo_com_sessao_de_dono() {
    let e = estado_compartilhando("w2");
    let dono = convidado_entra(&e, "w2", "dono", "Ana");
    for (metodo, url) in SO_LOCAIS {
        let corpo = (*metodo != "GET").then(|| json!({"workspaceId": "w2", "blueprintId": "p9"}));
        for cookie in [None, Some(dono.as_str())] {
            let r = pela_porta_dos_convidados(&e, &convidado(metodo, url, cookie, corpo.clone()));
            // sem sessão, o PUT dos dados para antes (401): de um jeito ou de outro, não passa
            let esperado = if *url == "/api/dados" && cookie.is_none() { "sem-sessao" } else { "so-local" };
            assert_eq!(codigo(&r), esperado, "{metodo} {url} com cookie={}: {}", cookie.is_some(), String::from_utf8_lossy(r.bytes()));
        }
    }
    // e o MCP sem chave nem sessão não vira o dono local: 401
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/api/mcp/blueprints", None, None));
    assert_eq!((r.status, codigo(&r)), (401, "sem-sessao".into()));
}

#[test]
fn convidado_sem_sessao_nao_le_nada_mas_recebe_a_pagina() {
    let e = estado_compartilhando("w2");
    for url in ["/api/dados", "/api/mcp/fila", "/api/mcp/blueprints", "/api/eventos?workspace=w2", "/api/historico?workspace=w2", "/api/pessoas?workspace=w2"] {
        let r = pela_porta_dos_convidados(&e, &convidado("GET", url, None, None));
        assert_eq!(r.status, 401, "{url}");
    }
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/api/sessao", None, None));
    assert_eq!(codigo(&r), "sem-sessao", "a página sabe que precisa do convite");
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/", None, None));
    assert_eq!(r.status, 200);
    assert_eq!(r.cabecalho("x-frame-options"), Some("DENY"), "os cabeçalhos de segurança do modo hospedado valem aqui");
}

#[test]
fn host_origin_e_https_sao_os_do_tunel() {
    let e = estado_compartilhando("w2");
    let mut p = convidado("GET", "/api/sessao", None, None);
    p.cabecalhos.retain(|(k, _)| k != "Host");
    p.cabecalhos.push(("Host".into(), "127.0.0.1:3000".into()));
    assert_eq!(codigo(&pela_porta_dos_convidados(&e, &p)), "host-invalido", "quem chega pela porta dos convidados com o Host local não vira local");
    let mut p = convidado("GET", "/api/sessao", None, None);
    p.cabecalhos.retain(|(k, _)| k != "X-Forwarded-Proto");
    assert_eq!(codigo(&pela_porta_dos_convidados(&e, &p)), "https-obrigatorio");
    let mut p = convidado("POST", "/api/convite/conferir", None, Some(json!({"segredo": "x"})));
    p.cabecalhos.retain(|(k, _)| k != "Origin");
    p.cabecalhos.push(("Origin".into(), "http://127.0.0.1:3000".into()));
    assert_eq!(codigo(&pela_porta_dos_convidados(&e, &p)), "origem-invalida");
}

#[test]
fn convidado_so_ve_o_workspace_compartilhado() {
    let e = estado_compartilhando("w2");
    let ana = convidado_entra(&e, "w2", "colaborador", "Ana");
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/api/dados", Some(&ana), None));
    let d = json_de(&r);
    assert_eq!(d["blueprints"], json!([]), "a raiz (sm) não aparece");
    assert_eq!(d["workspaces"].as_array().unwrap().len(), 1);
    assert_eq!(d["workspaces"][0]["id"], "w2");
    // o mesmo membro, com acesso também à raiz (de um compartilhamento anterior), só alcança o workspace compartilhado agora
    let membro = json_de(&pela_porta_dos_convidados(&e, &convidado("GET", "/api/sessao", Some(&ana), None)))["membro"]["id"].as_str().unwrap().to_string();
    crate::dados::membros::definir_papel(e.dados.lock().unwrap().conexao(), &membro, "sm", Papel::Dono).unwrap();
    let d = json_de(&pela_porta_dos_convidados(&e, &convidado("GET", "/api/dados", Some(&ana), None)));
    assert_eq!(d["blueprints"], json!([]));
    let s = json_de(&pela_porta_dos_convidados(&e, &convidado("GET", "/api/sessao", Some(&ana), None)));
    assert_eq!(s["acessos"].as_array().unwrap().len(), 1, "{s}");
    // gravar fora do workspace compartilhado: proibido
    let ops = json!({"base": 0, "cliente": "c1", "ops": [{"op": "alterar", "tipo": "blueprint", "id": "p1", "campos": {"nome": "X"}, "antes": {"nome": "Portal"}}]});
    let r = pela_porta_dos_convidados(&e, &convidado("POST", "/api/mudancas", Some(&ana), Some(ops)));
    assert_eq!(r.status, 403, "{}", String::from_utf8_lossy(r.bytes()));
}

#[test]
fn desligado_a_porta_dos_convidados_nao_atende() {
    let e = crate::http::tests::estado(false);
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/", None, None));
    assert_eq!(r.status, 503);
}

#[test]
fn porta_local_continua_igual_e_convites_saem_na_url_do_tunel() {
    let e = crate::http::tests::estado(false);
    // sem compartilhar: o modo local de sempre
    assert_eq!(json_de(&pela_porta_local(&e, &local("GET", "/api/sessao", None))), json!({"modo": "local"}));
    assert_eq!(codigo(&pela_porta_local(&e, &local("GET", "/api/convites?workspace=w2", None))), "so-servidor");
    assert_eq!(pela_porta_local(&e, &local("GET", "/api/dados", None)).status, 200, "a página local lê tudo sem cookie");
    e.compartilhar.ligado_para_teste(URL, "w2");
    assert_eq!(json_de(&pela_porta_local(&e, &local("GET", "/api/sessao", None))), json!({"modo": "local"}), "a página local continua local");
    let r = pela_porta_local(&e, &local("POST", "/api/convites", Some(json!({"workspaceId": "w2", "papel": "colaborador"}))));
    assert_eq!(r.status, 201);
    assert!(json_de(&r)["link"].as_str().unwrap().starts_with(&format!("{URL}/convite#")));
    for ws in ["sm", "*"] {
        let r = pela_porta_local(&e, &local("POST", "/api/convites", Some(json!({"workspaceId": ws, "papel": "dono"}))));
        assert_eq!(r.status, 403, "convite de {ws} recusado enquanto compartilha w2");
    }
    let lista = json_de(&pela_porta_local(&e, &local("GET", "/api/convites?workspace=w2", None)));
    assert_eq!(lista["convites"].as_array().unwrap().len(), 1);
    let ana = convidado_entra(&e, "w2", "colaborador", "Ana");
    let pessoas = json_de(&pela_porta_local(&e, &local("GET", "/api/pessoas?workspace=w2", None)));
    assert_eq!(pessoas["membros"][0]["nome"], "Ana");
    let id = pessoas["membros"][0]["id"].as_str().unwrap();
    let r = pela_porta_local(&e, &local("POST", "/api/pessoas/remover", Some(json!({"workspaceId": "w2", "membroId": id}))));
    assert_eq!(r.status, 200);
    let r = pela_porta_dos_convidados(&e, &convidado("GET", "/api/dados", Some(&ana), None));
    assert!(r.status == 401 || json_de(&r)["workspaces"] == json!([]), "removida, não lê mais nada: {}", String::from_utf8_lossy(r.bytes()));
    // o chave e a entrada pelo convite não são da página local
    assert_eq!(codigo(&pela_porta_local(&e, &local("POST", "/api/chave", Some(json!({}))))), "so-servidor");
}
