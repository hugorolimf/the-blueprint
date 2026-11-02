//! Testes de plano::arvore (seção 2 do contrato). Passa de 300 linhas: são os 15 casos que o contrato nomeia, sobre o mesmo exemplo, e a fatia não pode criar outro módulo de teste.
use super::arvore::{do_blueprint, hash_peca, projetar, recompor, Escopo};
use super::comparar::comparar_planos;
use super::formato::{PlanoDoc, FORMATO};
use serde_json::{json, Value};
use std::collections::HashSet;

const HOJE: &str = "2026-11-01";

fn exemplo() -> Value {
    let tudo: Value = serde_json::from_str(include_str!("../dados/tests/exemplo.json")).unwrap();
    tudo["blueprints"][0].clone()
}

fn doc(v: Value) -> PlanoDoc {
    PlanoDoc::de_json(&v).unwrap()
}

fn doc_de(acoes: Value) -> PlanoDoc {
    doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": acoes}]}))
}

fn acao<'a>(bp: &'a Value, id: &str) -> &'a Value {
    bp["acoes"].as_array().unwrap().iter().find(|a| a["id"] == id || a["ref"] == id).unwrap_or_else(|| panic!("sem ação {id}"))
}

fn plano<'a>(a: &'a Value, r: &str) -> &'a Value {
    a["planos"].as_array().unwrap().iter().find(|q| q["ref"] == r).unwrap_or_else(|| panic!("sem plano {r}"))
}

/// Tira o `ref` de toda parte, para comparar com a árvore original.
fn sem_ref(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(m.iter().filter(|(k, _)| *k != "ref").map(|(k, x)| (k.clone(), sem_ref(x))).collect()),
        Value::Array(l) => Value::Array(l.iter().map(sem_ref).collect()),
        _ => v.clone(),
    }
}

fn assert_tudo_igual(depois: &PlanoDoc, d: &PlanoDoc) {
    let c = comparar_planos(Some(depois), d);
    assert!(c.novas.is_empty() && c.removidas.is_empty(), "{c:?}");
    assert!(c.alteradas.is_empty(), "{:?}", c.alteradas);
}

#[test]
fn ida_e_volta_sem_mudanca() {
    let fixtures = [include_str!("fixtures/monitor/plano.json"), include_str!("fixtures/reforma/plano.json"), include_str!("fixtures/biblioteca/plano.json")];
    for texto in fixtures {
        let d = doc(serde_json::from_str(texto).unwrap());
        let vazio = projetar(&json!({"id": "b1", "nome": "x"}), &d, Escopo::Tudo, HOJE);
        assert_tudo_igual(&do_blueprint(&vazio), &d);
        assert_eq!(do_blueprint(&vazio).decisoes, d.decisoes);
        assert_eq!(do_blueprint(&vazio).perguntas, d.perguntas);
        assert_eq!(do_blueprint(&vazio).tema, d.tema);
        // Por cima de um blueprint que já tem outra coisa, e projetar de novo não muda nada.
        let cheio = projetar(&exemplo(), &d, Escopo::Tudo, HOJE);
        assert_tudo_igual(&do_blueprint(&cheio), &d);
        assert_eq!(projetar(&cheio, &d, Escopo::Tudo, HOJE), cheio);
    }
    // A árvore antiga, para doc e de volta.
    let b = exemplo();
    let d = do_blueprint(&b);
    assert_eq!(do_blueprint(&projetar(&b, &d, Escopo::Tudo, HOJE)), d);
}

#[test]
fn feito_sobrevive() {
    let b = exemplo();
    let mut d = do_blueprint(&b);
    d.projetos[0].acoes[0].planos[0].texto = "Criar rota nova".into();
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    let t1 = plano(acao(&depois, "f1"), "t1");
    assert_eq!((t1["id"].as_str(), t1["texto"].as_str(), &t1["feito"]), (Some("t1"), Some("Criar rota nova"), &json!(true)));
    assert_eq!(t1["agente"], b["acoes"][0]["planos"][0]["agente"]);
    let m1 = &acao(&depois, "f1")["checkpoints"][0];
    assert_eq!((&m1["feito"], m1["evidencia"].as_str(), m1["feitoEm"].as_str()), (&json!(true), Some("ok"), Some("2026-11-20T12:00:00Z")));
    assert_tudo_igual(&do_blueprint(&depois), &d);
}

