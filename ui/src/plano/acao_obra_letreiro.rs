//! O letreiro: a identidade de um plano escrita no vidro do monitor, quando a construção é uma Tela sem tela própria. Em vez de monitores vazios iguais, cada um mostra o seu número, pequeno em cima, e a sua entrega em até três linhas, como a tela provisória de um wireframe.
//! As letras saem das fontes do egui e vão uma a uma para o vidro em perspectiva (`Malha::na_face` e a câmera), na vez do vidro na ordem da cena. De longe, onde a letra não se lê, ficam só os traços de onde o texto está; de perto, os traços dão lugar às letras.

use super::super::modelo::Plano;
use super::obra_formas::{vidro, Forma};
use super::obra_tela::Composicao;
use crate::anatomia::{Camera, Malha, V3};
use crate::fonte::{familia, Peso, AVANCO};
use crate::tema::{MEDIO, TINTA};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{vec2, Color32, FontId, Painter, Pos2, Shape};

/// Quantas letras cabem numa linha: o texto quebra igual em qualquer zoom, só a câmera muda.
const COLUNAS: usize = 20;
const MAX_LINHAS: usize = 3;
/// A margem dos lados, em fração da largura do vidro.
const MARGEM: f32 = 0.045;
/// O número em cima, em fração da letra da entrega.
const NUMERO: f32 = 0.7;
/// De uma linha à seguinte, em múltiplos da letra.
const PASSO: f32 = 1.22;
const VAO_NUMERO: f32 = 0.3;
/// Abaixo desta largura na tela, o vidro fica liso.
const MIN_TRACOS: f32 = 14.0;
/// As letras só entram com o vidro desta largura na tela, e clareiam enquanto a letra cresce entre estes tamanhos (em pontos).
const MIN_LETRAS: f32 = 40.0;
const LETRA_LEGIVEL: (f32, f32) = (4.5, 7.0);
/// O quanto os traços aparecem: um esboço, não um texto.
const TRACO: f32 = 0.45;
/// Antes de a obra subir, o letreiro fica no vidro do fantasma, mais apagado.
const FANTASMA: f32 = 0.55;

/// O letreiro de uma construção, com o vidro onde ele vai e o quanto aparece.
pub struct Letreiro<'a> {
    numero: &'a str,
    texto: &'a str,
    cantos: [V3; 4],
    alfa: f32,
}

/// As medidas do letreiro num vidro, em unidades do mundo: as margens, a letra da entrega e quantas linhas dela cabem.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Grade {
    mx: f32,
    my: f32,
    letra: f32,
    linhas: usize,
}

/// Uma linha do letreiro: o texto, a letra, a cor e onde começa, a partir do topo do vidro.
struct Linha {
    texto: String,
    letra: f32,
    cor: Color32,
    topo: f32,
    peso: Peso,
}

