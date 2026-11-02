//! O que pode entrar como figura de um plano: o SVG passa por um tokenizador próprio com lista de elementos permitidos (sem regex, sem dependência nova) e a imagem é reconhecida pelos primeiros bytes. As mensagens vão ao agente, em inglês.
pub const LIMITE_SVG: usize = 2 << 20;
pub const LIMITE_IMAGEM: usize = 4 << 20;
pub const MAX_FIGURAS: usize = 500;

/// Em minúsculas e sem prefixo de namespace. Fica de fora tudo o que carrega ou executa algo: script, foreignObject, image, feImage, animate e set (que reescrevem href).
const PERMITIDOS: &[&str] = &[
    "svg", "g", "defs", "title", "desc", "metadata", "symbol", "use", "a", "path", "rect", "circle", "ellipse", "line", "polyline", "polygon", "text", "tspan", "textpath", "marker", "clippath", "mask", "pattern", "lineargradient", "radialgradient", "stop", "style", "filter", "feblend", "fecolormatrix", "fecomponenttransfer", "fecomposite",
    "feconvolvematrix", "fediffuselighting", "fedisplacementmap", "fedistantlight", "fedropshadow", "feflood", "fefunca", "fefuncb", "fefuncg", "fefuncr", "fegaussianblur", "femerge", "femergenode", "femorphology", "feoffset", "fepointlight", "fespecularlighting", "fespotlight", "feturbulence",
];

struct Etiqueta<'a> { nome: &'a str, atributos: Vec<(&'a str, &'a str)>, vazia: bool, fim: usize }

/// Lê `<nome a="b" ...>` a partir do `<`. Não entende aninhamento: quem chama anda de etiqueta em etiqueta.
fn ler_etiqueta(r: &str) -> Result<Etiqueta<'_>, String> {
    let separa = |c: char| c.is_whitespace() || c == '/' || c == '>';
    let tam = r[1..].find(separa).ok_or("SVG: tag without an end")? + 1;
    let nome = &r[1..tam];
    if !nome.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        return Err("SVG: stray \"<\" (write &lt; in text)".to_string());
    }
    let (mut i, mut atributos) = (tam, Vec::new());
    loop {
        i += r[i..].len() - r[i..].trim_start().len();
        let resto = &r[i..];
        if resto.is_empty() {
            return Err(format!("SVG: <{nome}> without an end"));
        }
        if resto.starts_with("/>") {
            return Ok(Etiqueta { nome, atributos, vazia: true, fim: i + 2 });
        }
        if resto.starts_with('>') {
            return Ok(Etiqueta { nome, atributos, vazia: false, fim: i + 1 });
        }
        if resto.starts_with('/') {
            i += 1;
            continue;
        }
        let tam = resto.find(|c: char| separa(c) || c == '=').unwrap_or(resto.len());
        if tam == 0 {
            return Err(format!("SVG: <{nome}> has a malformed attribute"));
        }
        let atributo = &resto[..tam];
        i += tam;
        i += r[i..].len() - r[i..].trim_start().len();
        if !r[i..].starts_with('=') {
            atributos.push((atributo, ""));
            continue;
        }
        i += 1;
        i += r[i..].len() - r[i..].trim_start().len();
        let resto = &r[i..];
        let valor = match resto.chars().next() {
            Some(q @ ('"' | '\'')) => {
                let fim = resto[1..].find(q).ok_or_else(|| format!("SVG: attribute {atributo} without closing quote"))?;
                i += fim + 2;
                &resto[1..fim + 1]
            }
            _ => {
                let fim = resto.find(|c: char| c.is_whitespace() || c == '>').unwrap_or(resto.len());
                i += fim;
                &resto[..fim]
            }
        };
        atributos.push((atributo, valor));
    }
}

/// Uma passada, como no XML: `&#x26;#106;` vira o texto `&#106;`, não `j`. Entidade com nome só as cinco do XML, porque `<!DOCTYPE` é recusado.
fn decodificar(s: &str) -> String {
    let (mut saida, mut resto) = (String::with_capacity(s.len()), s);
    while let Some(p) = resto.find('&') {
        saida.push_str(&resto[..p]);
        resto = &resto[p..];
        // Janela curta: `&&&&...` sem `;` não pode reler o texto inteiro a cada `&`.
        let trocado = resto.bytes().take(13).position(|b| b == b';').and_then(|f| {
            let c = match &resto[1..f] {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                e => e.strip_prefix('#').and_then(|n| match n.strip_prefix(['x', 'X']) {
                    Some(h) => u32::from_str_radix(h, 16).ok(),
                    None => n.parse().ok(),
                }).and_then(char::from_u32),
            };
            c.map(|c| (c, f + 1))
        });
        let (c, n) = trocado.unwrap_or(('&', 1));
        saida.push(c);
        resto = &resto[n..];
    }
    saida.push_str(resto);
    saida
}

