use super::*;

#[test]
fn o_dono_local_pode_tudo() {
    let c = Contexto::local();
    assert_eq!(c.papel, Papel::Dono);
    assert!(c.pode_ler() && c.pode_editar() && c.pode_administrar());
    assert!(c.exige_editar().is_ok());
}

#[test]
fn so_dono_e_colaborador_editam() {
    for (papel, edita) in [(Papel::Dono, true), (Papel::Colaborador, true), (Papel::Supervisor, false), (Papel::Visitante, false)] {
        let c = Contexto { usuario: "ana".into(), papel };
        assert!(c.pode_ler());
        assert_eq!(c.pode_editar(), edita, "{papel:?}");
        assert_eq!(c.exige_editar().is_ok(), edita, "{papel:?}");
    }
}

#[test]
fn so_o_dono_administra() {
    assert!(!Contexto { usuario: "ana".into(), papel: Papel::Colaborador }.pode_administrar());
}
