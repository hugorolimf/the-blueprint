//! A malha: vértices, faces e arestas. Toda a assinatura do Anatomy sai daqui, a cada quadro e de qualquer ângulo:
//! aresta com uma face à vista e outra de costas é **contorno**; com as duas à vista e um ângulo vivo entre elas é **vinco**; e cada face à vista ganha o tom da sua faixa de luz.
//! O kit chega no mesmo resultado separando o anel em trechos (`runsOf`); ter as faces ligadas pelas arestas vale para qualquer sólido e qualquer câmera.

use super::camera::Camera;
use super::tinta::Tinta;
use super::vetor::{cruz, dot, mistura, sub, unit, V3};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Painter, Pos2, Rect, Shape, Stroke, TextureId};

/// Acima deste cosseno entre as normais, a aresta é lisa (a lateral de um cilindro), e não um vinco.
const LISA: f32 = 0.93;
const TRACO_VISIVEL: (f32, f32) = (3.5, 3.0);

#[derive(Clone, Debug)]
struct Face {
    indices: Vec<u32>,
    normal: V3,
    centro: V3,
}

#[derive(Clone, Copy, Debug)]
struct Aresta {
    a: u32,
    b: u32,
    faces: (u32, u32),
    viva: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Malha {
    vertices: Vec<V3>,
    faces: Vec<Face>,
    arestas: Vec<Aresta>,
}

impl Malha {
    /// Liga as faces pelas arestas. Cada face vem como anel de índices, olhando para fora (a normal é a regra da mão direita).
    pub fn de_faces(vertices: Vec<V3>, aneis: Vec<Vec<u32>>) -> Malha {
        let faces: Vec<Face> = aneis.into_iter().map(|indices| {
            let (a, b, c) = (vertices[indices[0] as usize], vertices[indices[1] as usize], vertices[indices[(indices.len() + 1) / 2] as usize]);
            let normal = unit(cruz(sub(b, a), sub(c, a)));
            let soma = indices.iter().fold([0.0; 3], |s: V3, i| [s[0] + vertices[*i as usize][0], s[1] + vertices[*i as usize][1], s[2] + vertices[*i as usize][2]]);
            let n = indices.len() as f32;
            Face { indices, normal, centro: [soma[0] / n, soma[1] / n, soma[2] / n] }
        }).collect();
        let pontas = pontas_em_ordem(&faces, vertices.len());
        let mut arestas = Vec::with_capacity(pontas.len() / 2);
        let mut i = 0;
        while i < pontas.len() {
            let (a, b, f0) = pontas[i];
            let par = pontas.get(i + 1).filter(|p| p.0 == a && p.1 == b).map(|p| p.2);
            let f1 = par.unwrap_or(f0);
            let viva = dot(faces[f0 as usize].normal, faces[f1 as usize].normal) < LISA || par.is_none();
            arestas.push(Aresta { a, b, faces: (f0, f1), viva });
            i += if par.is_some() { 2 } else { 1 };
        }
        Malha { vertices, faces, arestas }
    }

    /// O meio da malha, para ordenar na cena.
    pub fn centro(&self) -> V3 {
        let n = self.vertices.len().max(1) as f32;
        let s = self.vertices.iter().fold([0.0; 3], |s: V3, v| [s[0] + v[0], s[1] + v[1], s[2] + v[2]]);
        [s[0] / n, s[1] / n, s[2] / n]
    }

    fn face_a_vista(&self, c: &Camera, f: &Face) -> bool {
        dot(f.normal, c.para_o_olho(f.centro)) > 1e-4
    }

    fn inteira(&self, c: &Camera) -> bool {
        self.vertices.iter().all(|v| c.alcanca(*v))
    }