#[test]
fn so_uma_acao() {
    let b = exemplo();
    let mut d = do_blueprint(&b);
    d.projetos[0].acoes[0].nome = "Base nova".into();
    d.projetos[0].acoes[1].nome = "F2 nova".into();
    d.projetos[0].acoes.extend(doc_de(json!([{"id": "n", "nome": "Nova"}])).projetos[0].acoes.clone());
    d.resumo = Some("resumo novo".into());

    let so = ["f1".to_string()];
    let depois = projetar(&b, &d, Escopo::So(&so), HOJE);
    let nomes: Vec<&str> = depois["acoes"].as_array().unwrap().iter().map(|a| a["nome"].as_str().unwrap()).collect();
    assert_eq!(nomes, ["Base nova", "Sem listas"], "f2 e a nova ficam de fora");
    assert_eq!(depois["descricao"], "resumo novo", "o que é do blueprint entra em qualquer escopo");

    let so = ["n".to_string()];
    let depois = projetar(&b, &d, Escopo::So(&so), HOJE);
    let nomes: Vec<&str> = depois["acoes"].as_array().unwrap().iter().map(|a| a["nome"].as_str().unwrap()).collect();
    assert_eq!(nomes, ["Base", "Sem listas", "Nova"], "a nova entra depois da anterior no doc");

    let fora = ["f1".to_string()];
    let depois = projetar(&b, &d, Escopo::SoFora(&fora), HOJE);
    let nomes: Vec<&str> = depois["acoes"].as_array().unwrap().iter().map(|a| a["nome"].as_str().unwrap()).collect();
    assert_eq!(nomes, ["Base", "F2 nova", "Nova"]);
}

#[test]
fn remove_acao() {
    let b = exemplo();
    let mut d = do_blueprint(&b);
    d.projetos[0].acoes[0].nome = "Mudou".into();
    d.projetos[0].acoes.retain(|a| a.id != "f2");

    let so = ["f2".to_string()];
    let depois = projetar(&b, &d, Escopo::So(&so), HOJE);
    assert_eq!(depois["acoes"].as_array().unwrap().len(), 1);
    assert_eq!(depois["acoes"][0]["nome"], "Base", "f1 não estava no escopo");

    assert_eq!(projetar(&b, &d, Escopo::Tudo, HOJE)["acoes"].as_array().unwrap().len(), 1);

    let fora = ["f2".to_string()];
    let depois = projetar(&b, &d, Escopo::SoFora(&fora), HOJE);
    assert_eq!(depois["acoes"].as_array().unwrap().len(), 2, "fora do escopo, a ausente fica");
    let so_outra = ["f1".to_string()];
    assert_eq!(projetar(&b, &d, Escopo::So(&so_outra), HOJE)["acoes"].as_array().unwrap().len(), 2);
}

#[test]
fn ids_de_dois_blueprints_nao_colidem() {
    let d = doc_de(json!([{"id": "1", "nome": "Um", "planos": [{"id": "1.1", "texto": "x"}], "checkpoints": [{"id": "c", "texto": "y"}]}]));
    let a = projetar(&json!({"id": "a"}), &d, Escopo::Tudo, HOJE);
    let b = projetar(&json!({"id": "b"}), &d, Escopo::Tudo, HOJE);
    assert_ne!(a["acoes"][0]["id"], b["acoes"][0]["id"]);
    assert_ne!(a["acoes"][0]["planos"][0]["id"], b["acoes"][0]["planos"][0]["id"]);
    assert_ne!(a["acoes"][0]["checkpoints"][0]["id"], b["acoes"][0]["checkpoints"][0]["id"]);
    assert_eq!(a["acoes"][0]["ref"], "1");
    // Projetar de novo acha pelo ref e não troca o id.
    assert_eq!(projetar(&a, &d, Escopo::Tudo, HOJE)["acoes"][0]["id"], a["acoes"][0]["id"]);
}

