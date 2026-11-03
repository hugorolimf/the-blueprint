//! Testes de mcp::planejar::escrita: abrir, escrever, figura e fechar, pelo caminho real (`despachar`), para valer também a versão dos dados (D4) e o que `rota_estado` faz.
//! Passa de 300 linhas: são os 18 casos que o contrato nomeia, e a fatia não pode criar outro módulo de teste.
use crate::contexto::Contexto;
use crate::http::pedido::Resposta;
use crate::http::rotas::despachar;
use crate::http::tests::{estado, json_de, pedido};
use crate::http::Estado;
use serde_json::{json, Value};
use std::sync::atomic::Ordering;

const SVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 90 40\"><rect width=\"10\" height=\"10\"/></svg>";

fn janela() -> Estado {
    let e = estado(false);
    e.mcp.ligar_janela();
    e
}

fn post(e: &Estado, rota: &str, corpo: Value) -> Resposta {
    despachar(e, &pedido("POST", &format!("/api/mcp/plano/{rota}"), &[], Some(corpo)))
}

fn ok(e: &Estado, rota: &str, corpo: Value) -> Value {
    let r = post(e, rota, corpo);
    assert_eq!(r.status, 200, "{rota}: {}", String::from_utf8_lossy(r.bytes()));
    json_de(&r)
}

fn abrir_em(e: &Estado, ws: &str, bp: &str, origem: &str, acoes: Value) -> Value {
    ok(e, "abrir", json!({"workspaceId": ws, "blueprintId": bp, "origem": origem, "agente": "Claude", "projetos": [{"acoes": acoes}]}))
}

fn abrir_vazio(e: &Estado, ws: &str, bp: &str) -> String {
    abrir_em(e, ws, bp, "escrita", json!([]))["escritaId"].as_str().unwrap().to_string()
}

fn escrever(e: &Estado, escrita: &str, acao: Value) -> Value {
    ok(e, "escrever", json!({"escritaId": escrita, "acao": acao}))
}

fn fechar(e: &Estado, escrita: &str) -> Value {
    ok(e, "fechar", json!({"escritaId": escrita}))
}

fn blueprint(e: &Estado, ws: &str, bp: &str) -> Value {
    let mut d = e.dados.lock().unwrap();
    let tudo = d.ler_tudo(&Contexto::local()).unwrap();
    let lista = if ws == "sm" { &tudo["blueprints"] } else { &tudo["workspaces"].as_array().unwrap().iter().find(|w| w["id"] == ws).unwrap()["blueprints"] };
    lista.as_array().unwrap().iter().find(|b| b["id"] == bp).unwrap().clone()
}

fn versao(e: &Estado, escrita: &str) -> crate::dados::planejar::VersaoLinha {
    e.dados.lock().unwrap().plano_escrita(escrita).unwrap().unwrap()
}

/// O que o usuário faz durante a escrita: `PUT /api/dados` por cima do que está gravado.
fn editar_arvore(e: &Estado, muda: impl FnOnce(&mut Value)) {
    let mut tudo = e.dados.lock().unwrap().ler_tudo(&Contexto::local()).unwrap().clone();
    muda(&mut tudo);
    let r = despachar(e, &pedido("PUT", "/api/dados", &[("X-Blueprint-Gesto", &e.mcp.ligar_janela())], Some(tudo)));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(r.bytes()));
}

fn t1_alterado(e: &Estado, escrita: &str) {
    escrever(e, escrita, json!({"id": "f1", "planos": [{"id": "t1", "texto": "Criar rota nova"}]}));
}