    /// As formas do sólido, na ordem de pintar, entregues a `por` (a `Cena` desenha ou guarda). O que já passou do olho não aparece, e o sólido
    /// todo fora de `recorte` (o recorte do pincel; `Rect::EVERYTHING` para guardar) não entrega nada: ali nada apareceria.
    pub fn formas_em(&self, c: &Camera, t: &Tinta, recorte: Rect, por: &mut impl FnMut(Shape)) {
        if !self.inteira(c) || t.alfa <= 0.0 {
            return;
        }
        let tela: Vec<Pos2> = self.vertices.iter().map(|v| c.ponto(*v)).collect();
        // tudo o que vem abaixo fica entre os vértices, mais a grossura do traço e o esfumado da borda
        let folga = t.traco.width.max(t.vinco.width) + 2.0;
        if !recorte.intersects(Rect::from_points(&tela).expand(folga)) {
            return;
        }
        let vista: Vec<bool> = self.faces.iter().map(|f| self.face_a_vista(c, f)).collect();
        // a profundidade de cada face uma vez só, e não a cada comparação; o `sort_by` estável deixa a mesma ordem
        let mut ordem: Vec<(f32, usize)> = (0..self.faces.len()).filter(|i| vista[*i]).map(|i| (c.profundidade(self.faces[i].centro), i)).collect();
        ordem.sort_by(|a, b| a.0.total_cmp(&b.0));
        let luz = c.luz();
        let mut malha = Mesh::default();
        for (_, i) in ordem {
            let f = &self.faces[i];
            let cor = if t.tracejado { t.tons[0].gamma_multiply(t.alfa) } else { t.tom(dot(f.normal, luz)) };
            let base = malha.vertices.len() as u32;
            for ind in &f.indices {
                malha.vertices.push(Vertex { pos: tela[*ind as usize], uv: WHITE_UV, color: cor });
            }
            for k in 1..f.indices.len() as u32 - 1 {
                malha.add_triangle(base, base + k, base + k + 1);
            }
        }
        por(Shape::mesh(malha));
        let (traco, vinco) = (Stroke::new(t.traco.width, t.traco.color.gamma_multiply(t.alfa)), Stroke::new(t.vinco.width, t.vinco.color.gamma_multiply(t.alfa)));
        for a in &self.arestas {
            let (va, vb) = (vista[a.faces.0 as usize], vista[a.faces.1 as usize]);
            let linha = [tela[a.a as usize], tela[a.b as usize]];
            let contorno = va != vb || (a.faces.0 == a.faces.1 && va);
            if t.tracejado {
                if a.viva {
                    let s = if contorno || (va && vb) { traco } else { vinco };
                    for d in Shape::dashed_line(&linha, s, TRACO_VISIVEL.0, TRACO_VISIVEL.1) {
                        por(d);
                    }
                }
            } else if contorno {
                por(Shape::line_segment(linha, traco));
            } else if va && vb && a.viva && vinco.width > 0.0 {
                por(Shape::line_segment(linha, vinco));
            }
        }
    }

    /// `pos` cai em cima de alguma face à vista?
    pub fn contem(&self, c: &Camera, pos: Pos2) -> bool {
        if !self.inteira(c) {
            return false;
        }
        // cada vértice projetado uma vez, e não uma vez por face que o usa
        let tela: Vec<Pos2> = self.vertices.iter().map(|v| c.ponto(*v)).collect();
        // `dentro` só aceita o ponto que fica entre os vértices da face: fora da caixa de todos (com 1 px de sobra), nenhuma face o tem
        if !Rect::from_points(&tela).expand(1.0).contains(pos) {
            return false;
        }
        self.faces.iter().filter(|f| self.face_a_vista(c, f)).any(|f| dentro(&tela, &f.indices, pos))
    }

