use super::*;

#[test]
fn sessao_ativa_ate_ser_encerrada() {
    let a = Acesso::novo();
    let ana = Contexto::membro("m1", "Ana", vec![]);
    assert!(a.sessao_ativa(&Contexto::local()));
    assert!(a.sessao_ativa(&ana));
    assert!(!a.sessao_ativa(&Contexto::anonimo()), "sem sessão não há o que manter aberto");
    a.encerrar("m1");
    assert!(!a.sessao_ativa(&ana));
    assert!(a.sessao_ativa(&Contexto::local()), "o local nunca é encerrado");
}
