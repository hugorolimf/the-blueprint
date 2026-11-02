//! Passa de 300 linhas: a fatia dona (F32) tem só este arquivo de teste e não pode criar módulo novo. Os treze casos do contrato ficam juntos e dividem os mesmos auxiliares; dividir depois das ondas paralelas.
//! A leitura do Planejar: a folha com e sem versão, o que chegou, os agentes e o pulso. As versões entram pela camada de dados: aqui só importa como elas são lidas.
use super::revisao::{hash_decisao, hash_detalhe, hash_diff};
use crate::contexto::Contexto;
use crate::dados::planejar::{FiguraLinha, Lancamento, Mudanca, NovaExecucao, NovoRecado};
use crate::http::rotas::despachar;
use crate::http::tests::{estado, fixture, json_de, pedido};
use crate::http::Estado;
use crate::plano::arvore::do_blueprint;
use crate::plano::formato::{AcaoDoc, Detalhe, PlanoDoc, PlanoItem, ProjetoDoc};
use crate::tempo::agora_ms;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

const FOLHA: &str = "/api/mcp/planejar/folha?workspace=sm&blueprint=b1";
const CHEGOU: &str = "/api/mcp/planejar/chegou?workspace=sm&blueprint=b1";
const AGENTES: &str = "/api/mcp/planejar/agentes?workspace=sm";
const PULSO: &str = "/api/mcp/planejar/pulso?workspace=sm&blueprint=b1";

fn arvore() -> Value {
    json!({"blueprints": [
        {"id": "b1", "nome": "Monitor", "descricao": "Painel", "obs": "anotado", "links": [{"id": "l1", "rotulo": "Docs", "url": "https://exemplo.dev"}],
         "planoDoc": {"decisoes": [{"id": "d1", "texto": "Usar a rede local", "acoes": ["1.1"], "pede_aprovacao": true}], "perguntas": [{"id": "q1", "texto": "Qual banco?", "opcoes": ["A", "B"], "recomendada": "A", "acoes": ["1.1"]}]},
         "acoes": [
            {"id": "a1", "ref": "1.1", "nome": "Coleta", "inicio": "2026-11-02", "fim": "2026-11-09", "status": "andamento", "obs": "Ler e guardar",
             "detalhes": [{"id": "d-a", "titulo": "Esquema", "pede_aprovacao": true}],
             "planos": [{"id": "q1", "ref": "1.1.1", "texto": "Ler os medidores", "area": "coleta", "feito": true}, {"id": "q2", "ref": "1.1.2", "texto": "Guardar as leituras", "area": "banco", "dependeDe": ["1.1.1"]}],
             "checkpoints": [{"id": "k1", "ref": "1.1.c1", "nome": "Primeiros dados", "data": "2026-11-09"}]},
            {"id": "a2", "ref": "1.2", "nome": "Painel", "inicio": "2026-11-10", "fim": "2026-11-20", "status": "planejado", "planos": [{"id": "q3", "ref": "1.2.1", "texto": "Somar por sala", "area": "painel"}], "checkpoints": []}]},
        {"id": "b2", "nome": "Biblioteca", "acoes": [{"id": "a3", "ref": "2.1", "nome": "Acervo", "inicio": "2026-11-02", "fim": "2026-11-09", "status": "planejado",
            "planos": [{"id": "q4", "ref": "2.1.1", "texto": "Cadastrar", "area": "coleta/livros"}, {"id": "q5", "ref": "2.1.2", "texto": "Emprestar", "area": "emprestimo", "feito": true}], "checkpoints": []}]}]})
}

fn montar() -> Estado {
    let e = estado(false);
    e.dados.lock().unwrap().gravar_tudo(&Contexto::local(), &arvore()).unwrap();
    e
}

fn ler(e: &Estado, url: &str) -> Value {
    let r = despachar(e, &pedido("GET", url, &[], None));
    assert_eq!(r.status, 200, "{url}: {}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)
}

/// A versão como o fechamento a deixa: pendente, sobre a árvore de agora, com a ação mudada por `mexer`.
fn pendente(e: &Estado, origem: &str, mexer: impl FnOnce(&mut PlanoDoc)) -> i64 {
    let mut d = e.dados.lock().unwrap();
    let tudo = d.ler_tudo(&Contexto::local()).unwrap().clone();
    let mut doc = do_blueprint(&tudo["blueprints"][0]);
    mexer(&mut doc);
    let (numero, escrita) = d.plano_abrir("sm", "b1", origem, "plano 1", &doc.para_json(), &[], agora_ms()).unwrap();
    d.plano_fechar(&escrita, agora_ms()).unwrap();
    numero
}

fn acao_de<'a>(folha: &'a Value, r: &str) -> &'a Value {
    folha["projetos"].as_array().unwrap().iter().flat_map(|p| p["acoes"].as_array().unwrap()).find(|a| a["ref"] == r).unwrap_or_else(|| panic!("sem a ação {r}: {folha}"))
}

