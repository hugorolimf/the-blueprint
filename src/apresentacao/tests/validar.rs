use super::{conteudo, figura};
use crate::apresentacao::validar::{achados, termos_de, validar};
use serde_json::{json, Value};

fn erro(c: &Value, f: &Value) -> (u16, String) {
    let e = validar(c, f).err().expect("devia recusar");
    (e.status, e.codigo.to_string())
}

#[test]
fn o_conteudo_do_contrato_passa_e_epico_so_vale_se_vier_escrito() {
    let v = validar(&conteudo(), &json!([figura("fichas")])).unwrap();
    assert_eq!((v.titulo.as_str(), v.publico.as_str(), v.nivel.as_str()), ("Atendimento de usuários", "cliente", "padrao"));
    let mut epico = conteudo();
    epico["nivel"] = json!("epico");
    assert_eq!(validar(&epico, &json!([])).unwrap().nivel, "epico");
    epico["nivel"] = json!("");
    assert_eq!(erro(&epico, &json!([])), (400, "nivel-invalido".into()), "nenhum nível é escolhido por omissão");
}

#[test]
fn recusa_o_que_fere_o_contrato() {
    let ok = json!([figura("fichas")]);
    assert_eq!(erro(&Value::Null, &ok).1, "falta-conteudo");
    assert_eq!(erro(&json!("texto"), &ok).1, "falta-conteudo");
    for (campo, valor, codigo) in [("titulo", json!("  "), "falta-titulo"), ("publico", json!("todos"), "publico-invalido"), ("nivel", json!("gigante"), "nivel-invalido")] {
        let mut c = conteudo();
        c[campo] = valor;
        assert_eq!(erro(&c, &ok), (400, codigo.into()));
    }
    let mut c = conteudo();
    c["interno"] = json!({"acoes": []});
    assert_eq!(erro(&c, &ok), (400, "interno-no-cliente".into()), "o bloco interno nunca chega ao cliente");
    c["publico"] = json!("interno");
    assert!(validar(&c, &ok).is_ok(), "no público interno ele vale");
}

#[test]
fn os_limites_do_contrato() {
    assert_eq!(erro(&conteudo(), &json!({"id": "x"})).1, "figuras-invalidas");
    let treze: Vec<Value> = (0..13).map(|i| figura(&format!("f{i}"))).collect();
    assert_eq!(erro(&conteudo(), &json!(treze)).1, "figuras-demais");
    let doze: Vec<Value> = (0..12).map(|i| figura(&format!("f{i}"))).collect();
    assert!(validar(&conteudo(), &json!(doze)).is_ok());
    let longo = "x".repeat(41);
    for id in ["", "Fichas", "-a", "a b", "a/../b", longo.as_str()] {
        assert_eq!(erro(&conteudo(), &json!([figura(id)])).1, "figura-id-invalido", "{id:?}");
    }
    assert_eq!(erro(&conteudo(), &json!([figura("a"), figura("a")])).1, "figura-id-repetido");
    let mut grande = figura("a");
    grande["codigo"] = json!("x".repeat(200 * 1024 + 1));
    assert_eq!(erro(&conteudo(), &json!([grande])), (413, "codigo-grande".into()));
    let mut torto = figura("a");
    torto["codigo"] = json!(42);
    assert_eq!(erro(&conteudo(), &json!([torto])).1, "codigo-invalido");
    let mut c = conteudo();
    c["resumo"] = json!(["x".repeat(128 * 1024)]);
    assert_eq!(erro(&c, &json!([])), (413, "conteudo-grande".into()));
    // 11 figuras de 190 KB: cada uma cabe, o conjunto passa de 2 MB
    let pesadas: Vec<Value> = (0..11).map(|i| json!({"id": format!("f{i}"), "impressao": "a", "codigo": "x".repeat(190 * 1024)})).collect();
    assert_eq!(erro(&conteudo(), &json!(pesadas)), (413, "apresentacao-grande".into()));
}

#[test]
fn os_avisos_nao_bloqueiam() {
    let semcodigo = json!([{"id": "fichas"}, {"id": "outra", "codigo": " ", "impressao": "x"}]);
    let v = validar(&conteudo(), &semcodigo).unwrap();
    let avisos = v.avisos.join("\n");
    assert!(avisos.contains("Figure \"fichas\" has no codigo") && avisos.contains("Figure \"outra\" has an empty codigo"), "{avisos}");
    assert!(avisos.contains("Figure \"fichas\" has no impressao"), "{avisos}");
    let desconhecida = validar(&conteudo(), &json!([figura("outra")])).unwrap().avisos.join("\n");
    assert!(desconhecida.contains("points to figure \"fichas\", which is not in figuras"), "{desconhecida}");
    assert!(validar(&conteudo(), &Value::Null).unwrap().avisos.iter().any(|a| a.contains("fichas")), "figuras ausente vale lista vazia");
}

