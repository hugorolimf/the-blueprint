//! Os sólidos do Anatomy, os mesmos do kit (`extrude`, `slabOf`, `cylinder`, `solidOf`, `sphereOf`), agora como malhas: extrusão de um anel, caixa, laje de cantos redondos, cilindro, sólido de revolução, esfera e viga entre dois pontos.
//! E os de pé, que o kit não tinha: a chapa (anéis empurrados na horizontal, como a frente de um cadeado ou o buraco da chave) e a alça em U, varrida como um tubo só.

use super::malha::Malha;
use super::vetor::{cruz, soma, sub, unit, vezes, V3};
use std::f32::consts::{PI, TAU};

pub const PASSOS_CIRCULO: usize = 28;
const PASSOS_CANTO: usize = 5;
/// Passos da meia volta da alça: 15 graus cada, abaixo do limite em que a aresta vira vinco, então o arco fica liso.
const PASSOS_ALCA: usize = 12;
/// Os lados do tubo da alça: o octógono tem face para fora, para dentro, para a frente e para trás.
const LADOS_ALCA: usize = 8;

/// O que há na ponta de um perfil de revolução: nada (fecha no eixo) ou uma tampa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ponta {
    Tampa,
    /// um tubo: a API do motor tem, mesmo que a planta ainda não use
    #[allow(dead_code)]
    Aberta,
}

type Faces = (Vec<V3>, Vec<Vec<u32>>);

fn area_com_sinal(anel: &[(f32, f32)]) -> f32 {
    let n = anel.len();
    (0..n).map(|i| anel[i].0 * anel[(i + 1) % n].1 - anel[(i + 1) % n].0 * anel[i].1).sum::<f32>() * 0.5
}

/// Empurra o anel `(a, b)`, posto em `origem + a·u + b·w`, por `passo`, e junta as faces em `f`. `u × w` aponta para o lado do `passo`: o anel, virado no sentido anti-horário, olha para lá.
fn empurrar((v, f): &mut Faces, anel: &[(f32, f32)], origem: V3, u: V3, w: V3, passo: V3) {
    let mut plano: Vec<(f32, f32)> = anel.to_vec();
    if area_com_sinal(&plano) < 0.0 {
        plano.reverse();
    }
    let (i0, n) = (v.len() as u32, plano.len() as u32);
    let atras: Vec<V3> = plano.iter().map(|(a, b)| soma(soma(origem, vezes(u, *a)), vezes(w, *b))).collect();
    let frente: Vec<V3> = atras.iter().map(|p| soma(*p, passo)).collect();
    v.extend(atras);
    v.extend(frente);
    f.push((i0 + n..i0 + n * 2).collect());
    f.push((i0..i0 + n).rev().collect());
    for i in 0..n {
        let j = (i + 1) % n;
        f.push(vec![i0 + i, i0 + j, i0 + j + n, i0 + i + n]);
    }
}

/// As faces da chapa: cada anel `(u, v)` de pé no plano vertical que passa por `base`, empurrado `espessura` para `fora`.
fn chapa_faces(base: V3, fora: (f32, f32), aneis: &[&[(f32, f32)]], espessura: f32) -> Faces {
    let n = unit([fora.0, fora.1, 0.0]);
    let u = [-n[1], n[0], 0.0];
    let mut faces: Faces = (Vec::new(), Vec::new());
    for anel in aneis {
        empurrar(&mut faces, anel, base, u, [0.0, 0.0, 1.0], vezes(n, espessura));
    }
    faces
}

/// O eixo do U, de uma perna à outra: sobe `perna` de `base - eixo·raio`, faz meia volta por cima e desce até `base + eixo·raio`.
fn caminho_da_alca(base: V3, eixo: V3, raio: f32, perna: f32) -> Vec<V3> {
    let ponto = |s: f32, h: f32| soma(soma(base, vezes(eixo, s)), [0.0, 0.0, h]);
    let com_perna = perna > 1e-4;
    let mut v = Vec::with_capacity(PASSOS_ALCA + 3);
    if com_perna {
        v.push(ponto(-raio, 0.0));
    }
    for k in 0..=PASSOS_ALCA {
        let t = PI * (1.0 - k as f32 / PASSOS_ALCA as f32);
        v.push(ponto(raio * t.cos(), perna.max(0.0) + raio * t.sin()));
    }
    if com_perna {
        v.push(ponto(raio, 0.0));
    }
    v
}

