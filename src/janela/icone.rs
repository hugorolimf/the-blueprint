//! O ícone do app (janela e bandeja): vem do próprio .exe, onde o `build.rs` põe `recursos/icone/icone.ico` (a folha de blueprint
//! com esquadro e compasso, desenhada por `scripts/icone/desenhar.mjs`). Um arquivo só para o Explorer, a janela e a bandeja.
use tao::platform::windows::IconExtWindows;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

/// A cor do papel do blueprint: é também o fundo da janela antes da página aparecer (sem flash branco).
pub const PAPEL: (u8, u8, u8, u8) = (0x0c, 0x3a, 0x78, 0xff);

/// O número do grupo de ícones no .exe (o `build.rs` grava o .ico com esse número).
const RECURSO: u16 = 1;

/// O ícone da janela; o Windows escolhe o tamanho da barra de título e do Alt+Tab dentro do .ico.
pub fn janela() -> Result<tao::window::Icon, String> {
    tao::window::Icon::from_resource(RECURSO, None).map_err(|e| e.to_string())
}

/// O ícone da bandeja, no tamanho pequeno da tela atual (16 px a 100%, 24 px a 150%).
pub fn bandeja() -> Result<tray_icon::Icon, String> {
    let lado = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16) as u32;
    tray_icon::Icon::from_resource(RECURSO, Some((lado, lado))).map_err(|e| e.to_string())
}