fn plano_de<'a>(acao: &'a Value, r: &str) -> &'a Value {
    acao["planos"].as_array().unwrap().iter().find(|q| q["ref"] == r).unwrap()
}

fn doc_acao_de(doc: &mut PlanoDoc, r: &str) -> usize {
    doc.projetos[0].acoes.iter().position(|a| a.id == r).unwrap()
}

/// Uma execução já lançada; devolve o id. `area` é a da linha.
fn lancada(e: &Estado, bp: &str, acao: &str, plano: &str, r: &str, area: &str) -> String {
    let mut d = e.dados.lock().unwrap();
    let id = d.execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: bp, acao_id: acao, plano_id: plano, plano_ref: r, area, pasta_principal: "/tmp/nao-existe", marcada_em: agora_ms() }).unwrap();
    assert!(d.execucao_reivindicar(&id).unwrap());
    assert!(d.execucao_lancar(&id, &Lancamento { area, pasta_trabalho: "/tmp/nao-existe/copia", repositorio: "r", ramo: &format!("agente/{r}"), base: "", agora_ms: agora_ms() }).unwrap());
    id
}

fn todos_os_get() -> Vec<String> {
    ["planejar/pulso", "planejar/folha", "planejar/chegou", "planejar/agentes", "planejar/config", "plano/ler", "plano/respostas", "plano/workspace"].iter().map(|r| format!("/api/mcp/{r}?workspace=sm&blueprint=b1")).collect()
}

#[test]
fn folha_de_blueprint_antigo() {
    let e = estado(false);
    let f = ler(&e, "/api/mcp/planejar/folha?workspace=sm&blueprint=p1");
    assert_eq!(f["blueprint"]["nome"], "Portal");
    assert_eq!(f["blueprint"]["resumo"], "d");
    assert!(f["escrita"].is_null() && f["pendente"].is_null());
    assert_eq!((f["semPasta"].as_bool(), f["limite"].as_u64()), (Some(true), Some(4)));
    assert_eq!(f["numeros"], json!({"aprovar": 0, "diffs": 0, "novas": 0}));
    assert_eq!(f["hoje"], crate::tempo::hoje_local());

    let a = acao_de(&f, "f1"); // sem `ref`: o ref é o próprio id, e o id real continua sendo o dele
    assert_eq!(a["id"], "f1");
    assert_eq!((a["de"].as_str(), a["ate"].as_str(), a["semData"].as_bool()), (Some("2026-11-01"), Some("2026-11-30"), Some(false)));
    assert_eq!((a["escrita"].as_str(), a["nova"].as_bool()), (Some("pronta"), Some(false)));
    assert!(a["diff"].is_null());
    assert_eq!(a["numeros"], json!({"planos": 2, "prontos": 1, "detalhes": 0, "decisoes": 0, "aprovar": 0}));
    let (livre, feito) = (plano_de(a, "t1"), plano_de(a, "t2"));
    assert_eq!((livre["id"].as_str(), livre["situacao"].as_str(), livre["instrucoes"].as_str()), (Some("t1"), Some("nao"), Some("x")));
    assert_eq!((feito["situacao"].as_str(), feito["feito"].as_bool()), (Some("pronto"), Some(true)));
    assert_eq!(a["checkpoints"][1], json!({"id": "m2", "nome": "Já feito", "data": "2026-11-25", "feito": true}));

    let r = despachar(&e, &pedido("GET", "/api/mcp/planejar/folha?workspace=sm&blueprint=nao-existe", &[], None));
    assert_eq!(r.status, 404);

    // os campos do blueprint novo: links, nota, decisões, perguntas e o projeto sem nome
    let m = montar();
    let f = ler(&m, FOLHA);
    assert_eq!(f["blueprint"]["temNota"], true);
    assert_eq!(f["blueprint"]["links"][0]["id"], "l1");
    assert_eq!(f["projetos"][0]["nome"], "");
    assert_eq!(f["decisoes"][0]["pedeAprovacao"], true);
    assert_eq!(f["perguntas"][0]["recomendada"], "A");
    assert_eq!(f["numeros"]["aprovar"], 2, "uma decisão e um detalhe");
    assert_eq!(acao_de(&f, "1.1")["numeros"]["aprovar"], 2);
    assert_eq!(acao_de(&f, "1.1")["temNota"], true);
}

