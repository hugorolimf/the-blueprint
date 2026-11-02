//! Testes de plano::comparar: os casos de `design/planejar/comparar.test.mjs`, um a um, e os buracos fechados.
use super::comparar::*;
use super::formato::*;
use serde_json::{json, Value};

fn sem_mais(p: &[Pedaco]) -> String {
    p.iter().filter_map(|x| match x { Pedaco::Igual(s) | Pedaco::Menos(s) => Some(s.as_str()), Pedaco::Mais(_) => None }).collect()
}

fn sem_menos(p: &[Pedaco]) -> String {
    p.iter().filter_map(|x| match x { Pedaco::Igual(s) | Pedaco::Mais(s) => Some(s.as_str()), Pedaco::Menos(_) => None }).collect()
}

fn q(id: &str, texto: &str) -> PlanoItem {
    PlanoItem { id: id.into(), texto: texto.into(), ..Default::default() }
}

fn det(id: &str) -> Detalhe {
    Detalhe { id: id.into(), ..Default::default() }
}

fn acao(id: &str, planos: Vec<PlanoItem>) -> AcaoDoc {
    AcaoDoc { id: id.into(), planos, ..Default::default() }
}

fn prazo(de: &str, ate: &str, planos: Vec<PlanoItem>, detalhes: Vec<Detalhe>) -> AcaoDoc {
    AcaoDoc { id: "1".into(), de: Some(de.into()), ate: Some(ate.into()), planos, detalhes, ..Default::default() }
}

fn a() -> AcaoDoc {
    prazo("2026-11-01", "2026-11-05", vec![q("1.1", "Ler"), q("1.2", "Guardar"), q("1.3", "Apagar")], vec![det("01")])
}

fn b() -> AcaoDoc {
    prazo("2026-11-01", "2026-11-09", vec![q("1.1", "Ler"), q("1.2", "Guardar tudo"), q("1.4", "Mostrar")], vec![det("01"), det("02")])
}

fn plano(acoes: Vec<AcaoDoc>) -> PlanoDoc {
    PlanoDoc { formato: FORMATO.into(), blueprint: "B".into(), projetos: vec![ProjetoDoc { nome: None, acoes }], ..Default::default() }
}

fn tipos(c: &CompAcao) -> Vec<TipoLinha> {
    c.linhas.iter().map(|l| l.tipo).collect()
}

fn doc(json_txt: &str) -> PlanoDoc {
    PlanoDoc::de_json(&serde_json::from_str::<Value>(json_txt).unwrap()).unwrap()
}

fn com_info(hash: &str) -> Detalhe {
    Detalhe { figura: Some("figuras/x.svg".into()), figura_info: Some(FiguraInfo { hash: hash.into(), tipo: "svg".into(), largura: None, altura: None }), ..det("01") }
}

#[test]
fn diff_preserva_os_dois_textos() {
    let d = diff_palavras("Contador na barra", "Contador vermelho na barra");
    assert_eq!(sem_mais(&d), "Contador na barra");
    assert_eq!(sem_menos(&d), "Contador vermelho na barra");
    assert!(d.iter().any(|p| matches!(p, Pedaco::Mais(s) if s.contains("vermelho"))));
}

#[test]
fn diff_junta_vizinhos_e_favorece_mais_no_empate() {
    assert_eq!(diff_palavras("um dois", "um tres"), vec![Pedaco::Igual("um ".into()), Pedaco::Mais("tres".into()), Pedaco::Menos("dois".into())]);
    assert_eq!(diff_palavras("", ""), vec![]);
    assert_eq!(diff_palavras("", "novo texto"), vec![Pedaco::Mais("novo texto".into())]);
    assert_eq!(diff_palavras("velho texto", ""), vec![Pedaco::Menos("velho texto".into())]);
}

#[test]
fn diff_preserva_espacos_e_acentos() {
    let (x, y) = ("  Ação  é\tmaçã ", "Ação é\n\nmaçã  verde");
    let d = diff_palavras(x, y);
    assert_eq!(sem_mais(&d), x);
    assert_eq!(sem_menos(&d), y);
}

