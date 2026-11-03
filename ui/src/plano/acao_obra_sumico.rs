//! O sumiço: o que está atrás da coluna de texto se dissolve, para ela ser lida direto sobre a planta, sem caixa por cima.
//! Quem desenha a coluna grava onde ela está; o palco lê no quadro seguinte e apaga cada desenho pelo quanto ele fica embaixo dela. A chapa e o papel ficam: é o mesmo desenho.

use super::super::modelo::Acao;
use crate::anatomia::{Camera, V3};
use egui::{Context, Id, Rect};

/// A rampa antes da borda esquerda da coluna: nada corta de repente.
const RAMPA: f32 = 120.0;
/// O que está atrás da coluna ainda se nota de leve.
const RESTO: f32 = 0.05;
/// Abaixo disto, o desenho nem entra na cena.
pub const SUMIU: f32 = 0.02;
/// Abaixo disto, ele ainda se nota, mas não pega o clique.
pub const CLICA: f32 = 0.35;

/// A chave de um desenho que fica `fica`: o que quase sumiu não responde.
pub fn chave_se_clica(chave: u32, fica: f32) -> u32 {
    if fica < CLICA { 0 } else { chave }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Sumico {
    coluna: Option<Rect>,
    t: f32,
}

fn id(a: &Acao) -> Id {
    Id::unique(("plano-obra-sumico", &a.referencia))
}

impl Sumico {
    /// Lê a coluna do quadro anterior e limpa: a coluna que parou de ser desenhada para de apagar a obra. Uma chamada por quadro, no palco.
    pub fn de(ctx: &Context, a: &Acao) -> Sumico {
        ctx.data_mut(|d| d.remove_temp::<Sumico>(id(a))).unwrap_or_default()
    }

    /// Grava a coluna, em coordenadas de tela, com `t` de 0 (fechada) a 1 (aberta). Duas no mesmo quadro (ficha e mesa) valem como a soma.
    pub fn gravar(ctx: &Context, a: &Acao, coluna: Rect, t: f32) {
        let antes = ctx.data(|d| d.get_temp::<Sumico>(id(a))).unwrap_or_default();
        let junta = Sumico { coluna: Some(antes.coluna.map_or(coluna, |c| c.union(coluna))), t: antes.t.max(t) };
        ctx.data_mut(|d| d.insert_temp(id(a), junta));
    }

    /// O quanto fica de um desenho nesta caixa da tela: 1 inteiro, perto de 0 embaixo da coluna.
    pub fn fica_rect(&self, r: Rect) -> f32 {
        let Some(coluna) = self.coluna.filter(|c| c.y_range().intersects(r.y_range())) else {
            return 1.0;
        };
        // o lado direito pesa mais: é ele que entra primeiro embaixo do texto
        let s = ((r.center().x + r.width() * 0.25 - (coluna.left() - RAMPA)) / RAMPA).clamp(0.0, 1.0);
        let suave = s * s * (3.0 - 2.0 * s);
        1.0 - suave * (1.0 - RESTO) * self.t.clamp(0.0, 1.0)
    }

    /// O mesmo, para pontos do mundo vistos por esta câmera.
    pub fn fica(&self, cam: &Camera, pontos: &[V3]) -> f32 {
        if self.coluna.is_none() || pontos.is_empty() {
            return 1.0;
        }
        let tela: Vec<_> = pontos.iter().map(|p| cam.ponto(*p)).collect();
        self.fica_rect(Rect::from_points(&tela))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2};

    fn caixa(x: f32) -> Rect {
        Rect::from_center_size(pos2(x, 100.0), vec2(40.0, 40.0))
    }

    #[test]
    fn so_some_o_que_fica_embaixo_da_coluna() {
        let s = Sumico { coluna: Some(Rect::from_min_max(pos2(500.0, 0.0), pos2(900.0, 400.0))), t: 1.0 };
        assert_eq!(s.fica_rect(caixa(100.0)), 1.0);
        assert!(s.fica_rect(caixa(700.0)) < 0.1);
        let na_rampa = s.fica_rect(caixa(430.0));
        assert!(na_rampa > 0.1 && na_rampa < 1.0);
    }

    #[test]
    fn sem_coluna_ou_fechada_nada_some() {
        assert_eq!(Sumico::default().fica_rect(caixa(700.0)), 1.0);
        let fechada = Sumico { coluna: Some(Rect::from_min_max(pos2(500.0, 0.0), pos2(900.0, 400.0))), t: 0.0 };
        assert_eq!(fechada.fica_rect(caixa(700.0)), 1.0);
    }
}