#[test]
fn folha_com_aguardando() {
    let e = montar();
    let mut doc = do_blueprint(&arvore()["blueprints"][0]);
    let vazia = AcaoDoc { id: "1.3".into(), nome: "Alertas".into(), ..AcaoDoc::default() };
    doc.projetos.push(ProjetoDoc { nome: None, acoes: vec![vazia.clone()] });
    let (numero, escrita) = e.dados.lock().unwrap().plano_abrir("sm", "b1", "escrita", "orquestrador", &doc.para_json(), &["1.3".to_string()], agora_ms()).unwrap();

    let f = ler(&e, FOLHA);
    assert_eq!((f["escrita"]["escritaId"].as_str(), f["escrita"]["agente"].as_str(), f["escrita"]["origem"].as_str()), (Some(escrita.as_str()), Some("orquestrador"), Some("escrita")));
    let a = acao_de(&f, "1.3");
    assert_eq!((a["escrita"].as_str(), a["nova"].as_bool(), a["semData"].as_bool()), (Some("aguardando"), Some(true), Some(true)));
    assert_eq!(a["id"], crate::plano::ids::id_real("b1", "acao", "1.3"));
    assert_eq!(acao_de(&f, "1.1")["escrita"], "pronta", "a que não está na escrita");
    assert_eq!(f["projetos"].as_array().unwrap().len(), 1, "entra no projeto de mesmo nome");

    // a peça chega: aparece na hora, sem ir à árvore
    let mut cheia = vazia;
    cheia.planos.push(PlanoItem { id: "1.3.1".into(), texto: "Avisar por e-mail".into(), ..PlanoItem::default() });
    cheia.detalhes.push(Detalhe { id: "fig".into(), figura: Some("figuras/a.svg".into()), ..Detalhe::default() });
    doc.projetos[1].acoes = vec![cheia];
    {
        let mut d = e.dados.lock().unwrap();
        d.figura_registrar(&FiguraLinha { blueprint_id: "b1".into(), numero, caminho: "figuras/a.svg".into(), hash: "h1".into(), tipo: "svg".into(), tamanho: 9, largura: Some(40), altura: Some(20) }).unwrap();
        d.plano_gravar(&escrita, &doc.para_json(), &[], &["1.3".to_string()], &[], agora_ms()).unwrap();
    }
    let f = ler(&e, FOLHA);
    let a = acao_de(&f, "1.3");
    assert_eq!(a["escrita"], "escrevendo");
    let q = plano_de(a, "1.3.1");
    assert_eq!((q["situacao"].as_str(), q["feito"].as_bool()), (Some("revisao"), Some(false)), "não pode começar antes de entrar na árvore");
    assert_eq!(a["detalhes"][0]["figura"], json!({"hash": "h1", "tipo": "svg", "largura": 40, "altura": 20}));
    let tudo = e.dados.lock().unwrap().ler_tudo(&Contexto::local()).unwrap().clone();
    assert_eq!(tudo["blueprints"][0]["acoes"].as_array().unwrap().len(), 2, "a árvore não muda");
}

