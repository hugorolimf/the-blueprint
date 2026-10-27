use super::*;
use crate::idioma::{com, Idioma};

#[test]
fn le_a_versao_do_app_que_responde_e_none_com_ele_fechado() {
    let base = crate::cli::doctor::tests::servidor_falso(VERSAO);
    assert_eq!(versao_do_app(&base).as_deref(), Some(VERSAO));
    assert_eq!(versao_do_app(&crate::cli::doctor::tests::porta_fechada()), None);
}

#[test]
fn app_aberto_na_porta_certa_e_ok() {
    assert_eq!(app("http://127.0.0.1:3000", true, Some(3000), true).estado, Estado::Ok);
    // um `blueprint serve` sem a janela também atende o agente
    assert_eq!(app("http://127.0.0.1:3000", true, None, false).estado, Estado::Ok);
    // sem a janela, o arquivo da porta é velho e não conta
    assert_eq!(app("http://127.0.0.1:3000", true, Some(53405), false).estado, Estado::Ok);
}

#[test]
fn dois_processos_e_porta_trocada_dizem_o_que_fazer() {
    com(Idioma::En, || {
        let dois = app("http://127.0.0.1:3000", true, Some(53405), true);
        assert_eq!(dois.estado, Estado::Falha);
        assert!(dois.texto.contains("two processes") && dois.texto.contains("Quit the app"), "{}", dois.texto);
        let outra = app("http://127.0.0.1:3000", false, Some(53405), true);
        assert_eq!(outra.estado, Estado::Falha);
        assert!(outra.texto.contains("BLUEPRINT_URL=http://127.0.0.1:53405"), "{}", outra.texto);
        assert_eq!(app("http://127.0.0.1:3000", false, Some(3000), true).estado, Estado::Falha);
    });
}

#[test]
fn app_fechado_manda_abrir() {
    com(Idioma::Pt, || {
        let item = app("http://127.0.0.1:3000", false, Some(3000), false);
        assert_eq!(item.estado, Estado::Falha);
        assert!(item.texto.contains("fechado") && item.texto.contains("menu Iniciar"), "{}", item.texto);
    });
}

#[test]
fn versao_diferente_e_aviso() {
    assert_eq!(versao(None).estado, Estado::Ok);
    assert_eq!(versao(Some(VERSAO)).estado, Estado::Ok);
    let outra = versao(Some("0.0.1"));
    assert_eq!(outra.estado, Estado::Aviso);
    assert!(outra.texto.contains("0.0.1"));
}
