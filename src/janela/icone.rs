//! O ícone do app (janela e bandeja): vem do próprio .exe, onde o `build.rs` põe `recursos/icone/icone.ico` (a folha de blueprint
//! com esquadro e compasso, desenhada por `scripts/icone/desenhar.mjs`). Um arquivo só para o Explorer, a janela e a bandeja.
#[cfg(windows)]
use tao::platform::windows::IconExtWindows;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

/// A cor do papel do blueprint: é também o fundo da janela antes da página aparecer (sem flash branco).
pub const PAPEL: (u8, u8, u8, u8) = (0x0c, 0x3a, 0x78, 0xff);

/// O número do grupo de ícones no .exe (o `build.rs` grava o .ico com esse número).
#[cfg(windows)]
const RECURSO: u16 = 1;

/// O ícone da janela; o Windows escolhe o tamanho da barra de título e do Alt+Tab dentro do .ico.
#[cfg(windows)]
pub fn janela() -> Result<tao::window::Icon, String> {
    tao::window::Icon::from_resource(RECURSO, None).map_err(|e| e.to_string())
}

/// O ícone da bandeja, no tamanho pequeno da tela atual (16 px a 100%, 24 px a 150%).
#[cfg(windows)]
pub fn bandeja() -> Result<tray_icon::Icon, String> {
    let lado = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16) as u32;
    tray_icon::Icon::from_resource(RECURSO, Some((lado, lado))).map_err(|e| e.to_string())
}

/// Fora do Windows não há recurso no executável: o mesmo `icone.ico` vai embutido e a imagem PNG do tamanho pedido (ou a maior) vira RGBA.
#[cfg(not(windows))]
fn rgba(lado: u32) -> Result<(Vec<u8>, u32, u32), String> {
    let ico: &[u8] = include_bytes!("../../recursos/icone/icone.ico");
    let entradas = (0..u16::from_le_bytes([ico[4], ico[5]]) as usize).map(|i| &ico[6 + 16 * i..][..16]);
    let largura = |e: &[u8]| if e[0] == 0 { 256 } else { u32::from(e[0]) };
    let e = entradas.clone().find(|e| largura(e) == lado).or_else(|| entradas.max_by_key(|e| largura(e))).ok_or("icone.ico vazio")?;
    let (tamanho, inicio) = (u32::from_le_bytes([e[8], e[9], e[10], e[11]]) as usize, u32::from_le_bytes([e[12], e[13], e[14], e[15]]) as usize);
    let mut leitor = png::Decoder::new(std::io::Cursor::new(&ico[inicio..][..tamanho])).read_info().map_err(|e| e.to_string())?;
    let mut pixels = vec![0; leitor.output_buffer_size().ok_or("imagem grande demais")?];
    let quadro = leitor.next_frame(&mut pixels).map_err(|e| e.to_string())?;
    if quadro.color_type != png::ColorType::Rgba || quadro.bit_depth != png::BitDepth::Eight {
        return Err("o icone.ico deve ter imagens PNG RGBA de 8 bits".into());
    }
    pixels.truncate(quadro.buffer_size());
    Ok((pixels, quadro.width, quadro.height))
}

#[cfg(not(windows))]
pub fn janela() -> Result<tao::window::Icon, String> {
    let (pixels, l, a) = rgba(64)?;
    tao::window::Icon::from_rgba(pixels, l, a).map_err(|e| e.to_string())
}

#[cfg(not(windows))]
pub fn bandeja() -> Result<tray_icon::Icon, String> {
    let (pixels, l, a) = rgba(32)?;
    tray_icon::Icon::from_rgba(pixels, l, a).map_err(|e| e.to_string())
}
