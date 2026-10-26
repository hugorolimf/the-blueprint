use super::*;
use crate::http::pedido::Pedido;

#[test]
fn reconhece_as_etiquetas_e_cai_no_ingles() {
    assert_eq!(Idioma::de_texto("pt-BR"), Some(Idioma::Pt));
    assert_eq!(Idioma::de_texto("pt_PT.UTF-8"), Some(Idioma::Pt));
    assert_eq!(Idioma::de_texto("en-US"), Some(Idioma::En));
    assert_eq!(Idioma::de_texto("fr"), None);
    assert_eq!(Idioma::de_accept_language("pt-BR,pt;q=0.9,en;q=0.8"), Idioma::Pt);
    assert_eq!(Idioma::de_accept_language("fr-FR,fr;q=0.9,pt;q=0.5"), Idioma::Pt); // a primeira com catálogo
    assert_eq!(Idioma::de_accept_language("fr-FR,de"), Idioma::En);
    assert_eq!(Idioma::de_accept_language("en-US,pt-BR"), Idioma::En);
}

#[test]
fn a_variavel_manda_depois_o_sistema_depois_ingles() {
    assert_eq!(escolher(Some("pt-BR"), Some(Idioma::En)), Idioma::Pt);
    assert_eq!(escolher(Some("en"), Some(Idioma::Pt)), Idioma::En);
    assert_eq!(escolher(Some("xx"), Some(Idioma::Pt)), Idioma::Pt);
    assert_eq!(escolher(None, None), Idioma::En);
}

#[test]
fn o_pedido_usa_accept_language_e_sem_ele_o_usuario() {
    let p = |cab: Vec<(String, String)>| Pedido::novo("GET", "/", cab, vec![]);
    assert_eq!(do_pedido(&p(vec![("Accept-Language".into(), "en-US,en".into())])), Idioma::En);
    assert_eq!(do_pedido(&p(vec![("accept-language".into(), "pt-BR".into())])), Idioma::Pt);
    assert_eq!(do_pedido(&p(vec![])), do_usuario());
}

#[test]
fn tr_segue_o_idioma_da_thread_e_volta_ao_anterior() {
    let n = 3;
    assert_eq!(com(Idioma::En, || crate::tr!("{n} plans", "{n} planos")), "3 plans");
    assert_eq!(com(Idioma::Pt, || crate::tr!("{n} plans", "{n} planos")), "3 planos");
    com(Idioma::En, || {
        com(Idioma::Pt, || assert!(!ingles()));
        assert!(ingles());
    });
    assert_eq!(atual(), do_usuario());
}
