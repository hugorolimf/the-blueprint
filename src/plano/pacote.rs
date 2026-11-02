//! A pasta do plano (`plano.json`, estilo e figuras), lida só no processo `--mcp`. Só o que o `plano.json` cita sai do disco, e nada fora da pasta. Quem valida o plano em si é `validar`. Mensagens em inglês: vão ao agente.
use super::svg::{css_seguro, imagem_tipo, medidas_imagem, medidas_svg, svg_seguro, LIMITE_IMAGEM, LIMITE_SVG, MAX_FIGURAS};
use serde_json::Value;
use std::io::Read;
use std::path::{Path, PathBuf};

/// `plano` é o JSON como veio; `figuras` traz o caminho exatamente como o `plano.json` o escreve.
#[derive(Debug, Clone, PartialEq)]
pub struct Pacote { pub plano: Value, pub estilo: Option<String>, pub figuras: Vec<(String, Vec<u8>)>, pub avisos: Vec<String> }

const LIMITE_PLANO: usize = 4 << 20;

/// Recusa o que sai da pasta antes de tocar o disco. `\` conta como separador nos dois sistemas e `:` nunca entra (unidade e fluxo alternativo do NTFS).
fn segmentos(caminho: &str) -> Result<Vec<&str>, String> {
    if caminho.is_empty() || caminho.contains(['\0', ':']) || caminho.starts_with(['/', '\\']) {
        return Err(format!("{caminho}: must be a relative path inside the plan folder"));
    }
    let partes: Vec<&str> = caminho.split(['/', '\\']).filter(|s| !s.is_empty() && *s != ".").collect();
    if partes.is_empty() || partes.contains(&"..") {
        return Err(format!("{caminho}: must be inside the plan folder"));
    }
    Ok(partes)
}

/// Anda componente a componente sem seguir link: link simbólico em qualquer um deles é recusado. O último tem de ser arquivo comum (uma FIFO prenderia a leitura).
fn resolver_na_pasta(pasta: &Path, caminho: &str) -> Result<PathBuf, String> {
    let partes = segmentos(caminho)?;
    let mut atual = pasta.to_path_buf();
    for (i, parte) in partes.iter().enumerate() {
        atual.push(parte);
        let tipo = std::fs::symlink_metadata(&atual).map_err(|_| format!("{caminho}: not found"))?.file_type();
        if tipo.is_symlink() {
            return Err(format!("{caminho}: symbolic links are not accepted"));
        }
        let ultimo = i + 1 == partes.len();
        if (ultimo && !tipo.is_file()) || (!ultimo && !tipo.is_dir()) {
            return Err(format!("{caminho}: not a regular file inside the plan folder"));
        }
    }
    Ok(atual)
}

/// O teto vale para o que foi lido, não para o que o `stat` disse: o arquivo pode ter crescido no meio.
fn ler_arquivo(pasta: &Path, caminho: &str, limite: usize) -> Result<Vec<u8>, String> {
    let arq = resolver_na_pasta(pasta, caminho)?;
    let mut bytes = Vec::new();
    let aberto = std::fs::File::open(&arq).map_err(|e| format!("{caminho}: {e}"))?;
    aberto.take(limite as u64 + 1).read_to_end(&mut bytes).map_err(|e| format!("{caminho}: {e}"))?;
    if bytes.len() > limite {
        return Err(format!("{caminho}: larger than {} MB", limite >> 20));
    }
    Ok(bytes)
}

fn extensao(caminho: &str) -> String {
    caminho.rsplit(['/', '\\']).next().and_then(|n| n.rsplit_once('.')).map_or(String::new(), |(_, e)| e.to_ascii_lowercase())
}

/// Figura: extensão, tamanho e conteúdo têm de concordar. Medida que falta não impede, só avisa: a prancha usa proporção 1.
fn ler_figura(pasta: &Path, caminho: &str, avisos: &mut Vec<String>) -> Result<Vec<u8>, String> {
    segmentos(caminho)?;
    let ext = extensao(caminho);
    let esperado = match ext.as_str() {
        "svg" => "svg",
        "png" => "png",
        "jpg" | "jpeg" => "jpg",
        "webp" => "webp",
        _ => return Err(format!("{caminho}: use SVG, PNG, JPG or WebP")),
    };
    let bytes = ler_arquivo(pasta, caminho, if ext == "svg" { LIMITE_SVG } else { LIMITE_IMAGEM })?;
    let medidas = if ext == "svg" {
        let texto = std::str::from_utf8(&bytes).map_err(|_| format!("{caminho}: SVG is not UTF-8"))?;
        svg_seguro(texto).map_err(|e| format!("{caminho}: {e}"))?;
        medidas_svg(texto)
    } else {
        if imagem_tipo(&bytes) != Some(esperado) {
            return Err(format!("{caminho}: the content is not a {esperado} file"));
        }
        medidas_imagem(&bytes)
    };
    if medidas.is_none() {
        avisos.push(format!("{caminho}: could not read its size; it will be drawn with proportion 1"));
    }
    Ok(bytes)
}

