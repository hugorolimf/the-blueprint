//! As execuções: uma viva por plano, passos condicionais e quem é o agente de uma pasta.
use super::super::Mudanca;
use super::{dados, lancamento, nova, rodando, BP};

#[test]
fn execucao_viva_unica() {
    let mut d = dados();
    let id = d.execucao_criar(&nova("t1")).unwrap();
    assert_eq!(id.len(), 16);
    assert!(d.execucao_criar(&nova("t1")).is_err(), "o mesmo plano não tem duas vivas");
    assert!(d.execucao_criar(&nova("t2")).is_ok());
    assert_eq!(d.execucao_viva(BP, "t1").unwrap().unwrap().id, id);
    assert!(d.execucao_cancelar(&id, 3000).unwrap());
    assert!(d.execucao_viva(BP, "t1").unwrap().is_none());
    assert!(d.execucao_criar(&nova("t1")).is_ok(), "cancelada libera");
    assert_eq!(d.execucoes_do_ws("sm").unwrap().len(), 3);
    assert_eq!(d.execucoes_vivas().unwrap().len(), 2);
}

#[test]
fn falhou_conta_como_viva() {
    let mut d = dados();
    let id = d.execucao_criar(&nova("t1")).unwrap();
    assert!(d.execucao_mudar(&id, &Mudanca { situacao: Some("falhou"), motivo: Some("base-em-conflito"), ..Default::default() }).unwrap());
    let linha = d.execucao_viva(BP, "t1").unwrap().unwrap();
    assert_eq!((linha.situacao.as_str(), linha.motivo.as_str()), ("falhou", "base-em-conflito"));
    assert!(d.execucao_criar(&nova("t1")).is_err(), "falhou ainda pode ser retomada");
    assert!(d.execucao_mudar(&id, &Mudanca { so_de: &["sumiu"], situacao: Some("marcada"), ..Default::default() }).is_ok_and(|mudou| !mudou), "so_de não confere");
    assert!(d.execucao_mudar(&id, &Mudanca { so_de: &["sumiu", "falhou"], situacao: Some("pronta"), terminada_em: Some(Some(9)), ..Default::default() }).unwrap());
    assert!(d.execucao_viva(BP, "t1").unwrap().is_none(), "pronta não é viva");
    assert!(d.execucao_mudar(&id, &Mudanca { terminada_em: Some(None), ..Default::default() }).unwrap());
    assert_eq!(d.execucao_de(&id).unwrap().unwrap().terminada_em, None, "Some(None) limpa");
    assert!(!d.execucao_mudar(&id, &Mudanca::default()).unwrap(), "nada a mudar");
}

#[test]
fn reivindicar_e_lancar_condicionais() {
    let mut d = dados();
    let id = d.execucao_criar(&nova("t2")).unwrap();
    assert!(!d.execucao_lancar(&id, &lancamento(2000)).unwrap(), "marcada não lança sem reivindicar");
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(!d.execucao_reivindicar(&id).unwrap(), "só uma vez");
    assert!(d.execucao_lancar(&id, &lancamento(2000)).unwrap());
    let primeira = d.execucao_de(&id).unwrap().unwrap();
    assert_eq!((primeira.situacao.as_str(), primeira.tentativa, primeira.segredo.len(), primeira.iniciada_em), ("rodando", 1, 32, Some(2000)));
    assert_eq!((primeira.ramo.as_str(), primeira.base.as_str()), ("agente/t2", "HEAD"));
    assert!(!format!("{primeira:?}").contains(&primeira.segredo), "o segredo não vai para o log");

    assert!(d.execucao_mudar(&id, &Mudanca { situacao: Some("marcada"), ..Default::default() }).unwrap());
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(d.execucao_cancelar(&id, 3000).unwrap(), "cancelou durante o git");
    assert!(!d.execucao_lancar(&id, &lancamento(4000)).unwrap(), "cancelada não lança");
    let cancelada = d.execucao_de(&id).unwrap().unwrap();
    assert_eq!((cancelada.situacao.as_str(), cancelada.cancelada_em, cancelada.parar, cancelada.terminada_em), ("cancelada", Some(3000), true, None));
    assert!(!d.execucao_cancelar(&id, 5000).unwrap(), "já não está viva");

    let marcada = d.execucao_criar(&nova("t3")).unwrap();
    assert!(d.execucao_cancelar(&marcada, 6000).unwrap());
    assert_eq!(d.execucao_de(&marcada).unwrap().unwrap().terminada_em, Some(6000), "marcada não tem processo: já termina");
}

#[test]
fn partida_uma_vez() {
    let mut d = dados();
    let id = d.execucao_criar(&nova("t2")).unwrap();
    assert!(d.execucao_partir(&id, 2500).unwrap().is_none(), "só com rodando");
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(d.execucao_lancar(&id, &lancamento(2000)).unwrap());
    let p = d.execucao_partir(&id, 2500).unwrap().unwrap();
    let primeiro = p.segredo.clone();
    assert_eq!((p.tentativa, p.linha.partida_em), (1, Some(2500)));
    assert!(d.execucao_partir(&id, 2600).unwrap().is_none(), "repetir não devolve o segredo");
    assert_eq!(d.execucao_de(&id).unwrap().unwrap().partida_em, Some(2500));
    assert!(d.execucao_mudar(&id, &Mudanca { situacao: Some("marcada"), ..Default::default() }).unwrap());
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(d.execucao_lancar(&id, &lancamento(3000)).unwrap());
    let p = d.execucao_partir(&id, 3100).unwrap().unwrap();
    assert_eq!(p.tentativa, 2);
    assert_ne!(p.segredo, primeiro, "tentativa nova, segredo novo");
}

#[test]
fn por_pasta_so_com_ramo() {
    let mut d = dados();
    let sem_ramo = d.execucao_criar(&nova("t1")).unwrap();
    assert!(d.execucao_mudar(&sem_ramo, &Mudanca { situacao: Some("rodando"), pasta_trabalho: Some("/tmp/sem-git"), ..Default::default() }).unwrap());
    assert!(d.execucao_por_pasta("/tmp/sem-git").unwrap().is_none(), "sem cópia própria, a sessão do usuário não vira o agente");
    let id = rodando(&mut d, "t2");
    assert_eq!(d.execucao_por_pasta("/tmp/copias/Abc").unwrap().unwrap().id, id);
    assert_eq!(d.execucao_por_pasta("/TMP/copias/abc/src/ui").unwrap().unwrap().id, id, "subpasta, com a mesma comparação de `pastas`");
    assert!(d.execucao_por_pasta("/tmp/copias/Abc2").unwrap().is_none(), "pasta irmã não conta");
    assert!(d.execucao_cancelar(&id, 3000).unwrap());
    assert!(d.execucao_por_pasta("/tmp/copias/Abc").unwrap().is_none(), "só viva");
}
