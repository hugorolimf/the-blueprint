//! A fonte: Cascadia Mono (OFL, da Microsoft), a que a página mostra no Windows pela pilha `ui-monospace, "Cascadia Mono", Consolas…`
//! (o Edge não conhece `ui-monospace`). Embutida, para a interface ficar igual em qualquer sistema. Três pesos tirados da fonte variável
//! (como o Windows a desenha): o normal (400, corpo), o médio (500, títulos) e o negrito (700).

use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::{Arc, OnceLock};

/// O avanço de cada letra, em múltiplos do tamanho (1200/2048).
pub const AVANCO: f32 = 0.585_937_5;
/// O quanto a linha de base fica abaixo do topo da letra, em múltiplos do tamanho (a subida da fonte, 1900/2048).
pub const SUBIDA: f32 = 0.927_734_4;
/// A altura da letra do topo ao pé (subida + descida, 2380/2048): a caixa que o CSS centraliza na linha.
pub const ALTURA: f32 = 1.162_109_4;

#[derive(Clone, Copy)]
pub enum Peso {
    Normal,
    Medio,
    Negrito,
}

/// A família do peso. Os nomes ficam guardados: cada texto pede a sua a cada quadro, e montar o nome de novo alocaria toda vez (o clone só soma no contador).
pub fn familia(peso: Peso) -> FontFamily {
    static MEDIO: OnceLock<FontFamily> = OnceLock::new();
    static NEGRITO: OnceLock<FontFamily> = OnceLock::new();
    match peso {
        Peso::Normal => FontFamily::Monospace,
        Peso::Medio => MEDIO.get_or_init(|| FontFamily::Name("medio".into())).clone(),
        Peso::Negrito => NEGRITO.get_or_init(|| FontFamily::Name("negrito".into())).clone(),
    }
}

pub fn instalar(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    let fontes: [(&str, &'static [u8]); 3] = [
        ("cascadia", include_bytes!("../assets/fontes/CascadiaMono-Regular.ttf")),
        ("cascadia-medio", include_bytes!("../assets/fontes/CascadiaMono-Medium.ttf")),
        ("cascadia-negrito", include_bytes!("../assets/fontes/CascadiaMono-Bold.ttf")),
    ];
    for (nome, bytes) in fontes {
        f.font_data.insert(nome.into(), Arc::new(FontData::from_static(bytes)));
    }
    // a nossa primeiro; as fontes de reserva do egui (símbolos, emojis) vêm depois
    let reserva: Vec<String> = f.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let com = |nome: &str| std::iter::once(nome.to_string()).chain(["cascadia".to_string()]).chain(reserva.clone()).collect::<Vec<_>>();
    f.families.insert(FontFamily::Monospace, com("cascadia"));
    f.families.insert(FontFamily::Proportional, com("cascadia"));
    f.families.insert(FontFamily::Name("medio".into()), com("cascadia-medio"));
    f.families.insert(FontFamily::Name("negrito".into()), com("cascadia-negrito"));
    ctx.set_fonts(f);
}
