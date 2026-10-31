//! The Blueprint (beta), interface nativa: o mesmo código abre uma janela (Windows, Linux) e roda no navegador (WebAssembly).
//! É cliente do servidor do Blueprint (`blueprint serve`, ou o app aberto): lê tudo de `/api/dados`.
//! O programa `blueprint` usa esta biblioteca para abrir a janela fora do Windows; `cargo run` aqui (`main.rs`) abre a mesma janela contra um servidor já ligado.

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod captura;
mod documento;
mod emblema;
mod fonte;
mod i18n;
#[cfg(target_arch = "wasm32")]
pub mod navegador;
mod planta;
pub mod rede;
#[cfg(not(target_arch = "wasm32"))]
mod roteiro;
mod telas;
mod tema;

/// Quem pede para a janela vir à frente (a segunda execução do programa, por exemplo): pode ser chamado de qualquer thread.
pub type Mostrar = Box<dyn Fn() + Send + Sync>;

/// Abre a janela e fica nela até ser fechada. `servidor` é o endereço do servidor (`http://127.0.0.1:3000`); `ao_abrir` recebe o `Mostrar` da janela.
#[cfg(not(target_arch = "wasm32"))]
pub fn executar(servidor: String, ao_abrir: impl FnOnce(Mostrar) + 'static) -> Result<(), String> {
    let janela = egui::ViewportBuilder::default()
        .with_title("The Blueprint (beta)")
        .with_app_id(std::env::var("BLUEPRINT_UI_APPID").unwrap_or_else(|_| "blueprint-ui".into()))
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([800.0, 500.0]);
    let opcoes = eframe::NativeOptions { viewport: janela, ..Default::default() };
    eframe::run_native("The Blueprint (beta)", opcoes, Box::new(move |cc| Ok(Box::new(app::App::nova(cc, servidor, Box::new(ao_abrir)))))).map_err(|e| e.to_string())
}