#[test]
fn abrir_cria_aguardando() {
    let e = janela();
    let r = abrir_em(&e, "w2", "p9", "escrita", json!([{"id": "1", "nome": "Um"}, {"id": "2", "nome": "Dois"}]));
    assert_eq!((&r["aguardando"], &r["continuacao"], &r["numero"], &r["blueprintId"]), (&json!(["1", "2"]), &json!(false), &json!(1), &json!("p9")));
    assert!(blueprint(&e, "w2", "p9")["acoes"].as_array().unwrap().is_empty(), "a árvore só muda ao fechar");
    let linha = versao(&e, r["escritaId"].as_str().unwrap());
    assert_eq!((linha.situacao.as_str(), linha.agente.as_str(), linha.aguardando.len()), ("escrevendo", "Claude", 2));

    // sem blueprintId: o blueprint nasce na hora, com a pasta, e a versão dos dados sobe
    let pasta = std::env::temp_dir().join(format!("bp-escrita-{}", crate::tempo::agora_ms()));
    std::fs::create_dir_all(&pasta).unwrap();
    let antes = e.versao_dados.load(Ordering::SeqCst);
    let r = ok(&e, "abrir", json!({"workspaceId": "w2", "blueprint": "Novo", "resumo": "R", "origem": "escrita", "pasta": pasta.to_str().unwrap(), "projetos": [{"nome": "Grupo", "acoes": [{"id": "a", "nome": "A"}]}]}));
    let id = r["blueprintId"].as_str().unwrap();
    assert!(e.versao_dados.load(Ordering::SeqCst) > antes);
    let novo = blueprint(&e, "w2", id);
    assert_eq!((&novo["nome"], &novo["descricao"]), (&json!("Novo"), &json!("R")));
    let ligada = e.dados.lock().unwrap().pasta_real_do_blueprint("w2", id).unwrap();
    assert_eq!(ligada.as_deref(), pasta.to_str());
    let r = post(&e, "abrir", json!({"workspaceId": "w2", "origem": "escrita"}));
    assert_eq!(r.status, 422, "blueprint novo pede nome");
    let r = post(&e, "abrir", json!({"workspaceId": "w2", "blueprint": "X", "origem": "escrita", "pasta": "/nao/existe/aqui"}));
    assert_eq!(r.status, 422, "pasta tem de ser pasta");
    std::fs::remove_dir_all(&pasta).unwrap();
}

#[test]
fn abrir_continua_a_mesma() {
    let e = janela();
    let a = abrir_em(&e, "w2", "p9", "escrita", json!([{"id": "1", "nome": "Um"}]));
    let escrita = a["escritaId"].as_str().unwrap();
    escrever(&e, escrita, json!({"id": "1", "planos": [{"id": "1.1", "texto": "Fazer"}]}));
    let b = abrir_em(&e, "w2", "p9", "escrita", json!([{"id": "2", "nome": "Dois"}]));
    assert_eq!((b["escritaId"].as_str(), &b["continuacao"], &b["numero"]), (Some(escrita), &json!(true), &a["numero"]));
    assert_eq!(b["aguardando"], json!(["2"]), "a já escrita não volta a aguardar");
    assert_eq!(versao(&e, escrita).plano["projetos"][0]["acoes"][0]["planos"][0]["texto"], "Fazer", "o escrito continua");

    let s = abrir_em(&e, "sm", "p1", "salvar", json!([{"id": "x", "nome": "X"}]));
    let escrita = s["escritaId"].as_str().unwrap();
    escrever(&e, escrita, json!({"id": "x", "planos": ["Fazer"]}));
    let de_novo = abrir_em(&e, "sm", "p1", "salvar", json!([{"id": "y", "nome": "Y"}]));
    assert_eq!((de_novo["escritaId"].as_str(), &de_novo["continuacao"]), (Some(escrita), &json!(true)));
    let linha = versao(&e, escrita);
    assert_eq!(linha.plano["projetos"][0]["acoes"].as_array().unwrap().len(), 1, "salvar reaberto zera o doc");
    assert_eq!(linha.plano["projetos"][0]["acoes"][0]["id"], "y");
}

#[test]
fn origem_diferente_409() {
    let e = janela();
    abrir_vazio(&e, "w2", "p9");
    let r = post(&e, "abrir", json!({"workspaceId": "w2", "blueprintId": "p9", "origem": "salvar"}));
    assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (409, Some("escrita-aberta")));
}