#[test]
fn sem_data_nao_vai_e_volta_com_hoje() {
    let d = doc_de(json!([{"id": "1", "nome": "Um"}]));
    let b = projetar(&json!({"id": "b"}), &d, Escopo::Tudo, HOJE);
    let a = &b["acoes"][0];
    assert_eq!((a["inicio"].as_str(), a["fim"].as_str(), &a["semData"]), (Some(HOJE), Some(HOJE), &json!(true)));
    let volta = do_blueprint(&b);
    assert_eq!((&volta.projetos[0].acoes[0].de, &volta.projetos[0].acoes[0].ate), (&None, &None));
    assert_tudo_igual(&volta, &d);

    let com_data = doc_de(json!([{"id": "1", "nome": "Um", "de": "2026-12-01", "ate": "2026-12-05"}]));
    let a = projetar(&b, &com_data, Escopo::Tudo, "2027-01-01");
    assert!(a["acoes"][0].get("semData").is_none());
    assert_eq!(a["acoes"][0]["inicio"], "2026-12-01");
    // Uma ação com datas que passa a não ter: o doc vale, e o `hoje` não substitui o que já havia para a web legada.
    let de_novo = projetar(&a, &d, Escopo::Tudo, "2027-01-01");
    assert_eq!((de_novo["acoes"][0]["semData"].as_bool(), de_novo["acoes"][0]["inicio"].as_str()), (Some(true), Some("2026-12-01")));
    assert_tudo_igual(&do_blueprint(&de_novo), &d);
}

#[test]
fn blueprint_antigo_ref_e_o_id() {
    let b = exemplo();
    let d = do_blueprint(&b);
    let f1 = &d.projetos[0].acoes[0];
    assert_eq!(f1.id, "f1");
    assert_eq!(f1.planos.iter().map(|q| q.id.as_str()).collect::<Vec<_>>(), ["t1", "t2", "t3", "t4"]);
    assert_eq!(f1.checkpoints[0].id, "m1");
    assert_eq!((f1.de.as_deref(), f1.ate.as_deref(), f1.texto.as_deref()), (Some("2026-11-01"), Some("2026-11-30"), Some("instruções")));
    assert_eq!(f1.planos[0].instrucoes.as_deref(), Some("x"));
    assert!(!f1.planos[0].id_gerado);
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    assert_eq!(acao(&depois, "f1")["id"], "f1");
    assert_eq!(plano(acao(&depois, "f1"), "t1")["id"], "t1");
    assert_eq!(acao(&depois, "f1")["checkpoints"][0]["id"], "m1");
}

#[test]
fn estado_nao_entra_em_extra() {
    let d = doc(json!({"formato": FORMATO, "blueprint": "B", "estado": "aprovado", "aprovado": true,
        "decisoes": [{"id": "d1", "texto": "t", "aprovado": true, "estado": "aprovada"}],
        "perguntas": [{"id": "q1", "texto": "t", "resposta": "x", "estado": "respondida"}],
        "projetos": [{"acoes": [{"id": "1", "nome": "Um", "estado": "x", "aprovado": true, "feito": true,
            "planos": [{"id": "1.1", "texto": "x", "feito": true, "estado": "pronto", "aprovado": true}],
            "checkpoints": [{"id": "c", "texto": "y", "feito": true, "evidencia": "forjada"}],
            "detalhes": [{"id": "d", "aprovado": true, "figura_info": {"hash": "forjado", "tipo": "svg"}}]}]}]}));
    let b = projetar(&json!({"id": "b"}), &d, Escopo::Tudo, HOJE);
    let a = &b["acoes"][0];
    assert_eq!(a["planos"][0]["feito"], false);
    assert_eq!(a["checkpoints"][0]["feito"], false);
    assert!(a["checkpoints"][0].get("evidencia").is_none());
    assert!(a["detalhes"][0].get("figura_info").is_none());
    let texto = b.to_string();
    for proibido in ["estado", "aprovado", "resposta", "forjad"] {
        assert!(!texto.contains(proibido), "{proibido} em {texto}");
    }
}

