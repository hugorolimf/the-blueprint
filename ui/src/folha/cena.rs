//! As peças do desenho da folha, nas coordenadas do SVG da página: o traço é centrado na geometria, o texto tem a linha de base em `y`,
//! o espaçamento de `.06em` e o contorno de 3 px na cor do papel que o CSS põe atrás das letras (`paint-order: stroke`).

use crate::fonte::{ALTURA, SUBIDA};
use crate::fonte::Peso;
use crate::planta::Texto;
use crate::tema::{BAIXO, MEDIO, PAPEL, TINTA};
use std::borrow::Cow;

use egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind};

/// A cor `--lo` das linhas leves (`.ln.l`).
pub const BAIXO_COR: Color32 = BAIXO;

/// Os 8 deslocamentos do contorno de 3 px na cor do papel em volta das letras.
const CONTORNO: [(f32, f32); 8] = [(-1.5, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5), (-1.1, -1.1), (1.1, -1.1), (-1.1, 1.1), (1.1, 1.1)];


#[derive(Clone, Copy, PartialEq)]
pub enum Ancora {
    Inicio,
    Meio,
    Fim,
}

/// O traço das classes `.ln`, `.ln.m` e `.ln.l` de web/css/base.css (`.guia` e `.eixo`, tracejados, entram com a linha do tempo).
#[derive(Clone, Copy)]
pub enum Fio {
    Ln,
    Medio,
    Baixo,
}

pub struct Cena<'a> {
    pub p: &'a Painter,
    pub o: Pos2,
}

