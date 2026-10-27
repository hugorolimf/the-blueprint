//! A página de uma apresentação: o modelo de `apresentacao-ia/modelo.html` com os campos `{{titulo}} {{idioma}} {{css}} {{kit}} {{biblioteca}} {{conteudo}} {{figuras}}`
//! trocados por texto puro (contrato em docs/interno/apresentacao-ia.md). A troca é de uma passada só: o que entra num campo nunca é lido como campo de novo.
use super::armazenar;
use serde_json::Value;
use std::path::Path;

/// Troca `</alvo` (sem diferenciar maiúsculas) por `<\/alvo`: o texto deixa de fechar o `<script>` ou o `<style>` e continua válido em JS e em CSS.
fn sem_fechamento(texto: &str, alvo: &str) -> String {
    let (bytes, n) = (texto.as_bytes(), alvo.len());
    let mut saida = String::with_capacity(texto.len() + 16);
    let mut inicio = 0;
    let mut i = 0;
    while i + 2 + n <= bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1] == b'/' && bytes[i + 2..i + 2 + n].eq_ignore_ascii_case(alvo.as_bytes()) {
            saida.push_str(&texto[inicio..i]);
            saida.push_str("<\\/");
            inicio = i + 2;
            i += 2;
        } else {
            i += 1;
        }
    }
    saida.push_str(&texto[inicio..]);
    saida
}

/// Texto dentro de um `<script>`: nem `</script` fecha o bloco nem `<!--` abre comentário (a mesma troca do `montar.mjs`, para o Rust e o Node darem a mesma página).
pub fn esc_script(texto: &str) -> String {
    sem_fechamento(texto, "script").replace("<!--", "<\\!--")
}

/// Os metadados das figuras como JSON puro: tudo menos o `codigo`, que roda em bloco próprio (`codigos_html`).
fn figuras_json(figuras: &Value) -> String {
    let lista: Vec<Value> = figuras.as_array().map(|l| l.iter().map(|f| {
        let mut f = f.clone();
        if let Some(o) = f.as_object_mut() { o.remove("codigo"); }
        f
    }).collect()).unwrap_or_default();
    Value::Array(lista).to_string()
}

/// Um `<script>` por figura, cada um com o `codigo` CRU: o sandbox não tem `eval`, então o código precisa ser código da página. Um erro de sintaxe derruba só o bloco dele
/// (a figura vira um cartão de erro) e não a página inteira. O formato é fixo e igual ao do `montar.mjs`.
fn codigos_html(figuras: &Value) -> String {
    figuras.as_array().into_iter().flatten().filter_map(|f| {
        let (id, codigo) = (f.get("id")?, f.get("codigo")?.as_str()?);
        Some(format!("<script>(window.AP_CODIGOS=window.AP_CODIGOS||{{}})[{}]=({});</script>", esc_script(&id.to_string()), esc_script(codigo)))
    }).collect::<Vec<_>>().join("
")
}

pub fn escapar_html(t: &str) -> String {
    t.chars().fold(String::with_capacity(t.len()), |mut s, c| {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            '\'' => s.push_str("&#39;"),
            _ => s.push(c),
        }
        s
    })
}

/// Uma passada: cada `{{nome}}` conhecido vira o valor dele; os desconhecidos ficam como estão.
fn trocar(modelo: &str, valores: &[(&str, &str)]) -> String {
    let mut saida = String::with_capacity(modelo.len() + valores.iter().map(|(_, v)| v.len()).sum::<usize>());
    let mut resto = modelo;
    while let Some(i) = resto.find("{{") {
        saida.push_str(&resto[..i]);
        let depois = &resto[i + 2..];
        match depois.find("}}").and_then(|f| valores.iter().find(|(n, _)| *n == &depois[..f]).map(|(_, v)| (f, *v))) {
            Some((f, valor)) => {
                saida.push_str(valor);
                resto = &depois[f + 2..];
            }
            None => {
                saida.push_str("{{");
                resto = depois;
            }
        }
    }
    saida.push_str(resto);
    saida
}

/// O idioma do `<html lang>`: só letras e hífen, senão inglês.
fn idioma_de(conteudo: &Value) -> &str {
    conteudo.get("idioma").and_then(Value::as_str).filter(|i| (2..=10).contains(&i.len()) && i.bytes().all(|c| c.is_ascii_alphabetic() || c == b'-')).unwrap_or("en")
}

/// Sem rede nem no arquivo exportado (onde o cabeçalho HTTP já não vale): o mesmo CSP, menos o `sandbox`, que só existe como cabeçalho.
pub const META_CSP: &str = "<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:\">";

/// `kit` fica vazio: o kit do anatomy já vem dentro de `dist/biblioteca.js`.
pub fn montar(modelo: &str, css: &str, biblioteca: &str, conteudo: &Value, figuras: &Value) -> String {
    let titulo = escapar_html(conteudo.get("titulo").and_then(Value::as_str).unwrap_or(""));
    let (css, biblioteca) = (sem_fechamento(css, "style"), esc_script(biblioteca));
    let (conteudo_js, figuras_js, codigos) = (esc_script(&conteudo.to_string()), esc_script(&figuras_json(figuras)), codigos_html(figuras));
    let pagina = trocar(modelo, &[("titulo", &titulo), ("idioma", idioma_de(conteudo)), ("css", &css), ("kit", ""), ("biblioteca", &biblioteca), ("conteudo", &conteudo_js), ("figuras", &figuras_js), ("codigos", &codigos)]);
    match pagina.find("<head>") {
        Some(i) if !pagina.contains("Content-Security-Policy") => format!("{}{META_CSP}{}", &pagina[..i + 6], &pagina[i + 6..]),
        _ => pagina,
    }
}

pub enum Falha {
    NaoExiste,
    /// a biblioteca não veio nesta compilação
    SemBiblioteca,
}

/// A página pronta de uma apresentação guardada.
pub fn pagina(pasta: &Path, id: &str) -> Result<(String, Value), Falha> {
    let registro = armazenar::ler(pasta, id).ok_or(Falha::NaoExiste)?;
    let arquivo = |n: &str| super::arquivo(n).ok_or(Falha::SemBiblioteca);
    let html = montar(arquivo("modelo.html")?, arquivo("dist/estilo.css")?, arquivo("dist/biblioteca.js")?, &registro["conteudo"], &registro["figuras"]);
    Ok((html, registro))
}