#[test]
fn diff_de_texto_enorme_nao_monta_a_tabela() {
    let x: String = (0..200_000).map(|i| format!("a{i} ")).collect();
    let y: String = (0..200_000).map(|i| format!("b{i} ")).collect();
    let d = diff_palavras(&x, &y);
    assert_eq!(sem_mais(&d), x);
    assert_eq!(sem_menos(&d), y);
}

#[test]
fn compara_acao_resumo() {
    assert_eq!(comparar_acao(&a(), &b()).resumo, Resumo { mais: 1, menos: 1, alterados: 2, detalhes: 1, outros: 0 });
}

#[test]
fn ordem_das_linhas() {
    assert_eq!(tipos(&comparar_acao(&a(), &b())), vec![TipoLinha::Prazo, TipoLinha::Igual, TipoLinha::Alterado, TipoLinha::Novo, TipoLinha::Removido]);
}

#[test]
fn igual_nao_mudou() {
    assert!(!comparar_acao(&a(), &a()).mudou);
}

fn os_dois_planos() -> CompPlano {
    let velho = plano(vec![a(), acao("2", vec![])]);
    let novo = plano(vec![b(), acao("2", vec![]), acao("3", vec![q("3.1", "Novo")])]);
    comparar_planos(Some(&velho), &novo)
}

#[test]
fn compara_planos() {
    let r = os_dois_planos();
    assert_eq!(r.novas, ["3"]);
    assert_eq!(r.iguais, ["2"]);
    assert_eq!(r.alteradas.keys().collect::<Vec<_>>(), ["1"]);
    assert!(r.removidas.is_empty());
}

#[test]
fn travados() {
    assert_eq!(planos_travados(&os_dois_planos()), ["1.2", "1.4"]);
}

#[test]
fn sem_anterior_tudo_novo() {
    let r = comparar_planos(None, &plano(vec![a()]));
    assert_eq!(r.novas, ["1"]);
    assert!(r.alteradas.is_empty() && r.iguais.is_empty() && r.removidas.is_empty());
}

fn so_um_plano(antes: PlanoItem, depois: PlanoItem) -> (CompAcao, Vec<String>) {
    let (velho, novo) = (plano(vec![acao("1", vec![antes])]), plano(vec![acao("1", vec![depois])]));
    let r = comparar_planos(Some(&velho), &novo);
    let travados = planos_travados(&r);
    (r.alteradas.get("1").cloned().unwrap_or_default(), travados)
}

#[test]
fn area_mudada_trava() {
    let (c, travados) = so_um_plano(PlanoItem { area: Some("app/barra".into()), ..q("1.1", "Ler") }, PlanoItem { area: Some("app/menu".into()), ..q("1.1", "Ler") });
    assert_eq!(travados, ["1.1"]);
    let l = &c.linhas[0];
    assert_eq!(l.tipo, TipoLinha::Alterado);
    assert_eq!(l.campos, ["area"]);
    assert!(l.palavras.is_empty(), "o texto é igual, então não há diff de palavras");
    assert_eq!(l.antes.as_ref().unwrap().area.as_deref(), Some("app/barra"));
}

#[test]
fn depende_mudado_trava() {
    let (c, travados) = so_um_plano(PlanoItem { depende_de: vec!["1.0".into()], ..q("1.1", "Ler") }, PlanoItem { depende_de: vec!["1.9".into()], ..q("1.1", "Ler") });
    assert_eq!(travados, ["1.1"]);
    assert_eq!(c.linhas[0].campos, ["depende_de"]);
}

#[test]
fn instrucoes_mudadas_travam() {
    let (c, travados) = so_um_plano(PlanoItem { instrucoes: Some("velha".into()), ..q("1.1", "Ler") }, PlanoItem { instrucoes: Some("nova".into()), ..q("1.1", "Ler") });
    assert_eq!(travados, ["1.1"]);
    assert_eq!(c.linhas[0].campos, ["instrucoes"]);
}

