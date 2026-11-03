//! Testes de `acao_blocos.rs`: cada peça no seu plano.
use super::super::super::modelo::{Diff, LinhaDiff};
use super::*;

fn plano(r: &str) -> Plano {
    Plano { id: r.into(), referencia: r.into(), texto: format!("texto {r}"), ..Plano::default() }
}

fn folha() -> Folha {
    let dec = |id: &str, q: &str| Decisao { id: id.into(), acoes: vec!["1".into()], plano: q.into(), ..Decisao::default() };
    Folha { decisoes: vec![dec("d1", "1.1"), dec("d2", ""), dec("d3", "sumiu")], perguntas: vec![Pergunta { id: "q1".into(), acoes: vec!["1".into()], plano: "1.2".into(), ..Pergunta::default() }], ..Folha::default() }
}

fn acao() -> Acao {
    let tela = Detalhe { id: "01".into(), plano: "1.2".into(), pede_aprovacao: true, ..Detalhe::default() };
    let k = |id: &str, q: &str| Checkpoint { id: id.into(), plano: q.into(), ..Checkpoint::default() };
    Acao { referencia: "1".into(), planos: vec![plano("1.1"), plano("1.2")], detalhes: vec![tela], checkpoints: vec![k("c1", "1.1"), k("c2", "")], ..Acao::default() }
}

#[test]
fn cada_peca_fica_no_seu_plano() {
    let (f, a) = (folha(), acao());
    let m = montar(&f, &a);
    assert_eq!(m.blocos.len(), 3, "da ação, 1.1 e 1.2");
    assert!(m.blocos[0].plano.is_none());
    assert_eq!(m.blocos[1].decisoes[0].id, "d1");
    assert_eq!(m.blocos[1].checkpoints[0].id, "c1");
    assert_eq!((m.blocos[2].telas[0].id.as_str(), m.blocos[2].perguntas[0].id.as_str()), ("01", "q1"));
    assert_eq!(m.soltos[0].id, "c2");
}

#[test]
fn plano_que_nao_existe_volta_para_a_acao() {
    let (f, a) = (folha(), acao());
    let ids: Vec<&str> = montar(&f, &a).blocos[0].decisoes.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(ids, ["d2", "d3"]);
}

#[test]
fn tela_sem_plano_nunca_vira_secao_e_vai_para_quem_combina() {
    let f = Folha::default();
    let tela = Detalhe { id: "01".into(), titulo: "O catálogo das peças".into(), ..Detalhe::default() };
    let q = |r: &str, t: &str| Plano { referencia: r.into(), texto: t.into(), ..Plano::default() };
    let a = Acao { planos: vec![q("1.1", "Guardar no banco"), q("1.2", "Montar o catálogo")], detalhes: vec![tela.clone()], ..Acao::default() };
    let m = montar(&f, &a);
    assert_eq!(m.blocos.len(), 2, "não sobra bloco da ação");
    assert_eq!(m.blocos[1].telas[0].id, "01");
    let sem_plano = Acao { detalhes: vec![tela], ..Acao::default() };
    assert_eq!(montar(&f, &sem_plano).blocos[0].telas.len(), 1, "sem plano nenhum, fica com a ação");
    let empate = Acao { planos: vec![q("1.1", "Um"), q("1.2", "Dois")], detalhes: vec![Detalhe { id: "02".into(), ..Detalhe::default() }], ..Acao::default() };
    assert_eq!(montar(&f, &empate).blocos[0].telas[0].id, "02", "no empate, o primeiro");
}

#[test]
fn sem_nada_da_acao_nao_ha_bloco_da_acao() {
    let f = Folha::default();
    let a = Acao { referencia: "1".into(), planos: vec![plano("1.1")], ..Acao::default() };
    let m = montar(&f, &a);
    assert_eq!(m.blocos.len(), 1);
    assert!(m.blocos[0].plano.is_some());
}

#[test]
fn o_que_falta_decidir_conta_tela_decisao_e_pergunta() {
    let (f, a) = (folha(), acao());
    let m = montar(&f, &a);
    assert_eq!(m.blocos[2].a_decidir(), (2, true));
    assert_eq!(m.blocos[1].a_decidir(), (0, false), "decisão sem pedido de aprovação não entra");
}

#[test]
fn os_planos_seguem_a_versao_pendente() {
    let linha = |r: &str, t: &str| LinhaDiff { referencia: r.into(), texto: t.into(), ..LinhaDiff::default() };
    let a = Acao { planos: vec![plano("1.1")], diff: Some(Diff { linhas: vec![linha("1.1", "novo texto"), linha("1.2", "chegou")], ..Diff::default() }), ..Acao::default() };
    let q = planos_da(&a);
    assert_eq!(q[0].texto, "novo texto");
    assert_eq!((q[1].referencia.as_str(), q[1].situacao), ("1.2", Situacao::Revisao));
}

#[test]
fn a_pilha_so_conta_o_que_existe() {
    assert_eq!(pilha(&[0.0, 0.0]), 0.0);
    assert_eq!(pilha(&[10.0, 0.0]), 10.0);
    assert_eq!(pilha(&[10.0, 20.0]), 30.0 + VAO_PECA);
}
