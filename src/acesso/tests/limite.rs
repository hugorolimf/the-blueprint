use crate::acesso::limite::*;

#[test]
fn dez_falhas_do_mesmo_ip_bloqueiam_ate_a_janela_passar() {
    let mut l = Limite::default();
    for i in 0..POR_IP as i64 {
        assert_eq!(l.bloqueado("1.2.3.4", i), None);
        l.falhou("1.2.3.4", i);
    }
    let espera = l.bloqueado("1.2.3.4", 1000).expect("bloqueado");
    assert!(espera > 800 && espera <= 900, "{espera}");
    assert_eq!(l.bloqueado("5.6.7.8", 1000), None, "outro IP segue");
    assert_eq!(l.bloqueado("1.2.3.4", JANELA_MS + 10), None, "a janela passou");
}

#[test]
fn duzentas_falhas_no_total_bloqueiam_todos() {
    let mut l = Limite::default();
    for i in 0..TOTAL {
        l.falhou(&format!("10.0.{}.{}", i / 250, i % 250), 0);
    }
    assert!(l.bloqueado("9.9.9.9", 1).is_some());
}
