//! `gesto/revisar` e `gesto/colocar`: a árvore é a base do diff, e o clique vem amarrado a `numero` e `hash`.
use super::{cena, json_de, par};
use crate::mcp::planejar::revisao::hash_decisao;
use crate::plano::formato::{AcaoDoc, PlanoDoc};
use serde_json::{json, Value};

fn trocar_texto(doc: &mut PlanoDoc, r: &str, texto: &str) {
    let q = doc.projetos.iter_mut().flat_map(|p| p.acoes.iter_mut()).flat_map(|a| a.planos.iter_mut()).find(|q| q.id == r).unwrap();
    q.texto = texto.into();
}

#[test]
fn aprovar_entra_na_arvore() {
    let c = cena();
    let n = c.pendente("escrita", |d| {
        trocar_texto(d, "1.1", "Fazer A melhor");
        trocar_texto(d, "2.1", "Fazer C melhor");
    });
    let antes = c.versao();
    let r = c.revisar("1", n, "aprovada", json!({}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(c.plano(0, 0)["texto"], "Fazer A melhor");
    assert_eq!(c.plano(1, 0)["texto"], "Fazer C", "só a ação aprovada");
    assert_eq!(c.plano(0, 1)["feito"], true, "feito de quem não mudou fica");
    assert_eq!(c.plano(0, 0)["id"], "q1", "reusa o id real");
    assert!(c.versao() > antes, "mudou a árvore: sobe a versão");
    assert!(c.pendente_existe(), "ainda falta a ação 2");
    assert_eq!(json_de(&c.revisar("2", n, "aprovada", json!({})))["resolvida"], true);
    assert_eq!(c.plano(1, 0)["texto"], "Fazer C melhor");
    assert!(!c.pendente_existe(), "nada falta");
}

#[test]
fn editar_troca_o_texto() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Texto do agente"));
    let r = c.revisar("1", n, "aprovada", json!({"textos": {"nao-existe": "x"}}));
    assert_eq!(par(&r), (422, "texto-invalido".to_string()));
    let r = c.revisar("1", n, "aprovada", json!({"textos": {"1.1": "Texto do usuário"}}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(c.plano(0, 0)["texto"], "Texto do usuário");
    assert!(!c.pendente_existe(), "o texto editado é o aprovado: não sobra diff");
}

#[test]
fn editar_mantem_a_versao_certa_com_outra_acao_pendente() {
    let c = cena();
    let n = c.pendente("escrita", |d| {
        trocar_texto(d, "1.1", "Texto do agente");
        trocar_texto(d, "2.1", "Outro");
    });
    assert_eq!(c.revisar("1", n, "aprovada", json!({"textos": {"1.1": "Texto do usuário"}})).status, 200);
    assert!(c.pendente_existe());
    assert_eq!(c.acao_nova("1").unwrap().planos[0].texto, "Texto do usuário", "a versão não volta a mostrar diff da ação 1");
}

#[test]
fn reprovar_nao_toca_arvore() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Ruim"));
    let (arvore, versao) = (c.arvore(), c.versao());
    let r = c.revisar("1", n, "reprovada", json!({}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(c.arvore(), arvore);
    assert_eq!(c.versao(), versao, "fora da árvore: a versão dos dados não sobe");
    let d = c.e.dados.lock().unwrap();
    let rev = d.revisoes_do("b1").unwrap();
    assert_eq!((rev[0].peca.as_str(), rev[0].estado.as_str()), ("diff:1", "reprovada"));
    assert_eq!(d.orq_ler("sm").unwrap().motivos.len(), 1, "acorda o orquestrador");
    assert!(d.plano_pendente("b1").unwrap().is_none(), "reprovada é decisão: a próxima escrita não parte do texto recusado");
}

#[test]
fn revisar_grava_observacao_e_deixa_pendente() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Quase"));
    assert_eq!(c.revisar("1", n, "revisar", json!({"observacao": "Falta o teste"})).status, 200);
    assert!(c.pendente_existe());
    let d = c.e.dados.lock().unwrap();
    assert_eq!(d.revisoes_do("b1").unwrap()[0].observacao, "Falta o teste");
    assert_eq!(d.orq_ler("sm").unwrap().motivos.len(), 1);
}

#[test]
fn remover_com_execucao_409() {
    let c = cena();
    let n = c.pendente("escrita", |d| {
        d.projetos[0].acoes.retain(|a| a.id != "2");
        d.projetos[0].acoes[0].planos.retain(|q| q.id != "1.1");
    });
    let viva = c.execucao("q3", "2.1");
    assert_eq!(par(&c.revisar("2", n, "aprovada", json!({}))), (409, "rodando".to_string()), "ação inteira");
    c.execucao("q1", "1.1");
    assert_eq!(par(&c.revisar("1", n, "aprovada", json!({}))), (409, "rodando".to_string()), "plano da ação");
    assert_eq!(c.arvore()["acoes"].as_array().unwrap().len(), 2);
    c.gesto("cancelar", json!({"execucaoId": viva}));
    assert_eq!(c.revisar("2", n, "aprovada", json!({})).status, 200, "cancelado, sai");
    assert_eq!(c.arvore()["acoes"].as_array().unwrap().len(), 1);
}

#[test]
fn aprovar_com_rodando_409() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Novo"));
    let id = c.execucao("q1", "1.1");
    assert_eq!(par(&c.revisar("1", n, "aprovada", json!({}))), (409, "rodando".to_string()));
    assert_eq!(c.plano(0, 0)["texto"], "Fazer A");
    assert_eq!(c.gesto("cancelar", json!({"execucaoId": id})).status, 200);
    assert_eq!(c.revisar("1", n, "aprovada", json!({})).status, 200);
    assert_eq!(c.plano(0, 0)["texto"], "Novo");
}

#[test]
fn execucao_em_plano_que_nao_mudou_nao_segura() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.2", "Outro texto"));
    c.execucao("q1", "1.1");
    assert_eq!(c.revisar("1", n, "aprovada", json!({})).status, 200);
}

#[test]
fn aprovar_versao_velha_409() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Novo"));
    assert_eq!(par(&c.revisar("1", n + 1, "aprovada", json!({}))), (409, "plano-mudou".to_string()), "numero errado");
    let r = c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": "b1", "peca": "diff:1", "numero": n, "hash": "0".repeat(64), "estado": "aprovada"}));
    assert_eq!(par(&r), (409, "plano-mudou".to_string()), "hash errado");
    let n2 = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Mais novo"));
    assert_ne!(n, n2);
    assert_eq!(par(&c.revisar("1", n, "aprovada", json!({}))), (409, "plano-mudou".to_string()), "fechou outra versão entre o desenho e o clique");
    assert_eq!(c.plano(0, 0)["texto"], "Fazer A");
    assert!(c.e.dados.lock().unwrap().revisoes_do("b1").unwrap().is_empty());
    assert_eq!(par(&c.revisar("2", n2, "aprovada", json!({}))), (409, "plano-mudou".to_string()), "ação sem diff");
}