#[test]
fn campos_alheios_sobrevivem() {
    let b = exemplo();
    let depois = projetar(&b, &do_blueprint(&b), Escopo::Tudo, HOJE);
    assert_eq!(sem_ref(&depois), b, "só o ref é novo");
    // Um campo qualquer, em cada nível.
    let mut outro = b.clone();
    outro["acoes"][0]["horas"] = json!(3);
    outro["acoes"][0]["planos"][1]["responsavel"] = json!("alguém");
    outro["acoes"][0]["checkpoints"][1]["novo"] = json!({"a": 1});
    let mut d = do_blueprint(&outro);
    d.projetos[0].acoes[0].nome = "Outro nome".into();
    let depois = projetar(&outro, &d, Escopo::Tudo, HOJE);
    assert_eq!(acao(&depois, "f1")["horas"], 3);
    assert_eq!(plano(acao(&depois, "f1"), "t2")["responsavel"], "alguém");
    assert_eq!(acao(&depois, "f1")["checkpoints"][1]["novo"], json!({"a": 1}));
    assert_eq!(acao(&depois, "f1")["prototipos"], b["acoes"][0]["prototipos"]);
    assert_eq!(plano(acao(&depois, "f1"), "t3")["novo"], b["acoes"][0]["planos"][2]["novo"]);
    assert_eq!(depois["links"], b["links"]);
}

#[test]
fn projeto_existente_mantem_id_e_descricao() {
    let mut b = exemplo();
    b["projetos"] = json!([{"id": "pj1", "nome": "Front", "descricao": "dd", "cor": "x"}, {"id": "pj2", "nome": "Sem uso"}]);
    let mut d = do_blueprint(&b);
    let acoes = std::mem::take(&mut d.projetos[0].acoes);
    d.projetos = vec![
        super::formato::ProjetoDoc { nome: Some("  front ".into()), acoes: acoes[..1].to_vec() },
        super::formato::ProjetoDoc { nome: Some("Back".into()), acoes: acoes[1..].to_vec() },
    ];
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    let projetos = depois["projetos"].as_array().unwrap();
    assert_eq!(projetos.len(), 3, "um novo; nenhum removido");
    assert_eq!(projetos[0], json!({"id": "pj1", "nome": "Front", "descricao": "dd", "cor": "x"}));
    assert_eq!(projetos[1]["id"], "pj2");
    assert_eq!(acao(&depois, "f1")["projetoId"], "pj1");
    assert_eq!(acao(&depois, "f2")["projetoId"], projetos[2]["id"]);
    assert_eq!(projetos[2]["id"], super::ids::id_real("p1", "projeto", "Back"));
    let volta = do_blueprint(&depois);
    assert_eq!(volta.projetos[0].nome.as_deref(), Some("Front"));
    assert_eq!(volta.projetos[2].nome.as_deref(), Some("Sem uso"), "projeto sem ação continua no doc");
    // Sem nome de projeto, a ação sai do grupo, e o projeto fica.
    let solto = doc_de(json!([{"id": "f1", "nome": "Base", "de": "2026-11-01", "ate": "2026-11-30"}]));
    let depois2 = projetar(&depois, &solto, Escopo::Tudo, HOJE);
    assert!(acao(&depois2, "f1").get("projetoId").is_none());
    assert_eq!(depois2["projetos"].as_array().unwrap().len(), 3);
}

