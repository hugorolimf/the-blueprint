//! Pequenas conversões sobre JSON que o `plano.mjs` fazia com `texto()`, `||` e `?.`.
use serde_json::Value;

/// `texto()` do JS: a string aparada, ou vazio se o valor não for string.
pub fn texto(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.trim().to_string(),
        _ => String::new(),
    }
}

/// `texto(obj?.chave)`
pub fn campo(v: &Value, chave: &str) -> String {
    texto(v.get(chave))
}

/// `obj.chave || ""` para um objeto que já existe nos dados (sem aparar)
pub fn str_de(v: &Value, chave: &str) -> String {
    v.get(chave).and_then(Value::as_str).unwrap_or("").to_string()
}

/// `a || b` entre textos
pub fn ou(a: String, b: String) -> String {
    if a.is_empty() { b } else { a }
}

/// `slice(0, n)` do JS (aqui em caracteres; difere só para emojis e afins)
pub fn truncar(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn tamanho(s: &str) -> usize {
    s.chars().count()
}

/// o valor como lista (vazia se não for lista)
pub fn lista(v: Option<&Value>) -> &[Value] {
    match v {
        Some(Value::Array(a)) => a.as_slice(),
        _ => &[],
    }
}

/// Só um aviso: a primeira palavra termina em ar, er, ir ou or. Não dá para checar verbo de verdade.
pub fn infinitivo(t: &str) -> bool {
    let primeira = t.split_whitespace().next().unwrap_or("").to_lowercase();
    ["ar", "er", "ir", "or"].iter().any(|s| primeira.ends_with(s))
}

/// ` "texto"` (com o espaço antes) ou nada, para montar rótulos como `Ação 1 "Base"`
pub fn entre_aspas(t: &str) -> String {
    if t.is_empty() { String::new() } else { format!(" \"{t}\"") }
}

/// `truthy` do JS
pub fn verdade(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
        Some(_) => true,
    }
}
