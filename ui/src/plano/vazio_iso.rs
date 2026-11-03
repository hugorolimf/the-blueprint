//! O desenho isométrico em traço (a linguagem Anatomy) feito direto no painter do egui: a câmera, a caixa de arestas finas, o tracejado e a grade do chão.
//! Mundo em unidades livres, eixo `z` para cima; `x` cresce para a direita e para baixo, `y` para a esquerda e para baixo.

use egui::{vec2, Color32, Painter, Pos2, Shape, Stroke};

pub type P3 = (f32, f32, f32);

/// Meia largura e meia altura de uma unidade na tela (30 graus).
pub const COS: f32 = 0.866;
const SEN: f32 = 0.5;

/// As cores das três faces de uma caixa e o traço das arestas.
#[derive(Clone, Copy)]
pub struct Tons {
    pub topo: Color32,
    pub esq: Color32,
    pub dir: Color32,
    pub traco: Stroke,
}

impl Tons {
    /// A mesma caixa com tudo mais apagado (para entrar e sair de cena).
    pub fn apagado(self, a: f32) -> Tons {
        Tons { topo: self.topo.gamma_multiply(a), esq: self.esq.gamma_multiply(a), dir: self.dir.gamma_multiply(a), traco: Stroke::new(self.traco.width, self.traco.color.gamma_multiply(a)) }
    }
}

pub struct Iso<'a> {
    pub p: &'a Painter,
    /// onde fica a origem do mundo na tela
    pub o: Pos2,
    /// pixels por unidade
    pub k: f32,
}

impl Iso<'_> {
    pub fn ponto(&self, (x, y, z): P3) -> Pos2 {
        self.o + vec2((x - y) * COS * self.k, (x + y) * SEN * self.k - z * self.k)
    }

    pub fn linha(&self, a: P3, b: P3, traco: Stroke) {
        self.p.line_segment([self.ponto(a), self.ponto(b)], traco);
    }

    /// Uma linha tracejada; `deslocamento` faz os traços correrem.
    pub fn tracejada(&self, pontos: &[P3], traco: Stroke, comprimento: f32, vao: f32, deslocamento: f32) {
        let tela: Vec<Pos2> = pontos.iter().map(|q| self.ponto(*q)).collect();
        self.p.extend(Shape::dashed_line_with_offset(&tela, traco, &[comprimento], &[vao], deslocamento));
    }

    /// Um polígono plano do mundo (uma face), com fundo e contorno.
    pub fn face(&self, pontos: &[P3], fundo: Color32, traco: Stroke) {
        let tela: Vec<Pos2> = pontos.iter().map(|q| self.ponto(*q)).collect();
        self.p.add(Shape::convex_polygon(tela, fundo, traco));
    }

    /// Um círculo deitado no plano `z`.
    pub fn circulo(&self, centro: (f32, f32), z: f32, raio: f32, traco: Stroke) {
        let pontos: Vec<Pos2> = (0..=48).map(|i| {
            let a = i as f32 / 48.0 * std::f32::consts::TAU;
            self.ponto((centro.0 + raio * a.cos(), centro.1 + raio * a.sin(), z))
        }).collect();
        self.p.add(Shape::line(pontos, traco));
    }

    /// A caixa cheia: as três faces que a câmera vê, de trás para a frente.
    pub fn caixa(&self, (x, y, z): P3, (w, d, h): P3, t: Tons) {
        let topo = [(x, y, z + h), (x + w, y, z + h), (x + w, y + d, z + h), (x, y + d, z + h)];
        let esq = [(x, y + d, z), (x + w, y + d, z), (x + w, y + d, z + h), (x, y + d, z + h)];
        let dir = [(x + w, y, z), (x + w, y + d, z), (x + w, y + d, z + h), (x + w, y, z + h)];
        self.face(&esq, t.esq, t.traco);
        self.face(&dir, t.dir, t.traco);
        self.face(&topo, t.topo, t.traco);
    }

    /// Só as arestas da caixa, tracejadas: o que ainda não existe.
    pub fn caixa_tracejada(&self, (x, y, z): P3, (w, d, h): P3, traco: Stroke, deslocamento: f32) {
        let (a, b) = (z, z + h);
        let anel = |zz: f32| [(x, y, zz), (x + w, y, zz), (x + w, y + d, zz), (x, y + d, zz), (x, y, zz)];
        self.tracejada(&anel(a), traco, 3.0, 3.0, deslocamento);
        self.tracejada(&anel(b), traco, 3.0, 3.0, deslocamento);
        for (px, py) in [(x + w, y), (x + w, y + d), (x, y + d)] {
            self.tracejada(&[(px, py, a), (px, py, b)], traco, 3.0, 3.0, deslocamento);
        }
    }

    /// A grade do chão, de `de` a `ate` de `passo` em `passo`.
    pub fn grade(&self, de: f32, ate: f32, passo: f32, traco: Stroke) {
        let mut v = de;
        while v <= ate + 0.001 {
            self.linha((v, de, 0.0), (v, ate, 0.0), traco);
            self.linha((de, v, 0.0), (ate, v, 0.0), traco);
            v += passo;
        }
    }
}
