use super::formato::*;
use serde_json::{json, Value};

const EXEMPLOS: [(&str, &str); 4] = [
    ("monitor", include_str!("fixtures/monitor/plano.json")),
    ("monitor/anterior", include_str!("fixtures/monitor/anterior.json")),
    ("biblioteca", include_str!("fixtures/biblioteca/plano.json")),
    ("reforma", include_str!("fixtures/reforma/plano.json")),
];

fn doc(json_txt: &str) -> PlanoDoc {
    PlanoDoc::de_json(&serde_json::from_str::<Value>(json_txt).unwrap()).unwrap()
}

fn minimo(acao: Value) -> Value {
    json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": [acao]}]})
}

fn completo() -> PlanoDoc {
    PlanoDoc::de_json(&json!({
        "formato": FORMATO, "blueprint": "B", "resumo": "", "estilo": "plano.css", "tema": {"papel": "#123"},
        "projetos": [{"nome": "P", "acoes": [{"id": "1", "nome": "A", "de": "2026-01-01", "ate": "2026-01-02", "texto": "t",
            "planos": [{"id": "1.1", "texto": "x", "area": "a/b", "depende_de": ["1.0"], "instrucoes": "i"}],
            "checkpoints": [{"id": "c", "texto": "k", "data": "2026-01-02"}],
            "detalhes": [{"id": "01", "titulo": "T", "legenda": "L", "grupo": "G", "figura": "f.svg", "pede_aprovacao": true}],
            "tabelas": [{"titulo": "Tb", "colunas": ["a", "b"], "larguras": [0.25, 0.75], "linhas": [["1", "2"]]}]}]}, {"acoes": []}],
        "decisoes": [{"id": "d", "rotulo": "R", "texto": "t", "data": "2026-01-01", "acoes": ["1"], "pede_aprovacao": true}],
        "perguntas": [{"id": "q", "texto": "t", "opcoes": ["a", "b"], "recomendada": "b", "acoes": ["1"]}]
    })).unwrap()
}

#[test]
fn formato_ida_e_volta() {
    let d = completo();
    assert_eq!(PlanoDoc::de_json(&d.para_json()).unwrap(), d);
    assert_eq!(d.resumo.as_deref(), Some(""), "apagar é mandar vazio: não vira ausente");
    assert_eq!(d.projetos[1].acoes, vec![], "projeto sem ação continua na volta");
    for (nome, txt) in EXEMPLOS {
        let d = doc(txt);
        assert_eq!(PlanoDoc::de_json(&d.para_json()).unwrap(), d, "{nome}");
    }
}

#[test]
fn plano_string_ganha_id() {
    let d = PlanoDoc::de_json(&minimo(json!({"id": "1", "nome": "A", "planos": ["Fazer", {"id": "1.9", "texto": "Outro"}, {"texto": "Sem id"}]}))).unwrap();
    let p = &d.projetos[0].acoes[0].planos;
    assert_eq!((p[0].id.as_str(), p[0].texto.as_str(), p[0].id_gerado), ("1.1", "Fazer", true));
    assert_eq!((p[1].id.as_str(), p[1].texto.as_str(), p[1].id_gerado), ("1.9", "Outro", false));
    assert_eq!((p[2].id.as_str(), p[2].id_gerado), ("1.3", true));
    let volta = d.para_json();
    assert_eq!(volta["projetos"][0]["acoes"][0]["planos"][0], json!({"id": "1.1", "texto": "Fazer"}), "na volta o id é explícito e `id_gerado` não vai");
}

#[test]
fn exemplos_decodificam() {
    for (nome, txt) in EXEMPLOS {
        let d = doc(txt);
        assert_eq!(d.formato, FORMATO, "{nome}");
        assert!(!d.blueprint.is_empty() && !d.projetos.is_empty(), "{nome}");
    }
    let m = doc(EXEMPLOS[0].1);
    assert_eq!(m.blueprint, "Monitor de consumo");
    let a = &m.projetos[0].acoes[1];
    assert_eq!((a.id.as_str(), a.planos.len()), ("1.2", 3));
    assert_eq!(a.planos[1].depende_de, ["1.2.1"]);
    assert_eq!(a.planos[1].area.as_deref(), Some("painel/tela"));
    assert_eq!(a.checkpoints[0].id, "1.2.c1", "o checkpoint sem id ganha o padrão");
    assert_eq!(a.detalhes[0].figura.as_deref(), Some("figuras/painel.svg"));
    assert!(a.detalhes[0].pede_aprovacao);
    assert_eq!(m.decisoes[0].acoes, ["1.1", "1.2"]);
    assert_eq!(m.perguntas[0].recomendada.as_deref(), Some("15 min"), "o índice dos exemplos vira o texto da opção");
    let b = doc(EXEMPLOS[2].1);
    assert_eq!(b.projetos[0].nome.as_deref(), Some("Versão 2.0"));
    assert_eq!(b.projetos[0].acoes[0].tabelas[0].larguras, [0.4, 0.4, 0.2]);
    assert_eq!(b.projetos[0].acoes[0].tabelas[0].linhas[1], ["agora()", "agora(fuso)", "sim"]);
}