/// Vale para qualquer valor, depois de decodificar: `url(` só para dentro, nada de `javascript:` e nada de escape CSS (`u\72l(`).
fn valor_seguro(valor: &str) -> Result<(), String> {
    let minusculo = valor.to_ascii_lowercase();
    if valor.contains('\\') {
        return Err(format!("SVG: value with a backslash is not allowed: {}", resumo(valor)));
    }
    let sem_espaco: String = minusculo.chars().filter(|c| !c.is_whitespace() && !c.is_control()).collect();
    if let Some(p) = ["javascript:", "image-set("].iter().find(|p| sem_espaco.contains(**p)) {
        return Err(format!("SVG: {p} is not allowed"));
    }
    let mut resto = sem_espaco.as_str();
    while let Some(p) = resto.find("url(") {
        resto = &resto[p + 4..];
        if !resto.trim_start_matches(['"', '\'']).starts_with('#') {
            return Err(format!("SVG: url() must point inside the figure (url(#id)): {}", resumo(valor)));
        }
    }
    Ok(())
}

fn resumo(s: &str) -> String { s.chars().take(60).collect() }

/// CSS de `<style>`, do atributo `style` e do arquivo de estilo do plano. Comentário sai antes, para `url(/**/http:...)` não passar.
pub fn css_seguro(css: &str) -> Result<(), String> {
    let limpo = super::estilo::sem_comentarios(css);
    if limpo.to_ascii_lowercase().contains("@import") {
        return Err("SVG: @import is not allowed in a style".to_string());
    }
    valor_seguro(&limpo)
}

fn conferir_atributo(nome: &str, valor: &str) -> Result<(), String> {
    let local = local_minusculo(nome);
    let local = local.as_str();
    if local.starts_with("on") {
        return Err(format!("SVG: event attribute {nome} is not allowed"));
    }
    let valor = decodificar(valor);
    if matches!(local, "href" | "src") && !valor.starts_with('#') {
        return Err(format!("SVG: {nome} must point inside the figure (#id): {}", resumo(&valor)));
    }
    if local == "style" {
        return css_seguro(&valor);
    }
    valor_seguro(&valor)
}

/// O miolo de `<style>`: texto e CDATA. Qualquer outra etiqueta lá dentro é recusada, porque seria um elemento de verdade para o XML.
fn conferir_style(miolo: &str) -> Result<(), String> {
    let (mut css, mut resto) = (String::with_capacity(miolo.len()), miolo);
    while let Some(p) = resto.find('<') {
        css.push_str(&decodificar(&resto[..p]));
        resto = &resto[p..];
        let dentro = resto.strip_prefix("<![CDATA[").ok_or("SVG: tag inside <style>")?;
        let fim = dentro.find("]]>").ok_or("SVG: CDATA without an end")?;
        css.push_str(&dentro[..fim]);
        resto = &dentro[fim + 3..];
    }
    css.push_str(&decodificar(resto));
    css_seguro(&css)
}

fn local_minusculo(nome: &str) -> String { nome.rsplit(':').next().unwrap_or(nome).to_ascii_lowercase() }

/// Roda duas vezes: antes e depois de `estilo::resolver`, porque valor vindo do CSS do plano só entra depois. O texto de `<text>` não é analisado.
pub fn svg_seguro(texto: &str) -> Result<(), String> {
    let (mut resto, mut viu_raiz) = (texto, false);
    while let Some(p) = resto.find('<') {
        let r = &resto[p..];
        let minusculo: String = r.chars().take(10).collect::<String>().to_ascii_lowercase();
        let ate = |marca: &str, de: usize, o_que: &str| r[de..].find(marca).map(|f| de + f).ok_or(format!("SVG: {o_que} without an end"));
        if r.starts_with("<!--") || r.starts_with("<![CDATA[") {
            let (de, marca) = if r.starts_with("<!--") { (4, "-->") } else { (9, "]]>") };
            let fim = ate(marca, de, "comment or CDATA")?;
            if r[de..fim].contains('<') {
                return Err("SVG: \"<\" inside a comment or CDATA is not allowed".to_string());
            }
            resto = &r[fim + 3..];
        } else if minusculo.starts_with("<!") {
            return Err("SVG: <!DOCTYPE and <!ENTITY are not allowed".to_string());
        } else if minusculo.starts_with("<?") {
            if !(minusculo.starts_with("<?xml") && minusculo[5..].starts_with(|c: char| c.is_whitespace() || c == '?')) {
                return Err("SVG: processing instruction other than <?xml is not allowed".to_string());
            }
            resto = &r[ate("?>", 2, "<?xml")? + 2..];
        } else if r.starts_with("</") {
            resto = &r[ate(">", 2, "closing tag")? + 1..];
        } else {
            let e = ler_etiqueta(r)?;
            let local = local_minusculo(e.nome);
            if !viu_raiz && local != "svg" {
                return Err(format!("SVG: the root must be <svg>, not <{}>", e.nome));
            }
            viu_raiz = true;
            if !PERMITIDOS.contains(&local.as_str()) {
                return Err(format!("SVG: <{}> is not allowed", e.nome));
            }
            for (nome, valor) in &e.atributos {
                conferir_atributo(nome, valor)?;
            }
            resto = &r[e.fim..];
            if local == "style" && !e.vazia {
                let fecha = format!("</{}", e.nome);
                let fim = resto.match_indices("</").map(|(i, _)| i).find(|i| resto.as_bytes().get(*i..i + fecha.len()).is_some_and(|b| b.eq_ignore_ascii_case(fecha.as_bytes()))).ok_or("SVG: <style> without an end")?;
                conferir_style(&resto[..fim])?;
                resto = &resto[fim..];
            }
        }
    }
    if viu_raiz { Ok(()) } else { Err("SVG: no <svg> element".to_string()) }
}

