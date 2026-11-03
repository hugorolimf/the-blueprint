//! O olhar sobre o palco: o quanto está perto e o ponto do chão que fica no meio da vista. A roda aproxima em torno do ponteiro, o outro botão arrastado desloca e o clique numa peça voa até ela; os três mexem no mesmo olhar, então a roda sempre traz de volta.
//! É em torno desse ponto que a obra gira: o que está no meio da vista fica no meio. O giro mora aqui também (arrastar com o botão principal). Vale para a planta e para a vista expandida, que é o mesmo palco.
//! Com o ponteiro no palco, a roda é só do zoom, e a página rola fora dele. A rolagem que já vinha descendo a página não vira zoom no caminho: segue até a roda parar.

use crate::anatomia::{camera_viva, Camera, V3};
use egui::{vec2, Context, Id, PointerButton, Pos2, Response, Vec2};

pub const MIN: f32 = 0.55;
pub const MAX: f32 = 7.0;
/// O alcance de quem anda solto: a vista expandida põe a pilha onde o enquadramento quer, mesmo longe do meio da chapa.
pub const SOLTO: Vec2 = Vec2::INFINITY;
/// Graus por pixel: a borda da obra acompanha a mão.
const GIRO_POR_PIXEL: f32 = 0.2;
const ELEVACAO_MENOS: f32 = -18.0;
/// Até quase de cima: a planta baixa.
const ELEVACAO_MAIS: f32 = 58.0;
const SEG_DESVIRAR: f32 = 0.6;
const POR_PIXEL: f32 = 0.0024;
const SEG_RODA: f32 = 0.12;
const SEG_VOO: f32 = 0.7;
/// Sem roda por este tempo, a rolagem da página acabou.
const PAUSA_DA_RODA: f64 = 0.3;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Olhar {
    k: f32,
    /// do foco enquadrado até o meio da vista, em x e y do mundo
    desvio: Vec2,
    seg: f32,
}

impl Default for Olhar {
    fn default() -> Self {
        Olhar { k: 1.0, desvio: Vec2::ZERO, seg: SEG_RODA }
    }
}

fn ler(ctx: &Context, id: Id) -> Olhar {
    ctx.data(|d| d.get_temp::<Olhar>(id)).unwrap_or_default()
}

/// De perto, o olhar anda até a borda de `alcance`; ao voltar para o enquadramento, volta ao meio. `SOLTO` não prende.
fn prende(d: Vec2, k: f32, alcance: Vec2) -> Vec2 {
    if !alcance.x.is_finite() {
        return d;
    }
    let folga = alcance * ((k - 1.0) * 2.0).clamp(0.0, 1.0);
    vec2(d.x.clamp(-folga.x, folga.x), d.y.clamp(-folga.y, folga.y))
}

/// O desvio que põe no meio da vista o ponto `p`, ou o que aparece no lugar dele, lá na altura do foco.
fn desvio_ate(base: &Camera, p: V3) -> Vec2 {
    let q = base.na_altura(p, base.foco[2]);
    vec2(q[0] - base.foco[0], q[1] - base.foco[1])
}

fn andou(base: &Camera, o: Olhar, tela: Vec2, k: f32, alcance: Vec2) -> Olhar {
    let d = base.da_tela(tela);
    let p = [base.foco[0] + o.desvio.x + d[0], base.foco[1] + o.desvio.y + d[1], base.foco[2] + d[2]];
    Olhar { k, desvio: prende(desvio_ate(base, p), k, alcance), seg: SEG_RODA }
}

/// O olhar com o zoom `fator`, sem mexer no que está sob o ponteiro `m` (relativo à origem da câmera).
fn com_zoom(base: &Camera, o: Olhar, fator: f32, m: Vec2, alcance: Vec2) -> Olhar {
    let k = (o.k * fator).clamp(MIN, MAX);
    andou(base, o, m * (1.0 / o.k - 1.0 / k), k, alcance)
}

/// Lê a roda e o outro botão arrastado. A rolagem sobre o palco nunca chega à página.
pub fn gesto(r: &Response, id: Id, base: &Camera, alcance: Vec2) {
    let ctx = &r.ctx;
    let id_pagina = Id::unique("plano-palco-pagina-rolando");
    let (roda, pinca, agora) = ctx.input(|i| (i.smooth_scroll_delta(), i.zoom_delta(), i.time));
    let girou = roda != Vec2::ZERO;
    if !r.contains_pointer() {
        if girou {
            ctx.data_mut(|d| d.insert_temp(id_pagina, agora));
        }
        return;
    }
    if girou && ctx.data(|d| d.get_temp::<f64>(id_pagina)).is_some_and(|t| agora - t < PAUSA_DA_RODA) {
        ctx.data_mut(|d| d.insert_temp(id_pagina, agora));
        return;
    }
    ctx.input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO);
    let fator = pinca * (roda.y * POR_PIXEL).exp();
    if fator != 1.0 {
        let m = ctx.pointer_hover_pos().unwrap_or(base.origem) - base.origem;
        let novo = com_zoom(base, ler(ctx, id), fator, m, alcance);
        ctx.data_mut(|d| d.insert_temp(id, novo));
        ctx.request_repaint();
    }
    if r.dragged_by(PointerButton::Secondary) || r.dragged_by(PointerButton::Middle) {
        let o = ler(ctx, id);
        let novo = andou(base, o, -r.drag_delta() / o.k, o.k, alcance);
        ctx.data_mut(|d| d.insert_temp(id, novo));
        ctx.request_repaint();
    }
}

