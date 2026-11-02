//! Testes de plano::escalonar e plano::frases: os casos de `design/planejar/agentes.test.mjs`, na mesma ordem, e os novos do contrato.
use super::escalonar::{areas_conflitam, escalonar, Causa, EntradaEsc, EstadoEsc, PlanoEsc, Situacao};
use super::formato::PlanoItem;
use super::frases::{avisos, diario, texto, texto_com, Frase};
use super::validar::achar_ciclo;
use crate::idioma::Idioma;
use std::collections::BTreeMap;

fn q(chave: &str, area: Option<&str>, depende: &[&str]) -> PlanoEsc {
    PlanoEsc { chave: chave.into(), area: area.map(String::from), exclusivo: None, depende_de: depende.iter().map(|d| d.to_string()).collect() }
}
fn lista(l: &[&str]) -> Vec<String> {
    l.iter().map(|x| x.to_string()).collect()
}
fn todos(planos: &[PlanoEsc]) -> Vec<String> {
    planos.iter().map(|p| p.chave.clone()).collect()
}
fn situacao(m: &BTreeMap<String, EstadoEsc>, k: &str) -> Situacao {
    m[k].situacao
}
fn item(id: &str, depende: &[&str]) -> PlanoItem {
    PlanoItem { id: id.into(), texto: id.into(), depende_de: lista(depende), ..Default::default() }
}
fn dez() -> Vec<PlanoEsc> {
    (0..10).map(|i| q(&format!("p{i}"), Some(&format!("area{i}")), &[])).collect()
}
fn area_abc() -> Vec<PlanoEsc> {
    vec![q("a", Some("app"), &[]), q("b", Some("app/barra"), &[]), q("c", Some("servidor"), &[])]
}
fn dep() -> Vec<PlanoEsc> {
    vec![q("a", Some("x"), &[]), q("b", Some("y"), &["a"])]
}

#[test]
fn dez_planos_dez_agentes() {
    let planos = dez();
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), limite: Some(10), ..Default::default() });
    assert_eq!(m.len(), 10);
    assert!(m.values().all(|e| e.situacao == Situacao::Trabalhando));
}

#[test]
fn limite_tres_deixa_sete_na_fila() {
    let planos = dez();
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), limite: Some(3), ..Default::default() });
    assert_eq!(m.values().filter(|e| e.situacao == Situacao::Fila).count(), 7);
    assert_eq!(m.values().filter(|e| e.situacao == Situacao::Trabalhando).count(), 3);
    // a ordem de documento decide quem entra; limite ausente é 4, e abaixo de 1 vale 1
    assert_eq!(situacao(&m, "p2"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "p3"), Situacao::Fila);
    let padrao = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!(padrao.values().filter(|e| e.situacao == Situacao::Trabalhando).count(), 4);
    let zero = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), limite: Some(0), ..Default::default() });
    assert_eq!(zero.values().filter(|e| e.situacao == Situacao::Trabalhando).count(), 1);
}

#[test]
fn nao_marcado_nao_roda() {
    let m = escalonar(&[q("a", None, &[])], &EntradaEsc::default());
    assert_eq!(situacao(&m, "a"), Situacao::Nao);
}

#[test]
fn alterado_em_revisao() {
    let m = escalonar(&[q("a", None, &[])], &EntradaEsc { iniciados: lista(&["a"]), travados: lista(&["a"]), ..Default::default() });
    assert_eq!(situacao(&m, "a"), Situacao::Revisao);
}

#[test]
fn areas_conflitam_casos() {
    assert!(areas_conflitam(Some("app"), Some("app/barra")));
    assert!(areas_conflitam(Some("app/barra/"), Some("app/barra")));
    assert!(!areas_conflitam(Some("app/barra"), Some("app/barra2")));
    assert!(!areas_conflitam(None, Some("app")));
    assert!(!areas_conflitam(Some("./"), Some("app")), "área que normaliza para nada não é a raiz de tudo");
}

#[test]
fn area_a_b_c() {
    let planos = area_abc();
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!([situacao(&m, "a"), situacao(&m, "b"), situacao(&m, "c")], [Situacao::Trabalhando, Situacao::Area, Situacao::Trabalhando]);
    assert_eq!(m["b"].por.as_deref(), Some("a"));
    assert_eq!(m["b"].area.as_deref(), Some("app/barra"));
}

#[test]
fn pronto_libera_area() {
    let planos = area_abc();
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), prontos: lista(&["a"]), ..Default::default() });
    assert_eq!(situacao(&m, "a"), Situacao::Pronto);
    assert_eq!(situacao(&m, "b"), Situacao::Trabalhando);
}

