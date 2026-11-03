//! A câmera: a mesma conta do kit (`iso-kit/camera.mjs`), com azimute e elevação livres e uma perspectiva que pode entrar aos poucos.
//! Com `perspectiva` zero o desenho é axonométrico, como as figuras do kit; quanto maior, mais perto o olho está do foco. É assim que se mergulha numa peça.

use super::vetor::{mistura, soma, sub, unit, vezes, V3};
use crate::plano::acao_movimento::{desligado, suave};
use egui::{Context, Id, Pos2, Rect};

/// O ângulo de sempre do Anatomy.
pub const AZIMUTE: f32 = 45.0;
pub const ELEVACAO: f32 = 30.0;
/// Um ponto mais perto do olho do que isto (fração da distância) já passou da câmera.
const LIMITE_OLHO: f32 = 0.9;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Camera {
    azimute: f32,
    elevacao: f32,
    /// pixels por unidade do mundo, no foco
    pub escala: f32,
    /// o ponto do mundo que cai em `origem`
    pub foco: V3,
    pub origem: Pos2,
    /// 1 / a distância do olho ao foco; zero: olho no infinito
    pub perspectiva: f32,
    seno: [f32; 4],
}

impl Camera {
    pub fn nova(azimute: f32, elevacao: f32, escala: f32, foco: V3, origem: Pos2, perspectiva: f32) -> Camera {
        let (a, e) = (azimute.to_radians(), elevacao.to_radians());
        Camera { azimute, elevacao, escala, foco, origem, perspectiva, seno: [a.sin(), a.cos(), e.sin(), e.cos()] }
    }

    /// O enquadramento do kit (`fitProjection`): cabe em `r` com folga `pad`. A origem fica relativa a `r.min` se `r` começar em zero.
    /// O foco é o meio dos pontos, não o zero do mundo: girar a câmera depois gira o desenho em torno do próprio centro.
    pub fn enquadrar(pontos: &[V3], r: Rect, pad: f32, azimute: f32, elevacao: f32) -> Camera {
        let base = Camera::nova(azimute, elevacao, 1.0, [0.0; 3], Pos2::ZERO, 0.0);
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        let (mut baixo, mut alto) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in pontos {
            let q = base.ponto(*p);
            (x0, x1, y0, y1) = (x0.min(q.x), x1.max(q.x), y0.min(q.y), y1.max(q.y));
            for i in 0..3 {
                (baixo[i], alto[i]) = (baixo[i].min(p[i]), alto[i].max(p[i]));
            }
        }
        let escala = ((r.width() - pad * 2.0) / (x1 - x0).max(1e-3)).min((r.height() - pad * 2.0) / (y1 - y0).max(1e-3));
        let foco = mistura(baixo, alto, 0.5);
        let q = base.ponto(foco);
        let origem = r.center() + egui::vec2(q.x - (x0 + x1) * 0.5, q.y - (y0 + y1) * 0.5) * escala;
        Camera::nova(azimute, elevacao, escala, foco, origem, 0.0)
    }

    /// A mesma câmera, girada: o foco fica onde está na tela.
    pub fn girada(&self, az: f32, el: f32) -> Camera {
        Camera::nova(self.azimute + az, self.elevacao + el, self.escala, self.foco, self.origem, self.perspectiva)
    }

    /// `d` pixels na tela, em unidades do mundo, no plano da vista. Sem perspectiva.
    pub fn da_tela(&self, d: egui::Vec2) -> V3 {
        let [sa, ca, se, ce] = self.seno;
        vezes([sa * d.x + ca * se * d.y, -ca * d.x + sa * se * d.y, -ce * d.y], 1.0 / self.escala.max(1e-3))
    }

    /// O ponto que, visto desta câmera sem perspectiva, cai no mesmo lugar que `p` e está na altura `z`.
    pub fn na_altura(&self, p: V3, z: f32) -> V3 {
        let olho = self.olhar();
        soma(p, vezes(olho, (z - p[2]) / olho[2].max(0.05)))
    }

    pub fn azimute(&self) -> f32 {
        self.azimute
    }

    pub fn elevacao(&self) -> f32 {
        self.elevacao
    }

    /// De onde o olho olha, em relação ao foco (de longe, a mesma direção para todo ponto).
    pub fn olhar(&self) -> V3 {
        let [sa, ca, se, ce] = self.seno;
        [ca * ce, sa * ce, se]
    }

    /// A luz anda com a câmera, como no kit: a mesma face fica sempre com o mesmo tom.
    pub fn luz(&self) -> V3 {
        let [sa, ca, _, _] = self.seno;
        unit([-sa, ca, 1.2])
    }

    /// De `p`, para onde fica o olho. Muda de ponto a ponto quando há perspectiva.
    pub fn para_o_olho(&self, p: V3) -> V3 {
        if self.perspectiva <= 0.0 {
            return self.olhar();
        }
        unit(sub(soma(self.foco, vezes(self.olhar(), 1.0 / self.perspectiva)), p))
    }

    /// Quanto `p` está à frente do foco, na direção do olho.
    pub fn profundidade(&self, p: V3) -> f32 {
        let [sa, ca, se, ce] = self.seno;
        let [x, y, z] = sub(p, self.foco);
        (x * ca + y * sa) * ce + z * se
    }

    /// O ponto ainda está na frente do olho?
    pub fn alcanca(&self, p: V3) -> bool {
        self.profundidade(p) * self.perspectiva < LIMITE_OLHO
    }