#[test]
fn feito_alterado_reabre() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.2", "Fazer B diferente"));
    let r = c.revisar("1", n, "aprovada", json!({}));
    assert_eq!(json_de(&r)["reabertos"], json!(["1.2"]));
    let q = c.plano(0, 1);
    assert_eq!(q["feito"], false);
    assert_eq!(q["agente"]["estado"], "pendente");
}

#[test]
fn feito_com_dependencia_mudada_nao_reabre() {
    let c = cena();
    let n = c.pendente("escrita", |d| d.projetos[0].acoes[0].planos[1].depende_de = vec!["1.1".into()]);
    assert_eq!(json_de(&c.revisar("1", n, "aprovada", json!({})))["reabertos"], json!([]));
    assert_eq!(c.plano(0, 1)["feito"], true);
}

#[test]
fn projecao_com_ciclo_409() {
    let c = cena();
    let n = c.pendente("escrita", |d| d.projetos[0].acoes[0].planos[0].depende_de = vec!["2.1".into()]);
    let (arvore, versao) = (c.arvore(), c.versao());
    assert_eq!(par(&c.revisar("1", n, "aprovada", json!({}))), (409, "ciclo".to_string()));
    assert_eq!(c.arvore(), arvore);
    assert_eq!(c.versao(), versao);
}

#[test]
fn dependencia_que_sumiu_so_avisa() {
    let c = cena();
    let n = c.pendente("escrita", |d| d.projetos[0].acoes[0].planos.retain(|q| q.id != "1.1"));
    let r = c.revisar("1", n, "aprovada", json!({}));
    assert_eq!(r.status, 200);
    assert!(json_de(&r)["avisos"].to_string().contains("1.1"));
}