#[test]
fn escrever_mescla_por_id() {
    let e = janela();
    let escrita = abrir_em(&e, "w2", "p9", "escrita", json!([{"id": "1", "nome": "Um"}, {"id": "2", "nome": "Dois"}]))["escritaId"].as_str().unwrap().to_string();
    let r = escrever(&e, &escrita, json!({"id": "1", "texto": "T", "planos": [{"id": "a", "texto": "A"}, {"id": "b", "texto": "B"}], "detalhes": [{"id": "d", "titulo": "Velho"}], "tabelas": [{"colunas": ["c"], "linhas": [["1"]]}]}));
    assert_eq!(r["faltam"], json!(["2"]));
    let r = ok(&e, "escrever", json!({"escritaId": escrita, "acao": {"id": "1", "planos": [{"id": "b", "texto": "B2"}, {"id": "c", "texto": "C"}], "removerPlanos": ["a"], "detalhes": [{"id": "d", "titulo": "Novo"}]}, "decisoes": [{"id": "q1", "texto": "D"}], "perguntas": [{"id": "p1", "texto": "P", "opcoes": ["x"]}]}));
    assert_eq!(r["faltam"], json!(["2"]));
    let linha = versao(&e, &escrita);
    let a = &linha.plano["projetos"][0]["acoes"][0];
    let planos: Vec<(&str, &str)> = a["planos"].as_array().unwrap().iter().map(|q| (q["id"].as_str().unwrap(), q["texto"].as_str().unwrap())).collect();
    assert_eq!(planos, [("b", "B2"), ("c", "C")]);
    assert_eq!((&a["nome"], &a["texto"], &a["detalhes"][0]["titulo"], a["tabelas"].as_array().unwrap().len()), (&json!("Um"), &json!("T"), &json!("Novo"), 1), "o que não veio fica");
    assert_eq!((&linha.plano["decisoes"][0]["id"], &linha.plano["perguntas"][0]["id"]), (&json!("q1"), &json!("p1")));
    assert_eq!(linha.tocadas, ["1", "2"]);
    // o reenvio é seguro: mesma peça, mesmo resultado
    let r = ok(&e, "escrever", json!({"escritaId": escrita, "acao": {"id": "2", "planos": [{"id": "z", "texto": "Z"}]}}));
    assert_eq!(r["faltam"], json!([]));
}

#[test]
fn peca_invalida_422() {
    let e = janela();
    let escrita = abrir_vazio(&e, "w2", "p9");
    let casos = [
        json!({"id": "a/b", "nome": "N"}),
        json!({"id": "1", "nome": "N", "planos": ["Sem id"]}),
        json!({"id": "1", "nome": "N", "planos": [{"id": "x", "texto": "A"}, {"id": "x", "texto": "B"}]}),
        json!({"id": "1", "nome": "N", "planos": [{"id": "x", "texto": "A", "depende_de": ["y"]}, {"id": "y", "texto": "B", "depende_de": ["x"]}]}),
        json!({"id": "1"}),
    ];
    for acao in casos {
        let r = post(&e, "escrever", json!({"escritaId": escrita, "acao": acao}));
        assert_eq!(r.status, 422, "{acao}");
        let v = json_de(&r);
        assert!(!v["erros"].as_array().unwrap().is_empty() && v["avisos"].is_array(), "{v}");
    }
    assert_eq!(versao(&e, &escrita).plano["projetos"], json!([]), "peça recusada não grava");
    assert_eq!(post(&e, "escrever", json!({"escritaId": "nao-existe", "acao": {"id": "1"}})).status, 404);
}

#[test]
fn depende_inexistente_so_no_fechar() {
    let e = janela();
    let escrita = abrir_vazio(&e, "w2", "p9");
    escrever(&e, &escrita, json!({"id": "1", "nome": "N", "planos": [{"id": "a", "texto": "A", "depende_de": ["z"]}]}));
    let r = post(&e, "fechar", json!({"escritaId": escrita}));
    assert_eq!(r.status, 422);
    let erros = json_de(&r)["erros"].to_string();
    assert!(erros.contains("does not exist") && erros.contains('z'), "{erros}");
    assert_eq!(versao(&e, &escrita).situacao, "escrevendo", "a escrita fica aberta");
    assert!(blueprint(&e, "w2", "p9")["acoes"].as_array().unwrap().is_empty());
    escrever(&e, &escrita, json!({"id": "2", "nome": "M", "planos": [{"id": "z", "texto": "Z"}]}));
    assert_eq!(fechar(&e, &escrita)["novas"], json!(["1", "2"]));
}