#[test]
fn dependencia_espera_e_libera() {
    let planos = dep();
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!(situacao(&m, "a"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "b"), Situacao::Dependencia);
    assert_eq!(m["b"].por.as_deref(), Some("a"));
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), prontos: lista(&["a"]), ..Default::default() });
    assert_eq!(situacao(&m, "b"), Situacao::Trabalhando);
}

#[test]
fn depender_de_nao_iniciado_avisa() {
    let planos = dep();
    let m = escalonar(&planos, &EntradaEsc { iniciados: lista(&["b"]), ..Default::default() });
    let lista_avisos = avisos(&planos, &m);
    assert_eq!(lista_avisos[0], Frase::DependeSemIniciar { id: "b".into(), faltam: lista(&["a"]) });
    crate::idioma::fixar(Idioma::Pt);
    assert!(texto(&lista_avisos[0]).contains("ainda não foi iniciado"));
    // a dependência que já roda foi iniciada: é só esperar
    let m = escalonar(&planos, &EntradaEsc { iniciados: lista(&["b"]), rodando: lista(&["a"]), ..Default::default() });
    assert_eq!(avisos(&planos, &m), vec![Frase::EsperaPlano { id: "b".into(), por: "a".into() }]);
    // dependência que não existe mais fica em Dependencia, com aviso próprio
    let orfao = vec![q("c", None, &["sumiu"])];
    let m = escalonar(&orfao, &EntradaEsc { iniciados: lista(&["c"]), ..Default::default() });
    assert_eq!(situacao(&m, "c"), Situacao::Dependencia);
    assert_eq!(avisos(&orfao, &m), vec![Frase::DependeFalta { id: "c".into(), dep: "sumiu".into() }]);
}

#[test]
fn dependencia_presa_para() {
    let planos = dep();
    let m = escalonar(&planos, &EntradaEsc { iniciados: lista(&["b"]), bloqueados: lista(&["a"]), ..Default::default() });
    assert_eq!(situacao(&m, "b"), Situacao::Parado);
    assert_eq!(m["b"].por.as_deref(), Some("a"));
    assert_eq!(avisos(&planos, &m), vec![Frase::EstaParado { id: "b".into(), por: "a".into() }]);
}

#[test]
fn rodando_segura_area() {
    let planos = vec![q("a", Some("app"), &[]), q("b", Some("app"), &[])];
    let m = escalonar(&planos, &EntradaEsc { iniciados: lista(&["b"]), rodando: lista(&["a"]), ..Default::default() });
    assert_eq!(situacao(&m, "a"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "b"), Situacao::Area);
}

#[test]
fn diario_ajuda_primeiro() {
    let planos = vec![q("a", Some("x"), &[]), q("b", Some("x"), &[]), q("c", Some("y"), &[]), q("d", Some("z"), &[])];
    let m = escalonar(&planos, &EntradaEsc { iniciados: lista(&["a", "b"]), prontos: lista(&["c"]), bloqueados: lista(&["d"]), ..Default::default() });
    let dia = diario(&planos, &m);
    assert_eq!(dia, vec![
        Frase::PediuAjuda { id: "d".into() },
        Frase::Chamou { id: "a".into() },
        Frase::Segurou { id: "b".into(), por: "a".into(), area: "x".into() },
        Frase::Conferiu { id: "c".into() },
    ]);
    crate::idioma::fixar(Idioma::Pt);
    assert_eq!(texto(&dia[0]), "d pediu ajuda", "desligado, ninguém está vendo com o agente");
    assert_eq!(texto_com(&dia[0], true), "d pediu ajuda: vendo com o agente");
    assert_eq!(texto(&dia[1]), "Chamou um agente para a");
    assert_eq!(texto(&dia[2]), "Segurou b: a já mexe em x");
    assert_eq!(texto(&dia[3]), "c: concluído com evidência", "desligado, o app viu a evidência e ninguém conferiu");
    assert_eq!(texto_com(&dia[3], true), "Conferiu c: concluído, com evidência");
}

#[test]
fn ciclo_abab() {
    let (a, b) = (item("a", &["b"]), item("b", &["a"]));
    assert_eq!(achar_ciclo(&[&a, &b]), Some(lista(&["a", "b", "a"])));
}

#[test]
fn sem_ciclo() {
    let (a, b) = (item("a", &[]), item("b", &["a"]));
    assert_eq!(achar_ciclo(&[&a, &b]), None);
}

#[test]
fn area_normalizada_conflita() {
    assert!(areas_conflitam(Some("./App//x/"), Some("app/x")));
    let planos = vec![q("a", Some("./App//x/"), &[]), q("b", Some("app/x"), &[])];
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!(situacao(&m, "b"), Situacao::Area);
}