impl<'a> Letreiro<'a> {
    /// O letreiro de `q`, se a construção é um monitor sem tela própria. `subiu` é o quanto a obra está de pé: o letreiro vai no vidro do que subiu, ou no do fantasma antes de a obra começar; enquanto o monitor é baixo demais para ter vidro, não há letreiro.
    pub fn do_plano(q: &'a Plano, comp: Composicao, (x, y, z): (f32, f32, f32), h: f32, subiu: f32, alfa: f32) -> Option<Letreiro<'a>> {
        if comp != Composicao::Sem || Forma::do_plano(q) != Forma::Tela {
            return None;
        }
        let (cantos, alfa) = if subiu > 0.0 { (vidro(x, y, z, h * subiu)?, alfa) } else { (vidro(x, y, z, h)?, alfa * FANTASMA) };
        let texto = if q.entrega.trim().is_empty() { q.texto.trim() } else { q.entrega.trim() };
        Some(Letreiro { numero: q.referencia.as_str(), texto, cantos, alfa })
    }

    /// Onde ele entra na ordem da cena: logo na frente do vidro, para pintar depois dele e antes do que está na frente.
    pub fn centro(&self) -> V3 {
        let [x, y, z] = Malha::na_face(&self.cantos, 0.5, 0.5);
        [x, y + 0.05, z]
    }

    /// Pinta no vidro, se ele está de frente para a câmera e grande o bastante na tela.
    pub fn pintar(&self, cam: &Camera, p: &Painter) {
        let c = &self.cantos;
        if self.alfa <= 0.01 || !c.iter().all(|v| cam.alcanca(*v)) {
            return;
        }
        let t = self.cantos.map(|v| cam.ponto(v));
        let (topo, lado) = (t[1] - t[0], t[3] - t[0]);
        // de costas: a câmera deu a volta no monitor
        if topo.x * lado.y - topo.y * lado.x <= 0.0 {
            return;
        }
        let (lw, lh) = (distancia(c[0], c[1]), distancia(c[0], c[3]));
        let largura = (topo.length() + (t[2] - t[3]).length()) * 0.5;
        let Some(g) = grade(lw, lh).filter(|_| largura >= MIN_TRACOS) else { return };
        // pontos da tela por unidade do mundo, no vidro: a letra sai do atlas no tamanho em que aparece
        let escala = (largura / lw).max((lado.length() + (t[2] - t[1]).length()) * 0.5 / lh);
        let letras = if largura < MIN_LETRAS { 0.0 } else { entre(LETRA_LEGIVEL.0, LETRA_LEGIVEL.1, g.letra * escala) };
        let no_vidro = |u: f32, v: f32| cam.ponto(Malha::na_face(c, u / lw, v / lh));
        let mut fileira = vec![Linha { texto: self.numero.to_string(), letra: g.letra * NUMERO, cor: MEDIO, topo: g.my, peso: Peso::Medio }];
        let topo_texto = g.my + g.letra * (NUMERO * PASSO + VAO_NUMERO);
        for (i, texto) in linhas(self.texto, COLUNAS, g.linhas).into_iter().enumerate() {
            fileira.push(Linha { texto, letra: g.letra, cor: TINTA, topo: topo_texto + i as f32 * g.letra * PASSO, peso: Peso::Normal });
        }
        let mut m = Mesh::default();
        // de longe, os traços de onde o texto está; de perto, eles dão lugar às letras
        let tracos = MEDIO.gamma_multiply(self.alfa * TRACO * (1.0 - letras));
        if tracos.a() > 0 {
            for l in &fileira {
                let (u0, u1) = (g.mx, g.mx + l.texto.chars().count() as f32 * AVANCO * l.letra);
                let (v0, v1) = (l.topo + l.letra * 0.3, l.topo + l.letra * 0.8);
                quad(&mut m, [no_vidro(u0, v0), no_vidro(u1, v0), no_vidro(u1, v1), no_vidro(u0, v1)], tracos);
            }
        }
        if letras > 0.0 {
            let galerias: Vec<_> = fileira.iter().map(|l| {
                let pt = degrau(l.letra * escala);
                (p.layout_no_wrap(l.texto.clone(), FontId::new(pt, familia(l.peso)), l.cor), l.letra / pt, l.cor.gamma_multiply(self.alfa * letras), l.topo)
            }).collect();
            // montar as letras pode aumentar o atlas: o tamanho dele é lido depois
            let [w, h] = p.fonts(|f| f.font_image_size());
            let normal = vec2(1.0 / w.max(1) as f32, 1.0 / h.max(1) as f32);
            for (galeria, k, cor, v0) in &galerias {
                for linha in &galeria.rows {
                    let base = m.vertices.len() as u32;
                    m.indices.extend(linha.visuals.mesh.indices.iter().map(|i| i + base));
                    m.vertices.extend(linha.visuals.mesh.vertices.iter().map(|vx| {
                        let pos = linha.pos + vx.pos.to_vec2();
                        Vertex { pos: no_vidro(g.mx + pos.x * *k, *v0 + pos.y * *k), uv: (vx.uv.to_vec2() * normal).to_pos2(), color: *cor }
                    }));
                }
            }
        }
        if !m.is_empty() {
            p.add(Shape::mesh(m));
        }
    }
}

fn distancia(a: V3, b: V3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Um quadrilátero liso, com o branco do atlas: a mesma malha das letras.
fn quad(m: &mut Mesh, cantos: [Pos2; 4], cor: Color32) {
    let base = m.vertices.len() as u32;
    for pos in cantos {
        m.vertices.push(Vertex { pos, uv: WHITE_UV, color: cor });
    }
    m.add_triangle(base, base + 1, base + 2);
    m.add_triangle(base, base + 2, base + 3);
}

/// De 0 a 1 enquanto `x` vai de `a` a `b`, suave nas pontas.
fn entre(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// O tamanho da letra em pontos, em degraus: o atlas de fontes não ganha um tamanho novo a cada quadro do zoom.
fn degrau(pt: f32) -> f32 {
    let passo = if pt < 12.0 { 0.5 } else if pt < 24.0 { 1.0 } else { 2.0 };
    ((pt / passo).round() * passo).clamp(4.0, 40.0)
}

/// As medidas num vidro `lw` × `lh`. `None` quando nem o número cabe.
fn grade(lw: f32, lh: f32) -> Option<Grade> {
    let mx = lw * MARGEM;
    let letra = (lw - mx * 2.0) / (COLUNAS as f32 * AVANCO);
    let my = (lh * 0.08).min(mx);
    let cabe = lh - my * 2.0 - letra * (NUMERO * PASSO + VAO_NUMERO);
    if letra <= 0.0 || cabe < 0.0 {
        return None;
    }
    Some(Grade { mx, my, letra, linhas: ((cabe / (letra * PASSO)).floor() as usize).min(MAX_LINHAS) })
}

/// O texto quebrado em linhas de até `colunas` letras, nas palavras; a palavra maior que a linha é cortada. Se passar de `max` linhas, a última termina em `…`.
fn linhas(texto: &str, colunas: usize, max: usize) -> Vec<String> {
    let colunas = colunas.max(2);
    let mut v: Vec<String> = Vec::new();
    let mut atual: Vec<char> = Vec::new();
    for palavra in texto.split_whitespace() {
        let mut resto: Vec<char> = palavra.chars().collect();
        loop {
            let livre = if atual.is_empty() { colunas } else { colunas.saturating_sub(atual.len() + 1) };
            if resto.len() <= livre {
                if !atual.is_empty() {
                    atual.push(' ');
                }
                atual.append(&mut resto);
                break;
            }
            if atual.is_empty() {
                v.push(resto.drain(..colunas).collect());
            } else {
                v.push(std::mem::take(&mut atual).into_iter().collect());
            }
        }
    }
    if !atual.is_empty() {
        v.push(atual.into_iter().collect());
    }
    if v.len() > max {
        v.truncate(max);
        if let Some(ultima) = v.last_mut() {
            let mut c: Vec<char> = ultima.chars().take(colunas - 1).collect();
            while c.last() == Some(&' ') {
                c.pop();
            }
            c.push('…');
            *ultima = c.into_iter().collect();
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_entrega_quebra_nas_palavras_e_corta_na_terceira_linha() {
        let v = linhas("Acrescentar ao catálogo em inglês os textos novos da tela", 20, 3);
        assert_eq!(v, vec!["Acrescentar ao", "catálogo em inglês", "os textos novos da…"]);
        assert!(v.iter().all(|l| l.chars().count() <= 20), "{v:?}");
        assert_eq!(linhas("Lista", 20, 3), vec!["Lista"]);
        assert_eq!(linhas("abcdefghij", 4, 3), vec!["abcd", "efgh", "ij"]);
        assert!(linhas("", 20, 3).is_empty());
    }

    #[test]
    fn o_vidro_inteiro_leva_tres_linhas_e_o_baixo_so_o_numero() {
        assert_eq!(grade(3.4, 1.6).map(|g| g.linhas), Some(3));
        assert_eq!(grade(3.4, 0.55).map(|g| g.linhas), Some(0));
        assert_eq!(grade(3.4, 0.2), None);
    }

    #[test]
    fn a_letra_anda_em_degraus() {
        assert_eq!(degrau(7.3), 7.5);
        assert_eq!(degrau(13.4), 13.0);
        assert_eq!(degrau(30.9), 30.0);
        assert_eq!((degrau(1.0), degrau(100.0)), (4.0, 40.0));
    }

    #[test]
    fn so_o_monitor_sem_tela_tem_letreiro() {
        let q = Plano { referencia: "1.2".into(), forma: "tela".into(), texto: "Texto".into(), entrega: "A lista nova".into(), ..Plano::default() };
        let l = Letreiro::do_plano(&q, Composicao::Sem, (0.0, 0.0, 0.0), 2.6, 1.0, 1.0).expect("monitor de pé");
        assert_eq!((l.numero, l.texto), ("1.2", "A lista nova"));
        assert!(Letreiro::do_plano(&q, Composicao::Parede, (0.0, 0.0, 0.0), 2.6, 1.0, 1.0).is_none());
        let dados = Plano { forma: "dados".into(), ..q.clone() };
        assert!(Letreiro::do_plano(&dados, Composicao::Sem, (0.0, 0.0, 0.0), 2.6, 1.0, 1.0).is_none());
        let fantasma = Letreiro::do_plano(&q, Composicao::Sem, (0.0, 0.0, 0.0), 2.6, 0.0, 1.0).expect("o fantasma tem vidro");
        assert!(fantasma.alfa < 1.0);
    }
}
