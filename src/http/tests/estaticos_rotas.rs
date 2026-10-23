use super::{estado, pedido, texto_de};
use crate::http::estaticos::permitido;
use crate::http::rotas::despachar;
use flate2::read::GzDecoder;
use std::io::Read;

#[test]
fn a_lista_do_que_pode_sair_e_a_do_servidor_node() {
    for ok in ["/blueprint.html", "/emblemas.mjs", "/agentes-figura.mjs", "/agentes-claude.mjs", "/web/css/tokens.css", "/web/js/inicio.js", "/kit/iso-kit.mjs", "/kit/lathe/torno.mjs"] {
        assert!(permitido(ok), "{ok}");
    }
    for fora in ["/", "/server.mjs", "/data.json", "/Cargo.toml", "/web/js/../../server.mjs", "/kit/../server.mjs", "/web/x.html", "/web/js/", "/kit/iso-kit.js", "/web//x.js", "/kit/a b.mjs", "/src/main.rs", "/prototipos/x.html"] {
        assert!(!permitido(fora), "{fora}");
    }
}

#[test]
fn a_pagina_e_os_estaticos_saem_com_o_tipo_certo() {
    let e = estado(false);
    for (url, tipo) in [("/", "text/html"), ("/blueprint.html", "text/html"), ("/emblemas.mjs", "text/javascript"), ("/agentes-figura.mjs", "text/javascript"), ("/agentes-claude.mjs", "text/javascript"), ("/kit/iso-kit.mjs", "text/javascript"), ("/kit/lathe/torno.mjs", "text/javascript"), ("/web/js/inicio.js", "text/javascript"), ("/web/css/tokens.css", "text/css")] {
        let r = despachar(&e, &pedido("GET", url, &[], None));
        assert_eq!(r.status, 200, "{url}");
        assert!(r.cabecalho("content-type").unwrap().starts_with(tipo), "{url}");
        assert!(r.cabecalho("etag").is_some(), "{url}");
    }
    assert!(texto_de(&despachar(&e, &pedido("GET", "/", &[], None))).contains("id=\"login-form\""));
    for url in ["/server.mjs", "/kit/../server.mjs", "/web/js/nao-existe.js", "/data.json", "/qualquer", "/api/ia"] {
        assert_eq!(despachar(&e, &pedido("GET", url, &[], None)).status, 404, "{url}");
    }
}

#[test]
fn o_embutido_e_igual_ao_arquivo_do_blueprint() {
    let e = estado(false);
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for url in ["/blueprint.html", "/emblemas.mjs", "/kit/iso-kit/camera.mjs", "/web/js/render.js", "/web/css/base.css"] {
        let r = despachar(&e, &pedido("GET", url, &[], None));
        assert_eq!(r.bytes(), std::fs::read(raiz.join(url.trim_start_matches('/'))).unwrap().as_slice(), "{url}");
    }
}

#[test]
fn etag_devolve_304_sem_corpo() {
    let e = estado(false);
    let etag = despachar(&e, &pedido("GET", "/web/js/render.js", &[], None)).cabecalho("etag").unwrap().to_string();
    let r = despachar(&e, &pedido("GET", "/web/js/render.js", &[("If-None-Match", &etag)], None));
    assert_eq!((r.status, r.bytes().len()), (304, 0));
    assert_eq!(despachar(&e, &pedido("GET", "/web/js/render.js", &[("If-None-Match", "\"outro\"")], None)).status, 200);
}

#[test]
fn gzip_so_quando_o_cliente_aceita_e_volta_igual() {
    let e = estado(false);
    let puro = despachar(&e, &pedido("GET", "/web/js/render.js", &[], None));
    assert!(puro.cabecalho("content-encoding").is_none());
    let gz = despachar(&e, &pedido("GET", "/web/js/render.js", &[("Accept-Encoding", "gzip, deflate, br")], None));
    assert_eq!(gz.cabecalho("content-encoding"), Some("gzip"));
    assert!(gz.bytes().len() < puro.bytes().len() / 2, "o gzip precisa compensar");
    let mut volta = Vec::new();
    GzDecoder::new(gz.bytes()).read_to_end(&mut volta).unwrap();
    assert_eq!(volta, puro.bytes());
    let pequeno = despachar(&e, &pedido("GET", "/web/css/tokens.css", &[("Accept-Encoding", "gzip")], None));
    assert!(pequeno.cabecalho("content-encoding").is_none(), "arquivo pequeno não compensa comprimir");
}
