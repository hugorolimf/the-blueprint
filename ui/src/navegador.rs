//! A interface no navegador: abre no canvas `blueprint` da página e tira a mensagem «Carregando…» quando começa (ou diz por que não abriu).

use wasm_bindgen::JsCast;

const CANVAS: &str = "blueprint";
const CARREGANDO: &str = "carregando";

pub fn abrir() {
    let _ = eframe::WebLogger::init(log::LevelFilter::Warn);
    wasm_bindgen_futures::spawn_local(async {
        let Some(canvas) = canvas() else {
            dizer("A página veio sem o canvas.");
            return;
        };
        let inicio = eframe::WebRunner::new().start(canvas, eframe::WebOptions::default(), Box::new(|cc| Ok(Box::new(crate::app::App::nova(cc, crate::rede::servidor(), Box::new(|_| {})))))).await;
        match inicio {
            Ok(()) => {
                if let Some(mensagem) = documento().and_then(|d| d.get_element_by_id(CARREGANDO)) {
                    mensagem.remove();
                }
            }
            Err(erro) => {
                log::error!("a interface não abriu: {erro:?}");
                dizer("Não foi possível abrir o The Blueprint neste navegador (ele precisa de WebGL 2).");
            }
        }
    });
}

fn documento() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn canvas() -> Option<web_sys::HtmlCanvasElement> {
    documento()?.get_element_by_id(CANVAS)?.dyn_into().ok()
}

fn dizer(texto: &str) {
    if let Some(mensagem) = documento().and_then(|d| d.get_element_by_id(CARREGANDO)) {
        mensagem.set_text_content(Some(texto));
    }
}