#[test]
fn termo_tecnico_vale_como_palavra_inteira_e_horas_tambem_contam() {
    let termos: Vec<String> = ["rust", "webhooks", "api", "api rest", "sqlite", "cli"].map(String::from).to_vec();
    let c = json!({"etapas": [{"texto": "Usamos Rust e webhooks"}, {"texto": "Confiança total, trust no processo"}], "resumo": ["Uma API REST simples", "SQLite em arquivo"]});
    let achou = achados(&c, &termos);
    assert_eq!(achou, ["etapas[0].texto: rust", "etapas[0].texto: webhooks", "resumo[0]: api", "resumo[0]: api rest", "resumo[1]: sqlite"], "trust não casa com rust (meio de palavra)");
    assert!(achados(&json!({"texto": "Tudo em linguagem simples, um aplicativo para o cliente"}), &termos).is_empty(), "api não casa aplicativo e cli não casa cliente");
    assert!(achados(&json!({"publico": "cliente", "id": "cli", "figura": "api"}), &termos).is_empty(), "identificadores do contrato não são texto");
    for texto in ["Leva 3 h", "Leva 1,5 horas", "cerca de 4h por etapa", "about 2 hours"] {
        assert_eq!(achados(&json!({"texto": texto}), &termos), ["texto: hours"], "{texto}");
    }
    assert!(achados(&json!({"texto": "Etapa 1 de 3, pronta em 2026-10-28, hoje não"}), &termos).is_empty(), "número sozinho ou perto de outra palavra não é hora");
    let muitos = json!({"a": (0..30).map(|_| "rust").collect::<Vec<_>>()});
    assert_eq!(achados(&muitos, &termos).len(), 10, "até 10 achados");
}

#[test]
fn a_lista_de_termos_aceita_as_formas_do_arquivo() {
    assert_eq!(termos_de(r#"["Rust", " Webhook ", ""]"#), ["rust", "webhook"]);
    assert_eq!(termos_de(r#"{"versao": 1, "termos": ["API"]}"#), ["api"]);
    assert_eq!(termos_de(r#"{"linguagens": ["rust"], "infra": ["docker"], "nota": "x"}"#), ["docker", "rust"], "o objeto sem `termos` vale por todas as listas, na ordem das chaves");
    assert!(termos_de("não é json").is_empty());
}

#[test]
fn a_secao_da_figura_e_conferida_e_so_avisa() {
    let com_secao = |id: &str, secao: Value| {
        let mut f = figura(id);
        f["secao"] = secao;
        f
    };
    let avisos = |c: &Value, f: Value| validar(c, &f).unwrap().avisos.join("
");
    assert!(!avisos(&conteudo(), json!([com_secao("a", json!("visao")), com_secao("b", json!("riscos")), figura("c")])).contains("secao"), "ids conhecidos e figura sem seção não avisam");
    let desconhecida = avisos(&conteudo(), json!([com_secao("a", json!("nada")), com_secao("b", json!(7))]));
    assert_eq!(desconhecida.matches("secao-desconhecida").count(), 2, "{desconhecida}");
    assert!(desconhecida.contains("Figure \"a\" has secao \"nada\"") && desconhecida.contains("use visao, recebe, etapas, linha, precisamos, decisoes, riscos"));
    let duplicada = avisos(&conteudo(), json!([com_secao("a", json!("visao")), com_secao("b", json!("visao")), com_secao("c", json!("riscos"))]));
    assert!(duplicada.contains("secao-duplicada: Section \"visao\" has 2 figures (a, b)") && !duplicada.contains("\"riscos\" has"), "{duplicada}");
    // `acoes` só existe na página interno
    assert!(avisos(&conteudo(), json!([com_secao("a", json!("acoes"))])).contains("secao-desconhecida"));
    let mut interno = conteudo();
    interno["publico"] = json!("interno");
    assert!(!avisos(&interno, json!([com_secao("a", json!("acoes"))])).contains("secao"));
    assert!(validar(&conteudo(), &json!([com_secao("a", json!("nada"))])).is_ok(), "aviso, não erro");
}
