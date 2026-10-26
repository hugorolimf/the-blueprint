use super::*;

#[test]
fn o_dono_local_pode_tudo() {
    let c = Contexto::local();
    assert_eq!((c.usuario.as_str(), c.papel, c.local, c.visitante), ("local", Papel::Dono, true, false));
    assert!(c.pode_ler() && c.pode_editar() && c.pode_administrar());
    assert!(c.exige_editar().is_ok());
    for acao in [Acao::Ler, Acao::Gravar, Acao::Administrar] {
        assert!(c.pode(acao, "*", None) && c.pode(acao, "w1", Some("b1")));
    }
}

#[test]
fn cada_papel_no_seu_escopo() {
    for (papel, grava, administra) in [(Papel::Dono, true, true), (Papel::Colaborador, true, false), (Papel::Supervisor, false, false)] {
        let c = Contexto::membro("m1", "Ana", vec![Acesso::novo("w1", "", papel)]);
        assert_eq!(c.papel, papel);
        assert!(c.pode(Acao::Ler, "w1", None) && c.pode(Acao::Ler, "w1", Some("b1")), "{papel:?}");
        assert_eq!(c.pode(Acao::Gravar, "w1", Some("b1")), grava, "{papel:?}");
        assert_eq!(c.pode(Acao::Administrar, "w1", None), administra, "{papel:?}");
        assert_eq!(c.exige(Acao::Gravar, "w1", Some("b1")).is_ok(), grava, "{papel:?}");
        // outro workspace: nada
        assert!(!c.pode(Acao::Ler, "w2", None) && !c.pode(Acao::Ler, "w2", Some("b1")), "{papel:?}");
        assert!(matches!(c.exige(Acao::Ler, "w2", None), Err(Erro::Proibido(_))));
    }
}

#[test]
fn vale_o_maior_papel_que_cobre_o_alvo() {
    let c = Contexto::membro("m1", "Ana", vec![Acesso::novo("w1", "", Papel::Supervisor), Acesso::novo("w1", "b1", Papel::Colaborador)]);
    assert!(c.pode(Acao::Gravar, "w1", Some("b1")));
    assert!(!c.pode(Acao::Gravar, "w1", Some("b2")));
    assert!(!c.pode(Acao::Gravar, "w1", None), "o acesso do blueprint não alcança o workspace inteiro");
    let so_bp = Contexto::membro("m2", "Bia", vec![Acesso::novo("w1", "b1", Papel::Colaborador)]);
    assert!(!so_bp.pode(Acao::Ler, "w1", None) && so_bp.pode(Acao::Ler, "w1", Some("b1")));
}

#[test]
fn so_o_dono_de_tudo_administra_o_servidor() {
    let todos = Contexto::membro("m1", "Ana", vec![Acesso::novo("*", "", Papel::Dono)]);
    assert!(todos.pode(Acao::Administrar, "*", None) && todos.pode(Acao::Gravar, "qualquer", Some("b")));
    let de_um = Contexto::membro("m2", "Bia", vec![Acesso::novo("w1", "", Papel::Dono)]);
    assert!(!de_um.pode(Acao::Administrar, "*", None));
}

#[test]
fn visitante_so_le_o_blueprint_do_link() {
    let v = Contexto::visitante("c1", "w1", "b1", "apresentacao");
    assert!(v.visitante && !v.local && v.usuario == "visitante:c1");
    assert_eq!(v.acessos[0].abre, "apresentacao");
    assert!(v.pode(Acao::Ler, "w1", Some("b1")));
    assert!(!v.pode(Acao::Ler, "w1", Some("b2")) && !v.pode(Acao::Ler, "w1", None));
    assert!(!v.pode(Acao::Gravar, "w1", Some("b1")) && !v.pode_editar());
}

#[test]
fn anonimo_nao_alcanca_nada() {
    let a = Contexto::anonimo();
    assert!(!a.autenticado() && !a.pode_ler() && !a.pode(Acao::Ler, "w1", Some("b1")));
}

#[test]
fn papel_de_e_para_texto() {
    for p in [Papel::Dono, Papel::Colaborador, Papel::Supervisor, Papel::Visitante] {
        assert_eq!(Papel::de_texto(p.texto()), Some(p));
    }
    assert_eq!(Papel::de_texto("admin"), None);
}
