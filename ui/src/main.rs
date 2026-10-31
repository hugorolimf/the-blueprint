//! The Blueprint (beta), interface nativa: o mesmo código abre uma janela (Windows, Linux) e roda no navegador (WebAssembly).
//! É cliente do servidor do Blueprint (`blueprint serve`, ou o app aberto): lê tudo de `/api/dados`.

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod captura;
mod documento;
mod emblema;
mod fonte;
mod i18n;
#[cfg(target_arch = "wasm32")]
mod navegador;
mod planta;
mod rede;
#[cfg(not(target_arch = "wasm32"))]
mod roteiro;
mod tema;
mod telas;

#[cfg(target_arch = "wasm32")]
fn main() {
    navegador::abrir();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let janela = egui::ViewportBuilder::default().with_title("The Blueprint (beta)").with_app_id(std::env::var("BLUEPRINT_UI_APPID").unwrap_or_else(|_| "blueprint-ui".into())).with_inner_size([1280.0, 800.0]).with_min_inner_size([800.0, 500.0]);
    let opcoes = eframe::NativeOptions { viewport: janela, ..Default::default() };
    eframe::run_native("The Blueprint (beta)", opcoes, Box::new(|cc| Ok(Box::new(app::App::nova(cc, rede::servidor())))))
}
