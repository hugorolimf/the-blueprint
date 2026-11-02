//! O movimento do Planejar nas telas Agentes, «O que chegou» e na escrita ao vivo: entrada em cascata, chegada de cada peça, cor que escorre,
//! pulso e a faixa de brilho dos esqueletos. Tudo respeita `BLUEPRINT_UI_SEM_ANIMACAO` e só pede quadro novo enquanto algo anda.

use super::linha::Pincel;
use egui::emath::easing::cubic_out;
use egui::epaint::Mesh;
use egui::{pos2, vec2, Color32, Context, Id, Painter, Pos2, Rect, Shape, Ui};
use std::hash::Hash;
use std::time::Duration;

/// Cerca de 30 quadros por segundo: o bastante para um pulso sem acordar a máquina inteira.
const QUADRO: Duration = Duration::from_millis(33);
/// Quanto o cartão sobe ao entrar.
const SOBE: f32 = 10.0;
const PASSO_CASCATA: f64 = 0.055;
const DURACAO_ENTRADA: f64 = 0.42;
/// Se a tela ficou tanto tempo sem ser desenhada, ao voltar ela entra de novo.
const RETORNO_S: f64 = 2.5;
const MAX_CASCATA: usize = 14;

/// `BLUEPRINT_UI_SEM_ANIMACAO=1`: tudo aparece pronto e parado (o «movimento reduzido» da página).
pub fn ligado() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static LIGADO: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *LIGADO.get_or_init(|| std::env::var_os("BLUEPRINT_UI_SEM_ANIMACAO").is_none())
    }
    #[cfg(target_arch = "wasm32")]
    true
}

pub fn agora(ctx: &Context) -> f64 {
    ctx.input(|i| i.time)
}

/// Pede o próximo quadro, mas só enquanto algo anda.
pub fn vivo(ctx: &Context) {
    if ligado() {
        ctx.request_repaint_after(QUADRO);
    }
}

/// Uma volta de 0 a 1 a cada `periodo` segundos. Sem animação fica numa posição fixa e agradável.
pub fn fase(ctx: &Context, periodo: f64) -> f32 {
    if !ligado() {
        return 0.3;
    }
    vivo(ctx);
    (agora(ctx) % periodo / periodo) as f32
}

/// De 0 a 1 e de volta a 0, suave: o fôlego do pulso.
pub fn onda(fase: f32) -> f32 {
    0.5 - 0.5 * (fase * std::f32::consts::TAU).cos()
}

/// Marca que a tela está aberta neste quadro; passado um tempo sem ser vista, a cascata recomeça.
pub fn abrir(ctx: &Context, tela: &'static str) {
    let id = Id::unique(("mov-tela", tela));
    let agora = agora(ctx);
    let (inicio, visto) = ctx.data(|d| d.get_temp::<(f64, f64)>(id)).unwrap_or((agora, agora));
    let inicio = if agora - visto > RETORNO_S { agora } else { inicio };
    ctx.data_mut(|d| d.insert_temp(id, (inicio, agora)));
}

/// O quanto (0 a 1) a peça `i` da tela já entrou: cada uma chega um pouco depois da anterior.
pub fn entrada(ctx: &Context, tela: &'static str, i: usize) -> f32 {
    if !ligado() {
        return 1.0;
    }
    let Some((inicio, _)) = ctx.data(|d| d.get_temp::<(f64, f64)>(Id::unique(("mov-tela", tela)))) else {
        return 1.0;
    };
    let t = ((agora(ctx) - inicio - i.min(MAX_CASCATA) as f64 * PASSO_CASCATA) / DURACAO_ENTRADA).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
    }
    cubic_out(t as f32)
}

/// A chegada de uma peça que nasce enquanto a tela está aberta (uma ação preenchida, um balão novo): 0 a 1 em `dur` segundos desde que foi vista.
pub fn chegada(ctx: &Context, chave: impl Hash + std::fmt::Debug, dur: f64) -> f32 {
    if !ligado() {
        return 1.0;
    }
    let id = Id::unique(("mov-chegada", chave));
    let agora = agora(ctx);
    let nasceu = ctx.data_mut(|d| *d.get_temp_mut_or_insert_with::<f64>(id, || agora));
    let t = ((agora - nasceu) / dur.max(0.01)).clamp(0.0, 1.0);
    if t < 1.0 {
        ctx.request_repaint();
    }
    cubic_out(t as f32)
}

