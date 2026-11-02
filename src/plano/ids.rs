//! Os ids do plano: o id do formato é o `ref`, o id da árvore é derivado dele. Também os nomes que saem daí (ramo, agente) e a área de cada plano.
use serde_json::Value;
use sha2::{Digest, Sha256};

/// O sha256 em hexadecimal.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// `"p"` + 16 hexadecimais. Plano e checkpoint passam `r = "<acao_ref>/<ref>"`, porque o id da árvore é global por tabela.
pub fn id_real(bp: &str, tipo: &str, r: &str) -> String {
    let hash = sha256_hex(format!("{bp}\0{tipo}\0{r}").as_bytes());
    format!("p{}", &hash[..16])
}

/// `^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$`, sem `..`, sem fim em `.lock` (em qualquer caixa: no NTFS é o mesmo arquivo) nem em `.`: vira nome de ramo e de pasta.
pub fn ref_valido(r: &str) -> bool {
    let b = r.as_bytes();
    let forma = !b.is_empty() && b.len() <= 64 && b[0].is_ascii_alphanumeric() && b.iter().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'));
    forma && !r.contains("..") && !r.ends_with('.') && !r.to_ascii_lowercase().ends_with(".lock")
}

/// `agente/<r>` se `r` é válido; senão, 12 hexadecimais do sha256 de `r`, para nada do texto chegar ao git.
pub fn ramo_de(r: &str) -> String {
    if ref_valido(r) {
        format!("agente/{r}")
    } else {
        format!("agente/{}", &sha256_hex(r.as_bytes())[..12])
    }
}

/// `plano <r>`, no máximo 40 caracteres (o limite do nome de agente em `mcp::acoes`).
pub fn nome_agente(r: &str) -> String {
    format!("plano {r}").chars().take(40).collect()
}

/// 1 a 120 caracteres: alfanumérico unicode, espaço, `.`, `_`, `-`, `/`. Não começa com `-` nem `/`, sem segmento `..`, e sobra algo depois de `area_normal`.
pub fn area_valida(a: &str) -> bool {
    let tamanho = a.chars().count();
    let forma = (1..=120).contains(&tamanho) && !a.starts_with(['-', '/']) && a.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '.' | '_' | '-' | '/'));
    forma && a.split('/').all(|s| s.trim() != "..") && !area_normal(a).is_empty()
}

/// Minúsculas, `\` vira `/`, `trim` em cada segmento, sem segmentos vazios nem `.`, sem `/` no fim. É a forma que `escalonar` compara.
pub fn area_normal(a: &str) -> String {
    let minusculas = a.to_lowercase().replace('\\', "/");
    let segmentos: Vec<&str> = minusculas.split('/').map(str::trim).filter(|s| !s.is_empty() && *s != ".").collect();
    segmentos.join("/")
}

fn acha_em<'a>(itens: impl Iterator<Item = &'a Value> + Clone, r: &str) -> Option<String> {
    let id_de = |i: &'a Value| i.get("id").and_then(Value::as_str);
    let por_id = itens.clone().find(|i| id_de(i) == Some(r));
    // o `extra` de cada linha volta no próprio objeto (`dados/formato.rs`), então `extra.ref` é o campo `ref` do item
    por_id.or_else(|| itens.clone().find(|i| i.get("ref").and_then(Value::as_str) == Some(r))).and_then(id_de).map(String::from)
}

/// O id real de um item do blueprint `bp`, procurado por id (vence) ou por `extra.ref`. `tipo`: `acao`, `plano` ou `checkpoint`.
pub fn traduzir(bp: &Value, r: &str, tipo: &str) -> Option<String> {
    if r.is_empty() {
        return None;
    }
    let acoes = bp.get("acoes").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let filhos = |lista: &'static str| acoes.iter().flat_map(move |a| a.get(lista).and_then(Value::as_array).map_or(&[][..], Vec::as_slice));
    match tipo {
        "acao" => acha_em(acoes.iter(), r),
        "plano" => acha_em(filhos("planos"), r),
        "checkpoint" => acha_em(filhos("checkpoints"), r),
        _ => None,
    }
}