#[test]
fn diff_na_acao() {
    let e = montar();
    let numero = pendente(&e, "escrita", |doc| {
        let i = doc_acao_de(doc, "1.1");
        let a = &mut doc.projetos[0].acoes[i];
        a.planos[0].texto = "Ler os medidores pela rede".into(); // está feito: reabre ao aprovar
        a.planos[1].depende_de = Vec::new(); // só a dependência: não reabre
        a.planos.push(PlanoItem { id: "1.1.3".into(), texto: "Reenviar".into(), ..PlanoItem::default() });
        a.ate = Some("2026-11-12".into());
        doc.projetos[0].acoes.retain(|x| x.id != "1.2");
    });

    let f = ler(&e, FOLHA);
    assert_eq!(f["pendente"], json!({"numero": numero, "agente": "plano 1"}));
    assert_eq!((f["numeros"]["diffs"].as_u64(), f["numeros"]["novas"].as_u64()), (Some(2), Some(0)));
    assert_eq!(f["numeros"]["aprovar"], 4, "decisão, detalhe e os dois diffs");

    let a = acao_de(&f, "1.1");
    let d = &a["diff"];
    let antes = do_blueprint(&arvore()["blueprints"][0]);
    let tree = e.dados.lock().unwrap().plano_pendente("b1").unwrap().unwrap();
    let doc = super::revisao::doc_da_versao(&e.dados.lock().unwrap(), &tree).unwrap();
    let (x, y) = (super::revisao::acao_do(&antes, "1.1"), super::revisao::acao_do(&doc, "1.1"));
    assert_eq!(d["hash"], hash_diff(x, y), "o hash é o que gesto/revisar confere");
    assert_eq!((d["numero"].as_i64(), d["agente"].as_str(), d["removida"].as_bool()), (Some(numero), Some("plano 1"), Some(false)));
    assert_eq!(d["resumo"], json!({"mais": 1, "menos": 0, "alterados": 3, "detalhes": 0, "outros": 0}));
    assert_eq!(d["prazo"], json!({"antes": "2026-11-02 → 2026-11-09", "depois": "2026-11-02 → 2026-11-12"}));
    let linhas = d["linhas"].as_array().unwrap();
    assert_eq!(linhas.iter().map(|l| l["tipo"].as_str().unwrap()).collect::<Vec<_>>(), ["prazo", "alterado", "alterado", "novo"]);
    assert_eq!((linhas[1]["ref"].as_str(), linhas[1]["antes"].as_str(), linhas[1]["situacao"].as_str()), (Some("1.1.1"), Some("Ler os medidores"), Some("reabre")));
    assert!(linhas[1]["palavras"].as_array().unwrap().iter().any(|p| p["t"] == "mais" && p["s"].as_str().unwrap().contains("rede")));
    assert_eq!((linhas[2]["campos"].clone(), linhas[2]["situacao"].as_str()), (json!(["depende_de"]), Some("")));

    // a folha continua mostrando a árvore: o diff é só a proposta
    assert_eq!(plano_de(a, "1.1.1")["texto"], "Ler os medidores");
    assert_eq!(plano_de(a, "1.1.1")["situacao"], "pronto", "pronto vence travado");
    assert_eq!(plano_de(a, "1.1.2")["situacao"], "revisao", "alterado não inicia");

    let saindo = acao_de(&f, "1.2");
    assert_eq!(saindo["diff"]["removida"], true);
    assert_eq!(saindo["diff"]["hash"], hash_diff(super::revisao::acao_do(&antes, "1.2"), None));
    assert_eq!(saindo["diff"]["linhas"][0]["tipo"], "removido");
    assert_eq!(saindo["diff"]["prazo"], Value::Null);
}

#[test]
fn revisao_some_se_peca_mudou() {
    let e = montar();
    let doc = do_blueprint(&arvore()["blueprints"][0]);
    let (dec, det) = (hash_decisao(&doc.decisoes[0]), hash_detalhe(&doc.projetos[0].acoes[0].detalhes[0]));
    {
        let mut d = e.dados.lock().unwrap();
        d.revisao_gravar("b1", "d:d1", 1, "aprovada", "", &dec, 7).unwrap();
        d.revisao_gravar("b1", "det:1.1/d-a", 1, "revisar", "mais claro", &det, 8).unwrap();
        d.revisao_gravar("b1", "q:q1", 0, "respondida", "B", &crate::plano::arvore::hash_peca(&doc.perguntas[0].para_json()), 9).unwrap();
    }
    let f = ler(&e, FOLHA);
    assert_eq!(f["decisoes"][0]["hash"], dec);
    assert_eq!(f["decisoes"][0]["revisao"], json!({"estado": "aprovada", "observacao": "", "em": 7}));
    assert_eq!(acao_de(&f, "1.1")["detalhes"][0]["revisao"], json!({"estado": "revisar", "observacao": "mais claro", "em": 8}));
    assert_eq!(f["perguntas"][0]["resposta"], "B");
    assert_eq!(f["numeros"]["aprovar"], 0);

    let mut nova = arvore();
    nova["blueprints"][0]["planoDoc"]["decisoes"][0]["texto"] = json!("Usar a nuvem");
    nova["blueprints"][0]["planoDoc"]["perguntas"][0]["texto"] = json!("Qual banco agora?");
    nova["blueprints"][0]["acoes"][0]["detalhes"][0]["titulo"] = json!("Esquema novo");
    e.dados.lock().unwrap().gravar_tudo(&Contexto::local(), &nova).unwrap();
    let f = ler(&e, FOLHA);
    assert!(f["decisoes"][0]["revisao"].is_null(), "a decisão reescrita pede revisão de novo");
    assert!(acao_de(&f, "1.1")["detalhes"][0]["revisao"].is_null());
    assert_eq!(f["perguntas"][0]["resposta"], "");
    assert_eq!(f["numeros"]["aprovar"], 2);

    // o diff: a mesma ação com outra versão muda o hash, e a revisão anterior não vale
    pendente(&e, "escrita", |d| d.projetos[0].acoes[1].nome = "Painel novo".into());
    let hash = ler(&e, FOLHA);
    let hash = acao_de(&hash, "1.2")["diff"]["hash"].as_str().unwrap().to_string();
    e.dados.lock().unwrap().revisao_gravar("b1", "diff:1.2", 2, "reprovada", "", &hash, 10).unwrap();
    let f = ler(&e, FOLHA);
    assert_eq!(acao_de(&f, "1.2")["diff"]["revisao"]["estado"], "reprovada");
    assert_eq!(plano_de(acao_de(&f, "1.2"), "1.2.1")["situacao"], "nao", "reprovada: fica o que está na árvore");
    pendente(&e, "escrita", |d| d.projetos[0].acoes[1].nome = "Painel outro".into());
    let f = ler(&e, FOLHA);
    assert!(acao_de(&f, "1.2")["diff"]["revisao"].is_null());
}

