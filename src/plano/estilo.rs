//! O estilo das figuras de um plano, pronto para quem não entende `var()` nem `color-mix()` (o resvg do app). Porte de `design/planejar/estilo.mjs`, sem regex (a raiz não tem o crate): cada laço anda para a frente, então CSS malformado não prende.
use std::collections::BTreeMap;

/// Igual a `TEMA` de `estilo.mjs`.
pub const TEMA: &[(&str, &str)] = &[
    ("papel", "#113360"), ("cartao", "#0c264b"), ("campo", "#081d3c"), ("botao", "#16407a"), ("escolhido", "#1f5299"), ("texto", "#c9dcf3"), ("forte", "#eaf4ff"), ("medio", "#9cc4ee"), ("linha", "#5b93d0"), ("tenue", "#2f66aa"), ("borda", "#3d72b3"),
    ("vermelho", "#ff8f80"), ("ambar", "#ffcf6b"), ("verde", "#8be3b5"), ("azul", "#bfe0ff"), ("destaque", "#d9b8ff"), ("mapa", "#0f2c53"), ("quadra", "#1a467d"), ("topo", "#194680"), ("face", "#123866"), ("sombra", "#0c2a55"),
];

/// Variável que cita variável pode crescer a cada passada. Passou disto, o resto fica como está; quem recebe confere o tamanho e `svg_seguro`.
const TETO_RESOLVIDO: usize = 16 << 20;

