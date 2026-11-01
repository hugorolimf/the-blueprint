//! Testar sem olhar a tela: com `BLUEPRINT_UI_CAPTURA=<arquivo.png>` a janela salva uma imagem de si mesma depois de 2 s e fecha.

use egui::{Context, ViewportCommand};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct Captura {
    destino: PathBuf,
    inicio: Instant,
    pedida: bool,
}

pub fn do_ambiente() -> Option<Captura> {
    let destino = std::env::var_os("BLUEPRINT_UI_CAPTURA").filter(|v| !v.is_empty())?;
    Some(Captura { destino: destino.into(), inicio: Instant::now(), pedida: false })
}

impl Captura {
    /// Chamado a cada quadro: passada a espera (a conexão com o servidor já respondeu) pede a imagem, grava quando ela chega e fecha.
    pub fn passo(&mut self, ctx: &Context) {
        ctx.request_repaint_after(Duration::from_millis(300));
        if self.pedida || self.inicio.elapsed() < espera() {
            return;
        }
        self.pedida = true;
        let (destino, janela) = (self.destino.clone(), ctx.clone());
        ctx.request_screenshot(move |imagem| {
            let [largura, altura] = imagem.size;
            let _ = image::save_buffer(&destino, imagem.as_raw(), largura as u32, altura as u32, image::ColorType::Rgba8);
            janela.send_viewport_cmd(ViewportCommand::Close);
            // a janela de teste pode estar escondida (outra área): o fechamento normal esperaria um quadro que não vem
            std::process::exit(0);
        });
    }
}

/// `BLUEPRINT_UI_ESPERA=<segundos>`: quanto esperar antes da imagem (padrão 2), para dar tempo ao roteiro.
fn espera() -> Duration {
    Duration::from_secs_f32(std::env::var("BLUEPRINT_UI_ESPERA").ok().and_then(|v| v.parse().ok()).unwrap_or(2.0))
}

/// Só nos testes (`BLUEPRINT_UI_TEMPO=1`): diz no stderr quanto o quadro levou para desenhar, para achar o que deixa a janela pesada.
pub struct Tempo(Option<std::time::Instant>);

impl Tempo {
    pub fn novo() -> Tempo {
        Tempo(std::env::var_os("BLUEPRINT_UI_TEMPO").map(|_| std::time::Instant::now()))
    }
}

impl Drop for Tempo {
    fn drop(&mut self) {
        if let Some(t) = self.0 {
            eprintln!("quadro: {:.1} ms", t.elapsed().as_secs_f32() * 1000.0);
        }
    }
}