#[test]
fn chegou_colocar_so_no_salvar() {
    let e = montar();
    let vazio = ler(&e, CHEGOU);
    assert_eq!((vazio["numero"].as_i64(), vazio["novas"].as_array().map(Vec::len)), (Some(0), Some(0)));

    let nova = |doc: &mut PlanoDoc| {
        doc.projetos[0].acoes[1].nome = "Painel por sala".into();
        doc.projetos[0].acoes.remove(0);
        let mut a = AcaoDoc { id: "1.3".into(), nome: "Alertas".into(), ..AcaoDoc::default() };
        a.planos.push(PlanoItem { id: "1.3.1".into(), texto: "Avisar".into(), ..PlanoItem::default() });
        a.detalhes.push(Detalhe { id: "x".into(), ..Detalhe::default() });
        doc.projetos[0].acoes.push(a);
    };
    let numero = pendente(&e, "salvar", nova);
    let c = ler(&e, CHEGOU);
    assert_eq!((c["numero"].as_i64(), c["agente"].as_str(), c["origem"].as_str(), c["iguais"].as_u64()), (Some(numero), Some("plano 1"), Some("salvar"), Some(0)));
    assert_eq!(c["novas"], json!([{"ref": "1.3", "nome": "Alertas", "planos": 1, "detalhes": 1, "checkpoints": 0, "podeColocar": true}]));
    assert_eq!(c["alteradas"][0]["ref"], "1.2");
    assert_eq!(c["alteradas"][0]["nome"], "Painel por sala");
    assert_eq!(c["alteradas"][0]["resumo"]["outros"], 1);
    assert_eq!(c["removidas"], json!([{"ref": "1.1", "nome": "Coleta"}]));
    assert_eq!(ler(&e, FOLHA)["numeros"]["novas"], 1);

    // a mesma chegada por escrita: o fechamento já pôs as novas, então daqui não há o que colocar
    pendente(&e, "escrita", nova);
    let c = ler(&e, CHEGOU);
    assert_eq!(c["origem"], "escrita");
    assert_eq!(c["novas"][0]["podeColocar"], false);
}