#[test]
fn figura_info_nao_vem_do_json() {
    let forjada = json!({"hash": "h", "tipo": "svg", "largura": 1, "altura": 1});
    let d = PlanoDoc::de_json(&minimo(json!({"id": "1", "nome": "A", "detalhes": [{"id": "01", "figura": "f.svg", "figura_info": forjada, "figuraInfo": forjada}]}))).unwrap();
    assert_eq!(d.projetos[0].acoes[0].detalhes[0].figura_info, None);
    assert_eq!(Detalhe::de_json(&json!({"id": "01", "figura_info": forjada})).unwrap().figura_info, None);
    // a volta do que o servidor gravou é explícita, por `FiguraInfo::de_json`
    let info = FiguraInfo { hash: "h".into(), tipo: "svg".into(), largura: Some(10), altura: None };
    let detalhe = Detalhe { id: "01".into(), figura_info: Some(info.clone()), ..Detalhe::default() }.para_json();
    assert_eq!(FiguraInfo::de_json(&detalhe["figura_info"]), Some(info));
}

#[test]
fn estado_no_json_nao_tem_efeito() {
    let com = minimo(json!({"id": "1", "nome": "A", "aprovado": true, "feito": true, "estado": "aprovada", "planos": [{"id": "1.1", "texto": "x", "feito": true, "aprovado": true, "desconhecido": 1}]}));
    let sem = minimo(json!({"id": "1", "nome": "A", "planos": [{"id": "1.1", "texto": "x"}]}));
    assert_eq!(PlanoDoc::de_json(&com).unwrap(), PlanoDoc::de_json(&sem).unwrap());
    let volta = PlanoDoc::de_json(&com).unwrap().para_json().to_string();
    assert!(!volta.contains("aprovado") && !volta.contains("feito") && !volta.contains("estado"), "{volta}");
}

#[test]
fn o_que_a_struct_nao_guarda_e_erro() {
    let erro = |v: Value| PlanoDoc::de_json(&v).unwrap_err().join("; ");
    assert!(erro(json!({"formato": "blueprint.plano/2", "blueprint": "B", "projetos": []})).contains("blueprint.plano/2"));
    assert!(erro(json!({"blueprint": "B", "projetos": []})).contains(FORMATO));
    assert!(erro(json!({"formato": FORMATO, "blueprint": "B"})).contains("projetos"));
    assert!(erro(json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"nome": "P"}]})).contains("acoes"), "`acoes` ausente é erro");
    assert!(PlanoDoc::de_json(&json!({"formato": FORMATO, "blueprint": "B", "projetos": [{"acoes": []}]})).is_ok(), "`acoes: []` passa; o aviso é de `validar::avisos`");
    assert!(erro(minimo(json!({"id": 1, "nome": "A"}))).contains("id must be a string"));
    assert!(erro(minimo(json!({"id": "1", "nome": "A", "planos": [7]}))).contains("plan 1.1"));
    assert!(erro(minimo(json!({"id": "1", "nome": "A", "planos": [{"id": "1.1", "depende_de": "x"}]}))).contains("depende_de"));
    assert!(erro(json!({"formato": FORMATO, "projetos": [], "tema": {"a": 1}})).contains("tema"));
    assert!(erro(json!({"formato": FORMATO, "projetos": [], "perguntas": [{"id": "q", "opcoes": ["a"], "recomendada": 5}]})).contains("recomendada"));
    assert!(erro(json!([])).contains("object"));
}

#[test]
fn pecas_sozinhas() {
    let a = AcaoDoc::de_json(&json!({"id": "1", "nome": "A", "planos": [{"id": "1.1", "texto": "x"}]})).unwrap();
    assert_eq!(a.planos[0].texto, "x");
    assert_eq!(a.para_json()["planos"][0], json!({"id": "1.1", "texto": "x"}));
    assert!(AcaoDoc::de_json(&json!({"id": 1})).is_err());
    assert_eq!(Decisao::de_json(&json!({"id": "d", "texto": "t", "pede_aprovacao": true})).unwrap().pede_aprovacao, true);
    assert_eq!(Pergunta::de_json(&json!({"id": "q", "opcoes": ["a"], "recomendada": 0})).unwrap().recomendada.as_deref(), Some("a"));
    assert_eq!(Tabela::de_json(&json!({"colunas": ["a"], "linhas": [[1]]})).unwrap().linhas[0], ["1"]);
}
