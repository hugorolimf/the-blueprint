//! The Blueprint (beta), interface nativa: o mesmo código abre uma janela (Windows, Linux) e roda no navegador (WebAssembly).
//! É cliente do servidor do Blueprint (`blueprint serve`, ou o app aberto): lê tudo de `/api/dados`.
//! O programa `blueprint` usa esta biblioteca para abrir a janela fora do Windows; `cargo run` aqui (`main.rs`) abre a mesma janela contra um servidor já ligado.

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod captura;
mod documento;
mod dominio;
mod emblema;
mod folha;
mod fonte;
mod i18n;
#[cfg(target_arch = "wasm32")]
pub mod navegador;
mod pesquisa;
mod planta;
pub mod rede;
#[cfg(not(target_arch = "wasm32"))]
mod roteiro;
mod telas;
mod tema;

/// Quem pede para a janela vir à frente (a segunda execução do programa, por exemplo): pode ser chamado de qualquer thread.
pub type Mostrar = Box<dyn Fn() + Send + Sync>;

/// O ícone do app (o mesmo `icone.ico` do .exe do Windows, de `recursos/icone/`): a imagem PNG de 64 px que está dentro dele.
#[cfg(not(target_arch = "wasm32"))]
fn icone() -> Option<egui::IconData> {
    let ico: &[u8] = include_bytes!("../../recursos/icone/icone.ico");
    let entradas = (0..u16::from_le_bytes([ico[4], ico[5]]) as usize).map(|i| &ico[6 + 16 * i..][..16]);
    let e = entradas.clone().find(|e| e[0] == 64).or_else(|| entradas.last())?;
    let (tamanho, inicio) = (u32::from_le_bytes([e[8], e[9], e[10], e[11]]) as usize, u32::from_le_bytes([e[12], e[13], e[14], e[15]]) as usize);
    let imagem = image::load_from_memory(ico.get(inicio..inicio + tamanho)?).ok()?.to_rgba8();
    Some(egui::IconData { width: imagem.width(), height: imagem.height(), rgba: imagem.into_raw() })
}

/// Abre a janela e fica nela até ser fechada. `servidor` é o endereço do servidor (`http://127.0.0.1:3000`); `ao_abrir` recebe o `Mostrar` da janela.
#[cfg(not(target_arch = "wasm32"))]
pub fn executar(servidor: String, ao_abrir: impl FnOnce(Mostrar) + 'static) -> Result<(), String> {
    let janela = egui::ViewportBuilder::default()
        .with_title("The Blueprint (beta)")
        .with_app_id(std::env::var("BLUEPRINT_UI_APPID").unwrap_or_else(|_| "blueprint-ui".into()))
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([800.0, 500.0]);
    let janela = match icone() {
        Some(i) => janela.with_icon(i),
        None => janela,
    };
    let opcoes = eframe::NativeOptions { viewport: janela, ..Default::default() };
    eframe::run_native("The Blueprint (beta)", opcoes, Box::new(move |cc| Ok(Box::new(app::App::nova(cc, servidor, Box::new(ao_abrir)))))).map_err(|e| e.to_string())
}