/// Varre o octógono de meia largura `meia` pelo `caminho`, deitado no plano de normal `plano`. Em cada dobra o anel fica na bissetriz, então as faces de um trecho e do outro se encontram sem fresta.
fn varrer(caminho: &[V3], plano: V3, meia: f32) -> Faces {
    let (m, ultimo) = (LADOS_ALCA as u32, caminho.len() - 1);
    let r = meia / (PI / LADOS_ALCA as f32).cos();
    let mut vertices = Vec::with_capacity(caminho.len() * LADOS_ALCA);
    for (k, p) in caminho.iter().enumerate() {
        let antes = unit(sub(*p, caminho[k.saturating_sub(1)]));
        let depois = unit(sub(caminho[(k + 1).min(ultimo)], *p));
        let frente = unit(soma(antes, depois));
        // `fora × plano = frente`: o anel gira no sentido anti-horário de quem olha para onde o tubo vai
        let fora = cruz(plano, frente);
        for j in 0..LADOS_ALCA {
            let a = (j as f32 + 0.5) / LADOS_ALCA as f32 * TAU;
            vertices.push(soma(*p, soma(vezes(fora, r * a.cos()), vezes(plano, r * a.sin()))));
        }
    }
    let n = caminho.len() as u32;
    let mut faces = Vec::with_capacity((n as usize - 1) * LADOS_ALCA + 2);
    for k in 0..n - 1 {
        for j in 0..m {
            let (a, b) = (k * m + j, k * m + (j + 1) % m);
            faces.push(vec![a, b, b + m, a + m]);
        }
    }
    faces.push((0..m).rev().collect());
    faces.push(((n - 1) * m..n * m).collect());
    (vertices, faces)
}

fn alca_faces(base: V3, eixo: (f32, f32), raio: f32, perna: f32, grossura: f32) -> Faces {
    let e = unit([eixo.0, eixo.1, 0.0]);
    varrer(&caminho_da_alca(base, e, raio, perna), cruz(e, [0.0, 0.0, 1.0]), grossura * 0.5)
}

