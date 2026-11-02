//! As cores do The Blueprint: as variáveis de `web/css/tokens.css` (`--paper`, `--ink`, `--mid`, `--lo`, `--faint`, `--erro`).
//! O app só tem este tema (a planta azul); o papel branco da web é só de impressão.

use egui::{Color32, Stroke, Visuals};

const fn rgb(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

pub const PAPEL: Color32 = rgb(0x0c3a78);
pub const TINTA: Color32 = rgb(0xeaf4ff);
pub const MEDIO: Color32 = rgb(0x9cc4ee);
pub const BAIXO: Color32 = rgb(0x5b93d0);
pub const TENUE: Color32 = rgb(0x2f66aa);
pub const ERRO: Color32 = rgb(0xffb4a8);

// As cores do Planejar (`plano/`), as mesmas do protótipo em `design/planejar`.
pub const VERMELHO: Color32 = rgb(0xff8f80);
pub const AMBAR: Color32 = rgb(0xffcf6b);
pub const VERDE: Color32 = rgb(0x8be3b5);
pub const AZUL: Color32 = rgb(0xbfe0ff);
pub const CARTAO: Color32 = rgb(0x082b5e);
pub const CAMPO: Color32 = rgb(0x081d3c);
pub const QUADRA: Color32 = rgb(0x14509b);

/// Uma cor `#rrggbb` com transparência `aa` (0 a 255), como o `#0a3069dd` do CSS.
pub fn rgba(c: u32, aa: u8) -> Color32 {
    Color32::from_rgba_unmultiplied((c >> 16) as u8, (c >> 8) as u8, c as u8, aa)
}

/// Liga o tema no egui: cantos retos (a planta não tem arredondado), linhas finas e as cores acima.
pub fn aplicar(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    // o Chromium desenha o texto claro sobre fundo escuro com traço mais fino que o padrão escuro do egui
    v.text_options.color_transfer_function = egui::epaint::FontColorTransferFunction::Off;
    v.panel_fill = rgb(0x0b3268);
    v.window_fill = rgb(0x082b5e);
    v.extreme_bg_color = rgb(0x082b5e);
    v.override_text_color = Some(TINTA);
    v.hyperlink_color = MEDIO;
    v.selection.bg_fill = TENUE;
    v.selection.stroke = Stroke::new(1.0, TINTA);
    v.window_stroke = Stroke::new(1.0, MEDIO);
    v.window_corner_radius = 0.into();
    v.menu_corner_radius = 0.into();
    v.window_shadow = egui::Shadow { offset: [0, 20], blur: 56, spread: 0, color: rgba(0x02132e, 0xe6) };
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
        w.corner_radius = 0.into();
        w.bg_stroke = Stroke::new(1.0, MEDIO);
        w.fg_stroke = Stroke::new(1.0, TINTA);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, TENUE);
    v.widgets.inactive.weak_bg_fill = rgb(0x0c3a78);
    v.widgets.hovered.weak_bg_fill = TINTA;
    v.widgets.active.weak_bg_fill = TINTA;
    ctx.set_visuals(v);
}