#[test]
fn fechar_novas_entram_na_escrita() {
    let e = janela();
    let escrita = abrir_em(&e, "w2", "p9", "escrita", json!([{"id": "1", "nome": "Um"}]))["escritaId"].as_str().unwrap().to_string();
    ok(&e, "escrever", json!({"escritaId": escrita, "acao": {"id": "1", "texto": "T", "projeto": "Grupo", "planos": [{"id": "1.1", "texto": "A", "area": "app"}], "checkpoints": [{"texto": "C"}]}, "decisoes": [{"id": "q", "texto": "D", "acoes": ["1"]}]}));
    let antes = e.versao_dados.load(Ordering::SeqCst);
    let r = fechar(&e, &escrita);
    assert_eq!((&r["novas"], &r["colocadas"], &r["alteradas"], &r["removidas"], &r["iguais"]), (&json!(["1"]), &json!(["1"]), &json!([]), &json!([]), &json!(0)));
    assert!(e.versao_dados.load(Ordering::SeqCst) > antes, "fechar com árvore mudada sobe a versão");
    let bp = blueprint(&e, "w2", "p9");
    let a = &bp["acoes"][0];
    assert_eq!((&a["ref"], &a["nome"], &a["obs"]), (&json!("1"), &json!("Um"), &json!("T")));
    assert_eq!((&a["planos"][0]["ref"], &a["planos"][0]["extra"], &a["planos"][0]["area"]), (&json!("1.1"), &Value::Null, &json!("app")));
    assert_eq!(a["id"].as_str().unwrap(), crate::plano::ids::id_real("p9", "acao", "1"));
    assert_eq!(bp["projetos"][0]["nome"], "Grupo");
    assert_eq!(bp["planoDoc"]["decisoes"][0]["id"], "q");
    let d = e.dados.lock().unwrap();
    assert_eq!(d.plano_escrita(&escrita).unwrap().unwrap().situacao, "resolvida", "nada ficou pendente");
    assert!(d.plano_pendente("p9").unwrap().is_none());
    drop(d);
    assert_eq!(post(&e, "fechar", json!({"escritaId": escrita})).status, 409, "fechar de novo");
}

#[test]
fn fechar_novas_esperam_no_salvar() {
    let e = janela();
    let antes = blueprint(&e, "sm", "p1");
    let escrita = ok(&e, "abrir", json!({"workspaceId": "sm", "blueprintId": "p1", "origem": "salvar", "resumo": "Novo resumo", "projetos": [{"acoes": [{"id": "x", "nome": "X"}]}]}))["escritaId"].as_str().unwrap().to_string();
    escrever(&e, &escrita, json!({"id": "x", "planos": ["Sem id vale aqui"]}));
    let r = fechar(&e, &escrita);
    assert_eq!((&r["novas"], &r["colocadas"], &r["removidas"]), (&json!(["x"]), &json!([]), &json!(["f1"])), "o que faltou é removido e entra no diff");
    let depois = blueprint(&e, "sm", "p1");
    assert_eq!(depois["descricao"], "Novo resumo", "o que é do blueprint entra já");
    assert_eq!(depois["acoes"], antes["acoes"], "novas esperam gesto/colocar; removida espera revisão");
    let p = e.dados.lock().unwrap().plano_pendente("p1").unwrap().unwrap();
    assert_eq!((p.tocadas, p.removidas), (vec!["x".to_string()], vec!["f1".to_string()]));
}

#[test]
fn alterada_nao_toca_arvore() {
    let e = janela();
    let antes = blueprint(&e, "sm", "p1");
    let escrita = abrir_vazio(&e, "sm", "p1");
    t1_alterado(&e, &escrita);
    let r = fechar(&e, &escrita);
    assert_eq!(r["alteradas"], json!([{"ref": "f1", "resumo": {"mais": 0, "menos": 0, "alterados": 1, "detalhes": 0, "outros": 0}}]));
    assert_eq!((&r["novas"], &r["colocadas"]), (&json!([]), &json!([])));
    assert_eq!(blueprint(&e, "sm", "p1"), antes);
    let p = e.dados.lock().unwrap().plano_pendente("p1").unwrap().unwrap();
    assert_eq!((p.escrita_id, p.tocadas), (escrita, vec!["f1".to_string()]));
    // feito fica na árvore; a versão só diz o texto
    assert_eq!(antes["acoes"][0]["planos"][1]["feito"], true);
}