/// A cor que escorre até o alvo em vez de pular (a situação muda de âmbar para azul).
pub fn suavizar(ctx: &Context, chave: impl Hash + std::fmt::Debug, alvo: Color32) -> Color32 {
    if !ligado() {
        return alvo;
    }
    let id = Id::unique(("mov-cor", chave));
    let atual = ctx.data(|d| d.get_temp::<Color32>(id)).unwrap_or(alvo);
    let k = 1.0 - (-ctx.input(|i| i.stable_dt.min(0.1)) * 14.0).exp();
    let passo = |a: u8, b: u8| if (i32::from(a) - i32::from(b)).abs() <= 2 { b } else { (f32::from(a) + (f32::from(b) - f32::from(a)) * k).round() as u8 };
    let novo = Color32::from_rgba_premultiplied(passo(atual.r(), alvo.r()), passo(atual.g(), alvo.g()), passo(atual.b(), alvo.b()), passo(atual.a(), alvo.a()));
    ctx.data_mut(|d| d.insert_temp(id, novo));
    if novo != alvo {
        ctx.request_repaint();
    }
    novo
}

/// O número que anda até o valor novo (a contagem dos indicadores).
pub fn suave(ctx: &Context, chave: impl Hash + std::fmt::Debug, alvo: f32) -> f32 {
    if ligado() {
        ctx.animate_value_with_time(Id::unique(("mov-valor", chave)), alvo, 0.45)
    } else {
        alvo
    }
}

/// Um painter que apaga e sobe com `t` (0 a 1), e o pincel já deslocado. O cartão inteiro entra com ele.
pub struct Entra {
    p: Painter,
    o: Pos2,
}

impl Entra {
    pub fn nova(c: &Pincel, t: f32) -> Entra {
        Entra::com_alfa(c, t, 1.0)
    }

    /// Como `nova`, com a peça ainda mais apagada por `alfa` (o balão velho da conversa).
    pub fn com_alfa(c: &Pincel, t: f32, alfa: f32) -> Entra {
        let mut p = c.p.clone();
        p.multiply_opacity(t * alfa);
        Entra { p, o: c.o + vec2(0.0, (1.0 - t) * SOBE) }
    }

    pub fn pincel(&self) -> Pincel<'_> {
        Pincel { p: &self.p, o: self.o }
    }
}

/// Desenha widgets (botões) com a mesma opacidade da peça que está entrando.
pub fn com_opacidade<R>(ui: &mut Ui, t: f32, f: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope(|ui| {
        ui.set_opacity(t);
        f(ui)
    })
    .inner
}

/// A faixa de brilho que passa por `r` (o esqueleto de quem ainda vai chegar). `fase` de 0 a 1 leva a faixa de um lado ao outro.
pub fn varredura(p: &Painter, r: Rect, fase: f32, cor: Color32) {
    let p = p.with_clip_rect(r.intersect(p.clip_rect()));
    let largura = (r.width() * 0.45).max(36.0);
    let inclina = r.height() * 0.6;
    let x = r.left() - largura - inclina + (r.width() + largura + inclina) * fase;
    let mut m = Mesh::default();
    let tons = [Color32::TRANSPARENT, cor, Color32::TRANSPARENT];
    for (i, tom) in tons.into_iter().enumerate() {
        let dx = x + largura * i as f32 / 2.0;
        m.colored_vertex(pos2(dx + inclina, r.top()), tom);
        m.colored_vertex(pos2(dx, r.bottom()), tom);
    }
    for k in 0..2u32 {
        let a = k * 2;
        m.add_triangle(a, a + 1, a + 2);
        m.add_triangle(a + 1, a + 3, a + 2);
    }
    p.add(Shape::mesh(m));
}
