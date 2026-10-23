use super::{pasta, sha256};
use crate::atualizador::hash::{confere, procurar, sha256_do_arquivo};

const ABC: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[test]
fn sha256_de_um_arquivo_bate_com_o_valor_conhecido() {
    let p = pasta("hash");
    std::fs::create_dir_all(&p).unwrap();
    std::fs::write(p.join("abc"), "abc").unwrap();
    assert_eq!(sha256_do_arquivo(&p.join("abc")).unwrap(), ABC);
    assert_eq!(sha256(b"abc"), ABC);
    assert!(confere(&p.join("abc"), &ABC.to_uppercase()), "maiúsculas valem");
    assert!(!confere(&p.join("abc"), &sha256(b"abd")));
    assert!(!confere(&p.join("nao-existe"), ABC));
}

#[test]
fn procura_a_linha_do_arquivo_no_sha256sums() {
    let sums = format!("{ABC}  Blueprint-0.2.0-instalador.exe\n{}  outro.zip\n{ABC} *com-asterisco.exe\n", sha256(b"x"));
    assert_eq!(procurar(&sums, "Blueprint-0.2.0-instalador.exe").as_deref(), Some(ABC));
    assert_eq!(procurar(&sums, "com-asterisco.exe").as_deref(), Some(ABC), "o formato binário do sha256sum");
    assert_eq!(procurar(&sums, "falta.exe"), None);
    assert_eq!(procurar("curto  Blueprint-0.2.0-instalador.exe", "Blueprint-0.2.0-instalador.exe"), None, "não parece um SHA-256");
}
