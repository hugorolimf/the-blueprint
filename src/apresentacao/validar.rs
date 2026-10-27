//! O que o agente manda em `anexar_apresentacao`: limites, ids e valores do contrato (erros que bloqueiam) e avisos que não bloqueiam (termo técnico no público
//! `cliente`, figura que nenhuma etapa usa, figura sem código). Tudo o que o agente lê aqui é inglês.
use crate::http::pedido::{falha, ErroHttp};
use serde_json::Value;
use std::collections::HashSet;

pub const CONTEUDO_MAX: usize = 128 * 1024;
pub const FIGURAS_MAX: usize = 12;
pub const CODIGO_MAX: usize = 200 * 1024;
/// O conteúdo e as figuras juntos.
pub const TOTAL_MAX: usize = 2 * 1024 * 1024;
const TITULO_MAX: usize = 120;
const PUBLICOS: [&str; 2] = ["cliente", "interno"];
/// `epico` vale, mas só quem pediu pelo nome o manda: nenhum padrão do app o escolhe.
const NIVEIS: [&str; 4] = ["rapido", "padrao", "completo", "epico"];
/// Os ids de seção da página (a figura que leva `secao` entra no painel dela); `acoes` só existe na página `interno`.
const SECOES: [&str; 8] = ["visao", "recebe", "etapas", "linha", "precisamos", "decisoes", "riscos", "acoes"];
/// Quantos achados de termo técnico o aviso lista.
const ACHADOS_MAX: usize = 10;

pub struct Validada {
    pub titulo: String,
    pub publico: String,
    pub nivel: String,
    pub avisos: Vec<String>,
}

fn id_de_figura_valido(id: &str) -> bool {
    let b = id.as_bytes();
    !b.is_empty() && b.len() <= 40 && b[0].is_ascii_alphanumeric() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn texto<'a>(v: &'a Value, chave: &str) -> &'a str {
    v.get(chave).and_then(Value::as_str).unwrap_or("")
}