fn ler_estilo(pasta: &Path, caminho: &str) -> Result<String, String> {
    segmentos(caminho)?;
    if extensao(caminho) != "css" {
        return Err(format!("{caminho}: the style must be a .css file"));
    }
    let bytes = ler_arquivo(pasta, caminho, LIMITE_SVG)?;
    let css = String::from_utf8(bytes).map_err(|_| format!("{caminho}: the style is not UTF-8"))?;
    css_seguro(&css).map_err(|e| format!("{caminho}: {e}"))?;
    Ok(css)
}

/// Os caminhos citados em `projetos[].acoes[].detalhes[].figura`, na ordem e sem repetir. Plano torto não é erro aqui: `formato` e `validar` respondem por ele.
fn figuras_citadas(plano: &Value) -> Vec<&str> {
    fn itens<'a>(v: &'a Value, chave: &str) -> &'a [Value] {
        v.get(chave).and_then(Value::as_array).map_or(&[], Vec::as_slice)
    }
    let mut citadas: Vec<&str> = Vec::new();
    for pr in itens(plano, "projetos") {
        for a in itens(pr, "acoes") {
            for d in itens(a, "detalhes") {
                if let Some(f) = d.get("figura").and_then(Value::as_str).filter(|f| !citadas.contains(f)) {
                    citadas.push(f);
                    if citadas.len() > MAX_FIGURAS {
                        return citadas; // quem chama recusa; a busca acima fica limitada
                    }
                }
            }
        }
    }
    citadas
}

/// Junta todos os erros de uma vez, para o agente corrigir tudo numa volta só.
pub fn ler_pasta(pasta: &Path) -> Result<Pacote, Vec<String>> {
    let bytes = ler_arquivo(pasta, "plano.json", LIMITE_PLANO).map_err(|e| vec![e])?;
    let plano: Value = serde_json::from_slice(&bytes).map_err(|e| vec![format!("plano.json: {e}")])?;
    if !plano.is_object() {
        return Err(vec!["plano.json: must be an object".to_string()]);
    }
    let (mut erros, mut avisos) = (Vec::new(), Vec::new());
    let mut estilo = None;
    match plano.get("estilo") {
        None | Some(Value::Null) => {}
        Some(Value::String(caminho)) => match ler_estilo(pasta, caminho) {
            Ok(css) => estilo = Some(css),
            Err(e) => erros.push(e),
        },
        Some(_) => erros.push("estilo: must be a path".to_string()),
    }
    let citadas = figuras_citadas(&plano);
    if citadas.len() > MAX_FIGURAS {
        erros.push(format!("detalhes: more than {MAX_FIGURAS} figures"));
        return Err(erros);
    }
    let mut figuras = Vec::with_capacity(citadas.len());
    for caminho in citadas {
        match ler_figura(pasta, caminho, &mut avisos) {
            Ok(b) => figuras.push((caminho.to_string(), b)),
            Err(e) => erros.push(e),
        }
    }
    if !erros.is_empty() {
        return Err(erros);
    }
    Ok(Pacote { plano, estilo, figuras, avisos })
}

