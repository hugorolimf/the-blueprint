//! A fonte: JetBrains Mono (OFL), a mesma que a página usa pela pilha `ui-monospace` neste sistema. Embutida, para a interface ficar igual
//! em qualquer sistema. Três pesos: o normal (corpo), o médio (500, títulos) e o negrito (700).

use egui::{FontData, FontDefinitions, FontFamily};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum Peso {
    Normal,
    Medio,
    Negrito,
}

pub fn familia(peso: Peso) -> FontFamily {
    match peso {
        Peso::Normal => FontFamily::Monospace,
        Peso::Medio => FontFamily::Name("medio".into()),
        Peso::Negrito => FontFamily::Name("negrito".into()),
    }
}

pub fn instalar(ctx: &egui::Context) {
    let mut f = FontDefinitions::default();
    let fontes: [(&str, &'static [u8]); 3] = [
        ("jb", include_bytes!("../assets/fontes/JetBrainsMono-Regular.ttf")),
        ("jb-medio", include_bytes!("../assets/fontes/JetBrainsMono-Medium.ttf")),
        ("jb-negrito", include_bytes!("../assets/fontes/JetBrainsMono-Bold.ttf")),
    ];
    for (nome, bytes) in fontes {
        f.font_data.insert(nome.into(), Arc::new(FontData::from_static(bytes)));
    }
    // a nossa primeiro; as fontes de reserva do egui (símbolos, emojis) vêm depois
    let reserva: Vec<String> = f.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    let com = |nome: &str| std::iter::once(nome.to_string()).chain(["jb".to_string()]).chain(reserva.clone()).collect::<Vec<_>>();
    f.families.insert(FontFamily::Monospace, com("jb"));
    f.families.insert(FontFamily::Proportional, com("jb"));
    f.families.insert(FontFamily::Name("medio".into()), com("jb-medio"));
    f.families.insert(FontFamily::Name("negrito".into()), com("jb-negrito"));
    ctx.set_fonts(f);
}