    /// Os pontos do quadrilátero `cantos` a fração `(u, v)`: para escrever e pintar numa face.
    pub fn na_face(cantos: &[V3; 4], u: f32, v: f32) -> V3 {
        mistura(mistura(cantos[0], cantos[1], u), mistura(cantos[3], cantos[2], u), v)
    }
}

/// Uma imagem colada num quadrilátero do mundo, na ordem `[topo esq, topo dir, base dir, base esq]`. Em pedaços, para a perspectiva não entortar.
pub fn imagem_na_face(c: &Camera, p: &Painter, textura: TextureId, cantos: &[V3; 4], alfa: f32) {
    if !cantos.iter().all(|v| c.alcanca(*v)) {
        return;
    }
    const PASSOS: u32 = 6;
    let mut m = Mesh::with_texture(textura);
    let cor = egui::Color32::WHITE.gamma_multiply(alfa);
    for j in 0..=PASSOS {
        for i in 0..=PASSOS {
            let (u, v) = (i as f32 / PASSOS as f32, j as f32 / PASSOS as f32);
            m.vertices.push(Vertex { pos: c.ponto(Malha::na_face(cantos, u, v)), uv: egui::pos2(u, v), color: cor });
        }
    }
    let linha = PASSOS + 1;
    for j in 0..PASSOS {
        for i in 0..PASSOS {
            let a = j * linha + i;
            m.add_triangle(a, a + 1, a + linha);
            m.add_triangle(a + 1, a + linha + 1, a + linha);
        }
    }
    p.add(Shape::mesh(m));
}

/// As pontas de aresta `(menor, maior, face)` na mesma ordem que um `sort` delas daria, em tempo linear (o sólido é refeito a cada quadro):
/// separa pelo menor vértice, contando, e ordena cada grupo, que é pequeno, pelo maior. As faces já chegam em ordem e a ordenação do grupo é estável.
fn pontas_em_ordem(faces: &[Face], n_vertices: usize) -> Vec<(u32, u32, u32)> {
    let ponta = |face: &Face, i: usize| {
        let (a, b) = (face.indices[i], face.indices[(i + 1) % face.indices.len()]);
        (a.min(b), a.max(b))
    };
    let mut inicio = vec![0usize; n_vertices + 1];
    for face in faces {
        for i in 0..face.indices.len() {
            inicio[ponta(face, i).0 as usize + 1] += 1;
        }
    }
    for v in 0..n_vertices {
        inicio[v + 1] += inicio[v];
    }
    let mut livre = inicio.clone();
    let mut pontas = vec![(0, 0, 0); inicio[n_vertices]];
    for (f, face) in faces.iter().enumerate() {
        for i in 0..face.indices.len() {
            let (a, b) = ponta(face, i);
            pontas[livre[a as usize]] = (a, b, f as u32);
            livre[a as usize] += 1;
        }
    }
    for v in 0..n_vertices {
        let grupo = &mut pontas[inicio[v]..inicio[v + 1]];
        for i in 1..grupo.len() {
            let mut j = i;
            while j > 0 && grupo[j - 1].1 > grupo[j].1 {
                grupo.swap(j - 1, j);
                j -= 1;
            }
        }
    }
    pontas
}

/// `q` cai dentro do anel `anel` de pontos de `tela`?
fn dentro(tela: &[Pos2], anel: &[u32], q: Pos2) -> bool {
    let n = anel.len();
    let mut sinal = 0.0_f32;
    for i in 0..n {
        let (a, b) = (tela[anel[i] as usize], tela[anel[(i + 1) % n] as usize]);
        let lado = (b.x - a.x) * (q.y - a.y) - (b.y - a.y) * (q.x - a.x);
        if lado.abs() < 1e-5 {
            continue;
        }
        if sinal != 0.0 && lado.signum() != sinal {
            return false;
        }
        sinal = lado.signum();
    }
    sinal != 0.0
}

#[cfg(test)]
mod tests {
    use super::super::camera::{Camera, AZIMUTE, ELEVACAO};
    use super::*;
    use egui::pos2;

    fn camera() -> Camera {
        Camera::nova(AZIMUTE, ELEVACAO, 20.0, [0.0; 3], pos2(200.0, 200.0), 0.0)
    }

    #[test]
    fn um_triangulo_tambem_tem_normal() {
        let m = Malha::de_faces(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![vec![0, 1, 2]]);
        assert_eq!(m.faces[0].normal, [0.0, 0.0, 1.0]);
    }

    #[test]
    fn uma_caixa_mostra_tres_faces_e_seis_arestas_de_contorno() {
        let m = Malha::caixa([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let c = camera();
        let vista: Vec<bool> = m.faces.iter().map(|f| m.face_a_vista(&c, f)).collect();
        assert_eq!(vista.iter().filter(|v| **v).count(), 3);
        let contorno = m.arestas.iter().filter(|a| vista[a.faces.0 as usize] != vista[a.faces.1 as usize]).count();
        assert_eq!(contorno, 6);
    }

    #[test]
    fn a_lateral_do_cilindro_e_lisa_e_a_tampa_e_viva() {
        let m = Malha::cilindro((0.0, 0.0), 2.0, 0.0, 2.0);
        let lisas = m.arestas.iter().filter(|a| !a.viva).count();
        assert_eq!(lisas, super::super::solidos::PASSOS_CIRCULO);
    }

    #[test]
    fn as_pontas_saem_na_ordem_do_sort() {
        let pano = Malha::de_faces(vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], vec![vec![0, 1, 2], vec![2, 1, 0]]);
        for m in [Malha::caixa([0.0; 3], [1.0, 2.0, 3.0]), Malha::esfera([0.0; 3], 1.0), Malha::laje([0.0; 3], [4.0, 3.0, 1.0], 0.5), pano] {
            let mut esperado: Vec<(u32, u32, u32)> = m.faces.iter().enumerate().flat_map(|(f, face)| {
                let n = face.indices.len();
                (0..n).map(move |i| (face.indices[i].min(face.indices[(i + 1) % n]), face.indices[i].max(face.indices[(i + 1) % n]), f as u32))
            }).collect();
            esperado.sort_unstable();
            assert_eq!(pontas_em_ordem(&m.faces, m.vertices.len()), esperado);
        }
    }

    #[test]
    fn o_clique_acha_a_caixa_e_erra_fora_dela() {
        let m = Malha::caixa([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let c = camera();
        assert!(m.contem(&c, c.ponto(m.centro())));
        assert!(!m.contem(&c, pos2(0.0, 0.0)));
    }
}