fn eh_nome(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

pub(super) fn sem_comentarios(css: &str) -> String {
    let (mut saida, mut resto) = (String::with_capacity(css.len()), css);
    while let Some(p) = resto.find("/*") {
        saida.push_str(&resto[..p]);
        resto = resto[p + 2..].find("*/").map_or("", |f| &resto[p + 2 + f + 2..]);
    }
    saida.push_str(resto);
    saida
}

/// Tira `@... {...}` com chaves aninhadas. `@` que chega num `;` antes de um `{` corta até esse `;`; sem nenhum dos dois, até o fim.
fn sem_arroba(css: &str) -> String {
    let (mut saida, mut resto) = (String::with_capacity(css.len()), css);
    while let Some(a) = resto.find('@') {
        saida.push_str(&resto[..a]);
        let depois = &resto[a..];
        resto = match depois.find(['{', ';']) {
            None => "",
            Some(i) if depois.as_bytes()[i] == b';' => &depois[i + 1..],
            Some(i) => {
                let mut prof = 1;
                let fim = depois[i + 1..].char_indices().find_map(|(j, c)| {
                    prof += match c { '{' => 1, '}' => -1, _ => 0 };
                    (prof == 0).then_some(i + 1 + j + 1)
                });
                fim.map_or("", |f| &depois[f..])
            }
        };
    }
    saida.push_str(resto);
    saida
}

/// As declarações `--nome: valor` de um corpo de regra.
fn declaracoes(corpo: &str) -> Vec<(&str, &str)> {
    let (mut achadas, mut i) = (Vec::new(), 0);
    while let Some(p) = corpo[i..].find("--") {
        let de = i + p;
        let fim_nome = corpo[de..].find(|c: char| !eh_nome(c)).map_or(corpo.len(), |f| de + f);
        i = fim_nome;
        let Some(valor) = corpo[fim_nome..].trim_start().strip_prefix(':') else { continue };
        if fim_nome == de + 2 {
            continue;
        }
        let valor = valor.split(';').next().unwrap_or("");
        i = corpo.len() - corpo[fim_nome..].trim_start().len() + 1 + valor.len();
        achadas.push((&corpo[de..fim_nome], valor.trim()));
    }
    achadas
}

/// Anda pelas regras sem chave aninhada: guarda as variáveis de `.classe { ... }` e joga fora, do CSS, toda regra que define variável. O resto fica como veio.
fn separar_variaveis(css: &str, vars: &mut BTreeMap<String, String>) -> String {
    let (mut saida, mut i) = (String::with_capacity(css.len()), 0);
    while let Some(p) = css[i..].find('{') {
        let abre = i + p;
        let Some(f) = css[abre + 1..].find(['{', '}']) else { break };
        let fecha = abre + 1 + f;
        if css.as_bytes()[fecha] == b'{' {
            saida.push_str(&css[i..=abre]);
            i = abre + 1;
            continue;
        }
        let achadas = declaracoes(&css[abre + 1..fecha]);
        if achadas.is_empty() {
            saida.push_str(&css[i..=fecha]);
        } else {
            let seletor = css[i..abre].trim_start_matches('}').trim();
            let de_classe = seletor.strip_prefix('.').is_some_and(|n| !n.is_empty() && n.chars().all(eh_nome));
            for (nome, valor) in achadas.into_iter().filter(|(_, v)| de_classe && !v.is_empty()) {
                vars.entry(nome.to_string()).or_insert_with(|| valor.to_string());
            }
        }
        i = fecha + 1;
    }
    saida.push_str(&css[i..]);
    saida
}

/// `css` do plano + `tema` do plano (variável -> nome de cor do app; sem nome, a cor original da figura) -> (CSS só com cores prontas, variáveis).
pub fn compilar_estilo(css: &str, tema: &BTreeMap<String, String>) -> (String, BTreeMap<String, String>) {
    let s = sem_arroba(&sem_comentarios(css));
    let mut vars: BTreeMap<String, String> = TEMA.iter().map(|(k, v)| (format!("--{k}"), (*v).to_string())).collect();
    let s = separar_variaveis(&s, &mut vars);
    for (v, nome) in tema {
        if let Some((_, cor)) = TEMA.iter().find(|(k, _)| k == nome) {
            vars.insert(v.clone(), (*cor).to_string());
        }
    }
    (resolver(&s, &vars), vars)
}

/// O `)` numa janela curta: `var(--a,` repetido sem `)` não pode reler o texto inteiro a cada ocorrência.
fn fecha_perto(s: &str) -> Option<usize> {
    s.bytes().take(256).position(|b| b == b')')
}

/// Depois de `var(`: `--nome`, opcionalmente `, alternativa`, e `)`. Devolve o nome, a alternativa e quanto andou.
fn ler_var(r: &str) -> Option<(&str, Option<&str>, usize)> {
    let de = r.strip_prefix("--")?;
    let tam = de.find(|c: char| !eh_nome(c)).unwrap_or(de.len()) + 2;
    let nome = &r[..tam];
    let depois = &r[tam..];
    if depois.starts_with(')') {
        return Some((nome, None, tam + 1));
    }
    let alt = depois.strip_prefix(',')?;
    let fecha = fecha_perto(alt)?;
    let limpo = alt[..fecha].trim_start();
    (!limpo.is_empty()).then_some((nome, Some(limpo), tam + 1 + fecha + 1))
}

fn passada_var(s: &str, vars: &BTreeMap<String, String>) -> String {
    let (mut saida, mut resto) = (String::with_capacity(s.len()), s);
    while let Some(p) = resto.find("var(") {
        saida.push_str(&resto[..p]);
        let depois = &resto[p + 4..];
        if saida.len() > TETO_RESOLVIDO {
            saida.push_str(&resto[p..]);
            return saida;
        }
        match ler_var(depois) {
            Some((nome, alt, andou)) => {
                saida.push_str(vars.get(nome).map(String::as_str).or(alt).unwrap_or("currentColor"));
                resto = &depois[andou..];
            }
            None => {
                saida.push_str("var(");
                resto = depois;
            }
        }
    }
    saida.push_str(resto);
    saida
}

fn hex(c: &str) -> Option<[u32; 3]> {
    let h = c.strip_prefix('#')?;
    if !h.chars().all(|x| x.is_ascii_hexdigit()) {
        return None;
    }
    let d = |x: &str| u32::from_str_radix(x, 16).ok();
    match h.len() {
        3 => {
            let v = |i: usize| d(&h[i..=i]).map(|n| n * 17);
            Some([v(0)?, v(1)?, v(2)?])
        }
        n if n >= 6 => Some([d(&h[..2])?, d(&h[2..4])?, d(&h[4..6])?]),
        _ => None,
    }
}

/// `in srgb, A P%, B)`: `None` se não for a forma que o `estilo.mjs` entende, e o texto fica como veio.
fn ler_mistura(r: &str) -> Option<(String, usize)> {
    let fecha = fecha_perto(r)?;
    let baixo = r[..fecha].to_ascii_lowercase();
    let dentro = baixo.strip_prefix("in srgb,")?;
    let (a, b) = dentro.split_once(',')?;
    let (cor, pct) = a.trim_start().split_once(char::is_whitespace)?;
    let p = pct.trim().strip_suffix('%')?;
    if p.is_empty() || !p.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let b = b.trim();
    if cor == "currentcolor" {
        return (b == "transparent").then(|| ("currentColor".to_string(), fecha + 1));
    }
    let p = p.parse::<u32>().ok()?.min(100);
    let x = hex(cor)?;
    let y = if b == "transparent" { x } else { hex(b)? };
    let canal = |i: usize| (x[i] * p + y[i] * (100 - p) + 50) / 100;
    Some((format!("#{:02x}{:02x}{:02x}", canal(0), canal(1), canal(2)), fecha + 1))
}

fn trocar_misturas(s: &str) -> String {
    const MARCA: &str = "color-mix(";
    let baixo = s.to_ascii_lowercase();
    let (mut saida, mut i) = (String::with_capacity(s.len()), 0);
    while let Some(p) = baixo[i..].find(MARCA) {
        let de = i + p;
        saida.push_str(&s[i..de]);
        let depois = de + MARCA.len();
        match ler_mistura(&s[depois..]) {
            Some((cor, andou)) => {
                saida.push_str(&cor);
                i = depois + andou;
            }
            None => {
                saida.push_str(&s[de..depois]);
                i = depois;
            }
        }
    }
    saida.push_str(&s[i..]);
    saida
}

/// Troca `var()` e `color-mix()` por cores prontas, no CSS ou nos atributos de uma figura. 4 passadas, como `estilo.mjs`.
pub fn resolver(s: &str, vars: &BTreeMap<String, String>) -> String {
    let mut saida = s.to_string();
    for _ in 0..4 {
        if !saida.contains("var(") || saida.len() > TETO_RESOLVIDO {
            break;
        }
        saida = passada_var(&saida, vars);
    }
    trocar_misturas(&saida)
}