#[test]
fn escrita_nao_sobe_versao() {
    let e = janela();
    let versao_antes = e.versao_dados.load(Ordering::SeqCst);
    let pulso = e.mcp.revisao_planejar();
    let escrita = abrir_em(&e, "sm", "p1", "escrita", json!([{"id": "n", "nome": "Nova"}]))["escritaId"].as_str().unwrap().to_string();
    assert!(e.mcp.revisao_planejar() > pulso, "abrir muda o pulso");
    let pulso = e.mcp.revisao_planejar();
    escrever(&e, &escrita, json!({"id": "n", "planos": [{"id": "n.1", "texto": "A"}]}));
    assert!(e.mcp.revisao_planejar() > pulso, "escrever muda o pulso");
    let pulso = e.mcp.revisao_planejar();
    ok(&e, "figura", json!({"escritaId": escrita, "caminho": "a.svg", "tipo": "svg", "svg": SVG}));
    assert!(e.mcp.revisao_planejar() > pulso, "figura muda o pulso");
    assert_eq!(e.versao_dados.load(Ordering::SeqCst), versao_antes, "escrever não dá 409 em quem está clicando");
    fechar(&e, &escrita);
    assert!(e.versao_dados.load(Ordering::SeqCst) > versao_antes);
}

#[test]
fn figura_hostil_recusada() {
    let e = janela();
    let escrita = abrir_vazio(&e, "w2", "p9");
    let casos = [
        json!({"caminho": "a.svg", "tipo": "svg", "svg": "<svg><script>alert(1)</script></svg>"}),
        json!({"caminho": "a.svg", "tipo": "svg", "svg": "<svg onload='x()'/>"}),
        json!({"caminho": "a.svg", "tipo": "svg"}),
        json!({"caminho": "../a.svg", "tipo": "svg", "svg": SVG}),
        json!({"caminho": "/etc/a.svg", "tipo": "svg", "svg": SVG}),
        json!({"caminho": "a.exe", "tipo": "exe", "base64": "AAAA"}),
        json!({"caminho": "a.png", "tipo": "svg", "svg": SVG}),
        json!({"caminho": "a.jpg", "tipo": "jpg", "base64": "PHN2Zz48L3N2Zz4="}), // jpg falso: é um SVG
        json!({"caminho": "a.png", "tipo": "png", "base64": "%%%"}),
    ];
    for mut c in casos {
        c["escritaId"] = json!(escrita);
        let r = post(&e, "figura", c.clone());
        assert_eq!(r.status, 422, "{c}");
        assert!(!json_de(&r)["erros"].as_array().unwrap().is_empty());
    }
    // valor hostil que só aparece depois de resolver o CSS do plano: o SVG sozinho é limpo, e o CSS passa em `css_seguro`
    let outra = json_de(&post(&e, "abrir", json!({"workspaceId": "sm", "blueprintId": "p1", "origem": "escrita", "estiloCss": ".f{--cor: x\" onload=\"a()}"})));
    let r = post(&e, "figura", json!({"escritaId": outra["escritaId"], "caminho": "b.svg", "tipo": "svg", "svg": "<svg><rect fill=\"var(--cor)\"/></svg>"}));
    assert_eq!(r.status, 422, "{}", String::from_utf8_lossy(r.bytes()));
    assert!(!e.mcp.pasta_figuras().exists() || std::fs::read_dir(e.mcp.pasta_figuras()).unwrap().next().is_none(), "nada foi gravado");
    // CSS que já vem hostil nem abre a escrita
    let r = post(&e, "abrir", json!({"workspaceId": "w2", "blueprintId": "p9", "origem": "escrita", "estiloCss": "@import url(http://exemplo.invalid/a.css);"}));
    assert_eq!(r.status, 422);
}