/// Valida o conteúdo e as figuras (`figuras` ausente vale lista vazia). O primeiro erro bloqueia; os avisos voltam na resposta.
pub fn validar(conteudo: &Value, figuras: &Value) -> Result<Validada, ErroHttp> {
    let Some(c) = conteudo.as_object() else {
        return Err(falha(400, crate::tr!("conteudo is required: the content JSON object of the presentation", "conteudo é obrigatório: o objeto JSON de conteúdo da apresentação")).codigo("falta-conteudo"));
    };
    let tamanho_conteudo = conteudo.to_string().len();
    if tamanho_conteudo > CONTEUDO_MAX {
        let kb = CONTEUDO_MAX / 1024;
        return Err(falha(413, crate::tr!("conteudo is over {kb} KB; shorten the texts", "conteudo passa de {kb} KB; encurte os textos")).codigo("conteudo-grande").params(serde_json::json!({"kb": kb})));
    }
    let titulo: String = texto(conteudo, "titulo").trim().chars().take(TITULO_MAX).collect();
    if titulo.is_empty() {
        return Err(falha(400, crate::tr!("conteudo.titulo is required", "conteudo.titulo é obrigatório")).codigo("falta-titulo"));
    }
    let publico = texto(conteudo, "publico");
    if !PUBLICOS.contains(&publico) {
        return Err(falha(400, crate::tr!("conteudo.publico must be \"cliente\" or \"interno\"", "conteudo.publico deve ser \"cliente\" ou \"interno\"")).codigo("publico-invalido"));
    }
    let nivel = texto(conteudo, "nivel");
    if !NIVEIS.contains(&nivel) {
        return Err(falha(400, crate::tr!("conteudo.nivel must be rapido, padrao, completo or epico", "conteudo.nivel deve ser rapido, padrao, completo ou epico")).codigo("nivel-invalido"));
    }
    if publico == "cliente" && c.contains_key("interno") {
        return Err(falha(400, crate::tr!("a \"cliente\" presentation must not carry the \"interno\" block (it would reach the client); remove it", "uma apresentação \"cliente\" não pode levar o bloco \"interno\" (chegaria ao cliente); remova-o")).codigo("interno-no-cliente"));
    }
    let lista: &[Value] = match figuras {
        Value::Null => &[],
        Value::Array(a) => a,
        _ => return Err(falha(400, crate::tr!("figuras must be a list", "figuras deve ser uma lista")).codigo("figuras-invalidas")),
    };
    if lista.len() > FIGURAS_MAX {
        let n = FIGURAS_MAX;
        return Err(falha(400, crate::tr!("at most {n} figures per presentation", "no máximo {n} figuras por apresentação")).codigo("figuras-demais").params(serde_json::json!({"n": n})));
    }
    let mut avisos = Vec::new();
    let mut ids = HashSet::new();
    let mut total = tamanho_conteudo;
    let mut por_secao: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for (i, f) in lista.iter().enumerate() {
        let id = texto(f, "id");
        if !f.is_object() || !id_de_figura_valido(id) {
            return Err(falha(400, crate::tr!("figuras[{i}].id must be 1 to 40 characters: lowercase letters, digits and hyphens", "figuras[{i}].id deve ter de 1 a 40 caracteres: minúsculas, dígitos e hífens")).codigo("figura-id-invalido").params(serde_json::json!({"i": i})));
        }
        if !ids.insert(id.to_string()) {
            return Err(falha(400, crate::tr!("figure id \"{id}\" is repeated", "o id de figura \"{id}\" está repetido")).codigo("figura-id-repetido").params(serde_json::json!({"id": id})));
        }
        match f.get("codigo") {
            None | Some(Value::Null) => avisos.push(format!("Figure \"{id}\" has no codigo: it will show an error card in the page.")),
            Some(Value::String(s)) if s.trim().is_empty() => avisos.push(format!("Figure \"{id}\" has an empty codigo: it will show an error card in the page.")),
            Some(Value::String(s)) if s.len() > CODIGO_MAX => {
                let kb = CODIGO_MAX / 1024;
                return Err(falha(413, crate::tr!("the codigo of figure \"{id}\" is over {kb} KB; the figure program must be small (the browser builds the SVG)", "o codigo da figura \"{id}\" passa de {kb} KB; o programa da figura deve ser pequeno (o navegador monta o SVG)")).codigo("codigo-grande").params(serde_json::json!({"id": id, "kb": kb})));
            }
            Some(Value::String(_)) => {}
            Some(_) => return Err(falha(400, crate::tr!("the codigo of figure \"{id}\" must be a string (a JavaScript function expression)", "o codigo da figura \"{id}\" deve ser texto (uma expressão de função JavaScript)")).codigo("codigo-invalido").params(serde_json::json!({"id": id}))),
        }
        if let Some(s) = f.get("secao").filter(|v| !v.is_null()) {
            let nome = s.as_str().unwrap_or("");
            if SECOES.contains(&nome) && !(nome == "acoes" && publico == "cliente") {
                por_secao.entry(nome).or_default().push(id);
            } else {
                avisos.push(format!("secao-desconhecida: Figure \"{id}\" has secao {s}, which is not a section of this page (use {}{}).", SECOES[..7].join(", "), if publico == "interno" { ", acoes" } else { "; acoes exists only in the interno page" }));
            }
        }
        if f.get("impressao").and_then(Value::as_str).is_none_or(str::is_empty) {
            avisos.push(format!("Figure \"{id}\" has no impressao: it cannot be kept when the presentation is regenerated and will be redrawn."));
        }
        total += f.to_string().len();
    }
    if total > TOTAL_MAX {
        let mb = TOTAL_MAX / 1024 / 1024;
        return Err(falha(413, crate::tr!("the presentation is over {mb} MB (content plus figures)", "a apresentação passa de {mb} MB (conteúdo mais figuras)")).codigo("apresentacao-grande").params(serde_json::json!({"mb": mb})));
    }
    for (secao, ids) in por_secao.iter().filter(|(_, ids)| ids.len() > 1) {
        avisos.push(format!("secao-duplicada: Section \"{secao}\" has {} figures ({}); the page shows one per section, give the others another section or a stage.", ids.len(), ids.join(", ")));
    }
    for etapa in conteudo.get("etapas").and_then(Value::as_array).into_iter().flatten() {
        let fig = texto(etapa, "figura");
        if !fig.is_empty() && !ids.contains(fig) {
            avisos.push(format!("Stage \"{}\" points to figure \"{fig}\", which is not in figuras.", texto(etapa, "id")));
        }
    }
    if publico == "cliente" {
        avisos.extend(auditar_cliente(conteudo));
    }
    Ok(Validada { titulo, publico: publico.to_string(), nivel: nivel.to_string(), avisos })
}

