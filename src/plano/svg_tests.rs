//! Testes de `plano::svg` e `plano::estilo`. Os de `ler_pasta` ficam em `pacote.rs`, por causa do filtro `plano::pacote`.
use super::estilo::{compilar_estilo, resolver, TEMA};
use super::svg::{imagem_tipo, medidas_imagem, medidas_svg, svg_seguro};
use std::collections::BTreeMap;

fn recusa(svg: &str) {
    assert!(svg_seguro(svg).is_err(), "devia recusar: {svg}");
}
fn aceita(svg: &str) {
    assert_eq!(svg_seguro(svg), Ok(()), "devia aceitar: {svg}");
}
fn vars(pares: &[(&str, &str)]) -> BTreeMap<String, String> {
    pares.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
}

#[test]
fn recusa_script() {
    recusa("<svg><script>alert(1)</script></svg>");
    recusa("<svg><SCRIPT>alert(1)</SCRIPT></svg>");
    recusa("<svg><x:script xmlns:x=\"http://www.w3.org/2000/svg\">a</x:script></svg>");
    recusa("<svg><script/></svg>");
    recusa("<svg><!-- <script>a</script> --></svg>");
    recusa("<svg><style><script>a</script></style></svg>");
    recusa("<script>a</script>");
    recusa("");
    recusa("sem etiqueta nenhuma");
}

#[test]
fn recusa_foreign_object() {
    recusa("<svg><foreignObject><div/></foreignObject></svg>");
    recusa("<svg><FOREIGNOBJECT/></svg>");
    recusa("<svg><image href=\"#a\"/></svg>");
    recusa("<svg><feImage href=\"#a\"/></svg>");
    recusa("<svg><animate attributeName=\"href\" to=\"javascript:a\"/></svg>");
    recusa("<svg><set attributeName=\"href\" to=\"x\"/></svg>");
}

#[test]
fn recusa_evento() {
    recusa("<svg onload=\"a()\"/>");
    recusa("<svg ONLOAD='a()'/>");
    recusa("<svg onload=a()>");
    recusa("<svg/onload=a()>");
    recusa("<svg><rect\n\tonclick = \"a()\"/></svg>");
    recusa("<svg><rect x:onclick=\"a()\"/></svg>");
}

#[test]
fn recusa_href_aspas_simples() {
    recusa("<svg><use href='http://x/y#a'/></svg>");
    recusa("<svg><use xlink:href='http://x/y#a'/></svg>");
    recusa("<svg><use XLINK:HREF=\"data:image/svg+xml,a\"/></svg>");
    recusa("<svg><a href='javascript:a()'><text>x</text></a></svg>");
    recusa("<svg><a href='&#106;avascript:a()'/></svg>");
    recusa("<svg><rect src='a.png'/></svg>");
    recusa("<svg><rect data-x='java\tscript:a'/></svg>");
    recusa("<svg><rect x='1' y='2/></svg>");
}

#[test]
fn recusa_url_externa_em_fill() {
    recusa("<svg><rect fill=\"url(http://x/y)\"/></svg>");
    recusa("<svg><rect fill='URL( \"http://x/y\" )'/></svg>");
    recusa("<svg><rect fill=\"&#117;rl(http://x/y)\"/></svg>");
    recusa("<svg><rect fill=\"u\\72l(http://x/y)\"/></svg>");
    recusa("<svg><rect style=\"fill:url(http://x/y)\"/></svg>");
    recusa("<svg><rect style=\"fill:url(/**/http://x/y)\"/></svg>");
    recusa("<svg><rect fill=\"url(#a) url(http://x/y)\"/></svg>");
    aceita("<svg><rect fill=\"url(#a)\" stroke=\"url('#b')\" style=\"fill:URL( #c )\"/></svg>");
}

#[test]
fn recusa_import_no_style() {
    recusa("<svg><style>@import 'http://x/y.css';</style></svg>");
    recusa("<svg><style>@IMPORT url(a.css);</style></svg>");
    recusa("<svg><style><![CDATA[@import 'a.css';]]></style></svg>");
    recusa("<svg><style>&#64;import 'a.css';</style></svg>");
    recusa("<svg><style>@/**/import 'a.css';</style></svg>");
    recusa("<svg><style>.a{fill:url(http://x/y)}</style></svg>");
    recusa("<svg><style>.a{fill:url(http://x/y)}</STYLE></svg>");
    recusa("<svg><style>.a{}");
    aceita("<svg><style>.a{fill:url(#b);stroke:#fff}</style><style/><rect class=\"a\"/></svg>");
    aceita("<svg><style><![CDATA[.a>.b{fill:red}]]></style></svg>");
}

