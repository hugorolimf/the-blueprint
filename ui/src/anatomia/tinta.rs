//! A tinta do Anatomy no tema da planta azul: o traço, o vinco e as faixas de luz. O kit tem `hi/mid/lo/faint`; aqui são `TINTA/MEDIO/BAIXO/TENUE`.
//! A luz é em faixas (`BANDAS`, as mesmas do kit): é o que faz parecer desenho, e não render.

use crate::tema::{CAMPO, CARTAO, MEDIO, PAPEL, QUADRA, TINTA};
use egui::{Color32, Stroke};

const BANDAS: [f32; 4] = [0.0, 0.2, 0.42, 0.62];

fn junta(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgba_premultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

/// Como um sólido é pintado.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tinta {
    /// as cinco faixas de tom, da sombra para a luz
    pub tons: [Color32; 5],
    pub traco: Stroke,
    /// as arestas de dentro; `Stroke::NONE` para não ter
    pub vinco: Stroke,
    /// só as arestas, tracejadas: o que ainda não existe
    pub tracejado: bool,
    /// 0 a 1: para entrar e sair de cena
    pub alfa: f32,
}

impl Tinta {
    /// O sólido comum, de `escura` a `clara`.
    pub fn de(escura: Color32, clara: Color32, traco: Color32) -> Tinta {
        let tons = [0.0, 0.25, 0.5, 0.75, 1.0].map(|t| junta(escura, clara, t));
        Tinta { tons, traco: Stroke::new(1.1, traco), vinco: Stroke::new(0.7, Self::mais_fraca(traco, 0.45)), tracejado: false, alfa: 1.0 }
    }

    fn mais_fraca(c: Color32, a: f32) -> Color32 {
        c.gamma_multiply(a)
    }

    /// A parede de uma construção.
    pub fn parede() -> Tinta {
        Tinta::de(CAMPO, QUADRA, TINTA)
    }

    /// O chão: quase o papel, com o traço discreto.
    pub fn chao() -> Tinta {
        Tinta::de(CAMPO, PAPEL, MEDIO)
    }

    /// Uma peça de `cor` que informa um estado.
    pub fn estado(cor: Color32) -> Tinta {
        Tinta::de(junta(CAMPO, cor, 0.25), junta(CARTAO, cor, 0.8), cor)
    }

    /// O que ainda não existe: só o traço, tracejado, sobre um véu do papel.
    pub fn fantasma(cor: Color32) -> Tinta {
        Tinta { tons: [PAPEL.gamma_multiply(0.55); 5], traco: Stroke::new(1.0, cor), vinco: Stroke::new(0.7, cor.gamma_multiply(0.35)), tracejado: true, alfa: 1.0 }
    }

    pub fn com_alfa(self, a: f32) -> Tinta {
        Tinta { alfa: self.alfa * a, ..self }
    }

    pub fn com_traco(self, cor: Color32, largura: f32) -> Tinta {
        Tinta { traco: Stroke::new(largura, cor), ..self }
    }

    /// O tom de uma face, pela luz que recebe (`dot` da normal com a luz).
    pub fn tom(&self, luz: f32) -> Color32 {
        let faixa = BANDAS.iter().take_while(|b| luz >= **b).count();
        self.tons[faixa].gamma_multiply(self.alfa)
    }
}