#[test]
fn desfazer() {
    let c = cena();
    let decisao = c.doc().decisoes[0].clone();
    let corpo = |estado: &str, hash: &str| json!({"workspaceId": "sm", "blueprintId": "b1", "peca": "d:d1", "numero": 0, "hash": hash, "estado": estado});
    let hash = hash_decisao(&decisao);
    assert_eq!(par(&c.gesto("revisar", corpo("aprovada", "velho"))), (409, "plano-mudou".to_string()));
    assert_eq!(c.gesto("revisar", corpo("aprovada", &hash)).status, 200);
    {
        let d = c.e.dados.lock().unwrap();
        let rev = d.revisoes_do("b1").unwrap();
        assert_eq!((rev[0].estado.as_str(), rev[0].hash_peca.as_str()), ("aprovada", hash.as_str()));
        assert!(d.orq_ler("sm").unwrap().motivos.is_empty(), "aprovar não acorda");
    }
    assert_eq!(c.gesto("revisar", corpo("pendente", &hash)).status, 200);
    assert!(c.e.dados.lock().unwrap().revisoes_do("b1").unwrap().is_empty());
    assert_eq!(c.gesto("revisar", corpo("reprovada", &hash)).status, 200, "reprovada não pede motivo");
    assert_eq!(c.e.dados.lock().unwrap().orq_ler("sm").unwrap().motivos.len(), 1);
    assert_eq!(par(&c.gesto("revisar", corpo("inventado", &hash))), (422, "estado-invalido".to_string()));
    let q = json!({"workspaceId": "sm", "blueprintId": "b1", "peca": "q:p1", "numero": 0, "hash": hash, "estado": "aprovada"});
    assert_eq!(par(&c.gesto("revisar", q)), (422, "peca-invalida".to_string()), "pergunta só por gesto/responder");
}

#[test]
fn colocar_so_no_salvar() {
    let c = cena();
    let nova = AcaoDoc { id: "3".into(), nome: "Nova".into(), ..AcaoDoc::default() };
    let n = c.pendente("salvar", |d| d.projetos[0].acoes.push(nova));
    let corpo = |acoes: Value| json!({"workspaceId": "sm", "blueprintId": "b1", "numero": n, "acoes": acoes});
    assert_eq!(par(&c.gesto("colocar", corpo(json!(["1"])))), (409, "plano-mudou".to_string()), "não é nova");
    let r = c.gesto("colocar", corpo(json!(["3"])));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(c.arvore()["acoes"].as_array().unwrap().len(), 3);
    assert!(!c.pendente_existe());
    let n = c.pendente("escrita", |d| d.projetos[0].acoes.push(AcaoDoc { id: "4".into(), nome: "Outra".into(), ..AcaoDoc::default() }));
    let r = c.gesto("colocar", json!({"workspaceId": "sm", "blueprintId": "b1", "numero": n, "acoes": ["4"]}));
    assert_eq!(par(&r), (409, "nao-e-salvar".to_string()));
}

/// O clique manda o `numero` e o `hash` que a folha mostrou: a tela e o gesto não podem calcular diferente.
#[test]
fn hash_da_folha_vale_no_clique() {
    let c = cena();
    let n = c.pendente("escrita", |d| trocar_texto(d, "1.1", "Novo"));
    let folha = json_de(&crate::http::rotas::despachar(&c.e, &crate::http::tests::pedido("GET", "/api/mcp/planejar/folha?workspace=sm&blueprint=b1", &[], None)));
    let acao = &folha["projetos"][0]["acoes"][0];
    assert_eq!((acao["diff"]["numero"].as_i64(), folha["pendente"]["numero"].as_i64()), (Some(n), Some(n)));
    let r = c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": "b1", "peca": "d:d1", "numero": n, "hash": folha["decisoes"][0]["hash"], "estado": "aprovada"}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    let r = c.gesto("revisar", json!({"workspaceId": "sm", "blueprintId": "b1", "peca": "diff:1", "numero": n, "hash": acao["diff"]["hash"], "estado": "aprovada"}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
    assert_eq!(c.plano(0, 0)["texto"], "Novo");
}