#[test]
fn recusa_doctype() {
    recusa("<!DOCTYPE svg><svg/>");
    recusa("<!doctype svg><svg/>");
    recusa("<?xml version=\"1.0\"?><!ENTITY a \"b\"><svg/>");
    recusa("<?xml-stylesheet href=\"a.css\"?><svg/>");
    recusa("<?php a ?><svg/>");
    aceita("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- gerado --><svg/>");
}

#[test]
fn aceita_href_interno() {
    aceita("<svg xmlns=\"http://www.w3.org/2000/svg\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><defs><linearGradient id=\"a\"><stop offset=\"0\"/></linearGradient></defs><use href=\"#a\"/><use xlink:href='#a'/><a href=\"#b\"><path d=\"M0 0\"/></a></svg>");
}

#[test]
fn aceita_on_dentro_de_text() {
    aceita("<svg><text x=\"1\">Plano on = sim, onload=\"a()\" &lt;script&gt; url(http://x) javascript:a</text></svg>");
    aceita("<svg><title>a on = b</title><desc>c onclick=d</desc></svg>");
    aceita("<svg><text>Recepção · 48 m² 日本語</text></svg>");
}

#[test]
fn valor_hostil_pego_depois_de_resolver() {
    for hostil in ["url(http://x/y)", "javascript:a()", "x\" onload=\"a()"] {
        let svg = "<svg><rect fill=\"var(--cor)\"/></svg>";
        aceita(svg);
        recusa(&resolver(svg, &vars(&[("--cor", hostil)])));
    }
    // Vindo do CSS do plano, pelo mesmo caminho do servidor.
    let (_, v) = compilar_estilo(".f{--cor: url(http://x/y)}", &BTreeMap::new());
    recusa(&resolver("<svg><rect fill=\"var(--cor)\"/></svg>", &v));
}

#[test]
fn resolver_var_e_color_mix() {
    let v = vars(&[("--a", "#ff0000"), ("--b", "var(--a)")]);
    assert_eq!(resolver("var(--a)", &v), "#ff0000");
    assert_eq!(resolver("var(--b)", &v), "#ff0000");
    assert_eq!(resolver("var(--x, #123)", &v), "#123");
    assert_eq!(resolver("var(--x)", &v), "currentColor");
    assert_eq!(resolver("var(--a, #123) e var(--x,  red)!", &v), "#ff0000 e red!");
    assert_eq!(resolver("color-mix(in srgb, #ffffff 50%, #000000)", &v), "#808080");
    assert_eq!(resolver("COLOR-MIX(IN SRGB, #FFF 25%, #000)", &v), "#404040");
    assert_eq!(resolver("color-mix(in srgb, #123456 40%, transparent)", &v), "#123456");
    assert_eq!(resolver("color-mix(in srgb, currentColor 40%, transparent)", &v), "currentColor");
    assert_eq!(resolver("color-mix(in srgb, var(--a) 50%, #000000)", &v), "#800000");
    assert_eq!(resolver("Recepção · color-mix(in srgb, #fff 100%, #000) m²", &v), "Recepção · #ffffff m²");
    // O que o protótipo não entende fica como veio.
    assert_eq!(resolver("color-mix(in oklab, #fff 50%, #000)", &v), "color-mix(in oklab, #fff 50%, #000)");
    assert_eq!(resolver("var(sem-tracos) var(--a", &v), "var(sem-tracos) var(--a");
    // Mistura depois de 100% não sai do intervalo.
    assert_eq!(resolver("color-mix(in srgb, #ffffff 900%, #000000)", &v), "#ffffff");
}

#[test]
fn compilar_estilo_como_o_prototipo() {
    let css = "/* nota */\n@media print { .x { --y: 1 } }\n.figura { --mesa: #abcdef; --cor: #111; --cartao: #222 }\n.a { fill: var(--mesa); stroke: var(--cor) }\n";
    let tema = vars(&[("--cor", "verde"), ("--outra", "nao-existe")]);
    let (saida, v) = compilar_estilo(css, &tema);
    assert_eq!(v["--mesa"], "#abcdef");
    assert_eq!(v["--cor"], "#8be3b5", "o tema do plano vence");
    assert_eq!(v["--cartao"], "#0c264b", "cor do app não é trocada pelo CSS");
    assert!(!v.contains_key("--outra"));
    assert!(!v.contains_key("--y"));
    assert_eq!(saida.trim(), ".a { fill: #abcdef; stroke: #8be3b5 }");
    assert_eq!(TEMA.len(), 21);
}

