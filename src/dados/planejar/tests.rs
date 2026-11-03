//! Testes de dados::planejar: as tabelas fora da árvore e a gravação da execução na árvore. Os de `execucoes` ficam em `tests/execucoes.rs`.
use super::{Lancamento, NovaExecucao, NovoRecado};
use crate::contexto::Contexto;
use crate::dados::tests::exemplo;
use crate::dados::Dados;
use crate::erro::Erro;
use serde_json::{json, Value};

mod execucoes;

const BP: &str = "p1";

fn dados() -> Dados {
    Dados::em_memoria().unwrap()
}

fn com_exemplo() -> (Dados, Contexto) {
    let mut d = dados();
    let ctx = Contexto::local();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    (d, ctx)
}

fn abrir(d: &mut Dados, bp: &str) -> (i64, String) {
    d.plano_abrir("sm", bp, "escrita", "Claude", &json!({"formato": "blueprint.plano/1"}), &[], 1000).unwrap()
}

fn nova<'a>(plano: &'a str) -> NovaExecucao<'a> {
    NovaExecucao { workspace_id: "sm", blueprint_id: BP, acao_id: "f1", plano_id: plano, plano_ref: plano, area: "app", pasta_principal: "/tmp/p", marcada_em: 1000 }
}

fn lancamento(agora_ms: i64) -> Lancamento<'static> {
    Lancamento { area: "app", pasta_trabalho: "/tmp/copias/Abc", repositorio: "/tmp/p/.git", ramo: "agente/t2", base: "HEAD", agora_ms }
}

/// Uma execução já em `rodando`, sem o invólucro ter partido.
fn rodando(d: &mut Dados, plano: &str) -> String {
    let id = d.execucao_criar(&nova(plano)).unwrap();
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(d.execucao_lancar(&id, &lancamento(2000)).unwrap());
    id
}

fn plano<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["blueprints"][0]["acoes"][0]["planos"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap()
}

fn checkpoint<'a>(v: &'a Value, id: &str) -> &'a Value {
    v["blueprints"][0]["acoes"][0]["checkpoints"].as_array().unwrap().iter().find(|t| t["id"] == id).unwrap()
}

#[test]
fn versao_uma_escrita() {
    let mut d = dados();
    let (numero, escrita) = abrir(&mut d, BP);
    assert_eq!((numero, escrita.len()), (1, 32));
    assert!(d.plano_abrir("sm", BP, "salvar", "", &json!({}), &[], 1).is_err(), "a segunda escrita aberta é recusada pelo índice");
    assert_eq!(abrir(&mut d, "outro").0, 1, "em outro blueprint pode");
    d.plano_gravar(&escrita, &json!({"resumo": "novo"}), &["1".into()], &["2".into()], &["3".into()], 5000).unwrap();
    let aberta = d.plano_aberta(BP).unwrap().unwrap();
    assert_eq!(aberta.escrita_id, escrita);
    assert_eq!((aberta.plano["resumo"].as_str(), aberta.aguardando, aberta.tocadas, aberta.removidas, aberta.atualizada_em), (Some("novo"), vec!["1".to_string()], vec!["2".to_string()], vec!["3".to_string()], 5000));
    assert_eq!(d.plano_escrita(&escrita).unwrap().unwrap().situacao, "escrevendo");
    assert!(d.plano_pendente(BP).unwrap().is_none());
    d.plano_descartar(&escrita).unwrap();
    assert!(d.plano_aberta(BP).unwrap().is_none());
    assert!(matches!(d.plano_descartar(&escrita), Err(Erro::NaoEncontrado(_))));
    assert!(matches!(d.plano_gravar(&escrita, &json!({}), &[], &[], &[], 1), Err(Erro::NaoEncontrado(_))));
}

#[test]
fn fechar_resolve_anterior() {
    let mut d = dados();
    let (_, primeira) = abrir(&mut d, BP);
    assert_eq!(d.plano_fechar(&primeira, 2000).unwrap(), 1);
    assert_eq!(d.plano_pendente(BP).unwrap().unwrap().fechada_em, Some(2000));
    let (_, segunda) = abrir(&mut d, BP);
    assert_eq!(d.plano_pendente(BP).unwrap().unwrap().numero, 1, "enquanto escreve, a anterior segue pendente");
    assert_eq!(d.plano_fechar(&segunda, 3000).unwrap(), 2);
    assert_eq!(d.plano_pendente(BP).unwrap().unwrap().numero, 2);
    assert_eq!(d.plano_escrita(&primeira).unwrap().unwrap().situacao, "resolvida");
    assert!(matches!(d.plano_fechar(&primeira, 4000), Err(Erro::NaoEncontrado(_))), "fechada não fecha de novo");
    d.plano_resolver(BP, 2).unwrap();
    assert!(d.plano_pendente(BP).unwrap().is_none());
}

