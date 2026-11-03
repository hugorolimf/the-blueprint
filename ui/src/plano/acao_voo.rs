//! O voo até a planta: o que se clica em outro lugar vai até a sua construção, para os olhos acompanharem de onde veio e para onde foi.
//! Quem clica guarda de onde sai; a tela de destino, ao desenhar o seu lugar, pergunta `em_voo` e desenha a peça por cima de tudo. Sem animação, não há voo.

use super::super::acao_movimento::{desligado, suave};
use egui::style::ScrollAnimation;
use egui::{Align, Context, Id, LayerId, Order, Painter, Pos2, Rangef, Rect, Ui};

const DURACAO: f64 = 0.5;

#[derive(Clone, Debug, Default)]
struct Voo {
    de: Pos2,
    k: f32,
    alvo: String,
    inicio: f64,
}

/// Onde a peça está agora e em que escala.
#[derive(Clone, Copy, Debug)]
pub struct Agora {
    pub o: Pos2,
    pub k: f32,
}

fn id(acao: &str) -> Id {
    Id::unique(("plano-voo", acao))
}

/// Solta a peça de `de`, na escala `k`, rumo a `alvo`.
pub fn decolar(ctx: &Context, acao: &str, de: Pos2, k: f32, alvo: &str) {
    if desligado() {
        return;
    }
    let voo = Voo { de, k, alvo: alvo.to_string(), inicio: ctx.input(|i| i.time) };
    ctx.data_mut(|d| d.insert_temp(id(acao), voo));
    ctx.request_repaint();
}

/// Se `alvo` está voando, onde está a caminho de `para`, na escala `k_para`.
pub fn em_voo(ctx: &Context, acao: &str, alvo: &str, para: Pos2, k_para: f32) -> Option<Agora> {
    let voo = ctx.data(|d| d.get_temp::<Voo>(id(acao))).filter(|v| v.alvo == alvo)?;
    let t = (ctx.input(|i| i.time) - voo.inicio) / DURACAO;
    if t >= 1.0 {
        ctx.data_mut(|d| d.remove_temp::<Voo>(id(acao)));
        return None;
    }
    ctx.request_repaint();
    let s = suave(t.max(0.0) as f32);
    Some(Agora { o: voo.de.lerp(para, s), k: voo.k + (k_para - voo.k) * s })
}

/// O pincel de cima de tudo, onde a peça voa.
pub fn camada(ctx: &Context) -> Painter {
    ctx.layer_painter(LayerId::new(Order::Foreground, Id::unique("plano-voo-camada")))
}

/// O começo de um caminho, de 0 a 1 do seu comprimento: é como um fio se desenha.
pub fn trecho(pts: &[Pos2], t: f32) -> Vec<Pos2> {
    if t >= 1.0 || pts.len() < 2 {
        return pts.to_vec();
    }
    let total: f32 = pts.windows(2).map(|p| p[0].distance(p[1])).sum();
    let mut falta = total * t.max(0.0);
    let mut saida = vec![pts[0]];
    for p in pts.windows(2) {
        let d = p[0].distance(p[1]);
        if d >= falta {
            saida.push(p[0].lerp(p[1], if d > 0.0 { falta / d } else { 0.0 }));
            break;
        }
        saida.push(p[1]);
        falta -= d;
    }
    saida
}

/// Leva `r` para a vista. Sem `alinhar`, só rola se ela estiver fora.
pub fn rolar(ui: &Ui, r: Rect, alinhar: Option<Align>) {
    let anda = if desligado() { ScrollAnimation::none() } else { ScrollAnimation::new(1800.0, Rangef::new(0.3, 0.65)) };
    ui.scroll_to_rect_animation(r, alinhar, anda);
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    #[test]
    fn o_trecho_para_no_meio_do_segmento_certo() {
        let pts = [pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 10.0)];
        assert_eq!(trecho(&pts, 0.25), vec![pos2(0.0, 0.0), pos2(5.0, 0.0)]);
        assert_eq!(trecho(&pts, 0.75), vec![pos2(0.0, 0.0), pos2(10.0, 0.0), pos2(10.0, 5.0)]);
        assert_eq!(trecho(&pts, 1.0), pts.to_vec());
    }
}