/// Magic bytes: png, jpg, webp.
pub fn imagem_tipo(bytes: &[u8]) -> Option<&'static str> {
    match bytes {
        [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, ..] => Some("png"),
        [0xff, 0xd8, 0xff, ..] => Some("jpg"),
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => Some("webp"),
        _ => None,
    }
}

fn numero_svg(s: &str) -> Option<f64> {
    s.trim().trim_end_matches("px").parse().ok().filter(|n: &f64| n.is_finite() && *n > 0.0)
}

/// `viewBox`, senão `width`/`height`, sempre da raiz. Medida em porcentagem ou em outra unidade não conta.
pub fn medidas_svg(texto: &str) -> Option<(u32, u32)> {
    let mut resto = texto;
    let raiz = loop {
        let r = &resto[resto.find('<')?..];
        if r.starts_with("<?") {
            resto = &r[r.find("?>")? + 2..];
        } else if r.starts_with("<!--") {
            resto = &r[r.find("-->")? + 3..];
        } else {
            break ler_etiqueta(r).ok()?;
        }
    };
    if local_minusculo(raiz.nome) != "svg" {
        return None;
    }
    let attr = |n: &str| raiz.atributos.iter().find(|(a, _)| a.eq_ignore_ascii_case(n)).map(|(_, v)| *v);
    let inteiro = |n: f64| (n.round() as u32).max(1);
    let da_caixa = attr("viewBox").and_then(|v| {
        let p: Vec<&str> = v.split(|c: char| c.is_whitespace() || c == ',').filter(|s| !s.is_empty()).collect();
        (p.len() == 4).then(|| Some((numero_svg(p[2])?, numero_svg(p[3])?))).flatten()
    });
    let (l, a) = da_caixa.or_else(|| Some((numero_svg(attr("width")?)?, numero_svg(attr("height")?)?)))?;
    Some((inteiro(l), inteiro(a)))
}

fn be16(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from(*b.get(i)?) << 8 | u32::from(*b.get(i + 1)?))
}

fn le(b: &[u8], i: usize, n: usize) -> Option<u32> {
    b.get(i..i + n).map(|p| p.iter().rev().fold(0, |v, x| v << 8 | u32::from(*x)))
}

fn medidas_jpg(b: &[u8]) -> Option<(u32, u32)> {
    let mut i = 2;
    loop {
        if *b.get(i)? != 0xff {
            return None;
        }
        while *b.get(i)? == 0xff {
            i += 1;
        }
        let marca = *b.get(i)?;
        i += 1;
        match marca {
            0x01 | 0xd0..=0xd8 => continue,
            0xd9 | 0xda => return None,
            0xc0..=0xcf if !matches!(marca, 0xc4 | 0xc8 | 0xcc) => return Some((be16(b, i + 5)?, be16(b, i + 3)?)),
            _ => i += usize::try_from(be16(b, i)?).ok()?.max(2),
        }
    }
}

fn medidas_webp(b: &[u8]) -> Option<(u32, u32)> {
    match b.get(12..16)? {
        b"VP8 " if b.get(23..26)? == [0x9d, 0x01, 0x2a] => Some((le(b, 26, 2)? & 0x3fff, le(b, 28, 2)? & 0x3fff)),
        b"VP8L" if *b.get(20)? == 0x2f => {
            let v = le(b, 21, 4)?;
            Some(((v & 0x3fff) + 1, ((v >> 14) & 0x3fff) + 1))
        }
        b"VP8X" => Some((le(b, 24, 3)? + 1, le(b, 27, 3)? + 1)),
        _ => None,
    }
}

/// Cabeçalho: IHDR, SOF, VP8/VP8L/VP8X. Medida zero é arquivo quebrado.
pub fn medidas_imagem(bytes: &[u8]) -> Option<(u32, u32)> {
    let m = match imagem_tipo(bytes)? {
        "png" if bytes.get(12..16)? == b"IHDR" => (u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?), u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?)),
        "jpg" => medidas_jpg(bytes)?,
        "webp" => medidas_webp(bytes)?,
        _ => return None,
    };
    (m.0 > 0 && m.1 > 0).then_some(m)
}
