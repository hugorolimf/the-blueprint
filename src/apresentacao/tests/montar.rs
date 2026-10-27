use super::conteudo;
use crate::apresentacao::montar::{esc_script, escapar_html, montar, META_CSP};
use serde_json::{json, Value};

const MODELO: &str = "<!doctype html><html lang=\"{{idioma}}\"><head><title>{{titulo}}</title><style>{{css}}</style></head><body>{{kit}}<script>{{biblioteca}}</script>{{codigos}}<script>const C={{conteudo}};const F={{figuras}};{{desconhecido}}</script></body></html>";

#[test]
fn troca_cada_campo_uma_vez_e_deixa_o_desconhecido() {
    let html = montar(MODELO, "b{color:red}", "const L=1;", &conteudo(), &json!([{"id": "a"}]));
    assert!(html.contains("<title>Atendimento de usuários</title>") && html.contains("lang=\"pt-BR\"") && html.contains("<style>b{color:red}</style>"));
    assert!(html.contains("<script>const L=1;</script>") && html.contains("const F=[{\"id\":\"a\"}];") && html.contains("{{desconhecido}}"));
    assert!(!html.contains("{{conteudo}}") && !html.contains("{{kit}}"), "kit fica vazio: o kit vai dentro da biblioteca");
}

#[test]
fn o_que_entra_num_campo_nunca_e_lido_como_campo() {
    let mut c = conteudo();
    c["titulo"] = json!("{{figuras}} {{css}}");
    c["resumo"] = json!(["{{biblioteca}}"]);
    let html = montar(MODELO, "CSS", "LIB", &c, &json!([]));
    assert!(html.contains("<title>{{figuras}} {{css}}</title>"), "o título aparece como foi escrito");
    assert!(html.contains("[\"{{biblioteca}}\"]"), "o conteúdo não puxa a biblioteca");
    assert_eq!(html.matches("LIB").count(), 1);
}

#[test]
fn json_e_titulo_nao_escapam_do_lugar_deles() {
    let c = json!({"titulo": "</script><img onerror=1>", "texto": "x </script> y <!-- z", "idioma": "pt-BR\"><b>"});
    let html = montar(MODELO, "", "", &c, &json!([{"id": "f", "codigo": "a</script>b"}]));
    assert_eq!(html.matches("</script>").count(), 3, "só os fechamentos do modelo e o do bloco da figura: {html}");
    assert!(html.contains("<title>&lt;/script&gt;&lt;img onerror=1&gt;</title>") && html.contains("lang=\"en\""), "idioma torto cai em inglês");
    assert!(!html.contains("<!-- z"), "comentário não abre dentro do script");
    assert_eq!(esc_script("a</SCRIPT>b </script x <!-- c </style>"), "a<\\/SCRIPT>b <\\/script x <\\!-- c </style>", "a mesma troca do montar.mjs: mantém as letras, só fecha a porta");
    assert_eq!(escapar_html("<a href=\"x\">&'</a>"), "&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;");
}

#[test]
fn cada_figura_ganha_o_proprio_script_e_o_json_fica_sem_codigo() {
    let figuras = json!([{"id": "fichas", "titulo": "Fichas", "codigo": "function (L) { return 1; }"}, {"id": "sem-codigo", "titulo": "X"}, {"id": "b\"</script>", "codigo": "() => 2"}]);
    let html = montar(MODELO, "", "", &conteudo(), &figuras);
    assert!(html.contains("const F=[{\"id\":\"fichas\",\"titulo\":\"Fichas\"},{\"id\":\"sem-codigo\",\"titulo\":\"X\"},{\"id\":\"b\\\"<\\/script>\"}];"), "metadados sem codigo: {html}");
    assert!(html.contains("<script>(window.AP_CODIGOS=window.AP_CODIGOS||{})[\"fichas\"]=(function (L) { return 1; });</script>\n<script>(window.AP_CODIGOS=window.AP_CODIGOS||{})[\"b\\\"<\\/script>\"]=(() => 2);</script>"), "um bloco por figura com codigo, na ordem: {html}");
    assert!(!html.contains("sem-codigo\"]=("), "figura sem codigo não ganha bloco");
}

#[test]
fn a_pagina_leva_um_csp_sem_rede_para_o_arquivo_exportado() {
    let html = montar(MODELO, "", "", &conteudo(), &json!([]));
    assert!(html.starts_with("<!doctype html><html lang=\"pt-BR\"><head><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:\">"), "{html}");
    let ja_tem = "<head><meta http-equiv=\"Content-Security-Policy\" content=\"x\"></head>{{conteudo}}";
    assert_eq!(montar(ja_tem, "", "", &conteudo(), &json!([])).matches("Content-Security-Policy").count(), 1, "não duplica o do modelo");
    assert!(!montar("sem cabeça {{titulo}}", "", "", &conteudo(), &json!([])).contains("<meta"));
}

#[test]
fn a_biblioteca_embutida_monta_uma_pagina_de_verdade() {
    let (Some(modelo), Some(css), Some(lib)) = (crate::apresentacao::arquivo("modelo.html"), crate::apresentacao::arquivo("dist/estilo.css"), crate::apresentacao::arquivo("dist/biblioteca.js")) else {
        eprintln!("apresentacao-ia ainda não está nesta compilação: teste da biblioteca real pulado");
        return;
    };
    let html = montar(modelo, css, lib, &conteudo(), &json!([]));
    assert!(html.contains("Atendimento de usuários"), "o título do conteúdo entra na página");
    for campo in ["{{titulo}}", "{{idioma}}", "{{css}}", "{{kit}}", "{{biblioteca}}", "{{conteudo}}", "{{figuras}}", "{{codigos}}"] {
        assert!(!html.contains(campo), "campo sem trocar: {campo}");
    }
    assert!(html.len() < 2 * 1024 * 1024, "a página montada cabe nos limites do contrato");
}

/// O mesmo caso que `apresentacao-ia/montar.test.mjs` confere no Node: as duas pontas têm de dar a MESMA página, byte a byte (as chaves do caso estão em ordem alfabética,
/// que é a ordem do `serde_json`; o Node mantém a ordem em que vêm).
#[test]
fn o_caso_fixo_da_biblioteca_da_a_a_mesma_pagina_que_o_node() {
    let caso: Value = serde_json::from_str(include_str!("../../../apresentacao-ia/teste-montagem-fixo.json")).unwrap();
    let texto = |k: &str| caso[k].as_str().unwrap().to_string();
    let pagina = montar(&texto("modelo"), &texto("css"), &texto("biblioteca"), &caso["conteudo"], &caso["figuras"]);
    assert_eq!(pagina.replacen(META_CSP, "", 1), texto("esperado"), "igual, fora o <meta> de CSP que só o servidor acrescenta para o arquivo exportado");
}