#[test]
fn reter_20() {
    let mut d = dados();
    for _ in 0..25 {
        let (_, e) = abrir(&mut d, BP);
        d.plano_fechar(&e, 2000).unwrap();
    }
    let (numero, _) = abrir(&mut d, BP);
    assert_eq!(numero, 26);
    let contar = |d: &Dados, tabela: &str| -> i64 { d.conexao().query_row(&format!("SELECT COUNT(*) FROM {tabela} WHERE blueprint_id = 'p1'"), [], |l| l.get(0)).unwrap() };
    assert_eq!(contar(&d, "plano_versoes"), 20);
    let menor: i64 = d.conexao().query_row("SELECT MIN(numero) FROM plano_versoes", [], |l| l.get(0)).unwrap();
    assert_eq!(menor, 7);
    assert_eq!(d.plano_pendente(BP).unwrap().unwrap().numero, 25, "só as resolvidas saem");
    assert_eq!(d.plano_aberta(BP).unwrap().unwrap().numero, 26);
    let figura = |numero: i64| super::FiguraLinha { blueprint_id: BP.into(), numero, caminho: "a.svg".into(), hash: "h".into(), tipo: "svg".into(), tamanho: 3, largura: Some(10), altura: None };
    d.figura_registrar(&figura(7)).unwrap();
    d.figura_registrar(&figura(26)).unwrap();
    d.plano_fechar(&d.plano_aberta(BP).unwrap().unwrap().escrita_id, 3000).unwrap();
    abrir(&mut d, BP);
    assert!(d.figura_de(BP, 7, "a.svg").unwrap().is_none(), "a figura sai com a versão");
    assert_eq!(d.figura_de(BP, 26, "a.svg").unwrap().unwrap().largura, Some(10));
    assert_eq!(d.figura_por_hash("h").unwrap().unwrap().numero, 26);
}

#[test]
fn sobrevive_ao_gravar_tudo() {
    let (mut d, ctx) = com_exemplo();
    let (_, escrita) = abrir(&mut d, BP);
    d.revisao_gravar(BP, "d:a", 1, "aprovada", "", "h", 1).unwrap();
    let id = rodando(&mut d, "t2");
    d.recado_gravar(&NovoRecado { workspace_id: "sm", de: "usuario", texto: "oi", em: 1, ..Default::default() }).unwrap();
    d.config_gravar("limite:sm", "3").unwrap();
    d.orq_ligar("sm", true).unwrap();
    d.gravar_tudo(&ctx, &exemplo()).unwrap();
    assert_eq!(d.plano_aberta(BP).unwrap().unwrap().escrita_id, escrita);
    assert_eq!(d.revisoes_do(BP).unwrap().len(), 1);
    assert_eq!(d.execucao_de(&id).unwrap().unwrap().situacao, "rodando");
    assert_eq!(d.recados_do("sm", 20).unwrap().len(), 1);
    assert_eq!(d.config_ler("limite:sm").unwrap().as_deref(), Some("3"));
    assert!(d.orq_ler("sm").unwrap().ligado);
    d.gravar_tudo(&ctx, &json!({"titulo": "vazio"})).unwrap();
    assert_eq!(d.plano_aberta(BP).unwrap().unwrap().escrita_id, escrita, "nem com a árvore toda apagada");
    assert_eq!(d.execucao_de(&id).unwrap().unwrap().situacao, "rodando");
}

#[test]
fn limpar_por_blueprint() {
    let mut d = dados();
    let (n, _) = abrir(&mut d, BP);
    abrir(&mut d, "outro");
    let figura = |bp: &str| super::FiguraLinha { blueprint_id: bp.into(), numero: n, caminho: "a.png".into(), hash: "h".into(), tipo: "png".into(), tamanho: 1, largura: None, altura: None };
    for bp in [BP, "outro"] {
        d.figura_registrar(&figura(bp)).unwrap();
        d.revisao_gravar(bp, "d:a", n, "aprovada", "", "h", 1).unwrap();
    }
    let execucao = d.execucao_criar(&nova("t1")).unwrap();
    d.planejar_limpar(BP).unwrap();
    assert!(d.plano_aberta(BP).unwrap().is_none());
    assert!(d.figura_de(BP, n, "a.png").unwrap().is_none());
    assert!(d.revisoes_do(BP).unwrap().is_empty());
    assert!(d.execucao_de(&execucao).unwrap().is_some(), "execução é cancelada pelo motor, não apagada");
    assert!(d.plano_aberta("outro").unwrap().is_some());
    assert!(d.figura_de("outro", n, "a.png").unwrap().is_some());
    assert_eq!(d.revisoes_do("outro").unwrap().len(), 1);
}