    /// O aumento de `p` por estar mais perto ou mais longe do que o foco.
    pub fn aumento(&self, p: V3) -> f32 {
        // sem perspectiva a conta abaixo dá sempre 1; `ponto` pergunta isto a cada vértice, a cada quadro
        if self.perspectiva == 0.0 {
            return 1.0;
        }
        1.0 / (1.0 - (self.profundidade(p) * self.perspectiva).min(LIMITE_OLHO))
    }

    /// Onde `p` cai na tela.
    pub fn ponto(&self, p: V3) -> Pos2 {
        let [sa, ca, se, ce] = self.seno;
        let [x, y, z] = sub(p, self.foco);
        let k = self.escala * self.aumento(p);
        self.origem + egui::vec2((x * sa - y * ca) * k, ((x * ca + y * sa) * se - z * ce) * k)
    }

    /// O caminho até `para`, de 0 a 1. A escala anda em proporção, para a chegada não parecer mais rápida que a saída.
    pub fn rumo(&self, para: &Camera, t: f32) -> Camera {
        let junta = |a: f32, b: f32| a + (b - a) * t;
        let escala = self.escala.max(1e-3) * (para.escala.max(1e-3) / self.escala.max(1e-3)).powf(t);
        let origem = self.origem.lerp(para.origem, t);
        Camera::nova(junta(self.azimute, para.azimute), junta(self.elevacao, para.elevacao), escala, mistura(self.foco, para.foco, t), origem, junta(self.perspectiva, para.perspectiva))
    }
}

#[derive(Clone, Copy, Default)]
struct Trilho {
    de: Camera,
    para: Camera,
    inicio: f64,
    seg: f32,
}

/// A câmera a caminho de `alvo`, partindo de onde estava quando o alvo mudou. Sem animação, já no alvo.
pub fn viva(ctx: &Context, id: Id, alvo: Camera, seg: f32) -> Camera {
    if desligado() {
        return alvo;
    }
    let agora = ctx.input(|i| i.time);
    let antes = ctx.data(|d| d.get_temp::<Trilho>(id));
    let t = |x: &Trilho| suave((((agora - x.inicio) as f32) / x.seg.max(1e-3)).clamp(0.0, 1.0));
    let trilho = match antes {
        Some(x) if x.para == alvo => x,
        Some(x) => Trilho { de: x.de.rumo(&x.para, t(&x)), para: alvo, inicio: agora, seg },
        None => Trilho { de: alvo, para: alvo, inicio: agora, seg: 0.0 },
    };
    ctx.data_mut(|d| d.insert_temp(id, trilho));
    let k = t(&trilho);
    if k < 1.0 {
        ctx.request_repaint();
    }
    trilho.de.rumo(&trilho.para, k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn sem_perspectiva_e_a_conta_do_kit() {
        let c = Camera::nova(45.0, 30.0, 10.0, [0.0; 3], pos2(100.0, 100.0), 0.0);
        let p = c.ponto([1.0, 0.0, 0.0]);
        assert!((p.x - (100.0 + 7.071)).abs() < 0.01 && (p.y - (100.0 + 3.536)).abs() < 0.01);
        assert!(c.ponto([0.0, 0.0, 1.0]).y < 100.0, "z sobe na tela");
    }

    #[test]
    fn o_foco_fica_na_origem_e_o_perto_aumenta() {
        let c = Camera::nova(45.0, 30.0, 10.0, [5.0, 5.0, 0.0], pos2(50.0, 50.0), 0.05);
        assert_eq!(c.ponto([5.0, 5.0, 0.0]), pos2(50.0, 50.0));
        assert!(c.aumento([6.0, 6.0, 0.0]) > 1.0 && c.aumento([4.0, 4.0, 0.0]) < 1.0);
    }

    #[test]
    fn enquadrar_gira_em_torno_do_meio_do_desenho() {
        let pontos = [[10.0, 10.0, 0.0], [30.0, 10.0, 0.0], [30.0, 30.0, 0.0], [10.0, 30.0, 0.0]];
        let r = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 300.0));
        let c = Camera::enquadrar(&pontos, r, 20.0, AZIMUTE, ELEVACAO);
        assert_eq!(c.foco, [20.0, 20.0, 0.0]);
        let meio = c.ponto(c.foco);
        assert_eq!(c.girada(70.0, 10.0).ponto(c.foco), meio);
        assert!((meio - r.center()).length() < 0.01);
    }

    #[test]
    fn da_tela_desfaz_o_ponto() {
        let c = Camera::nova(70.0, 40.0, 12.0, [1.0, 2.0, 3.0], pos2(50.0, 80.0), 0.0);
        let d = egui::vec2(36.0, -18.0);
        let p = soma(c.foco, c.da_tela(d));
        assert!((c.ponto(p) - (c.origem + d)).length() < 0.01);
        assert!((c.ponto(c.na_altura(p, 0.5)) - c.ponto(p)).length() < 0.01);
    }

    #[test]
    fn enquadrar_cabe_no_retangulo() {
        let pontos = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0], [10.0, 10.0, 3.0]];
        let r = Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 300.0));
        let c = Camera::enquadrar(&pontos, r, 20.0, AZIMUTE, ELEVACAO);
        for p in pontos {
            assert!(r.shrink(19.0).contains(c.ponto(p)));
        }
    }
}