// Ficam aqui, e não em `svg_tests.rs`, para o filtro `plano::pacote` do critério de pronto achá-los.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static SEQ: AtomicU32 = AtomicU32::new(0);

    struct Temp(PathBuf);
    impl Temp {
        fn nova(plano: &str) -> Temp {
            let d = std::env::temp_dir().join(format!("bp-pacote-{}-{}", std::process::id(), SEQ.fetch_add(1, Ordering::Relaxed)));
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("plano.json"), plano).unwrap();
            Temp(d)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn com_figura(f: &str) -> String {
        serde_json::json!({"formato":"blueprint.plano/1","blueprint":"B","projetos":[{"acoes":[{"id":"1","nome":"A","detalhes":[{"id":"01","figura":f}]}]}]}).to_string()
    }
    fn tem(r: Result<Pacote, Vec<String>>, trecho: &str) {
        let erros = r.expect_err("devia recusar");
        assert!(erros.iter().any(|e| e.contains(trecho)), "{erros:?}");
    }

    #[cfg(unix)]
    #[test]
    fn pasta_com_link_simbolico_recusada() {
        // Link no meio do caminho, link na própria figura e no próprio plano.json: nenhum é seguido.
        let fora = Temp::nova("{}");
        std::fs::write(fora.0.join("a.svg"), "<svg viewBox=\"0 0 1 1\"/>").unwrap();
        let t = Temp::nova(&com_figura("figs/a.svg"));
        std::os::unix::fs::symlink(&fora.0, t.0.join("figs")).unwrap();
        tem(ler_pasta(&t.0), "symbolic");
        let t = Temp::nova(&com_figura("a.svg"));
        std::os::unix::fs::symlink(fora.0.join("a.svg"), t.0.join("a.svg")).unwrap();
        tem(ler_pasta(&t.0), "symbolic");
        let t = Temp::nova("{}");
        std::fs::remove_file(t.0.join("plano.json")).unwrap();
        std::os::unix::fs::symlink(fora.0.join("plano.json"), t.0.join("plano.json")).unwrap();
        tem(ler_pasta(&t.0), "symbolic");
        let t = Temp::nova(&serde_json::json!({"estilo":"e.css"}).to_string());
        std::fs::write(fora.0.join("e.css"), "").unwrap();
        std::os::unix::fs::symlink(fora.0.join("e.css"), t.0.join("e.css")).unwrap();
        tem(ler_pasta(&t.0), "symbolic");
    }

    #[test]
    fn figura_fora_da_pasta_recusada() {
        for caminho in ["../fora.svg", "figs/../../fora.svg", "..\\fora.svg", "/etc/passwd.svg", "\\fora.svg", "C:\\fora.svg", "c:/fora.svg", "a.svg:fluxo"] {
            let t = Temp::nova(&com_figura(caminho));
            tem(ler_pasta(&t.0), "plan folder");
        }
        let t = Temp::nova(&serde_json::json!({"estilo":"../fora.css"}).to_string());
        tem(ler_pasta(&t.0), "plan folder");
        let t = Temp::nova(&com_figura("nao-existe.svg"));
        tem(ler_pasta(&t.0), "not found");
    }

    #[test]
    fn jpg_falso_recusado() {
        let t = Temp::nova(&com_figura("a.JPG"));
        std::fs::write(t.0.join("a.JPG"), b"<svg onload='x'/>").unwrap();
        tem(ler_pasta(&t.0), "not a jpg");
        // PNG de verdade com nome de JPG também não: extensão e conteúdo têm de concordar.
        let t = Temp::nova(&com_figura("a.jpeg"));
        std::fs::write(t.0.join("a.jpeg"), b"\x89PNG\r\n\x1a\nresto").unwrap();
        tem(ler_pasta(&t.0), "not a jpg");
        let t = Temp::nova(&com_figura("a.exe"));
        std::fs::write(t.0.join("a.exe"), b"MZ").unwrap();
        tem(ler_pasta(&t.0), "use SVG");
        let t = Temp::nova(&com_figura("a.svg"));
        std::fs::write(t.0.join("a.svg"), "<svg><script>x</script></svg>").unwrap();
        tem(ler_pasta(&t.0), "<script> is not allowed");
        let t = Temp::nova(&com_figura("a.png"));
        std::fs::write(t.0.join("a.png"), vec![0u8; LIMITE_IMAGEM + 1]).unwrap();
        tem(ler_pasta(&t.0), "larger than");
        let t = Temp::nova(&serde_json::json!({"estilo":"e.txt"}).to_string());
        std::fs::write(t.0.join("e.txt"), "").unwrap();
        tem(ler_pasta(&t.0), ".css");
    }

    #[test]
    fn exemplo_reforma_le() {
        let pasta = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures/reforma");
        let p = ler_pasta(&pasta).unwrap();
        assert_eq!(p.plano["blueprint"], "Reforma do escritório");
        assert_eq!(p.figuras.len(), 1);
        assert_eq!(p.figuras[0].0, "figuras/planta.svg");
        assert!(p.estilo.is_none() && p.avisos.is_empty(), "{:?}", p.avisos);
        assert_eq!(medidas_svg(std::str::from_utf8(&p.figuras[0].1).unwrap()), Some((900, 460)));
    }

    #[test]
    fn os_tres_exemplos_leem_e_estilo_vem_junto() {
        for nome in ["biblioteca", "monitor", "reforma"] {
            ler_pasta(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/plano/fixtures").join(nome)).unwrap();
        }
        let t = Temp::nova(&serde_json::json!({"estilo":"./e.CSS"}).to_string());
        std::fs::write(t.0.join("e.CSS"), ".a{fill:red}").unwrap();
        assert_eq!(ler_pasta(&t.0).unwrap().estilo.as_deref(), Some(".a{fill:red}"));
        std::fs::write(t.0.join("e.CSS"), ".a{fill:url(http://x/y)}").unwrap();
        tem(ler_pasta(&t.0), "url()");
    }
}