#[test]
fn concluir_sobe_revisao() {
    let (mut d, ctx) = com_exemplo();
    assert_eq!(plano(d.ler_tudo(&ctx).unwrap(), "t3")["novo"], json!({"a": [1, 2, {"b": null}]}));
    let antes = d.revisao_dados();
    d.concluir_por_execucao("t2", "testes passando", "2026-11-01T10:00:00Z").unwrap();
    assert_eq!(d.revisao_dados(), antes + 1);
    let t2 = plano(d.ler_tudo(&ctx).unwrap(), "t2").clone();
    assert_eq!(t2["feito"], true);
    assert_eq!(t2["agente"]["estado"], "concluido");
    assert_eq!(t2["agente"]["comentarios"][0]["tipo"], "relatorio");
    assert_eq!(t2["agente"]["comentarios"][0]["texto"], "testes passando");

    // plano sem `agente` nasce com ele; `feito` sobrevive a um PUT, porque é da árvore
    d.concluir_por_execucao("t3", "ok", "2026-11-01T11:00:00Z").unwrap();
    let tudo = d.ler_tudo(&ctx).unwrap().clone();
    assert_eq!(plano(&tudo, "t3")["novo"], json!({"a": [1, 2, {"b": null}]}), "campo alheio fica");
    d.gravar_tudo(&ctx, &tudo).unwrap();
    assert_eq!(plano(d.ler_tudo(&ctx).unwrap(), "t2")["feito"], true);

    let antes = d.revisao_dados();
    d.desfazer_por_execucao("t2", "a tela quebrou", "2026-11-01T12:00:00Z").unwrap();
    assert_eq!(d.revisao_dados(), antes + 1);
    let t2 = plano(d.ler_tudo(&ctx).unwrap(), "t2").clone();
    assert_eq!(t2["feito"], false);
    assert_eq!(t2["agente"]["estado"], "pendente");
    let comentarios = t2["agente"]["comentarios"].as_array().unwrap();
    assert_eq!(comentarios.len(), 2);
    assert!(comentarios[1]["texto"].as_str().unwrap().contains("a tela quebrou"));

    assert!(matches!(d.concluir_por_execucao("nao", "x", "e"), Err(Erro::NaoEncontrado(_))));
    assert!(matches!(d.desfazer_por_execucao("nao", "x", "e"), Err(Erro::NaoEncontrado(_))));
}

#[test]
fn checkpoint_por_execucao() {
    let (mut d, ctx) = com_exemplo();
    let antes = d.revisao_dados();
    d.checkpoint_por_execucao("m2", true, "no ar", "2026-11-30T10:00:00Z").unwrap();
    assert_eq!(d.revisao_dados(), antes + 1);
    let m2 = checkpoint(d.ler_tudo(&ctx).unwrap(), "m2").clone();
    assert_eq!((&m2["feito"], &m2["evidencia"], &m2["nome"]), (&json!(true), &json!("no ar"), &json!("Fim")));
    d.checkpoint_por_execucao("m2", false, "caiu", "2026-11-30T11:00:00Z").unwrap();
    let m2 = checkpoint(d.ler_tudo(&ctx).unwrap(), "m2").clone();
    assert_eq!(m2["feito"], false);
    assert!(m2.get("evidencia").is_none(), "{m2}");
    assert!(matches!(d.checkpoint_por_execucao("nao", true, "x", "e"), Err(Erro::NaoEncontrado(_))));
}