#[test]
fn area_entre_blueprints() {
    let planos = vec![q("bp1|a", Some("app"), &[]), q("bp2|b", Some("app/barra"), &[]), q("bp2|c", Some("outra"), &[])];
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!(situacao(&m, "bp1|a"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "bp2|b"), Situacao::Area);
    assert_eq!(m["bp2|b"].por.as_deref(), Some("bp1|a"));
    assert_eq!(situacao(&m, "bp2|c"), Situacao::Trabalhando);
    // a chave fica na frase; a pessoa lê só o ref
    crate::idioma::fixar(Idioma::Pt);
    let frase = Frase::EsperaArea { id: "bp2|b".into(), por: "bp1|a".into(), area: "app/barra".into() };
    assert_eq!(avisos(&planos, &m), vec![frase.clone()]);
    assert_eq!(texto(&frase), "b espera a: os dois mexem em app/barra.");
}

#[test]
fn sumiu_segura_area_sem_vaga() {
    let planos = vec![q("a", Some("app"), &[]), q("b", Some("app"), &[]), q("c", Some("outra"), &[])];
    let m = escalonar(&planos, &EntradaEsc {
        iniciados: lista(&["b", "c"]),
        bloqueados: lista(&["a"]),
        seguram_area: lista(&["a"]),
        limite: Some(1),
        ..Default::default()
    });
    assert_eq!(situacao(&m, "a"), Situacao::Bloqueado);
    assert_eq!(situacao(&m, "b"), Situacao::Area, "o trabalho ficou na cópia: a área segue presa");
    assert_eq!(m["b"].por.as_deref(), Some("a"));
    assert_eq!(situacao(&m, "c"), Situacao::Trabalhando, "o agente saiu: a vaga está livre");
}

#[test]
fn ajuda_segura_vaga() {
    let planos = vec![q("a", Some("app"), &[]), q("b", Some("app"), &[]), q("c", Some("outra"), &[])];
    let m = escalonar(&planos, &EntradaEsc {
        iniciados: lista(&["b", "c"]),
        bloqueados: lista(&["a"]),
        rodando: lista(&["a"]),
        causas: BTreeMap::from([("a".to_string(), Causa::Ajuda)]),
        limite: Some(1),
        ..Default::default()
    });
    assert_eq!(situacao(&m, "a"), Situacao::Bloqueado, "bloqueado vence rodando");
    assert_eq!(m["a"].causa, Some(Causa::Ajuda));
    assert_eq!(situacao(&m, "b"), Situacao::Area);
    assert_eq!(situacao(&m, "c"), Situacao::Fila, "quem pediu ajuda segue vivo e ocupa a vaga");
}

#[test]
fn causa_sumiu_vira_agente_sumiu() {
    let planos = vec![q("a", None, &[]), q("b", None, &[]), q("c", None, &[])];
    let causas = BTreeMap::from([("a".to_string(), Causa::Sumiu), ("b".to_string(), Causa::Falhou("base-em-conflito".into())), ("c".to_string(), Causa::Ajuda)]);
    let m = escalonar(&planos, &EntradaEsc { bloqueados: todos(&planos), causas, ..Default::default() });
    assert_eq!(m["a"].causa, Some(Causa::Sumiu));
    let sumiu = Frase::AgenteSumiu { id: "a".into() };
    let falhou = Frase::FalhouAoIniciar { id: "b".into(), motivo: "base-em-conflito".into() };
    assert_eq!(diario(&planos, &m), vec![sumiu.clone(), falhou.clone(), Frase::PediuAjuda { id: "c".into() }]);
    assert_eq!(avisos(&planos, &m), vec![sumiu, falhou], "pedido de ajuda fica no diário");
}

#[test]
fn exclusivo_nao_junta_na_pasta() {
    let pasta = |chave: &str, p: &str| PlanoEsc { exclusivo: Some(p.into()), ..q(chave, None, &[]) };
    let planos = vec![pasta("bp1|a", "/obra"), pasta("bp2|b", "/obra"), pasta("bp2|c", "/outra"), q("bp3|d", None, &[])];
    let m = escalonar(&planos, &EntradaEsc { iniciados: todos(&planos), ..Default::default() });
    assert_eq!(situacao(&m, "bp1|a"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "bp2|b"), Situacao::Area);
    assert_eq!(m["bp2|b"].por.as_deref(), Some("bp1|a"));
    assert_eq!(m["bp2|b"].area.as_deref(), Some("/obra"), "sem área, o que está em disputa é a pasta");
    assert_eq!(situacao(&m, "bp2|c"), Situacao::Trabalhando);
    assert_eq!(situacao(&m, "bp3|d"), Situacao::Trabalhando);
}
