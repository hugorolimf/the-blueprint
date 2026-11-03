//! Testes de `acao_mapa_forma.rs`: a disposição das lajes.
use super::*;

fn plano(r: &str, depende: &[&str]) -> Plano {
    Plano { id: format!("id-{r}"), referencia: r.into(), depende_de: depende.iter().map(|d| d.to_string()).collect(), ..Plano::default() }
}

fn lugares(planos: &[Plano], cabem: usize) -> Vec<(usize, f32)> {
    let q: Vec<&Plano> = planos.iter().collect();
    dispor(&q, cabem).iter().map(|l| (l.col, l.lin)).collect()
}

#[test]
fn o_losango_abre_em_duas_linhas_e_centraliza_as_pontas() {
    let p = [plano("1.1", &[]), plano("1.2", &["1.1"]), plano("1.3", &["1.1"]), plano("1.4", &["1.2", "1.3"])];
    assert_eq!(lugares(&p, 6), [(0, 0.5), (1, 0.0), (1, 1.0), (2, 0.5)]);
}

#[test]
fn sem_dependencia_os_planos_correm_em_fila_e_quebram() {
    let p = [plano("1", &[]), plano("2", &[]), plano("3", &[])];
    assert_eq!(lugares(&p, 2), [(0, 0.0), (1, 0.0), (0, 1.0)]);
}

#[test]
fn a_corrente_longa_quebra_sem_cair_em_cima_de_outra_laje() {
    let p = [plano("1", &[]), plano("2", &["1"]), plano("3", &["2"]), plano("4", &["3"])];
    let l = lugares(&p, 2);
    assert_eq!(l, [(0, 0.0), (1, 0.0), (0, 1.0), (1, 1.0)]);
}

#[test]
fn dependencia_de_fora_da_acao_nao_conta() {
    let p = [plano("1", &["9.9"]), plano("2", &["9.9"])];
    assert_eq!(lugares(&p, 4), [(0, 0.0), (1, 0.0)]);
}

#[test]
fn a_dependencia_tambem_vale_pelo_id() {
    let p = [plano("1", &[]), plano("2", &["id-1"])];
    assert_eq!(lugares(&p, 4), [(0, 0.0), (1, 0.0)]);
    let q: Vec<&Plano> = p.iter().collect();
    assert_eq!(niveis(&q), [0, 1]);
}

#[test]
fn um_ciclo_nao_trava_e_nao_sai_do_tamanho() {
    let p = [plano("1", &["2"]), plano("2", &["1"])];
    let q: Vec<&Plano> = p.iter().collect();
    assert!(niveis(&q).iter().all(|n| *n < p.len()));
    assert_eq!(dispor(&q, 4).len(), 2);
}