/// Voa até `ponto` em `seg` segundos. É só outro olhar: a roda continua valendo a partir dele.
pub fn focar(ctx: &Context, id: Id, base: &Camera, ponto: V3, k: f32, alcance: Vec2, seg: f32) {
    // quem nunca foi desenhado começa enquadrado, para o voo sair de algum lugar
    let _ = aplicar(ctx, id, *base);
    let k = k.clamp(MIN, MAX);
    ctx.data_mut(|d| d.insert_temp(id, Olhar { k, desvio: prende(desvio_ate(base, ponto), k, alcance), seg }));
    ctx.request_repaint();
}

/// O quanto o olhar está perto, para onde ele está indo.
pub fn perto(ctx: &Context, id: Id) -> f32 {
    ler(ctx, id).k
}

/// A câmera com o olhar. O voo usa outra câmera só como carona: anda a escala em proporção e o foco em linha reta. O giro já vem em `base` e responde na hora.
pub fn aplicar(ctx: &Context, id: Id, base: Camera) -> Camera {
    let o = ler(ctx, id);
    let voo = camera_viva(ctx, id.with("voo"), Camera::nova(0.0, 0.0, o.k, [o.desvio.x, o.desvio.y, 0.0], Pos2::ZERO, 0.0), o.seg);
    let foco = [base.foco[0] + voo.foco[0], base.foco[1] + voo.foco[1], base.foco[2]];
    Camera::nova(base.azimute(), base.elevacao(), base.escala * voo.escala, foco, base.origem, base.perspectiva)
}

/// De volta ao enquadramento, voando.
pub fn zerar(ctx: &Context, id: Id) {
    ctx.data_mut(|d| d.insert_temp(id, Olhar { seg: SEG_VOO, ..Olhar::default() }));
    ctx.request_repaint();
}

fn id_orbita() -> Id {
    Id::unique("plano-obra-orbita")
}

/// Gira a obra: o arrastar do mouse e o passo `obra-orbita` do roteiro chegam aqui. O giro entra na hora.
pub fn orbitar(ctx: &Context, az: f32, el: f32) {
    let [a0, e0, _] = ctx.data(|d| d.get_temp::<[f32; 3]>(id_orbita())).unwrap_or_default();
    let volta = (a0 + az + 180.0).rem_euclid(360.0) - 180.0;
    ctx.data_mut(|d| d.insert_temp(id_orbita(), [volta, (e0 + el).clamp(ELEVACAO_MENOS, ELEVACAO_MAIS), 0.0]));
}

/// O arrastar com o botão principal: o que está na frente anda junto com a mão.
pub fn arrastar(r: &Response) {
    if r.dragged_by(PointerButton::Primary) {
        let d = r.drag_delta() * GIRO_POR_PIXEL;
        orbitar(&r.ctx, d.x, d.y);
        r.ctx.request_repaint();
    }
}

/// De volta ao ângulo de sempre, devagar.
pub fn desvirar(ctx: &Context) {
    ctx.data_mut(|d| d.insert_temp(id_orbita(), [0.0, 0.0, SEG_DESVIRAR]));
}

/// O quanto a obra está girada neste quadro.
pub fn orbita(ctx: &Context) -> [f32; 2] {
    let [az, el, seg] = ctx.data(|d| d.get_temp::<[f32; 3]>(id_orbita())).unwrap_or_default();
    let v = camera_viva(ctx, id_orbita().with("voo"), Camera::nova(az, el, 1.0, [0.0; 3], Pos2::ZERO, 0.0), seg);
    [v.azimute(), v.elevacao()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::pos2;

    const LONGE: Vec2 = vec2(1000.0, 1000.0);

    fn base() -> Camera {
        Camera::nova(70.0, 40.0, 10.0, [2.0, 3.0, 1.0], pos2(400.0, 200.0), 0.0)
    }

    fn vista(o: Olhar) -> Camera {
        let b = base();
        Camera::nova(b.azimute(), b.elevacao(), b.escala * o.k, [b.foco[0] + o.desvio.x, b.foco[1] + o.desvio.y, b.foco[2]], b.origem, 0.0)
    }

    #[test]
    fn o_que_esta_sob_o_ponteiro_nao_sai_do_lugar() {
        let antes = Olhar { k: 2.0, desvio: vec2(1.0, -1.0), seg: SEG_RODA };
        let m = vec2(120.0, -60.0);
        let depois = com_zoom(&base(), antes, 1.5, m, LONGE);
        let sob = base().da_tela(m / antes.k);
        let p = [base().foco[0] + antes.desvio.x + sob[0], base().foco[1] + antes.desvio.y + sob[1], base().foco[2] + sob[2]];
        assert!((vista(depois).ponto(p) - (base().origem + m)).length() < 0.05);
    }

    #[test]
    fn afastar_de_novo_volta_ao_enquadramento() {
        let perto = com_zoom(&base(), Olhar::default(), 3.0, vec2(100.0, 50.0), vec2(20.0, 20.0));
        assert_ne!(perto.desvio, Vec2::ZERO);
        let longe = com_zoom(&base(), perto, 0.1, vec2(-100.0, 0.0), vec2(20.0, 20.0));
        assert_eq!(longe.k, MIN);
        assert_eq!(longe.desvio, Vec2::ZERO);
    }

    #[test]
    fn o_ponto_focado_fica_no_meio_da_vista_e_gira_no_lugar() {
        let ponto = [8.0, -4.0, 2.5];
        let o = Olhar { k: 3.0, desvio: desvio_ate(&base(), ponto), seg: SEG_VOO };
        assert!((vista(o).ponto(ponto) - base().origem).length() < 0.01);
        let meio = vista(o).ponto([base().foco[0] + o.desvio.x, base().foco[1] + o.desvio.y, base().foco[2]]);
        assert_eq!(meio, base().origem);
    }
}