#[test]
fn texto_e_campo_mudados_trazem_os_dois() {
    let (c, _) = so_um_plano(q("1.1", "Ler"), PlanoItem { area: Some("x".into()), ..q("1.1", "Ler tudo") });
    assert_eq!(c.linhas[0].campos, ["area"]);
    assert_eq!(sem_menos(&c.linhas[0].palavras), "Ler tudo");
}

#[test]
fn opcional_ausente_na_nova_nao_muda() {
    // `projetar` não apaga campo ausente: se contasse como mudança, o diff nunca sumiria depois de aprovado.
    let antes = PlanoItem { area: Some("app".into()), instrucoes: Some("i".into()), ..q("1.1", "Ler") };
    let (c, travados) = so_um_plano(antes.clone(), q("1.1", "Ler"));
    assert!(!c.mudou && travados.is_empty());
    let (c, travados) = so_um_plano(q("1.1", "Ler"), PlanoItem { area: Some(String::new()), ..q("1.1", "Ler") });
    assert!(!c.mudou && travados.is_empty(), "vazio é o mesmo que nunca ter tido");
    let (c, _) = so_um_plano(antes, PlanoItem { area: Some(String::new()), ..q("1.1", "Ler") });
    assert_eq!(c.linhas[0].campos, ["area"], "apagar é mandar vazio");
}

#[test]
fn reordenar_nao_muda() {
    let velha = AcaoDoc {
        planos: vec![q("1.1", "A"), PlanoItem { depende_de: vec!["x".into(), "y".into()], ..q("1.2", "B") }],
        checkpoints: vec![CheckpointDoc { id: "c1".into(), texto: "k".into(), data: None }, CheckpointDoc { id: "c2".into(), texto: "l".into(), data: None }],
        detalhes: vec![det("01"), det("02")],
        ..acao("1", vec![])
    };
    let mut nova = velha.clone();
    nova.planos.reverse();
    nova.planos[0].depende_de.reverse();
    nova.checkpoints.reverse();
    nova.detalhes.reverse();
    assert!(!comparar_acao(&velha, &nova).mudou);
}

#[test]
fn removida_aparece() {
    let r = comparar_planos(Some(&plano(vec![a(), acao("2", vec![q("2.1", "X")])])), &plano(vec![a()]));
    assert_eq!(r.removidas, ["2"]);
    assert_eq!(r.iguais, ["1"]);
    assert!(r.alteradas.is_empty() && r.novas.is_empty());
    assert!(planos_travados(&r).is_empty(), "plano que saiu não tem o que travar");
}

#[test]
fn figura_trocada_conta() {
    let velha = AcaoDoc { detalhes: vec![com_info("aaa")], ..acao("1", vec![]) };
    let nova = AcaoDoc { detalhes: vec![com_info("bbb")], ..acao("1", vec![]) };
    let c = comparar_acao(&velha, &nova);
    assert_eq!(c.detalhes_mudados, ["01"]);
    assert_eq!(c.resumo.detalhes, 1);
    assert!(c.mudou && c.detalhes_novos.is_empty() && c.detalhes_removidos.is_empty());
    assert!(!comparar_acao(&velha, &velha).mudou);
}

#[test]
fn figura_com_mesmo_hash_nao_conta_mesmo_renomeada() {
    let velha = AcaoDoc { detalhes: vec![com_info("aaa")], ..acao("1", vec![]) };
    let mut nova = velha.clone();
    nova.detalhes[0].figura = Some("figuras/outro.svg".into());
    assert!(!comparar_acao(&velha, &nova).mudou);
    nova.detalhes[0].figura_info = None;
    assert_eq!(comparar_acao(&velha, &nova).detalhes_mudados, ["01"], "sem hash dos dois lados, vale o caminho");
}

