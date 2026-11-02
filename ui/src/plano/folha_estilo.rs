//! O estilo e o movimento da folha do Planejar: as cores de situação, a curva `cubic-bezier(.32,.72,0,1)` do Anatomy, o pulso e a passagem do mouse.
//! Tudo que anda passa por aqui para respeitar `BLUEPRINT_UI_SEM_ANIMACAO` e só pedir quadro novo enquanto algo se mexe.

use crate::anima;
use crate::tema::{AMBAR, BAIXO, MEDIO, QUADRA, VERDE};
use egui::{Color32, Context, Id};
use std::sync::OnceLock;
use std::time::Duration;

/// O que está feito: verde-água.
pub const FEITO: Color32 = VERDE;
/// O que está andando: um azul vivo, que se destaca do papel azul da planta.
pub const ANDANDO: Color32 = Color32::from_rgb(0x6c, 0xb4, 0xff);
/// O que espera alguém: âmbar.
pub const ESPERA: Color32 = AMBAR;
/// O que ainda não começou: só o contorno.
pub const PLANEJADO: Color32 = MEDIO;
/// O fundo dos chips de número e das etiquetas.
pub const CHIP: Color32 = QUADRA;
/// O traço fino das divisórias.
pub const FIO: Color32 = BAIXO;

/// A curva do Anatomy (`ease`): sai rápido e chega devagar.
const CURVA: [f32; 4] = [0.32, 0.72, 0.0, 1.0];

/// `BLUEPRINT_UI_SEM_ANIMACAO=1` desliga todo movimento: o que seria animado já aparece no destino.
pub fn ligado() -> bool {
    static LIGADO: OnceLock<bool> = OnceLock::new();
    *LIGADO.get_or_init(|| {
        #[cfg(not(target_arch = "wasm32"))]
        return std::env::var_os("BLUEPRINT_UI_SEM_ANIMACAO").is_none();
        #[cfg(target_arch = "wasm32")]
        true
    })
}

/// A curva do Anatomy no instante `x` (0 a 1).
pub fn suave(x: f32) -> f32 {
    anima::curva(CURVA, x)
}

/// Quanto (0 a 1) do estado `alvo` já chegou em `id`: anda em `dur` segundos pela curva do Anatomy. Sem animação, é 0 ou 1 na hora.
pub fn tempo(ctx: &Context, id: Id, alvo: bool, dur: f32) -> f32 {
    if ligado() {
        ctx.animate_bool_with_time_and_easing(id, alvo, dur, suave)
    } else {
        f32::from(u8::from(alvo))
    }
}

/// A passagem do mouse: 0 a 1 em 160 ms.
pub fn sobre(ctx: &Context, id: Id, em_cima: bool) -> f32 {
    tempo(ctx, id, em_cima, 0.16)
}

/// O relógio do pulso, de 0 a 1 e de volta a cada `periodo` segundos. Pede o próximo quadro: só chame de quem está na tela e pulsa.
pub fn pulso(ctx: &Context, periodo: f32) -> f32 {
    if !ligado() {
        return 0.5;
    }
    ctx.request_repaint_after(Duration::from_millis(66));
    let t = ctx.input(|i| i.time) as f32;
    0.5 - 0.5 * (std::f32::consts::TAU * t / periodo).cos()
}

/// `a` até `b` em `t` (0 a 1).
pub fn misturar(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t.clamp(0.0, 1.0)).round() as u8;
    Color32::from_rgba_unmultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

/// A cor `c` com a transparência `a` (0 a 1), sem depender de ela já ser opaca.
pub fn com_alfa(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (a.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// Onde a linha de base fica para o texto de tamanho `tam` parecer no meio de `y` (a altura das maiúsculas da Cascadia).
pub fn meio(tam: f32) -> f32 {
    tam * 0.35
}

/// Há quantos milissegundos esta folha (`chave`) entrou na tela. Recomeça quando ela volta depois de ter ficado fora por uns quadros.
fn relogio(ctx: &Context, chave: &str) -> f32 {
    let id = Id::unique(("folha-entrada", chave));
    let (agora, quadro) = (ctx.input(|i| i.time), ctx.cumulative_frame_nr());
    let guardado: Option<(f64, u64)> = ctx.data(|d| d.get_temp(id));
    let inicio = guardado.filter(|(_, q)| q + 3 >= quadro).map_or(agora, |(i, _)| i);
    ctx.data_mut(|d| d.insert_temp(id, (inicio, quadro)));
    ((agora - inicio) * 1000.0) as f32
}

/// A entrada de uma peça da folha: 0 a 1, começando `atraso` ms depois de a folha abrir e levando `dur` ms.
pub fn entrada(ctx: &Context, chave: &str, atraso: f32, dur: f32) -> f32 {
    if !ligado() {
        return 1.0;
    }
    let t = ((relogio(ctx, chave) - atraso) / dur).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
    }
    suave(t)
}

/// A vez de cada linha na entrada em cascata: 0 a 1, com a linha `i` começando 32 ms depois da anterior (até a 24ª). Só pede quadro enquanto a última não chegou.
pub fn cascata(ctx: &Context, chave: &str, n: usize) -> impl Fn(usize) -> f32 {
    let ms = relogio(ctx, chave);
    let ligada = ligado() && ms < n.min(24) as f32 * 32.0 + 520.0;
    if ligada {
        ctx.request_repaint();
    }
    move |i: usize| if ligada { suave(((ms - i.min(24) as f32 * 32.0) / 480.0).clamp(0.0, 1.0)) } else { 1.0 }
}
