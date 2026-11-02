//! O movimento do painel da ação, do diff, da revisão e dos cartões de detalhe: entrada em cascata, mistura de cor no hover e opacidade.
//! Tudo respeita `BLUEPRINT_UI_SEM_ANIMACAO` (aparece pronto) e só pede quadro enquanto algo anda: terminou, o app volta a dormir.

use crate::anima;
use egui::{Context, Id, Painter};
use std::hash::Hash;

/// `cubic-bezier(.32,.72,0,1)`: sai rápido e assenta devagar (o `ease` do anatomy).
pub const EASE: [f32; 4] = [0.32, 0.72, 0.0, 1.0];
/// Quanto cada bloco espera pelo anterior, e quanto dura a chegada de um bloco.
const PASSO_MS: f32 = 70.0;
const CHEGADA_MS: f32 = 340.0;

/// `BLUEPRINT_UI_SEM_ANIMACAO=1`: tudo aparece pronto.
pub fn desligado() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    return std::env::var_os("BLUEPRINT_UI_SEM_ANIMACAO").is_some();
    #[cfg(target_arch = "wasm32")]
    false
}

/// `EASE` como função solta, para o `animate_bool_with_time_and_easing` do egui.
pub fn suave(x: f32) -> f32 {
    anima::curva(EASE, x)
}

/// Milissegundos desde que `chave` apareceu na tela. Recomeça quando ela some e volta (não foi desenhada no quadro anterior).
/// Chame uma vez por quadro e por chave. Desligado, devolve um tempo que já acabou tudo.
pub fn desde_que_abriu(ctx: &Context, chave: impl Hash + std::fmt::Debug) -> f32 {
    if desligado() {
        return 1.0e6;
    }
    let (id, agora, quadro) = (Id::unique(("plano-entrada", chave)), ctx.input(|i| i.time), ctx.cumulative_frame_nr());
    let antes: Option<(f64, u64)> = ctx.data(|d| d.get_temp(id));
    let inicio = match antes {
        Some((inicio, visto)) if visto + 1 >= quadro => inicio,
        _ => agora,
    };
    ctx.data_mut(|d| d.insert_temp(id, (inicio, quadro)));
    ((agora - inicio) * 1000.0) as f32
}

/// O quanto (0 a 1) do bloco `indice` já chegou, `ms` depois da abertura. Pede quadro enquanto algum bloco ainda anda.
pub fn cascata(ctx: &Context, ms: f32, indice: usize, atraso_extra: f32) -> f32 {
    let atraso = indice as f32 * PASSO_MS + atraso_extra;
    if ms < atraso + CHEGADA_MS {
        ctx.request_repaint();
    }
    suave((ms - atraso) / CHEGADA_MS)
}

/// De 0 a 1 em `seg` segundos na direção de `alvo`, com o `EASE`. Sem animação, já no alvo.
pub fn mistura(ctx: &Context, id: impl Hash + std::fmt::Debug, alvo: bool, seg: f32) -> f32 {
    if desligado() {
        return if alvo { 1.0 } else { 0.0 };
    }
    ctx.animate_bool_with_time_and_easing(Id::unique(("plano-mistura", id)), alvo, seg, suave)
}

/// O hover de uma peça: 140 ms.
pub fn hover(ctx: &Context, id: impl Hash + std::fmt::Debug, sobre: bool) -> f32 {
    mistura(ctx, id, sobre, 0.14)
}

/// Uma cópia do pincel com a opacidade multiplicada por `a`: o que se desenha nela chega aos poucos.
pub fn com_alfa(p: &Painter, a: f32) -> Painter {
    let mut q = p.clone();
    q.multiply_opacity(a);
    q
}

/// Um número que vai e volta entre 0 e 1 a cada `periodo` segundos (respirar, brilhar). Pede quadro a ~25/s.
pub fn pulso(ctx: &Context, periodo: f64) -> f32 {
    if desligado() {
        return 0.5;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(40));
    let t = ctx.input(|i| i.time) / periodo;
    (0.5 - 0.5 * (t * std::f64::consts::TAU).cos()) as f32
}

/// Como `pulso`, mas só nos primeiros `ate_s` segundos de `chave` na tela: chama a atenção e descansa, e o quadro volta a dormir.
pub fn pulso_breve(ctx: &Context, chave: impl Hash + std::fmt::Debug, periodo: f64, ate_s: f32) -> f32 {
    if desde_que_abriu(ctx, ("pulso", chave)) > ate_s * 1000.0 {
        return 0.35;
    }
    pulso(ctx, periodo)
}

/// Uma fase de 0 a 1 que só anda (varredura do esqueleto, barra indeterminada). Pede quadro a ~25/s.
pub fn fase(ctx: &Context, periodo: f64) -> f32 {
    if desligado() {
        return 0.4;
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(40));
    ((ctx.input(|i| i.time) / periodo).fract()) as f32
}

/// Quando algo aconteceu agora (um clique): guarda o instante de `chave`.
pub fn marcar_instante(ctx: &Context, chave: impl Hash + std::fmt::Debug) {
    let agora = ctx.input(|i| i.time);
    ctx.data_mut(|d| d.insert_temp(Id::unique(("plano-instante", chave)), agora));
    ctx.request_repaint();
}

/// O quanto (1 a 0) do clarão de `chave` ainda dura, em `seg` segundos. Pede quadro enquanto dura.
pub fn clarao(ctx: &Context, chave: impl Hash + std::fmt::Debug, seg: f32) -> f32 {
    let Some(inicio) = ctx.data(|d| d.get_temp::<f64>(Id::unique(("plano-instante", chave)))) else { return 0.0 };
    let t = ((ctx.input(|i| i.time) - inicio) as f32) / seg;
    if t >= 1.0 || desligado() {
        return 0.0;
    }
    ctx.request_repaint();
    1.0 - suave(t.max(0.0))
}