impl Malha {
    /// O anel do plano sobe de `z` até `z + altura`.
    pub fn extrusao(anel: &[(f32, f32)], z: f32, altura: f32) -> Malha {
        let mut faces: Faces = (Vec::with_capacity(anel.len() * 2), Vec::with_capacity(anel.len() + 2));
        empurrar(&mut faces, anel, [0.0, 0.0, z], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, altura]);
        Malha::de_faces(faces.0, faces.1)
    }

    /// A chapa: anéis convexos `(u, v)` de pé no plano vertical que passa por `base` e olha para `fora` (na horizontal), empurrados `espessura` para fora.
    /// `u` anda para a direita de quem olha a chapa de frente e `v` sobe a partir de `base`. Vários anéis numa malha só pintam juntos: o buraco da chave é um círculo e uma fenda.
    #[allow(dead_code)] // API do motor: o cadeado saiu do chão da planta, mas a chapa e o tubo varrido continuam valendo
    pub fn chapa(base: V3, fora: (f32, f32), aneis: &[&[(f32, f32)]], espessura: f32) -> Malha {
        let (vertices, faces) = chapa_faces(base, fora, aneis, espessura);
        Malha::de_faces(vertices, faces)
    }

    /// A alça de um cadeado: um U de pé no plano vertical de `eixo` (na horizontal), com as pernas a `raio` de `base` (o meio entre elas) subindo `perna`, fechado em cima por meia volta de `raio`.
    /// Uma malha só, varrida com um octógono de `grossura` e tampada no pé das pernas; com o arco em passos curtos, as arestas dele ficam lisas.
    #[allow(dead_code)] // API do motor: o cadeado saiu do chão da planta, mas a chapa e o tubo varrido continuam valendo
    pub fn alca(base: V3, eixo: (f32, f32), raio: f32, perna: f32, grossura: f32) -> Malha {
        let (vertices, faces) = alca_faces(base, eixo, raio, perna, grossura);
        Malha::de_faces(vertices, faces)
    }

    /// A caixa com o canto em `pos` e o tamanho `tam`.
    pub fn caixa(pos: V3, tam: V3) -> Malha {
        let [x, y, z] = pos;
        let [w, d, h] = tam;
        Malha::extrusao(&[(x, y), (x + w, y), (x + w, y + d), (x, y + d)], z, h)
    }

    /// A laje de cantos redondos (`roundedPlan` do kit).
    pub fn laje(pos: V3, tam: V3, raio: f32) -> Malha {
        let [x, y, z] = pos;
        let [w, d, h] = tam;
        let r = raio.min(w / 2.0).min(d / 2.0).max(0.0);
        let cantos = [(x + w - r, y + r, -90.0_f32), (x + w - r, y + d - r, 0.0), (x + r, y + d - r, 90.0), (x + r, y + r, 180.0)];
        let mut anel = Vec::with_capacity(4 * (PASSOS_CANTO + 1));
        for (cx, cy, inicio) in cantos {
            for p in 0..=PASSOS_CANTO {
                let a = (inicio + 90.0 * p as f32 / PASSOS_CANTO as f32).to_radians();
                anel.push((cx + r * a.cos(), cy + r * a.sin()));
            }
        }
        Malha::extrusao(&anel, z, h)
    }

    pub fn cilindro(centro: (f32, f32), raio: f32, z: f32, altura: f32) -> Malha {
        let anel: Vec<(f32, f32)> = (0..PASSOS_CIRCULO).map(|i| {
            let a = i as f32 / PASSOS_CIRCULO as f32 * TAU;
            (centro.0 + raio * a.cos(), centro.1 + raio * a.sin())
        }).collect();
        Malha::extrusao(&anel, z, altura)
    }

    /// O perfil `[(altura, raio)]` gira em volta do eixo vertical em `centro` (`solidOf` do kit): sino, copo, garrafa, flange.
    pub fn torno(centro: (f32, f32), z: f32, perfil: &[(f32, f32)], ponta: Ponta) -> Malha {
        let m = PASSOS_CIRCULO as u32;
        let mut vertices = Vec::with_capacity(perfil.len() * PASSOS_CIRCULO);
        for (h, r) in perfil {
            for i in 0..PASSOS_CIRCULO {
                let a = i as f32 / PASSOS_CIRCULO as f32 * TAU;
                vertices.push([centro.0 + r * a.cos(), centro.1 + r * a.sin(), z + h]);
            }
        }
        let mut faces = Vec::new();
        for l in 0..perfil.len() as u32 - 1 {
            for i in 0..m {
                let j = (i + 1) % m;
                let (a, b) = (l * m + i, l * m + j);
                faces.push(vec![a, b, b + m, a + m]);
            }
        }
        if ponta == Ponta::Tampa {
            faces.push((0..m).rev().collect());
            let topo = (perfil.len() as u32 - 1) * m;
            faces.push((topo..topo + m).collect());
        }
        Malha::de_faces(vertices, faces)
    }

    /// A esfera (`sphereOf` do kit), como um sólido de revolução fechado.
    pub fn esfera(centro: V3, raio: f32) -> Malha {
        let perfil: Vec<(f32, f32)> = (0..=10).map(|i| {
            let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 10.0;
            (raio * a.sin(), (raio * a.cos()).max(raio * 0.02))
        }).collect();
        Malha::torno((centro[0], centro[1]), centro[2], &perfil, Ponta::Tampa)
    }

    /// Uma viga de `a` até `b`, deitada: a largura fica no plano, a altura para cima. É o duto que liga uma construção à outra.
    pub fn viga(a: V3, b: V3, largura: f32, altura: f32) -> Malha {
        let lado = vezes(unit(cruz(sub(b, a), [0.0, 0.0, 1.0])), largura * 0.5);
        let cima = [0.0, 0.0, altura];
        let cantos = |p: V3| [sub(p, lado), soma(p, lado), soma(soma(p, lado), cima), soma(sub(p, lado), cima)];
        let (ca, cb) = (cantos(a), cantos(b));
        let vertices = vec![ca[0], ca[1], ca[2], ca[3], cb[0], cb[1], cb[2], cb[3]];
        let faces = vec![vec![0, 1, 2, 3], vec![4, 7, 6, 5], vec![0, 3, 7, 4], vec![1, 5, 6, 2], vec![2, 6, 7, 3], vec![0, 4, 5, 1]];
        Malha::de_faces(vertices, faces)
    }
}