#[test]
fn resumo_ausente_nao_apaga() {
    let mut b = exemplo();
    b["acoes"][0]["planos"][1]["area"] = json!("app");
    let mut d = do_blueprint(&b);
    d.resumo = None;
    d.projetos[0].acoes[0].texto = None;
    d.projetos[0].acoes[0].planos[0].instrucoes = None;
    d.projetos[0].acoes[0].planos[1].area = None;
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    assert_eq!(depois["descricao"], "d");
    assert_eq!(acao(&depois, "f1")["obs"], "instruções");
    assert_eq!(plano(acao(&depois, "f1"), "t1")["descricao"], "x");
    assert_eq!(plano(acao(&depois, "f1"), "t2")["area"], "app");
    assert_tudo_igual(&do_blueprint(&depois), &d);

    d.resumo = Some(String::new());
    d.projetos[0].acoes[0].texto = Some(String::new());
    d.projetos[0].acoes[0].planos[0].instrucoes = Some(String::new());
    d.projetos[0].acoes[0].planos[1].area = Some(String::new());
    let vazio = do_blueprint(&projetar(&b, &d, Escopo::Tudo, HOJE));
    assert_eq!(vazio.resumo, None, "apagar é mandar vazio");
    let f1 = &vazio.projetos[0].acoes[0];
    assert_eq!((&f1.texto, &f1.planos[0].instrucoes, &f1.planos[1].area), (&None, &None, &None));
    assert_tudo_igual(&vazio, &d);
}

#[test]
fn recompor_mantem_tocadas() {
    let base_inicial = doc_de(json!([{"id": "1", "nome": "Um"}, {"id": "2", "nome": "Dois"}, {"id": "3", "nome": "Três"}, {"id": "apagada", "nome": "Vai sumir"}]));
    // O que o agente escreveu: mudou a 1, removeu a 3, criou a nova, e o doc ainda tem a 2 e a "apagada" como eram.
    let mut escrito = base_inicial.clone();
    escrito.resumo = Some("do agente".into());
    escrito.projetos[0].acoes[0].nome = "Um do agente".into();
    escrito.projetos[0].acoes.retain(|a| a.id != "3");
    escrito.projetos[0].acoes.insert(1, doc_de(json!([{"id": "nova", "nome": "Nova"}])).projetos[0].acoes[0].clone());
    // Enquanto isso, o usuário editou a 2, criou a sua e apagou a "apagada".
    let mut agora = base_inicial.clone();
    agora.projetos[0].acoes[1].nome = "Dois do usuário".into();
    agora.projetos[0].acoes.retain(|a| a.id != "apagada");
    agora.projetos[0].acoes.push(doc_de(json!([{"id": "minha", "nome": "Minha"}])).projetos[0].acoes[0].clone());

    let feito = recompor(&escrito, &agora, &["1".to_string(), "nova".to_string()], &["3".to_string()]);
    let nomes: Vec<(&str, &str)> = feito.projetos[0].acoes.iter().map(|a| (a.id.as_str(), a.nome.as_str())).collect();
    assert_eq!(nomes, [("1", "Um do agente"), ("nova", "Nova"), ("2", "Dois do usuário"), ("minha", "Minha")]);
    assert_eq!(feito.resumo.as_deref(), Some("do agente"));

    // Tocada que mudou de projeto vai para o projeto do doc.
    let base = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"nome": "A", "acoes": [{"id": "1", "nome": "Um"}, {"id": "2", "nome": "Dois"}]}]}));
    let movido = doc(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"nome": "A", "acoes": [{"id": "2", "nome": "Dois"}]}, {"nome": "B", "acoes": [{"id": "1", "nome": "Um"}]}]}));
    let r = recompor(&movido, &base, &["1".to_string()], &[]);
    assert_eq!(r.projetos[0].acoes.len(), 1);
    assert_eq!((r.projetos[1].nome.as_deref(), r.projetos[1].acoes[0].id.as_str()), (Some("B"), "1"));
}