#[test]
fn figura_ausente_no_fechar_422() {
    let e = janela();
    let escrita = abrir_vazio(&e, "w2", "p9");
    escrever(&e, &escrita, json!({"id": "1", "nome": "N", "detalhes": [{"id": "d", "figura": "falta.svg"}]}));
    let r = post(&e, "fechar", json!({"escritaId": escrita}));
    assert_eq!(r.status, 422);
    assert!(json_de(&r)["erros"].to_string().contains("falta.svg"));
    ok(&e, "figura", json!({"escritaId": escrita, "caminho": "falta.svg", "tipo": "svg", "svg": SVG}));
    fechar(&e, &escrita);
    let det = &blueprint(&e, "w2", "p9")["acoes"][0]["detalhes"][0];
    assert_eq!(det["figura_info"]["tipo"], "svg");
    assert_eq!(det["figura_info"]["hash"].as_str().unwrap().len(), 64);
}

#[test]
fn segunda_escrita_preserva_pendente() {
    let e = janela();
    let antes = blueprint(&e, "sm", "p1");
    let v1 = abrir_vazio(&e, "sm", "p1");
    t1_alterado(&e, &v1);
    fechar(&e, &v1);

    let v2 = abrir_em(&e, "sm", "p1", "escrita", json!([{"id": "n", "nome": "Nova"}]))["escritaId"].as_str().unwrap().to_string();
    assert_ne!(v1, v2);
    let aberta = versao(&e, &v2);
    let f1 = aberta.plano["projetos"][0]["acoes"].as_array().unwrap().iter().find(|a| a["id"] == "f1").unwrap().clone();
    assert_eq!(f1["planos"][0]["texto"], "Criar rota nova", "a base é a pendente, não a árvore");
    escrever(&e, &v2, json!({"id": "n", "planos": [{"id": "n.1", "texto": "A"}]}));
    let r = fechar(&e, &v2);
    assert_eq!(r["alteradas"][0]["ref"], "f1", "a mudança à espera continua à espera");
    assert_eq!(r["colocadas"], json!(["n"]));
    let d = e.dados.lock().unwrap();
    assert_eq!(d.plano_escrita(&v1).unwrap().unwrap().situacao, "resolvida");
    let p = d.plano_pendente("p1").unwrap().unwrap();
    assert_eq!((p.escrita_id, p.tocadas), (v2, vec!["f1".to_string()]));
    drop(d);
    let depois = blueprint(&e, "sm", "p1");
    assert_eq!(depois["acoes"][0], antes["acoes"][0], "f1 segue como estava na árvore");
    assert_eq!(depois["acoes"].as_array().unwrap().len(), 2);

    // remover também sobrevive à escrita seguinte
    let v3 = ok(&e, "abrir", json!({"workspaceId": "sm", "blueprintId": "p1", "origem": "escrita", "removerAcoes": ["n"]}))["escritaId"].as_str().unwrap().to_string();
    assert_eq!(fechar(&e, &v3)["removidas"], json!(["n"]));
    let v4 = abrir_vazio(&e, "sm", "p1");
    let r = fechar(&e, &v4);
    assert_eq!((&r["removidas"], &r["alteradas"][0]["ref"]), (&json!(["n"]), &json!("f1")));
}

#[test]
fn segunda_versao_nao_pede_figura_antiga() {
    let e = janela();
    let v1 = abrir_vazio(&e, "w2", "p9");
    ok(&e, "figura", json!({"escritaId": v1, "caminho": "a.svg", "tipo": "svg", "svg": SVG}));
    escrever(&e, &v1, json!({"id": "1", "nome": "N", "detalhes": [{"id": "d", "figura": "a.svg"}]}));
    fechar(&e, &v1);
    let hash = blueprint(&e, "w2", "p9")["acoes"][0]["detalhes"][0]["figura_info"]["hash"].clone();

    let v2 = abrir_vazio(&e, "w2", "p9");
    escrever(&e, &v2, json!({"id": "1", "texto": "Agora com texto", "detalhes": [{"id": "d", "titulo": "Com título", "figura": "a.svg"}]}));
    let r = fechar(&e, &v2);
    assert_eq!(r["alteradas"][0]["ref"], "1", "{r}");
    let p = e.dados.lock().unwrap().plano_pendente("p9").unwrap().unwrap();
    assert_eq!(p.plano["projetos"][0]["acoes"][0]["detalhes"][0]["figura_info"]["hash"], hash, "fica a medida da versão anterior");

    // a pendente também: a terceira versão parte dela e não pede a figura de novo
    let v3 = abrir_vazio(&e, "w2", "p9");
    escrever(&e, &v3, json!({"id": "1", "texto": "Mais texto"}));
    fechar(&e, &v3);
    // citar outra figura, sem mandar, volta a pedir
    let v4 = abrir_vazio(&e, "w2", "p9");
    escrever(&e, &v4, json!({"id": "1", "detalhes": [{"id": "d", "figura": "b.svg"}]}));
    assert_eq!(post(&e, "fechar", json!({"escritaId": v4})).status, 422);
}

