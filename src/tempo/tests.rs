use super::*;

#[test]
fn iso_igual_ao_to_iso_string_do_js() {
    assert_eq!(iso_utc(0), "1970-01-01T00:00:00.000Z");
    assert_eq!(iso_utc(1_000_000_000_123), "2001-09-09T01:46:40.123Z");
    assert_eq!(iso_utc(1_782_094_715_000), "2026-06-22T02:18:35.000Z");
    assert_eq!(iso_utc(951_782_400_000), "2000-02-29T00:00:00.000Z");
    assert_eq!(iso_utc(-1), "1969-12-31T23:59:59.999Z");
}

#[test]
fn o_formato_de_agora_e_o_de_hoje() {
    let agora = iso_utc_agora();
    assert_eq!(agora.len(), 24);
    assert!(agora.ends_with('Z') && agora.as_bytes()[10] == b'T');
    let hoje = hoje_local();
    assert_eq!(hoje.len(), 10);
    assert!(crate::blueprint::dia_valido(&hoje), "{hoje}");
}