impl Cena<'_> {
    pub fn pt(&self, x: f32, y: f32) -> Pos2 {
        self.o + vec2(x, y)
    }

    pub fn retangulo(&self, x: f32, y: f32, w: f32, h: f32, fundo: Option<Color32>, traco: Option<(Color32, f32)>) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w, h));
        if let Some(f) = fundo {
            self.p.rect_filled(r, 0, f);
        }
        if let Some((cor, largura)) = traco {
            self.p.rect_stroke(r, 0, Stroke::new(largura, cor), StrokeKind::Middle);
        }
    }

    /// Retângulo só de contorno, com traço contínuo (`ln`) ou tracejado `6 4` (as ações a fazer).
    pub fn moldura(&self, x: f32, y: f32, w: f32, h: f32, cor: Color32, largura: f32, tracejado: bool) {
        if !tracejado {
            return self.retangulo(x, y, w, h, None, Some((cor, largura)));
        }
        let pts = [self.pt(x, y), self.pt(x + w, y), self.pt(x + w, y + h), self.pt(x, y + h), self.pt(x, y)];
        self.p.extend(Shape::dashed_line(&pts, Stroke::new(largura, cor), 6.0, 4.0));
    }

    /// Segmentos soltos `(x1, y1, x2, y2)` com o traço de `fio`.
    pub fn fios(&self, fio: Fio, segmentos: &[(f32, f32, f32, f32)]) {
        let cor = match fio {
            Fio::Ln => TINTA,
            Fio::Medio => MEDIO,
            Fio::Baixo => BAIXO,
        };
        for &(x1, y1, x2, y2) in segmentos {
            self.p.line_segment([self.pt(x1, y1), self.pt(x2, y2)], Stroke::new(1.0, cor));
        }
    }

    /// Fio com traço e espessura livres (`stroke-width` do `style` da página).
    pub fn fio(&self, cor: Color32, largura: f32, de: (f32, f32), ate: (f32, f32)) {
        self.p.line_segment([self.pt(de.0, de.1), self.pt(ate.0, ate.1)], Stroke::new(largura, cor));
    }

    /// Escreve `s` com a linha de base em `y`. Devolve a largura. `cap`: caixa alta (a classe `cap`).
    #[allow(clippy::too_many_arguments)]
    pub fn texto(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32, ancora: Ancora, cap: bool) -> f32 {
        self.escrever(s, x, y, tam, peso, cor, ancora, cap, true)
    }

    /// O mesmo texto sem o contorno do papel (`.botao-t`: texto sobre fundo cheio).
    #[allow(clippy::too_many_arguments)]
    pub fn texto_liso(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32, ancora: Ancora) -> f32 {
        self.escrever(s, x, y, tam, peso, cor, ancora, false, false)
    }

    /// Texto com a linha de sublinhado de `.lk` (3 px abaixo da base, na cor `BAIXO`).
    pub fn texto_sublinhado(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32, ancora: Ancora, cap: bool) -> f32 {
        let largura = self.escrever(s, x, y, tam, peso, cor, ancora, cap, true);
        let esquerda = match ancora {
            Ancora::Inicio => x,
            Ancora::Meio => x - largura / 2.0,
            Ancora::Fim => x - largura,
        };
        self.fio(BAIXO, 1.0, (esquerda, y + 3.5), (esquerda + largura, y + 3.5));
        largura
    }

    /// Traço com um padrão de tracos e vãos que se repete (`stroke-dasharray`). Só pinta o trecho que cruza o recorte do pincel:
    /// os guias da régua descem a página inteira e o resto não aparece.
    pub fn tracejado(&self, cor: Color32, largura: f32, de: (f32, f32), ate: (f32, f32), padrao: &[f32]) {
        let comprimento = ((ate.0 - de.0).powi(2) + (ate.1 - de.1).powi(2)).sqrt();
        let periodo: f32 = padrao.iter().sum();
        if comprimento <= 0.0 || periodo <= 0.0 {
            return;
        }
        let (dx, dy) = ((ate.0 - de.0) / comprimento, (ate.1 - de.1) / comprimento);
        // o trecho do fio (em distância desde `de`) que cai no recorte, em coordenadas da página
        let recorte = self.p.clip_rect().expand(largura).translate(-self.o.to_vec2());
        let (mut t0, mut t1) = (0.0f32, comprimento);
        for (o, d, lo, hi) in [(de.0, dx, recorte.min.x, recorte.max.x), (de.1, dy, recorte.min.y, recorte.max.y)] {
            if d.abs() < 1e-6 {
                if o < lo || o > hi {
                    return;
                }
            } else {
                let (a, b) = ((lo - o) / d, (hi - o) / d);
                t0 = t0.max(a.min(b));
                t1 = t1.min(a.max(b));
            }
        }
        if t0 > t1 {
            return;
        }
        // começa no período em que o trecho visível começa: cada traço fica no mesmo lugar de quando se contava desde o começo
        let k = (t0 / periodo).floor().max(0.0);
        let (mut pos, mut i) = (k * periodo, k as usize * padrao.len());
        while pos < comprimento && pos <= t1 {
            let passo = padrao[i % padrao.len()].min(comprimento - pos);
            if i % 2 == 0 {
                self.fio(cor, largura, (de.0 + dx * pos, de.1 + dy * pos), (de.0 + dx * (pos + passo), de.1 + dy * (pos + passo)));
            }
            pos += padrao[i % padrao.len()];
            i += 1;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn escrever(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, cor: Color32, ancora: Ancora, cap: bool, contorno: bool) -> f32 {
        let s: Cow<str> = if cap { Cow::Owned(s.to_uppercase()) } else { Cow::Borrowed(s) };
        // uma galeria só, na cor de reserva: mede, faz o contorno e o texto (o tessellator troca PLACEHOLDER pela cor de cada pintura)
        let g = Texto::novo(&*s, tam).peso(peso).trilha(0.06).cor(Color32::PLACEHOLDER).galeria(self.p);
        // a largura do CSS: o espaçamento também vem depois da última letra (`Texto::largura`)
        let largura = g.size().x + 0.06 * tam;
        let esquerda = match ancora {
            Ancora::Inicio => x,
            Ancora::Meio => x - largura / 2.0,
            Ancora::Fim => x - largura,
        };
        let topo = self.pt(esquerda, y - SUBIDA * tam);
        // fora do recorte nada disso aparece (o tessellator já descartava): basta a largura
        if s.is_empty() || !self.p.clip_rect().intersects(Rect::from_min_size(topo, g.size()).expand(tam)) {
            return largura;
        }
        // abrindo a tela: o texto se escreve em três trechos, cada um chegando na sua vez (anima.rs)
        let escrita = crate::anima::atual(self.p.ctx()).and_then(|a| a.escrita(&s, topo.x, topo.y, topo.y + ALTURA * tam));
        let Some(alfas) = escrita.filter(|a| a.iter().any(|v| *v < 1.0)) else {
            if contorno {
                self.p.extend(CONTORNO.iter().map(|&(dx, dy)| Shape::galley(topo + vec2(dx, dy), g.clone(), PAPEL)));
            }
            self.p.galley(topo, g, cor);
            return largura;
        };
        let letras: Vec<char> = s.chars().collect();
        let cortes = crate::anima::tercos(letras.len());
        for k in 0..3 {
            let antes: String = letras[..cortes[k]].iter().collect();
            let parte: String = letras[cortes[k]..cortes[k + 1]].iter().collect();
            let dx = if antes.is_empty() { 0.0 } else { Texto::novo(antes, tam).peso(peso).trilha(0.06).largura(self.p) };
            self.pintar(&parte, topo + vec2(dx, 0.0), tam, peso, cor, contorno, alfas[k]);
        }
        largura
    }

    /// Pinta o texto pronto com o canto de cima em `topo`, com o contorno na cor do papel se pedido, na opacidade `alfa`.
    #[allow(clippy::too_many_arguments)]
    fn pintar(&self, s: &str, topo: Pos2, tam: f32, peso: Peso, cor: Color32, contorno: bool, alfa: f32) {
        if alfa <= 0.0 || s.is_empty() {
            return;
        }
        // o contorno de 3 px na cor do papel: o texto corta a grade sem apagar o fundo em volta
        if contorno {
            let papel = PAPEL.gamma_multiply(alfa);
            let g = Texto::novo(s, tam).peso(peso).trilha(0.06).cor(papel).galeria(self.p);
            for (dx, dy) in CONTORNO {
                self.p.galley(topo + vec2(dx, dy), g.clone(), papel);
            }
        }
        Texto::novo(s, tam).peso(peso).trilha(0.06).cor(cor.gamma_multiply(alfa)).em(self.p, topo);
    }

    /// A legenda da marca-d'água: `legenda-marca` (espaçamento .16em) com a opacidade de `.marca-fundo` (18%), centrada em `x`, sem contorno.
    pub fn texto_apagado(&self, s: &str, x: f32, y: f32, tam: f32) {
        let t = Texto::novo(s, tam).trilha(0.16).cor(MEDIO.gamma_multiply(0.18));
        let largura = t.largura(self.p);
        t.em(self.p, self.pt(x - largura / 2.0, y - SUBIDA * tam));
    }

    /// O retângulo de `.pesq-realce` atrás dos trechos achados de um texto: `trechos` em letras do original; `cap` se o texto aparece em caixa alta;
    /// `trilha` é o `letter-spacing` em múltiplos do tamanho. O texto é escrito depois, por cima.
    pub fn realce(&self, s: &str, x: f32, y: f32, tam: f32, peso: Peso, trilha: f32, cap: bool, trechos: &[(usize, usize)]) {
        let letras: Vec<char> = if cap { s.to_uppercase().chars().collect() } else { s.chars().collect() };
        let largura = |de: usize, ate: usize| -> f32 {
            let parte: String = letras[de.min(letras.len())..ate.min(letras.len())].iter().collect();
            if parte.is_empty() { 0.0 } else { Texto::novo(parte, tam).peso(peso).trilha(trilha).largura(self.p) }
        };
        for &(a, b) in trechos {
            let (x0, w) = (x + largura(0, a), largura(a, b));
            self.retangulo(x0, y - SUBIDA * tam, w, ALTURA * tam, Some(BAIXO.gamma_multiply(0.7)), None);
        }
    }

    /// O hachurado de `pattern#hach`: linhas a 45° espaçadas de 6 px, em `#9cc4ee` com 1,5 px.
    pub fn hachura(&self, x: f32, y: f32, w: f32, h: f32) {
        let r = Rect::from_min_size(self.pt(x, y), vec2(w, h));
        let recorte = r.intersect(self.p.clip_rect());
        // fora da janela as linhas seriam todas cortadas
        if !recorte.is_positive() {
            return;
        }
        let p = self.p.with_clip_rect(recorte);
        let passo = 6.0 * 2f32.sqrt();
        let mut k = -h;
        while k < w + h {
            p.line_segment([pos2(r.min.x + k, r.max.y), pos2(r.min.x + k + h, r.min.y)], Stroke::new(1.5, MEDIO));
            k += passo;
        }
    }

    /// O losango quadrado de meia-diagonal `r` (`M x yc-r l r r l -r r l -r -r z`), cheio de tinta se feito.
    pub fn losango_r(&self, x: f32, yc: f32, r: f32, feito: bool, traco: f32) {
        let pts = vec![self.pt(x, yc - r), self.pt(x + r, yc), self.pt(x, yc + r), self.pt(x - r, yc)];
        let (fundo, borda) = if feito { (TINTA, PAPEL) } else { (PAPEL, TINTA) };
        self.p.add(Shape::convex_polygon(pts, fundo, Stroke::new(traco, borda)));
    }

    /// O ícone de nota (folha com a ponta dobrada) das observações e instruções (`notaSvg`).
    pub fn nota(&self, x: f32, y: f32, cheio: bool) {
        let (fundo, traco) = if cheio { (TINTA, PAPEL) } else { (PAPEL, TINTA) };
        let corpo = vec![self.pt(x, y), self.pt(x + 9.0, y), self.pt(x + 14.0, y + 5.0), self.pt(x + 14.0, y + 16.0), self.pt(x, y + 16.0)];
        self.p.add(Shape::convex_polygon(corpo, fundo, Stroke::new(1.2, TINTA)));
        for (de, ate) in [((9.0, 0.0), (9.0, 5.0)), ((9.0, 5.0), (14.0, 5.0)), ((3.0, 9.0), (11.0, 9.0)), ((3.0, 12.5), (11.0, 12.5))] {
            self.fio(traco, 1.2, (x + de.0, y + de.1), (x + ate.0, y + ate.1));
        }
    }

    /// O losango de checkpoint (`.dm`): cheio de tinta se feito.
    pub fn losango(&self, x: f32, yc: f32, feito: bool) {
        let pts = vec![self.pt(x, yc - 5.0), self.pt(x + 4.0, yc), self.pt(x, yc + 5.0), self.pt(x - 4.0, yc)];
        let (fundo, traco) = if feito { (TINTA, PAPEL) } else { (PAPEL, TINTA) };
        self.p.add(Shape::convex_polygon(pts, fundo, Stroke::new(1.4, traco)));
    }
}
