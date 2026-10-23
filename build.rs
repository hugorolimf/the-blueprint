//! Embute no binário os arquivos que o servidor entrega (a página, `web/`, o kit do anatomy e `emblemas.mjs`):
//! gera `$OUT_DIR/assets.rs` com uma tabela (caminho na URL, bytes). Mesma lista que o servidor Node deixa sair; nada mais da pasta fica exposto.
//! No Windows (MSVC) também transforma `recursos/icone/icone.ico` num arquivo de recursos (.res) que o linker põe no .exe: é o ícone do
//! Explorer, da janela e da bandeja (grupo de ícones de número 1). Feito à mão para não depender do rc.exe do Windows SDK.
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

fn le32(b: &[u8], i: usize) -> usize {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize
}

/// Uma entrada de um .res de 32 bits: tamanho, cabeçalho de 32 bytes (tipo e nome por número), dados alinhados a 4 bytes.
fn recurso(res: &mut Vec<u8>, tipo: u16, nome: u16, flags: u16, dados: &[u8]) {
    res.extend((dados.len() as u32).to_le_bytes());
    res.extend(32u32.to_le_bytes());
    for palavra in [0xFFFF, tipo, 0xFFFF, nome] {
        res.extend(palavra.to_le_bytes());
    }
    res.extend(0u32.to_le_bytes()); // versão dos dados
    res.extend(flags.to_le_bytes());
    res.extend(0u16.to_le_bytes()); // idioma neutro
    res.extend([0u8; 8]); // versão e características
    res.extend(dados);
    res.resize(res.len().next_multiple_of(4), 0);
}

/// .ico → .res: cada imagem vira um RT_ICON (3) e o diretório vira o RT_GROUP_ICON (14) de número 1.
fn res_do_ico(ico: &[u8]) -> Vec<u8> {
    let n = u16::from_le_bytes([ico[4], ico[5]]);
    let mut res = Vec::new();
    recurso(&mut res, 0, 0, 0, &[]); // a entrada vazia que abre todo .res
    let mut grupo = vec![0, 0, 1, 0];
    grupo.extend(n.to_le_bytes());
    for i in 0..n {
        let e = &ico[6 + 16 * i as usize..][..16];
        recurso(&mut res, 3, i + 1, 0x1010, &ico[le32(e, 12)..][..le32(e, 8)]);
        grupo.extend(&e[..12]);
        grupo.extend((i + 1).to_le_bytes());
    }
    recurso(&mut res, 14, 1, 0x1030, &grupo);
    res
}

fn main() {
    let raiz = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let mut arquivos: Vec<PathBuf> = vec![raiz.join("blueprint.html"), raiz.join("emblemas.mjs"), raiz.join("agentes-figura.mjs"), raiz.join("agentes-claude.mjs")];
    coletar(&raiz.join("web"), &["css", "js"], &mut arquivos);
    coletar(&raiz.join("kit"), &["mjs"], &mut arquivos);
    let mut codigo = String::from("pub static ASSETS: &[(&str, &[u8])] = &[\n");
    for a in &arquivos {
        let url = format!("/{}", a.strip_prefix(&raiz).expect("dentro da raiz").to_string_lossy().replace('\\', "/"));
        let caminho = a.to_string_lossy().replace('\\', "/");
        codigo.push_str(&format!("    ({url:?}, include_bytes!({caminho:?})),\n"));
    }
    codigo.push_str("];\n");
    let saida = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    fs::write(saida.join("assets.rs"), codigo).expect("gravar assets.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let ico = fs::read(raiz.join("recursos/icone/icone.ico")).expect("ler recursos/icone/icone.ico (gere com node scripts/icone/desenhar.mjs)");
        let res = saida.join("icone.res");
        fs::write(&res, res_do_ico(&ico)).expect("gravar icone.res");
        println!("cargo:rustc-link-arg-bins={}", res.display());
    }
    for pasta in ["web", "kit", "blueprint.html", "emblemas.mjs", "agentes-figura.mjs", "agentes-claude.mjs", "recursos/icone/icone.ico", "build.rs"] {
        println!("cargo:rerun-if-changed={pasta}");
    }
}