#[test]
fn css_malformado_nao_laca() {
    let vazio = BTreeMap::new();
    assert_eq!(compilar_estilo("@import 'a.css'", &vazio).0, "");
    assert_eq!(compilar_estilo("@import 'a.css'; .a{fill:red}", &vazio).0.trim(), ".a{fill:red}");
    assert_eq!(compilar_estilo(".a{fill:red} @media x { .b{", &vazio).0.trim(), ".a{fill:red}");
    for feio in ["@", "@@@@", "{", "}", "{{{", "}}}", "/*", "a{b", "a{--", "a{--:", "--", "var(", "var(--", "@;@;", "é{ç}@ã", "color-mix(", "color-mix(in srgb,", "color-mix(in srgb, é 1%, ç)", ".é{--ç:ã;--"] {
        compilar_estilo(feio, &vazio);
        resolver(feio, &vazio);
        let _ = svg_seguro(feio);
    }
    // Variável que se cita: as 4 passadas acabam, e variável que multiplica para no teto.
    assert!(resolver("var(--a)", &vars(&[("--a", "var(--a)")])).contains("var("));
    let grande = "var(--a)".repeat(20000);
    let v = vars(&[("--a", &grande)]);
    assert!(resolver(&grande, &v).len() < 512 << 20);
    // No tamanho máximo, tudo termina depressa: nenhuma destas repetições relê o resto do texto a cada ocorrência.
    let n = 1 << 18;
    for peca in ["&", "var(--a,", "color-mix(", "--a:", "@{", "{"] {
        let feio = peca.repeat(n / peca.len());
        compilar_estilo(&feio, &vazio);
        resolver(&feio, &vazio);
        let _ = svg_seguro(&format!("<svg a=\"{feio}\"/>"));
        let _ = svg_seguro(&format!("<svg><style>{feio}</style></svg>"));
    }
    aceita(&format!("<svg>{}</svg>", "<style></style>".repeat(n / 15)));
}

fn png(l: u32, a: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    b.extend(l.to_be_bytes());
    b.extend(a.to_be_bytes());
    b
}

#[test]
fn medidas_dos_quatro_tipos() {
    assert_eq!(medidas_svg("<?xml version=\"1.0\"?><!-- a --><svg xmlns=\"x\" viewBox=\"0 0 900 460\" width=\"1\" height=\"1\"/>"), Some((900, 460)));
    assert_eq!(medidas_svg("<svg viewBox='0,0,10.4,20.6'/>"), Some((10, 21)));
    assert_eq!(medidas_svg("<svg width=\"300px\" height=\"150\"/>"), Some((300, 150)));
    assert_eq!(medidas_svg("<svg width=\"100%\" height=\"150\"/>"), None);
    assert_eq!(medidas_svg("<svg viewBox=\"0 0 0 10\"/>"), None);
    assert_eq!(medidas_svg("<rect width=\"1\" height=\"1\"/>"), None);

    let p = png(640, 480);
    assert_eq!(imagem_tipo(&p), Some("png"));
    assert_eq!(medidas_imagem(&p), Some((640, 480)));
    assert_eq!(medidas_imagem(&png(0, 480)), None);
    assert_eq!(medidas_imagem(&p[..20]), None);

    // SOI, APP0 de 4 bytes, DHT (0xC4 tem forma de SOF e não é), SOF0: altura 200, largura 300.
    let jpg = [0xff, 0xd8, 0xff, 0xe0, 0, 4, 1, 2, 0xff, 0xc4, 0, 2, 0xff, 0xff, 0xc0, 0, 11, 8, 0, 200, 1, 44, 3];
    assert_eq!(imagem_tipo(&jpg), Some("jpg"));
    assert_eq!(medidas_imagem(&jpg), Some((300, 200)));
    assert_eq!(medidas_imagem(&[0xff, 0xd8, 0xff, 0xda, 0, 2]), None);
    assert_eq!(medidas_imagem(&[0xff, 0xd8, 0xff, 0xe0, 0, 0]), None);

    let webp = |tipo: &[u8], resto: &[u8]| -> Vec<u8> { [b"RIFF\0\0\0\0WEBP".as_slice(), tipo, &[0, 0, 0, 0], resto].concat() };
    let vp8 = webp(b"VP8 ", &[0, 0, 0, 0x9d, 0x01, 0x2a, 0x40, 0x01, 0xf0, 0x00]);
    assert_eq!(imagem_tipo(&vp8), Some("webp"));
    assert_eq!(medidas_imagem(&vp8), Some((320, 240)));
    // 14 bits largura-1 = 319, 14 bits altura-1 = 239.
    let bits: u32 = 319 | 239 << 14;
    let vp8l = webp(b"VP8L", &[&[0x2f][..], &bits.to_le_bytes()].concat());
    assert_eq!(medidas_imagem(&vp8l), Some((320, 240)));
    let vp8x = webp(b"VP8X", &[0, 0, 0, 0, 0x3f, 0x01, 0, 0xef, 0, 0]);
    assert_eq!(medidas_imagem(&vp8x), Some((320, 240)));

    assert_eq!(imagem_tipo(b"<svg/>"), None);
    assert_eq!(imagem_tipo(b"RIFF\0\0\0\0WAVE"), None);
    assert_eq!(medidas_imagem(b"nada"), None);
}