#[test]
fn revisao_desfazer() {
    let mut d = dados();
    d.revisao_gravar(BP, "det:1/a", 1, "revisar", "mude o título", "h1", 10).unwrap();
    d.revisao_gravar(BP, "det:1/a", 2, "aprovada", "", "h2", 20).unwrap();
    d.revisao_gravar(BP, "d:b", 2, "reprovada", "", "h3", 30).unwrap();
    assert!(d.revisao_gravar(BP, "d:c", 2, "pendente", "", "", 30).is_err(), "pendente não é estado: é apagar");
    let linhas = d.revisoes_do(BP).unwrap();
    assert_eq!(linhas.len(), 2, "uma por peça");
    let a = linhas.iter().find(|r| r.peca == "det:1/a").unwrap();
    assert_eq!((a.numero, a.estado.as_str(), a.observacao.as_str(), a.hash_peca.as_str(), a.em), (2, "aprovada", "", "h2", 20));
    d.revisao_limpar(BP, "det:1/a").unwrap();
    let linhas = d.revisoes_do(BP).unwrap();
    assert_eq!(linhas.iter().map(|r| r.peca.as_str()).collect::<Vec<_>>(), ["d:b"]);
}

#[test]
fn recados_reter_200() {
    let mut d = dados();
    for i in 0..205 {
        let texto = format!("r{i}");
        d.recado_gravar(&NovoRecado { workspace_id: "sm", execucao_id: if i == 204 { "e1" } else { "" }, de: "agente", texto: &texto, em: i, ..Default::default() }).unwrap();
    }
    d.recado_gravar(&NovoRecado { workspace_id: "outro", de: "usuario", texto: "fica", em: 1, ..Default::default() }).unwrap();
    assert_eq!(d.recados_do("sm", 1000).unwrap().len(), 200);
    let ultimos = d.recados_do("sm", 20).unwrap();
    assert_eq!((ultimos.len(), ultimos[0].texto.as_str(), ultimos[19].texto.as_str()), (20, "r185", "r204"), "os últimos, em ordem");
    assert_eq!(d.recados_para("e1").unwrap()[0].texto, "r204");
    assert_eq!(d.recados_do("outro", 20).unwrap().len(), 1, "cada workspace com os seus");
    assert!(d.recado_gravar(&NovoRecado { workspace_id: "sm", de: "estranho", texto: "x", em: 1, ..Default::default() }).is_err());
}

#[test]
fn orquestrador_junta_acorda_e_conta_a_hora() {
    let mut d = dados();
    assert!(!d.orq_ler("sm").unwrap().ligado, "sem linha é desligado");
    assert_eq!(d.orq_inicio("sm", 1000).unwrap(), Some(vec![]));
    d.orq_fim("sm", 1500).unwrap();
    d.orq_ligar("sm", true).unwrap();
    d.orq_pedir("sm", "pronta:a", false).unwrap();
    d.orq_pedir("sm", "pronta:a", false).unwrap();
    d.orq_pedir("sm", "mensagem", true).unwrap();
    d.orq_pedir("sm", "ajuda:b", false).unwrap();
    let o = d.orq_ler("sm").unwrap();
    assert_eq!((o.motivos.clone(), o.urgente, o.ligado), (vec!["pronta:a".to_string(), "mensagem".into(), "ajuda:b".into()], true, true));
    assert_eq!(d.orq_inicio("sm", 2000).unwrap().unwrap().len(), 3);
    assert_eq!(d.orq_inicio("sm", 2100).unwrap(), None, "já rodando");
    let o = d.orq_ler("sm").unwrap();
    assert!(o.rodando && !o.urgente && o.motivos.is_empty() && o.ligado);
    assert_eq!((o.na_hora, o.hora_inicio, o.ultimo_em), (2, Some(1000), Some(2000)));
    d.orq_fim("sm", 3000).unwrap();
    assert_eq!(d.orq_ler("sm").unwrap().ultimo_em, Some(3000));
    let depois = 1000 + 3_600_000;
    assert_eq!(d.orq_ler("sm").unwrap().na_hora_em(depois), 0, "a hora passou");
    d.orq_inicio("sm", depois).unwrap();
    let o = d.orq_ler("sm").unwrap();
    assert_eq!((o.na_hora, o.hora_inicio), (1, Some(depois)));
    d.orq_zerar_rodando().unwrap();
    assert!(!d.orq_ler("sm").unwrap().rodando);
}

#[test]
fn config_le_e_substitui() {
    let mut d = dados();
    assert_eq!(d.config_ler("comando_agente").unwrap(), None);
    d.config_gravar("comando_agente", "[\"claude\"]").unwrap();
    d.config_gravar("comando_agente", "[\"codex\"]").unwrap();
    assert_eq!(d.config_ler("comando_agente").unwrap().as_deref(), Some("[\"codex\"]"));
}
