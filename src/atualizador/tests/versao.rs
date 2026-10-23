use crate::atualizador::versao::{mais_nova, Versao};

#[test]
fn compara_como_semver() {
    assert!(mais_nova("0.1.0", "0.1.1"));
    assert!(mais_nova("0.1.9", "0.2.0"));
    assert!(mais_nova("0.9.0", "v1.0.0"), "a tag pode vir com v");
    assert!(!mais_nova("0.1.1", "0.1.1"));
    assert!(!mais_nova("0.2.0", "0.1.9"));
}

#[test]
fn pre_lancamento_vale_menos_que_a_versao_final() {
    assert!(mais_nova("0.2.0-beta.1", "0.2.0"));
    assert!(!mais_nova("0.2.0", "0.2.0-beta.1"));
    assert!(mais_nova("0.2.0-beta.1", "0.2.0-beta.2"));
    assert!(mais_nova("0.2.0-beta.2", "0.2.0-beta.10"), "números comparam como números");
    assert!(mais_nova("0.2.0-alpha", "0.2.0-beta"));
    assert!(mais_nova("0.2.0-1", "0.2.0-beta"), "só números vale menos que letras");
    assert!(Versao::ler("1.0.0-rc.1").unwrap() < Versao::ler("1.0.0").unwrap());
}

#[test]
fn texto_que_nao_e_versao_nunca_e_mais_novo() {
    for ruim in ["", "latest", "1.2", "1.2.3.4", "1.2.x", "v", "1.0.0-", "a.b.c"] {
        assert!(!mais_nova("0.1.0", ruim), "{ruim:?}");
        assert!(!mais_nova(ruim, "9.9.9"), "{ruim:?} como instalada");
    }
}
