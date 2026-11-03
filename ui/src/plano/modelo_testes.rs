//! Os testes de `plano::modelo`, com o JSON gravado pelo teste de ponta a ponta do servidor (`src/mcp/planejar/fluxo_tests.rs`).

use super::*;

const FOLHA: &str = include_str!("fixtures/folha.json");
const AGENTES: &str = include_str!("fixtures/agentes.json");
const CHEGOU: &str = include_str!("fixtures/chegou.json");

/// Decodifica e escreve de volta: dá o mesmo JSON, então nenhum campo do servidor ficou sem lugar no modelo.
fn sem_perder_campo<T: serde::de::DeserializeOwned + Serialize>(texto: &str) -> T {
    let cru: Value = serde_json::from_str(texto).unwrap();
    let lido: T = serde_json::from_value(cru.clone()).unwrap();
    assert_eq!(serde_json::to_value(&lido).unwrap(), cru);
    lido
}

#[test]
fn folha_decodifica() {
    let f: Folha = sem_perder_campo(FOLHA);
    assert_eq!((f.blueprint.nome.as_str(), f.limite, f.sem_pasta), ("Monitor de consumo", 4, false));
    assert!(f.escrita.is_none());
    assert_eq!(f.pendente, Some(Pendente { numero: 2, agente: "exemplo".into() }));
    assert_eq!(f.numeros, NumerosFolha { aprovar: 2, diffs: 1, novas: 1 });
    assert_eq!(f.decisoes[0].revisao.as_ref().unwrap().estado, EstadoRevisao::Aprovada);
    assert_eq!(f.perguntas[0].recomendada, "15 min");

    let acoes = &f.projetos[0].acoes;
    assert_eq!(acoes.iter().map(|a| a.referencia.as_str()).collect::<Vec<_>>(), ["1.1", "1.2", "1.3"]);
    assert!(acoes[2].nova);
    assert_eq!(acoes[0].numeros, NumerosAcao { planos: 2, prontos: 2, detalhes: 0, decisoes: 1, aprovar: 0 });

    let painel = &acoes[1];
    assert_eq!(painel.planos.iter().map(|p| p.situacao).collect::<Vec<_>>(), [Situacao::Trabalhando, Situacao::Dependencia, Situacao::Nao]);
    assert_eq!(painel.planos[1].por, "1.2.1");
    assert_eq!(painel.detalhes[0].figura.as_ref().map(|x| (x.largura, x.altura)), Some((Some(900), Some(420))));
    let diff = painel.diff.as_ref().unwrap();
    assert_eq!(diff.resumo, Resumo { mais: 1, menos: 1, alterados: 1, detalhes: 1, outros: 0 });
    assert_eq!(diff.linhas.iter().map(|l| l.tipo).collect::<Vec<_>>(), [TipoLinha::Alterado, TipoLinha::Igual, TipoLinha::Novo, TipoLinha::Removido]);
    assert_eq!(diff.linhas[0].palavras[1], Palavra { t: TipoPalavra::Mais, s: " por sala".into() });
    assert_eq!(diff.detalhes_novos[0].id, "02");
}

#[test]
fn agentes_decodifica() {
    let a: Agentes = sem_perder_campo(AGENTES);
    assert_eq!(a.limite, 4);
    assert_eq!(a.numeros, NumerosAgentes { trabalhando: 1, fila: 0, esperando: 1, parados: 0, prontos: 2 });
    assert!(!a.orquestrador.ligado);
    assert_eq!(a.orquestrador.agora.len(), 3);
    assert_eq!(a.planos.iter().map(|p| p.situacao).collect::<Vec<_>>(), [Situacao::Pronto, Situacao::Pronto, Situacao::Trabalhando, Situacao::Dependencia]);
    assert!(a.planos[0].pode_integrar && !a.planos[0].pode_cancelar);
    assert_eq!((a.planos[0].destino.as_str(), a.planos[3].por.as_str()), ("main", "1.2.1"));
}

#[test]
fn chegou_decodifica() {
    let c: Chegou = sem_perder_campo(CHEGOU);
    assert_eq!((c.numero, c.origem.as_str(), c.iguais), (2, "salvar", 1));
    assert!(c.novas[0].pode_colocar);
    assert_eq!(c.alteradas[0].resumo.alterados, 2);
}

#[test]
fn campo_ausente_e_situacao_nova_nao_derrubam() {
    let p: Plano = serde_json::from_str(r#"{"id":"x","situacao":"inventada"}"#).unwrap();
    assert_eq!((p.id.as_str(), p.situacao, p.feito), ("x", Situacao::Outra, false));
    let t: Tabela = serde_json::from_str(r#"{"colunas":["a"],"linhas":[["1"]]}"#).unwrap();
    assert_eq!((t.titulo.as_str(), t.linhas.len(), t.larguras.len()), ("", 1, 0));
}
