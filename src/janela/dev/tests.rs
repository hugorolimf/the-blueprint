use super::*;
use std::fs::{self, File};
use std::time::Duration;

fn pasta(nome: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("blueprint-vigia-{nome}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(p.join("web/css")).unwrap();
    fs::create_dir_all(p.join("web/js")).unwrap();
    fs::write(p.join("blueprint.html"), "<html></html>").unwrap();
    fs::write(p.join("web/css/a.css"), "a{}").unwrap();
    fs::write(p.join("web/js/b.js"), "1").unwrap();
    p
}

/// Avança a data do arquivo em 5 s: não depende do relógio nem da pressa do teste.
fn tocar(caminho: &Path) {
    let f = File::options().write(true).open(caminho).unwrap();
    f.set_modified(f.metadata().unwrap().modified().unwrap() + Duration::from_secs(5)).unwrap();
}

#[test]
fn sem_mudanca_nao_avisa() {
    let p = pasta("quieto");
    let mut v = Vigia::novo(p.clone());
    assert_eq!(v.mudou(), None);
    assert_eq!(v.mudou(), None);
    let _ = fs::remove_dir_all(p);
}

#[test]
fn so_css_troca_o_estilo_e_o_resto_recarrega() {
    let p = pasta("tipos");
    let mut v = Vigia::novo(p.clone());
    tocar(&p.join("web/css/a.css"));
    assert_eq!(v.mudou(), Some(true));
    assert_eq!(v.mudou(), None, "a mudança já foi avisada");
    tocar(&p.join("web/js/b.js"));
    assert_eq!(v.mudou(), Some(false));
    tocar(&p.join("web/css/a.css"));
    tocar(&p.join("blueprint.html"));
    assert_eq!(v.mudou(), Some(false), "CSS junto com outro arquivo recarrega a página");
    let _ = fs::remove_dir_all(p);
}

#[test]
fn arquivo_novo_e_apagado_recarregam_e_outras_extensoes_sao_ignoradas() {
    let p = pasta("novos");
    let mut v = Vigia::novo(p.clone());
    fs::write(p.join("web/js/notas.txt"), "x").unwrap();
    assert_eq!(v.mudou(), None);
    fs::write(p.join("web/js/c.js"), "2").unwrap();
    assert_eq!(v.mudou(), Some(false));
    fs::remove_file(p.join("web/js/c.js")).unwrap();
    assert_eq!(v.mudou(), Some(false));
    let _ = fs::remove_dir_all(p);
}