#[test]
fn acao_criada_durante_escrita_nao_some() {
    let e = janela();
    let escrita = abrir_vazio(&e, "sm", "p1");
    t1_alterado(&e, &escrita);
    editar_arvore(&e, |t| {
        t["blueprints"][0]["acoes"].as_array_mut().unwrap().push(json!({"id": "f9", "nome": "Do usuário", "status": "planejado", "planos": [], "checkpoints": []}));
    });
    let r = fechar(&e, &escrita);
    assert_eq!((&r["removidas"], &r["iguais"]), (&json!([]), &json!(1)), "{r}");
    let bp = blueprint(&e, "sm", "p1");
    assert_eq!(bp["acoes"].as_array().unwrap().len(), 2);
    let p = e.dados.lock().unwrap().plano_pendente("p1").unwrap().unwrap();
    let doc = p.plano["projetos"].to_string();
    assert!(doc.contains("f9"), "a versão também a tem, para aprovar não a tirar: {doc}");
}

#[test]
fn texto_editado_durante_escrita_nao_reverte() {
    let e = janela();
    let escrita = abrir_em(&e, "sm", "p1", "escrita", json!([{"id": "n", "nome": "Nova"}]))["escritaId"].as_str().unwrap().to_string();
    editar_arvore(&e, |t| {
        t["blueprints"][0]["nome"] = json!("Renomeado");
        t["blueprints"][0]["acoes"][0]["planos"][0]["texto"] = json!("Editado pelo usuário");
    });
    let r = fechar(&e, &escrita);
    assert_eq!((&r["alteradas"], &r["iguais"], &r["removidas"]), (&json!([]), &json!(1), &json!([])), "{r}");
    let bp = blueprint(&e, "sm", "p1");
    assert_eq!(bp["acoes"][0]["planos"][0]["texto"], "Editado pelo usuário");
    assert_eq!(bp["nome"], "Renomeado", "o agente não mandou nome: fica o do usuário");
    assert_eq!(bp["acoes"][1]["ref"], "n");
}

#[test]
fn sem_janela_409() {
    let e = estado(false);
    for rota in ["abrir", "escrever", "figura", "fechar", "recado"] {
        let r = post(&e, rota, json!({"workspaceId": "w2", "blueprintId": "p9", "origem": "escrita"}));
        assert_eq!((r.status, json_de(&r)["codigo"].as_str()), (409, Some("sem-janela")), "{rota}");
    }
    assert_eq!(e.dados.lock().unwrap().plano_aberta("p9").unwrap(), None);
}

#[test]
fn recado_grava_para_a_execucao() {
    let e = janela();
    let antes = e.mcp.revisao_planejar();
    ok(&e, "recado", json!({"workspaceId": "w2", "blueprintId": "p9", "para": "e1", "execucaoId": "e2", "texto": " Olá ", "de": "orquestrador"}));
    assert!(e.mcp.revisao_planejar() > antes);
    let d = e.dados.lock().unwrap();
    let l = d.recados_para("e1").unwrap();
    assert_eq!((l.len(), l[0].texto.as_str(), l[0].de.as_str(), l[0].blueprint_id.as_str()), (1, "Olá", "orquestrador", "p9"));
    drop(d);
    assert_eq!(post(&e, "recado", json!({"workspaceId": "w2", "texto": "x", "de": "usuario"})).status, 400, "o usuário fala por gesto");
    assert_eq!(post(&e, "recado", json!({"workspaceId": "w2", "texto": " ", "de": "agente"})).status, 400);
    assert_eq!(post(&e, "recado", json!({"workspaceId": "nao-existe", "texto": "x", "de": "agente"})).status, 404);
}
