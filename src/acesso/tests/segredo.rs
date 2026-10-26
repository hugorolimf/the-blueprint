use crate::acesso::segredo::*;

#[test]
fn base64url_sem_preenchimento() {
    assert_eq!(base64url(b""), "");
    assert_eq!(base64url(b"f"), "Zg");
    assert_eq!(base64url(b"fo"), "Zm8");
    assert_eq!(base64url(b"foo"), "Zm9v");
    assert_eq!(base64url(&[0xfb, 0xff]), "-_8");
}

#[test]
fn segredo_tem_256_bits_e_nunca_se_repete() {
    let (a, b) = (gerar(), gerar());
    assert_eq!(a.len(), 43);
    assert!(forma_valida(&a) && forma_valida(&b));
    assert_ne!(a, b);
    assert!(!forma_valida("curto") && !forma_valida(&format!("{}=", &a[..42])));
    assert_eq!(id().len(), 16);
    assert_ne!(id(), id());
}

#[test]
fn hash_e_sha256_em_hexadecimal() {
    assert_eq!(hash("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}