/// A lista de `apresentacao-ia/termos-tecnicos.json`: uma lista de textos, ou um objeto com `termos`, ou um objeto cujos valores são listas.
pub fn termos_de(json: &str) -> Vec<String> {
    let textos = |v: &Value| -> Vec<String> { v.as_array().into_iter().flatten().filter_map(Value::as_str).map(|t| t.trim().to_lowercase()).filter(|t| !t.is_empty()).collect() };
    let Ok(v) = serde_json::from_str::<Value>(json) else { return Vec::new() };
    match &v {
        Value::Array(_) => textos(&v),
        Value::Object(m) => m.get("termos").map(textos).unwrap_or_else(|| m.values().flat_map(textos).collect()),
        _ => Vec::new(),
    }
}

/// O termo aparece em `texto` (já em minúsculas) como palavra inteira? (`api` casa `api` e não `aplicativo`; `cli` não casa `cliente`; o plural vem escrito na lista)
fn aparece(texto: &str, termo: &str) -> bool {
    let borda = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
    texto.match_indices(termo).any(|(i, _)| borda(texto[..i].chars().next_back()) && borda(texto[i + termo.len()..].chars().next()))
}

/// Um número seguido de `h`, `hs`, `hr(s)`, `hora(s)` ou `hour(s)`: horas por plano não entram no público cliente (a lista de termos não pega números).
fn tem_horas(texto: &str) -> bool {
    let c: Vec<char> = texto.chars().collect();
    let mut i = 0;
    while i < c.len() {
        if !c[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut fim = i;
        while fim < c.len() && (c[fim].is_ascii_digit() || c[fim] == '.' || c[fim] == ',') {
            fim += 1;
        }
        let depois = fim + c[fim..].iter().take_while(|x| **x == ' ').count();
        let palavra: String = c[depois..].iter().take_while(|x| x.is_alphanumeric()).collect();
        if matches!(palavra.as_str(), "h" | "hs" | "hr" | "hrs" | "hora" | "horas" | "hour" | "hours") {
            return true;
        }
        i = fim;
    }
    false
}

/// Campos que guardam identificador, data ou valor fixo do contrato, não texto para o cliente ler: ficam fora da auditoria.
const ESTRUTURA: [&str; 11] = ["versao", "publico", "nivel", "idioma", "data", "id", "figura", "inicio", "fim", "ate", "estado"];

/// Cada texto do conteúdo contra a lista (e contra horas): `caminho: termo`, até `ACHADOS_MAX`.
pub fn achados(conteudo: &Value, termos: &[String]) -> Vec<String> {
    fn percorrer(v: &Value, caminho: String, termos: &[String], saida: &mut Vec<String>) {
        if saida.len() >= ACHADOS_MAX {
            return;
        }
        match v {
            Value::String(s) => {
                let minusculas = s.to_lowercase();
                for t in termos.iter().filter(|t| aparece(&minusculas, t)) {
                    saida.push(format!("{caminho}: {t}"));
                }
                if tem_horas(&minusculas) {
                    saida.push(format!("{caminho}: hours"));
                }
            }
            Value::Array(a) => a.iter().enumerate().for_each(|(i, x)| percorrer(x, format!("{caminho}[{i}]"), termos, saida)),
            Value::Object(m) => m.iter().filter(|(k, _)| !ESTRUTURA.contains(&k.as_str())).for_each(|(k, x)| percorrer(x, if caminho.is_empty() { k.clone() } else { format!("{caminho}.{k}") }, termos, saida)),
            _ => {}
        }
    }
    let mut saida = Vec::new();
    percorrer(conteudo, String::new(), termos, &mut saida);
    saida.truncate(ACHADOS_MAX);
    saida
}

fn auditar_cliente(conteudo: &Value) -> Vec<String> {
    let Some(lista) = super::arquivo("termos-tecnicos.json") else {
        return vec!["The technical-terms list is not part of this build: the client audience was not audited.".into()];
    };
    let achou = achados(conteudo, &termos_de(lista));
    if achou.is_empty() {
        return Vec::new();
    }
    vec![format!("The client presentation uses technical terms; rewrite them in plain language ({}).", achou.join("; "))]
}