#[test]
fn hash_peca_estavel() {
    let a: Value = serde_json::from_str(r#"{"b":1,"a":{"y":[1,{"q":2,"p":3}],"x":"é"}}"#).unwrap();
    let b: Value = serde_json::from_str(r#"{"a":{"x":"é","y":[1,{"p":3,"q":2}]},"b":1}"#).unwrap();
    assert_eq!(hash_peca(&a), hash_peca(&b));
    assert_eq!(hash_peca(&a).len(), 64);
    assert_ne!(hash_peca(&a), hash_peca(&json!({"b": 2, "a": {"x": "é", "y": [1, {"p": 3, "q": 2}]}})));
    assert_ne!(hash_peca(&json!([1, 2])), hash_peca(&json!([2, 1])), "a ordem de lista conta");
    assert_eq!(hash_peca(&json!({"a": "x"})), hash_peca(&json!({"a": "x"})));
}

#[test]
fn ref_igual_id_real_nao_herda_feito() {
    let b = exemplo(); // t1 é feito, na ação f1
    let mut d = do_blueprint(&b);
    // A f2 ganha um plano cujo ref imita o id real de t1; uma ação nova também.
    let imitadores = doc_de(json!([{"id": "f2", "nome": "Sem listas", "de": "2026-12-01", "ate": "2026-12-10", "planos": [{"id": "t1", "texto": "imita"}], "checkpoints": [{"id": "m1", "texto": "imita"}]}, {"id": "nova", "nome": "Nova", "planos": [{"id": "t1", "texto": "imita"}]}]));
    d.projetos[0].acoes.truncate(1);
    d.projetos[0].acoes.extend(imitadores.projetos[0].acoes.clone());
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    assert_eq!(plano(acao(&depois, "f1"), "t1")["feito"], true);
    let f2 = plano(acao(&depois, "f2"), "t1");
    assert_eq!(f2["feito"], false);
    assert_ne!(f2["id"], "t1");
    assert!(f2.get("agente").is_none());
    assert_eq!(acao(&depois, "f2")["checkpoints"][0]["feito"], false);
    assert_ne!(acao(&depois, "f2")["checkpoints"][0]["id"], "m1");
    assert_eq!(plano(acao(&depois, "nova"), "t1")["feito"], false);

    // Dentro da mesma ação, um item só responde por um ref: um novo com o id real de um renomeado não copia o estado.
    let renomeado = projetar(&b, &do_blueprint(&b), Escopo::Tudo, HOJE);
    let mut ja = renomeado.clone();
    ja["acoes"][0]["planos"][0]["ref"] = json!("x");
    let dois = doc_de(json!([{"id": "f1", "nome": "Base", "de": "2026-11-01", "ate": "2026-11-30", "planos": [{"id": "t1", "texto": "imita"}, {"id": "x", "texto": "o verdadeiro"}]}]));
    let saida = projetar(&ja, &dois, Escopo::Tudo, HOJE);
    assert_eq!(plano(acao(&saida, "f1"), "x")["feito"], true);
    assert_eq!(plano(acao(&saida, "f1"), "t1")["feito"], false);
    assert_ne!(plano(acao(&saida, "f1"), "t1")["id"], "t1");
}

#[test]
fn mover_plano_nao_duplica_id() {
    let b = exemplo();
    let mut d = do_blueprint(&b);
    let movido = d.projetos[0].acoes[0].planos.remove(0); // t1, feito
    d.projetos[0].acoes[1].planos.push(movido);
    let depois = projetar(&b, &d, Escopo::Tudo, HOJE);
    let ids: Vec<&str> = depois["acoes"].as_array().unwrap().iter().flat_map(|a| a["planos"].as_array().into_iter().flatten()).map(|q| q["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), ids.iter().collect::<HashSet<_>>().len(), "{ids:?}");
    assert!(!ids.contains(&"t1"), "mover é remover e criar");
    let novo = plano(acao(&depois, "f2"), "t1");
    assert_eq!(novo["feito"], false);
    assert!(novo.get("agente").is_none());
    assert!(acao(&depois, "f1")["planos"].as_array().unwrap().iter().all(|q| q["ref"] != "t1"));

    // Aprovar só o destino deixa os dois, ainda com ids diferentes.
    let so = ["f2".to_string()];
    let parcial = projetar(&b, &d, Escopo::So(&so), HOJE);
    assert_ne!(plano(acao(&parcial, "f2"), "t1")["id"], "t1");
    assert_eq!(acao(&parcial, "f1")["planos"][0]["id"], "t1");
}