#[test]
fn agentes_lista_entre_blueprints() {
    let e = montar();
    let git_dir = std::env::temp_dir().join(format!("bp-leitura-{}-{}", std::process::id(), agora_ms()));
    std::fs::create_dir_all(&git_dir).unwrap();
    let init = std::process::Command::new("git").args(["init", "-q", "-b", "principal"]).current_dir(&git_dir).status().unwrap();
    assert!(init.success());

    let rodando = lancada(&e, "b1", "a1", "q2", "1.1.2", "banco");
    let ajuda = lancada(&e, "b1", "a2", "q3", "1.2.1", "painel");
    let marcada = {
        let mut d = e.dados.lock().unwrap();
        let marcada = d.execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: "b2", acao_id: "a3", plano_id: "q4", plano_ref: "2.1.1", area: "banco/livros", pasta_principal: "", marcada_em: agora_ms() }).unwrap();
        d.execucao_mudar(&ajuda, &Mudanca { situacao: Some("ajuda"), ultimo_sinal: Some(agora_ms() - 121_000), ..Mudanca::default() }).unwrap();
        d.orq_ligar("sm", true).unwrap();
        d.recado_gravar(&NovoRecado { workspace_id: "sm", de: "orquestrador", texto: "Estou olhando", em: 1, ..NovoRecado::default() }).unwrap();
        d.config_gravar("limite:sm", "3").unwrap();
        marcada
    };
    let a = ler(&e, AGENTES);
    let planos = a["planos"].as_array().unwrap();
    assert_eq!(planos.iter().map(|q| q["ref"].as_str().unwrap()).collect::<Vec<_>>(), ["1.1.2", "1.2.1", "2.1.1"], "em ordem de documento; os feitos à mão e os livres ficam de fora");
    assert_eq!(a["limite"], 3);

    let linha = |r: &str| planos.iter().find(|q| q["ref"] == r).unwrap();
    let q = linha("1.1.2");
    assert_eq!((q["blueprintId"].as_str(), q["blueprintNome"].as_str(), q["acaoId"].as_str(), q["id"].as_str()), (Some("b1"), Some("Monitor"), Some("a1"), Some("q2")));
    assert_eq!((q["situacao"].as_str(), q["execucaoId"].as_str(), q["ramo"].as_str()), (Some("trabalhando"), Some(rodando.as_str()), Some("agente/1.1.2")));
    assert_eq!((q["podeCancelar"].as_bool(), q["podeIntegrar"].as_bool(), q["podeRetomar"].as_bool()), (Some(true), Some(false), Some(false)));

    let q = linha("1.2.1");
    assert_eq!((q["situacao"].as_str(), q["causa"].as_str(), q["semSinal"].as_bool(), q["podeRetomar"].as_bool()), (Some("bloqueado"), Some("ajuda"), Some(true), Some(true)));

    // área de outro blueprint segura: `banco/livros` dentro de `banco`
    let q = linha("2.1.1");
    assert_eq!((q["blueprintNome"].as_str(), q["situacao"].as_str(), q["por"].as_str(), q["execucaoId"].as_str()), (Some("Biblioteca"), Some("area"), Some("1.1.2"), Some(marcada.as_str())));
    assert_eq!(a["numeros"], json!({"trabalhando": 1, "fila": 0, "esperando": 1, "parados": 1, "prontos": 0}));
    assert!(a["avisos"].as_array().unwrap().iter().any(|x| x.as_str().unwrap().contains("2.1.1")), "{a}");
    let agora: Vec<&str> = a["orquestrador"]["agora"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect();
    assert!(agora[0].contains("1.2.1"), "a ajuda vem primeiro: {agora:?}");
    assert!(agora.contains(&"Estou olhando"));
    assert_eq!((a["orquestrador"]["ligado"].as_bool(), a["orquestrador"]["acordado"].as_bool()), (Some(true), Some(false)));

    // pronta sem integrar: pode integrar, com o destino lido da pasta principal; integrada, sai da lista
    {
        let mut d = e.dados.lock().unwrap();
        let caminho = git_dir.to_string_lossy().to_string();
        let id = d.execucao_criar(&NovaExecucao { workspace_id: "sm", blueprint_id: "b2", acao_id: "a3", plano_id: "q5", plano_ref: "2.1.2", area: "emprestimo", pasta_principal: &caminho, marcada_em: agora_ms() }).unwrap();
        d.execucao_mudar(&id, &Mudanca { situacao: Some("pronta"), ramo: Some("agente/2.1.2"), ..Mudanca::default() }).unwrap();
    }
    let a = ler(&e, AGENTES);
    let q = a["planos"].as_array().unwrap().iter().find(|q| q["ref"] == "2.1.2").unwrap().clone();
    assert_eq!((q["situacao"].as_str(), q["podeIntegrar"].as_bool(), q["destino"].as_str(), q["podeCancelar"].as_bool()), (Some("pronto"), Some(true), Some("principal"), Some(false)));
    assert_eq!(a["numeros"]["prontos"], 1);
    let id = q["execucaoId"].as_str().unwrap().to_string();
    e.dados.lock().unwrap().execucao_mudar(&id, &Mudanca { integrada_em: Some(Some(1)), ..Mudanca::default() }).unwrap();
    assert!(ler(&e, AGENTES)["planos"].as_array().unwrap().iter().all(|q| q["ref"] != "2.1.2"));
    let _ = std::fs::remove_dir_all(&git_dir);

    // plano/workspace traz as mesmas linhas, e os recados
    e.mcp.ligar_janela();
    let w = ler(&e, "/api/mcp/plano/workspace?workspace=sm");
    assert_eq!(w["agentes"].as_array().unwrap().len(), 3);
    assert_eq!(w["blueprints"].as_array().unwrap().iter().map(|b| b["id"].as_str().unwrap()).collect::<Vec<_>>(), ["b1", "b2"]);
    assert_eq!(w["recados"][0]["texto"], "Estou olhando");
    assert_eq!(w["hoje"], crate::tempo::hoje_local());

    // workspace que não existe: 404, não uma lista vazia
    assert_eq!(despachar(&e, &pedido("GET", "/api/mcp/planejar/agentes?workspace=nao-existe", &[], None)).status, 404);
}

