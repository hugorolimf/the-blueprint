//! Idioma: o texto em português é a chave (`tr("Atualizar")`), como na página; sem tradução volta ao português e nunca quebra.
//! O inglês vem do catálogo da página (`web/i18n/en-*.js`, exportado por `node scripts/exportar-i18n.mjs`) mais os poucos textos que só
//! existem aqui. O idioma segue o do sistema (`LC_ALL`, `LC_MESSAGES`, `LANG`; no navegador, `navigator.language`) e, fora do português, é o inglês.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Textos que só a interface nativa tem (os da página já estão no catálogo).
const LOCAIS: &[(&str, &str)] = &[
    ("Salvar", "Save"),
    ("Ex.: Portal do fornecedor", "E.g.: Supplier portal"),
    ("Para que serve", "What it is for"),
    ("Não foi possível salvar.", "Could not save."),
    ("Abrir na versão web", "Open in the web version"),
    ("A folha do blueprint, as ações e os planos ainda abrem na versão web.", "The blueprint sheet, actions and plans still open in the web version."),
    ("Breve descrição do workspace", "Short description of the workspace"),
    ("Carregando…", "Loading…"),
    ("Excluir workspace…", "Delete workspace…"),
    ("Os dados mudaram em outra janela. Atualizei: tente de novo.", "The data changed in another window. I refreshed: try again."),
    ("Pesquisar ações, planos e checkpoints", "Search actions, plans and checkpoints"),
    ("Pesquisar blueprints, ações, planos", "Search blueprints, actions, plans"),
    ("Não foi possível marcar o item.", "Could not mark the item."),
    ("sem blueprints", "no blueprints"),
    ("{pct}% CONSTRUÍDA", "{pct}% BUILT"),
    ("Tentar de novo", "Try again"),
];

fn catalogo() -> &'static HashMap<String, Value> {
    static CATALOGO: OnceLock<HashMap<String, Value>> = OnceLock::new();
    CATALOGO.get_or_init(|| serde_json::from_str(include_str!("../assets/i18n-en.json")).unwrap_or_default())
}

pub fn ingles() -> bool {
    static INGLES: OnceLock<bool> = OnceLock::new();
    *INGLES.get_or_init(|| !idioma_do_sistema().to_lowercase().starts_with("pt"))
}

#[cfg(not(target_arch = "wasm32"))]
fn idioma_do_sistema() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"].iter().filter_map(|v| std::env::var(v).ok()).find(|v| !v.is_empty() && v != "C" && v != "POSIX").unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
fn idioma_do_sistema() -> String {
    web_sys::window().and_then(|w| w.navigator().language()).unwrap_or_default()
}

/// A tradução de `pt`, ou o próprio `pt` se não houver (ou se o idioma for o português).
pub fn tr(pt: &'static str) -> &'static str {
    if !ingles() {
        return pt;
    }
    LOCAIS.iter().find(|(chave, _)| *chave == pt).map(|(_, en)| *en).or_else(|| catalogo().get(pt).and_then(Value::as_str)).unwrap_or(pt)
}

/// `tr` com os `{nome}` do texto trocados pelos valores.
pub fn tr_com(pt: &'static str, valores: &[(&str, &str)]) -> String {
    troca(tr(pt), valores)
}

/// Plural como a página (`trn`): `um` é a chave do catálogo (`{n} blueprint`) e `varios` o plural em português. No português 0 e 1 usam o singular.
pub fn trn(n: usize, um: &'static str, varios: &'static str) -> String {
    let formas = if ingles() { catalogo().get(um).and_then(Value::as_array).map(|l| (l.first().and_then(Value::as_str).unwrap_or(um), l.get(1).and_then(Value::as_str).unwrap_or(varios))) } else { None };
    let (singular, plural) = formas.unwrap_or((um, varios));
    let escolhida = if (ingles() && n == 1) || (!ingles() && n <= 1) { singular } else { plural };
    troca(escolhida, &[("n", &n.to_string())])
}

fn troca(texto: &str, valores: &[(&str, &str)]) -> String {
    valores.iter().fold(texto.to_string(), |s, (chave, valor)| s.replace(&format!("{{{chave}}}"), valor))
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::path::Path;

    /// Os textos passados a `tr("...")` no código: lê os arquivos e pega o que vem entre as aspas.
    fn chaves(pasta: &Path, achadas: &mut Vec<String>) {
        for entrada in std::fs::read_dir(pasta).unwrap().flatten() {
            let caminho = entrada.path();
            if caminho.is_dir() {
                chaves(&caminho, achadas);
            } else if caminho.extension().is_some_and(|e| e == "rs") {
                // as linhas de comentário citam `tr("...")` só como exemplo
                let fonte: String = std::fs::read_to_string(&caminho).unwrap().lines().filter(|l| !l.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
                for prefixo in ["tr(\"", "tr_com(\""] {
                    for (i, _) in fonte.match_indices(prefixo) {
                        let resto = &fonte[i + prefixo.len()..];
                        let mut fim = 0;
                        while fim < resto.len() && !(resto.as_bytes()[fim] == b'"' && (fim == 0 || resto.as_bytes()[fim - 1] != b'\\')) {
                            fim += 1;
                        }
                        achadas.push(resto[..fim].replace("\\\"", "\"").replace("\\n", "\n"));
                    }
                }
            }
        }
    }

    #[test]
    fn todo_texto_do_app_tem_ingles() {
        let mut achadas = Vec::new();
        chaves(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut achadas);
        let sem: Vec<_> = achadas.iter().filter(|c| !c.is_empty() && !LOCAIS.iter().any(|(k, _)| k == c) && !catalogo().contains_key(c.as_str())).collect();
        assert!(sem.is_empty(), "sem inglês (acrescente em LOCAIS ou no catálogo da página): {sem:#?}");
    }

    #[test]
    fn plural_como_a_pagina() {
        assert!(catalogo().contains_key("{n} blueprint"), "o catálogo traz a lista [um, outros]");
        // em inglês só o 1 é singular; em português o 0 também
        assert_eq!(troca("{n} blueprints", &[("n", "2")]), "2 blueprints");
    }
}
