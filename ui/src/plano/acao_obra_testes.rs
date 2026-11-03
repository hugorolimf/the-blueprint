//! Testes de `acao_obra_figura.rs`: nenhum módulo cai em cima de outro e quem depende fica um degrau acima.
use super::*;

fn plano(r: &str, depende: &[&str]) -> Plano {
    Plano { id: format!("id-{r}"), referencia: r.into(), depende_de: depende.iter().map(|d| d.to_string()).collect(), ..Plano::default() }
}

fn sem_sobrepor(o: &Obra) {
    for (i, a) in o.modulos.iter().enumerate() {
        for b in &o.modulos[i + 1..] {
            assert!((a.x - b.x).abs() >= LADO || (a.y - b.y).abs() >= LADO, "{a:?} em cima de {b:?}");
        }
    }
}

#[test]
fn o_losango_nao_sobrepoe_e_quem_depende_fica_mais_alto() {
    let p = [plano("1.1", &[]), plano("1.2", &["1.1"]), plano("1.3", &["1.1"]), plano("1.4", &["1.2", "1.3"])];
    let q: Vec<&Plano> = p.iter().collect();
    let o = Obra::de(&q);
    sem_sobrepor(&o);
    assert_eq!(o.niveis, 3);
    assert!(o.modulos[0].base < o.modulos[1].base && o.modulos[1].base < o.modulos[3].base);
    assert_eq!(o.modulos[1].base, o.modulos[2].base);
}

#[test]
fn oito_planos_soltos_nao_se_encostam() {
    let p: Vec<Plano> = (0..8).map(|i| plano(&i.to_string(), &[])).collect();
    let q: Vec<&Plano> = p.iter().collect();
    sem_sobrepor(&Obra::de(&q));
}

#[test]
fn um_plano_so_tambem_tem_obra() {
    let p = [plano("1", &[])];
    let q: Vec<&Plano> = p.iter().collect();
    let o = Obra::de(&q);
    assert_eq!((o.modulos.len(), o.niveis), (1, 1));
}

#[test]
fn a_camera_de_fora_enquadra_a_obra_inteira() {
    let p = [plano("1.1", &[]), plano("1.2", &["1.1"]), plano("1.3", &["1.1"]), plano("1.4", &["1.2", "1.3"])];
    let q: Vec<&Plano> = p.iter().collect();
    let o = Obra::de(&q);
    let r = Rect::from_min_size(Pos2::ZERO, vec2(900.0, 500.0));
    let c = Camera::enquadrar(&o.pontos(), r, 20.0, AZIMUTE, ELEVACAO);
    for ponto in o.pontos() {
        assert!(r.contains(c.ponto(ponto)));
    }
}