#[test]
fn pulso_muda() {
    use super::motor::{passo, Ambiente, Relogio};
    let e = montar();
    let token = e.mcp.ligar_janela();
    let revisao = || ler(&e, PULSO)["revisao"].as_u64().unwrap();
    let corpo_de = |url: &str, cab: &[(&str, &str)], corpo: Value| {
        let r = despachar(&e, &pedido("POST", url, cab, Some(corpo)));
        assert_eq!(r.status, 200, "{url}: {}", String::from_utf8_lossy(r.bytes()));
    };
    let inicio = revisao();
    ler(&e, FOLHA);
    ler(&e, AGENTES);
    assert_eq!(revisao(), inicio, "ler não muda o pulso");

    corpo_de("/api/mcp/gesto/limite", &[("X-Blueprint-Gesto", &token)], json!({"workspaceId": "sm", "limite": 3}));
    let depois_do_gesto = revisao();
    assert!(depois_do_gesto > inicio, "gesto");

    corpo_de("/api/mcp/plano/recado", &[], json!({"workspaceId": "sm", "de": "orquestrador", "texto": "Olhando"}));
    let depois_do_recado = revisao();
    assert!(depois_do_recado > depois_do_gesto, "recado");

    let id = lancada(&e, "b1", "a2", "q3", "1.2.1", "painel");
    let segredo = e.dados.lock().unwrap().execucao_de(&id).unwrap().unwrap().segredo;
    corpo_de("/api/mcp/execucao/sinal", &[], json!({"id": id, "segredo": segredo}));
    let depois_do_sinal = revisao();
    assert!(depois_do_sinal > depois_do_recado, "sinal");

    // o motor percebe o agente que parou de sinalizar: a tela vê sem ninguém clicar
    let agora = agora_ms() + 121_000;
    let amb = Ambiente { lancar: Box::new(|_, _, _| Ok("falso")), git: "git".into(), pasta_copias: std::env::temp_dir() };
    let mut relogio = Relogio { partida_ms: agora - 600_000, ultimo_ms: agora };
    passo(&amb, &e.dados, &e.mcp, agora, &mut relogio);
    assert_eq!(e.dados.lock().unwrap().execucao_de(&id).unwrap().unwrap().situacao, "sumiu");
    assert!(revisao() > depois_do_sinal, "motor");
}

#[test]
fn pulso_traz_versao_dados() {
    let e = estado(false);
    let antes = e.versao_dados.load(Ordering::SeqCst);
    assert_eq!(despachar(&e, &pedido("PUT", "/api/dados", &[], Some(fixture()))).status, 200);
    let depois = e.versao_dados.load(Ordering::SeqCst);
    assert!(depois > antes);
    let p = ler(&e, PULSO);
    assert_eq!(p, json!({"versaoDados": depois, "revisao": 0}));
    assert_eq!(ler(&e, "/api/mcp/planejar/folha?workspace=sm&blueprint=p1")["versaoDados"], depois);
    e.mcp.planejar_mudou();
    assert_eq!(ler(&e, PULSO)["revisao"], 1);
    assert_eq!(ler(&e, "/api/mcp/planejar/folha?workspace=sm&blueprint=p1")["revisao"], 1);
}

#[test]
fn token_nao_aparece_em_get() {
    let e = montar();
    let token = e.mcp.ligar_janela();
    lancada(&e, "b1", "a1", "q2", "1.1.2", "banco");
    for url in todos_os_get() {
        let r = despachar(&e, &pedido("GET", &url, &[("X-Blueprint-Gesto", &token)], None));
        assert_eq!(r.status, 200, "{url}");
        assert!(!String::from_utf8_lossy(r.bytes()).contains(&token), "o token saiu em {url}");
    }
}

#[test]
fn segredo_nao_aparece_em_get() {
    let e = montar();
    e.mcp.ligar_janela();
    let id = lancada(&e, "b1", "a1", "q2", "1.1.2", "banco");
    let segredo = e.dados.lock().unwrap().execucao_de(&id).unwrap().unwrap().segredo;
    assert_eq!(segredo.len(), 32);
    let mut viu_a_execucao = false;
    for url in todos_os_get().into_iter().chain([format!("/api/mcp/plano/respostas?workspace=sm&blueprint=b1&execucao={id}")]) {
        let corpo = String::from_utf8_lossy(despachar(&e, &pedido("GET", &url, &[], None)).bytes()).into_owned();
        assert!(!corpo.contains(&segredo), "o segredo saiu em {url}");
        viu_a_execucao |= corpo.contains(&id);
    }
    assert!(viu_a_execucao, "a execução está mesmo nas respostas lidas");
}

