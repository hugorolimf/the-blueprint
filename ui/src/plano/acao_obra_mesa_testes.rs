//! Testes de `acao_obra_mesa.rs`: o arranjo das colunas, que `altura` e `desenhar` usam juntos.
use super::*;

fn todas(k: &[Coluna]) -> Vec<Parte> {
    let mut v: Vec<Parte> = k.iter().flat_map(|c| c.partes.iter().copied()).collect();
    v.sort_by_key(|p| *p as u8);
    v
}

#[test]
fn embaixo_do_palco_e_uma_coluna_so() {
    assert_eq!(colunas(880.0, false), None);
    assert_eq!(arranjo(true, 880.0, false).len(), 1);
}

#[test]
fn estreita_demais_nao_divide() {
    assert_eq!(colunas(LARGURA_DUAS_COLUNAS - 1.0, true), None);
}

#[test]
fn as_duas_colunas_somam_a_largura() {
    let (e, d) = colunas(900.0, true).expect("duas colunas");
    assert!((e + d + VAO_COLUNAS - 900.0).abs() < 0.01);
    assert!(e > 250.0 && d > 250.0);
}

#[test]
fn da_acao_fica_na_direita() {
    let k = arranjo(true, 900.0, true);
    assert_eq!(k[1].partes, vec![Parte::DaAcao]);
    assert_eq!(k[0].partes.first(), Some(&Parte::Cabeca));
}

#[test]
fn sem_da_acao_os_agentes_vao_para_a_direita() {
    let k = arranjo(false, 900.0, true);
    assert_eq!(k[1].partes, vec![Parte::DaAcao, Parte::Agentes, Parte::Soltos]);
}

#[test]
fn toda_parte_aparece_uma_vez() {
    let uma = todas(&arranjo(true, 500.0, true));
    assert_eq!(uma.len(), 6);
    for (tem, w, larga) in [(true, 900.0, true), (false, 900.0, true), (true, 900.0, false)] {
        assert_eq!(todas(&arranjo(tem, w, larga)), uma);
    }
}
