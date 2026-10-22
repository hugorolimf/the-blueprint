//! Embute no binário os arquivos que o servidor entrega (a página, `web/`, o kit do anatomy e `emblemas.mjs`):
//! gera `$OUT_DIR/assets.rs` com uma tabela (caminho na URL, bytes). Mesma lista que o servidor Node deixa sair; nada mais da pasta fica exposto.
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn coletar(dir: &Path, extensoes: &[&str], saida: &mut Vec<PathBuf>) {
    let Ok(entradas) = fs::read_dir(dir) else { return };
    let mut caminhos: Vec<PathBuf> = entradas.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    caminhos.sort();
    for p in caminhos {
        if p.is_dir() {
            coletar(&p, extensoes, saida);
        } else if p.extension().and_then(|e| e.to_str()).is_some_and(|e| extensoes.contains(&e)) {
            saida.push(p);
        }
    }
}

fn main() {
    let raiz = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut arquivos: Vec<PathBuf> = vec![raiz.join("blueprint.html"), raiz.join("emblemas.mjs"), raiz.join("agentes-figura.mjs")];
    coletar(&raiz.join("web"), &["css", "js"], &mut arquivos);
    coletar(&raiz.join("kit"), &["mjs"], &mut arquivos);
    let mut codigo = String::from("pub static ASSETS: &[(&str, &[u8])] = &[\n");
    for a in &arquivos {
        let url = format!("/{}", a.strip_prefix(&raiz).expect("dentro da raiz").to_string_lossy().replace('\\', "/"));
        let caminho = a.to_string_lossy().replace('\\', "/");
        codigo.push_str(&format!("    ({url:?}, include_bytes!({caminho:?})),\n"));
    }
    codigo.push_str("];\n");
    fs::write(PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("assets.rs"), codigo).expect("gravar assets.rs");
    for pasta in ["web", "kit", "blueprint.html", "emblemas.mjs", "agentes-figura.mjs", "build.rs"] {
        println!("cargo:rerun-if-changed={pasta}");
    }
}