#[test]
fn parada_depois_de_5_min() {
    let e = montar();
    let agora = agora_ms();
    let doc = do_blueprint(&arvore()["blueprints"][0]).para_json();
    let escrita = e.dados.lock().unwrap().plano_abrir("sm", "b1", "escrita", "a", &doc, &[], agora - 299_000).unwrap().1;
    let f = ler(&e, FOLHA);
    assert_eq!(f["escrita"]["parada"], false);
    assert!(f["escrita"]["paradaMs"].as_i64().unwrap() >= 299_000);
    e.dados.lock().unwrap().plano_gravar(&escrita, &doc, &[], &[], &[], agora - 301_000).unwrap();
    assert_eq!(ler(&e, FOLHA)["escrita"]["parada"], true);
    e.dados.lock().unwrap().plano_gravar(&escrita, &doc, &[], &[], &[], agora_ms()).unwrap();
    assert_eq!(ler(&e, FOLHA)["escrita"]["parada"], false, "peça nova tira da parada");
}

#[test]
fn respostas_sem_efeito() {
    let e = montar();
    e.mcp.ligar_janela();
    let id = lancada(&e, "b1", "a1", "q2", "1.1.2", "banco");
    {
        let mut d = e.dados.lock().unwrap();
        d.revisao_gravar("b1", "diff:1.1", 3, "reprovada", "muito grande", "h", 5).unwrap();
        d.revisao_gravar("b1", "q:q1", 3, "respondida", "B", "h", 6).unwrap();
        d.recado_gravar(&NovoRecado { workspace_id: "sm", blueprint_id: "b1", de: "usuario", texto: "do blueprint", em: 1, ..NovoRecado::default() }).unwrap();
        d.recado_gravar(&NovoRecado { workspace_id: "sm", blueprint_id: "b2", de: "usuario", texto: "de outro", em: 2, ..NovoRecado::default() }).unwrap();
        d.recado_gravar(&NovoRecado { workspace_id: "sm", blueprint_id: "b1", execucao_id: &id, de: "orquestrador", texto: "para o agente", em: 3, ..NovoRecado::default() }).unwrap();
    }
    let url = "/api/mcp/plano/respostas?workspace=sm&blueprint=b1";
    let pulso = ler(&e, PULSO);
    let primeira = ler(&e, url);
    assert_eq!(primeira["revisoes"], json!([{"peca": "diff:1.1", "estado": "reprovada", "observacao": "muito grande", "numero": 3}]));
    assert_eq!(primeira["respostas"], json!([{"pergunta": "q1", "resposta": "B"}]));
    assert_eq!(primeira["recados"].as_array().unwrap().iter().map(|r| r["texto"].as_str().unwrap()).collect::<Vec<_>>(), ["do blueprint", "para o agente"]);
    assert_eq!(ler(&e, url), primeira, "ler de novo dá o mesmo");
    assert_eq!(ler(&e, PULSO), pulso, "ler não acorda ninguém");
    let dela = ler(&e, &format!("{url}&execucao={id}"));
    assert_eq!(dela["recados"].as_array().unwrap().iter().map(|r| r["texto"].as_str().unwrap()).collect::<Vec<_>>(), ["para o agente"]);
    assert_eq!(e.dados.lock().unwrap().revisoes_do("b1").unwrap().len(), 2);

    // plano/ler: o doc da árvore, a escrita aberta e o que falta
    let l = ler(&e, "/api/mcp/plano/ler?workspace=sm&blueprint=b1");
    assert_eq!(l["plano"]["formato"], "blueprint.plano/1");
    assert!(l["escrita"].is_null() && l["pendente"].is_null());
    let doc = do_blueprint(&arvore()["blueprints"][0]).para_json();
    let escrita = e.dados.lock().unwrap().plano_abrir("sm", "b1", "escrita", "a", &doc, &["1.2".to_string()], agora_ms()).unwrap().1;
    let l = ler(&e, "/api/mcp/plano/ler?workspace=sm&blueprint=b1");
    assert_eq!((l["escrita"]["escritaId"].as_str(), l["aguardando"].clone()), (Some(escrita.as_str()), json!(["1.2"])));
}

#[test]
fn sem_pasta_quando_nao_e_pasta() {
    let e = montar();
    assert_eq!(ler(&e, FOLHA)["semPasta"], true, "nunca ligada");
    let pasta = std::env::temp_dir().join(format!("bp-sem-pasta-{}-{}", std::process::id(), agora_ms()));
    std::fs::create_dir_all(&pasta).unwrap();
    e.dados.lock().unwrap().vincular_pasta(&Contexto::local(), &pasta.to_string_lossy(), "sm", "b1").unwrap();
    assert_eq!(ler(&e, FOLHA)["semPasta"], false);
    std::fs::remove_dir_all(&pasta).unwrap();
    assert_eq!(ler(&e, FOLHA)["semPasta"], true, "a pasta foi apagada");
}
