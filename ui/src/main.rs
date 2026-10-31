//! A janela da interface nativa contra um servidor já ligado (`BLUEPRINT_URL`, padrão a porta 3000). No navegador (WebAssembly) abre no canvas da página.

#[cfg(target_arch = "wasm32")]
fn main() {
    blueprint_ui::navegador::abrir();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), String> {
    blueprint_ui::executar(blueprint_ui::rede::servidor(), |_| {})
}