#[cfg(test)]
mod tests {
    use super::super::camera::{Camera, AZIMUTE, ELEVACAO};
    use super::super::vetor::dot;
    use super::*;
    use egui::pos2;

    /// O volume com sinal (teorema da divergência): positivo e do tamanho certo só se toda face olha para fora e a malha fecha.
    fn volume((v, f): &Faces) -> f32 {
        let tri = |a: u32, b: u32, c: u32| dot(v[a as usize], cruz(v[b as usize], v[c as usize]));
        f.iter().map(|anel| (1..anel.len() - 1).map(|k| tri(anel[0], anel[k], anel[k + 1])).sum::<f32>()).sum::<f32>() / 6.0
    }

    const QUADRADO: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

    #[test]
    fn os_solidos_fechados_olham_para_fora() {
        let c = Camera::nova(AZIMUTE, ELEVACAO, 10.0, [0.0; 3], pos2(0.0, 0.0), 0.0);
        let chapa = Malha::chapa([0.0; 3], (0.0, 1.0), &[&QUADRADO[..]], 0.5);
        for m in [Malha::caixa([0.0; 3], [1.0, 2.0, 3.0]), Malha::cilindro((0.0, 0.0), 1.0, 0.0, 1.0), Malha::viga([0.0; 3], [3.0, 0.0, 1.0], 1.0, 0.5), Malha::esfera([0.0; 3], 1.0), chapa] {
            assert!(m.contem(&c, c.ponto(m.centro())), "o centro aparece na frente de alguma face");
        }
    }

    #[test]
    fn o_anel_ao_contrario_da_o_mesmo_solido() {
        let (a, b) = (Malha::extrusao(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)], 0.0, 1.0), Malha::extrusao(&[(1.0, 1.0), (1.0, 0.0), (0.0, 0.0)], 0.0, 1.0));
        assert_eq!(a.centro(), b.centro());
    }

    #[test]
    fn a_chapa_sai_para_onde_olha_e_fecha_para_fora() {
        let m = Malha::chapa([0.0; 3], (1.0, 0.0), &[&QUADRADO[..]], 0.5);
        let c = m.centro();
        assert!((c[0] - 0.25).abs() < 1e-5 && (c[1] - 0.5).abs() < 1e-5 && (c[2] - 0.5).abs() < 1e-5, "{c:?}");
        let anel_invertido: Vec<(f32, f32)> = QUADRADO.iter().rev().copied().collect();
        for fora in [(1.0, 0.0), (0.0, -1.0)] {
            let v = volume(&chapa_faces([0.0; 3], fora, &[&QUADRADO[..], &anel_invertido[..]], 0.5));
            assert!((v - 1.0).abs() < 1e-4, "dois quadrados de meio de espessura: {v}");
        }
    }

    #[test]
    fn a_alca_e_um_tubo_fechado_olhando_para_fora() {
        let (raio, perna, grossura) = (0.2, 0.3, 0.08);
        for eixo in [(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0)] {
            let f = alca_faces([1.0, 2.0, 0.5], eixo, raio, perna, grossura);
            let meia = grossura * 0.5;
            let secao = 8.0 * meia * meia * (PI / 8.0).tan();
            let esperado = secao * (2.0 * perna + PI * raio);
            let v = volume(&f);
            assert!(v > 0.0 && (v - esperado).abs() < esperado * 0.03, "volume {v}, esperado {esperado}");
            let (baixo, alto) = f.0.iter().fold((f32::MAX, f32::MIN), |(b, a), p| (b.min(p[2]), a.max(p[2])));
            assert!((baixo - 0.5).abs() < 1e-4, "as pernas nascem na base: {baixo}");
            assert!((alto - (0.5 + perna + raio + meia)).abs() < 1e-3, "o topo do arco: {alto}");
        }
    }

    #[test]
    fn a_alca_sem_perna_e_so_o_arco() {
        let f = alca_faces([0.0; 3], (1.0, 0.0), 0.2, 0.0, 0.06);
        assert_eq!(f.0.len(), (PASSOS_ALCA + 1) * LADOS_ALCA);
        assert!(volume(&f) > 0.0);
    }
}
