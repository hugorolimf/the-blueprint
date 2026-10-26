//! HTTPS pelo proxy, Host, Origin, cabeçalhos de segurança e as rotas que só existem no app local.
use super::*;

fn sem(p: &mut Pedido, nome: &str) {
    p.cabecalhos.retain(|(k, _)| !k.eq_ignore_ascii_case(nome));
}

fn codigo(r: &Resposta) -> String {
    json_de(r)["codigo"].as_str().unwrap_or("").to_string()
}

#[test]
fn sem_https_do_proxy_nada_sai_nem_a_pagina() {
    let e = estado();
    let mut p = pedido("GET", "/", None, None);
    sem(&mut p, "x-forwarded-proto");
    let r = despachar(&e, &p);
    assert_eq!((r.status, codigo(&r).as_str()), (403, "https-obrigatorio"));
    let mut http = pedido("GET", "/", None, None);
    http.cabecalhos.push(("X-Forwarded-Proto".into(), "http".into()));
    sem(&mut http, "x-forwarded-proto");
    http.cabecalhos.push(("X-Forwarded-Proto".into(), "http".into()));
    assert_eq!(despachar(&e, &http).status, 403);
    // quem fala direto com o app (fora do proxy) não pode fingir o cabeçalho
    let mut direto = pedido("GET", "/", None, None);
    direto.ip = "198.51.100.9".into();
    assert_eq!(codigo(&despachar(&e, &direto)), "https-obrigatorio");
    let mut sem_ip = pedido("GET", "/", None, None);
    sem_ip.ip = String::new();
    assert_eq!(despachar(&e, &sem_ip).status, 403, "par desconhecido não é o proxy");
}

#[test]
fn proxy_em_outra_maquina_so_com_blueprint_proxy() {
    let h = Hospedagem::de(URL, "0.0.0.0", Some("10.0.0.5"), false).unwrap();
    assert!(cabecalhos::par_confiavel(&h, "10.0.0.5") && cabecalhos::par_confiavel(&h, "127.0.0.1"));
    assert!(!cabecalhos::par_confiavel(&h, "10.0.0.6"));
    assert!(Hospedagem::de(URL, "0.0.0.0", None, false).is_err(), "HOST fora de loopback sem BLUEPRINT_PROXY");
    assert!(Hospedagem::de("http://equipe.teste", "127.0.0.1", None, false).is_err(), "só https");
    assert!(Hospedagem::de("https://equipe.teste/", "127.0.0.1", None, false).is_err(), "sem barra no fim");
    assert!(Hospedagem::de("https://equipe.teste", "127.0.0.1", Some("não-ip"), false).is_err());
}

#[test]
fn host_diferente_do_publico_e_recusado() {
    let e = estado();
    let mut p = pedido("GET", "/", None, None);
    sem(&mut p, "host");
    p.cabecalhos.push(("Host".into(), "outro.exemplo".into()));
    let r = despachar(&e, &p);
    assert_eq!((r.status, codigo(&r).as_str()), (403, "host-invalido"));
}

#[test]
fn a_pagina_sai_com_os_cabecalhos_de_seguranca() {
    let e = estado();
    for url in ["/", "/convite"] {
        let r = despachar(&e, &pedido("GET", url, None, None));
        assert_eq!(r.status, 200, "{url}");
        assert_eq!(r.cabecalho("referrer-policy"), Some("no-referrer"));
        assert_eq!(r.cabecalho("x-content-type-options"), Some("nosniff"));
        assert_eq!(r.cabecalho("x-frame-options"), Some("DENY"));
        assert_eq!(r.cabecalho("strict-transport-security"), Some("max-age=31536000"));
    }
    let api = despachar(&e, &pedido("GET", "/api/dados", None, None));
    assert_eq!(api.cabecalho("cache-control"), Some("no-store"));
}

#[test]
fn post_com_cookie_precisa_da_origin_publica() {
    let e = estado();
    let dono = pessoa(&e, Papel::Dono, "*", "", "Dona");
    for origem in [Some("https://evil.exemplo"), Some("http://equipe.teste"), None] {
        let mut p = pedido("POST", "/api/chave", Some(&dono), Some(json!({})));
        sem(&mut p, "origin");
        if let Some(o) = origem {
            p.cabecalhos.push(("Origin".into(), o.into()));
        }
        let r = despachar(&e, &p);
        assert_eq!((r.status, codigo(&r).as_str()), (403, "origem-invalida"), "{origem:?}");
    }
    assert_eq!(despachar(&e, &pedido("POST", "/api/chave", Some(&dono), Some(json!({})))).status, 201);
}

#[test]
fn rotas_so_locais_nao_existem_no_servidor() {
    let e = estado();
    let dono = pessoa(&e, Papel::Dono, "*", "", "Dona");
    for (metodo, url) in [("POST", "/api/mcp/iniciar"), ("POST", "/api/mcp/conversa"), ("GET", "/api/mcp/conversa"), ("POST", "/api/mcp/hook/sessao"), ("GET", "/api/mcp/sessoes"), ("GET", "/api/mcp/pastas"), ("POST", "/api/mcp/pasta/vincular"), ("GET", "/api/terminal/sessoes"), ("GET", "/api/atualizacao"), ("POST", "/api/app/mostrar")] {
        let r = despachar(&e, &pedido(metodo, url, Some(&dono), Some(json!({}))));
        assert_eq!((r.status, codigo(&r).as_str()), (404, "so-local"), "{url}");
    }
}

#[test]
fn sem_sessao_so_a_pagina_e_o_convite() {
    let e = estado();
    for url in ["/api/dados", "/api/mcp/blueprints", "/api/mcp/fila", "/api/pessoas?workspace=sm", "/api/chave"] {
        let r = despachar(&e, &pedido("GET", url, None, None));
        assert_eq!((r.status, codigo(&r).as_str()), (401, "sem-sessao"), "{url}");
    }
    let r = despachar(&e, &pedido("GET", "/api/sessao", None, None));
    assert_eq!((r.status, codigo(&r).as_str()), (401, "sem-sessao"));
    let falso = "A".repeat(43);
    assert_eq!(despachar(&e, &pedido("GET", "/api/dados", Some(&falso), None)).status, 401, "cookie inventado não é sessão");
}

#[test]
fn modo_local_continua_igual() {
    let mut e = estado();
    e.config.hospedagem = None;
    let mut p = pedido("GET", "/api/sessao", None, None);
    p.cabecalhos = vec![("Host".into(), "127.0.0.1:3000".into())];
    assert_eq!(json_de(&despachar(&e, &p)), json!({"modo": "local"}));
    p.caminho = "/api/pessoas".into();
    assert_eq!(codigo(&despachar(&e, &p)), "so-servidor");
    p.caminho = "/convite".into();
    assert_eq!(despachar(&e, &p).status, 404, "a página do convite só existe no servidor");
    p.caminho = "/api/dados".into();
    let r = despachar(&e, &p);
    assert_eq!(json_de(&r), fixture(), "o dono local lê tudo, sem filtro");
    assert!(r.cabecalho("referrer-policy").is_none(), "os cabeçalhos do servidor não mudam o app local");
}