#[test]
fn detalhe_com_legenda_nova_conta_e_detalhe_removido_conta() {
    let velha = AcaoDoc { detalhes: vec![det("01"), det("02")], ..acao("1", vec![]) };
    let nova = AcaoDoc { detalhes: vec![Detalhe { legenda: Some("outra".into()), ..det("01") }], ..acao("1", vec![]) };
    let c = comparar_acao(&velha, &nova);
    assert_eq!(c.detalhes_mudados, ["01"]);
    assert_eq!(c.detalhes_removidos, ["02"]);
    assert_eq!(c.resumo.detalhes, 2);
    assert!(c.mudou);
}

#[test]
fn outros_campos_da_acao() {
    let velha = AcaoDoc { nome: "N".into(), texto: Some("t".into()), ..acao("1", vec![]) };
    let nova = AcaoDoc {
        nome: "M".into(),
        texto: Some("u".into()),
        checkpoints: vec![CheckpointDoc { id: "c".into(), texto: "k".into(), data: None }],
        tabelas: vec![Tabela { colunas: vec!["a".into()], ..Default::default() }],
        ..acao("1", vec![])
    };
    let c = comparar_acao(&velha, &nova);
    assert_eq!(c.outros, ["nome", "texto", "checkpoints", "tabelas"]);
    assert_eq!(c.resumo.outros, 4);
    assert!(c.mudou);
    let r = comparar_planos(Some(&plano(vec![velha])), &plano(vec![nova]));
    assert_eq!(r.alteradas.keys().collect::<Vec<_>>(), ["1"]);
    assert!(planos_travados(&r).is_empty(), "mudar o nome da ação não trava plano nenhum");
}

#[test]
fn sair_de_data_para_sem_data_e_prazo() {
    let sem = acao("1", vec![]);
    let com = prazo("2026-01-01", "2026-01-02", vec![], vec![]);
    assert_eq!(tipos(&comparar_acao(&com, &sem)), [TipoLinha::Prazo]);
    assert_eq!(tipos(&comparar_acao(&sem, &com)), [TipoLinha::Prazo]);
}

#[test]
fn exemplo_monitor_altera_1_2() {
    let anterior = doc(include_str!("fixtures/monitor/anterior.json"));
    let atual = doc(include_str!("fixtures/monitor/plano.json"));
    let r = comparar_planos(Some(&anterior), &atual);
    assert_eq!(r.novas, ["1.3"]);
    assert_eq!(r.iguais, ["1.1"]);
    assert_eq!(r.alteradas.keys().collect::<Vec<_>>(), ["1.2"]);
    assert!(r.removidas.is_empty());
    let c = &r.alteradas["1.2"];
    assert_eq!(tipos(c), [TipoLinha::Prazo, TipoLinha::Alterado, TipoLinha::Igual, TipoLinha::Novo, TipoLinha::Removido]);
    assert_eq!(c.resumo, Resumo { mais: 1, menos: 1, alterados: 2, detalhes: 1, outros: 0 });
    assert_eq!(c.detalhes_novos.iter().map(|d| d.id.as_str()).collect::<Vec<_>>(), ["02"]);
    assert_eq!(c.linhas[1].plano.as_ref().unwrap().id, "1.2.1");
    assert_eq!(sem_mais(&c.linhas[1].palavras), "Somar as leituras");
    assert_eq!(sem_menos(&c.linhas[1].palavras), "Somar as leituras por sala");
    assert_eq!(c.linhas[4].plano.as_ref().unwrap().id, "1.2.4");
    assert_eq!(planos_travados(&r), ["1.2.1", "1.2.3"]);
}

#[test]
fn id_gerado_nao_e_mudanca() {
    let v: Value = json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [{"id": "1", "nome": "A", "planos": ["Fazer"]}]}]});
    let novo = PlanoDoc::de_json(&v).unwrap();
    let mut velho = novo.clone();
    velho.projetos[0].acoes[0].planos[0].id_gerado = false;
    assert_eq!(comparar_planos(Some(&velho), &novo).iguais, ["1"]);
}
