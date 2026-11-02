//! A janela: o cabeçalho, o estado da conexão e a tela do momento. Quem lê e grava no servidor é o `app`: as telas só pedem uma `Acao`,
//! ele aplica no documento e manda. Se a gravação falhar, o documento volta ao que era antes (nada fica pela metade na tela).

use crate::documento::Documento;
use crate::i18n::tr;
use crate::rede::{self, Falha, Msg};
use crate::telas::{self, Acao, Vista};
use crate::{fonte, tema};

mod acoes;
mod recarga;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;

enum Estado {
    Carregando,
    Erro(String),
    Pronto(Documento),
}

const CHAVE_VISTA: &str = "vista";

pub struct App {
    servidor: String,
    estado: Estado,
    vista: Vista,
    /// o documento de antes da gravação em curso: volta se ela falhar
    gravando: Option<Value>,
    /// a busca do documento em curso é uma recarga no lugar: a tela segue à vista e falha não vira a tela de erro
    recarga_em_curso: bool,
    envio: Sender<Msg>,
    recebe: Receiver<Msg>,
    /// alguém pediu a janela à frente (`Mostrar`)
    mostrar: Arc<AtomicBool>,
    /// só para o registro de testes (`BLUEPRINT_UI_LOG`)
    ultima_tela: String,
    /// o clique de teste em curso: posição, botão e se já é a hora de soltar
    clique: Option<(egui::Pos2, egui::PointerButton, bool)>,
    /// a última busca de agentes (a cada poucos segundos, só com a janela aberta)
    agentes_em: Option<web_time::Instant>,
    #[cfg(not(target_arch = "wasm32"))]
    captura: Option<crate::captura::Captura>,
    #[cfg(not(target_arch = "wasm32"))]
    roteiro: Option<crate::roteiro::Roteiro>,
}

impl App {
    pub fn nova(cc: &eframe::CreationContext<'_>, servidor: String, ao_abrir: Box<dyn FnOnce(crate::Mostrar)>) -> App {
        let (envio, recebe) = channel();
        let mostrar = Arc::new(AtomicBool::new(false));
        ao_abrir(Box::new({
            let (mostrar, ctx) = (mostrar.clone(), cc.egui_ctx.clone());
            move || {
                mostrar.store(true, Ordering::SeqCst);
                ctx.request_repaint();
            }
        }));
        #[cfg_attr(target_arch = "wasm32", allow(unused_mut))]
        let mut app = App {
            servidor,
            estado: Estado::Carregando,
            vista: Vista::nova(),
            gravando: None,
            recarga_em_curso: false,
            envio,
            recebe,
            mostrar,
            ultima_tela: String::new(),
            clique: None,
            agentes_em: None,
            #[cfg(not(target_arch = "wasm32"))]
            captura: crate::captura::do_ambiente(),
            #[cfg(not(target_arch = "wasm32"))]
            roteiro: crate::roteiro::do_ambiente(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        crate::roteiro::abrir(&mut app.vista);
        if !app.em_teste() {
            if let Some(salvo) = cc.storage.and_then(|s| s.get_string(CHAVE_VISTA)).and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
                app.vista.restaurar(&salvo);
            }
        }
        fonte::instalar(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        tema::aplicar(&cc.egui_ctx);
        rede::buscar(&app.servidor, &cc.egui_ctx, app.envio.clone());
        app
    }

    /// Rodando um roteiro ou uma captura de teste: nada é lido nem guardado, para o teste não herdar nem sujar o que o usuário deixou.
    fn em_teste(&self) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        return self.captura.is_some() || self.roteiro.is_some();
        #[cfg(target_arch = "wasm32")]
        false
    }

    fn recarregar(&mut self, ctx: &egui::Context) {
        self.estado = Estado::Carregando;
        self.gravando = None;
        self.recarga_em_curso = false;
        self.vista.plano.recarregar_doc = false;
        rede::buscar(&self.servidor, ctx, self.envio.clone());
    }

    /// Trata o que chegou do servidor desde o último quadro.
    fn receber(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.recebe.try_recv() {
            match msg {
                Msg::Dados(resultado) if self.recarga_em_curso => self.recarga_chegou(resultado),
                Msg::Dados(Ok(doc)) => self.estado = Estado::Pronto(doc),
                Msg::Dados(Err(motivo)) => self.estado = Estado::Erro(motivo),
                Msg::Gravado(Ok(versao)) => {
                    if let Estado::Pronto(doc) = &mut self.estado {
                        doc.versao = versao;
                    }
                    self.gravando = None;
                    self.vista.aviso.clear();
                    if let Some(tela) = self.vista.depois.take() {
                        self.vista.tela = tela;
                    }
                }
                Msg::Gravado(Err(falha)) => {
                    self.vista.depois = None;
                    match falha {
                        Falha::Desatualizado => {
                            self.recarregar(ctx);
                            self.vista.erro = tr("Os dados mudaram em outra janela. Atualizei: tente de novo.").to_string();
                        }
                        Falha::Outra(motivo) => {
                            if let (Estado::Pronto(doc), Some(antes)) = (&mut self.estado, self.gravando.take()) {
                                doc.raw = antes;
                            }
                            self.vista.erro = motivo;
                        }
                    }
                }
                Msg::Exemplo(Ok(modelo)) => self.aplicar(ctx, Acao::ExemploPronto(Box::new(modelo))),
                Msg::Exemplo(Err(motivo)) => self.vista.erro = motivo,
                Msg::Agentes(lista, vida) => {
                    self.vista.agentes = lista;
                    self.vista.agentes_vida = vida;
                }
            }
        }
    }


}

impl eframe::App for App {
    /// Guarda o que o usuário espera achar como deixou: as seções abertas e os agentes que ele já viu (`concluidosVistos` da página).
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.em_teste() {
            return;
        }
        if let Ok(t) = serde_json::to_string(&self.vista.para_guardar()) {
            storage.set_string(CHAVE_VISTA, t);
        }
    }

    /// Só nos testes: transforma o passo `clicar:x/y` do roteiro num clique de ponteiro de verdade (mover, apertar; no quadro seguinte, soltar).
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if let Some((pos, button, solta)) = self.clique.take() {
            if solta {
                raw.events.push(egui::Event::PointerButton { pos, button, pressed: false, modifiers: Default::default() });
            } else {
                raw.events.push(egui::Event::PointerMoved(pos));
                raw.events.push(egui::Event::PointerButton { pos, button, pressed: true, modifiers: Default::default() });
                self.clique = Some((pos, button, true));
                ctx.request_repaint();
            }
        } else if let Some((pos, button)) = self.vista.clique_pedido.take() {
            raw.events.push(egui::Event::PointerMoved(pos));
            self.clique = Some((pos, button, false));
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        let _tempo = crate::captura::Tempo::novo();
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("BLUEPRINT_UI_CAUSAS").is_some() {
            eprintln!("causas: {:?}", ui.ctx().repaint_causes().iter().map(|c| format!("{}:{}", c.file, c.line)).collect::<Vec<_>>());
        }
        let ctx = ui.ctx().clone();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(captura) = &mut self.captura {
            captura.passo(&ctx);
        }
        #[cfg(not(target_arch = "wasm32"))]
        crate::roteiro::escala(&ctx);
        if !self.em_teste() || std::env::var_os("BLUEPRINT_UI_AJUSTAR").is_some() {
            ajustar_a_largura(&ctx);
        }
        if self.mostrar.swap(false, Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        self.receber(&ctx);
        self.recarregar_no_lugar(&ctx);
        #[cfg(not(target_arch = "wasm32"))]
        if std::env::var_os("BLUEPRINT_UI_LOG").is_some() {
            let tela = format!("{:?}", self.vista.tela);
            if self.ultima_tela != tela {
                eprintln!("tela: {tela} (janela {:.0}x{:.0}, zoom {:.2})", ctx.content_rect().width(), ctx.content_rect().height(), ctx.zoom_factor());
                self.ultima_tela = tela;
            }
            if let Some(p) = ctx.input(|i| i.pointer.press_origin()) {
                eprintln!("clique em {:.0},{:.0}", p.x, p.y);
            }
        }
        const INTERVALO: std::time::Duration = std::time::Duration::from_secs(3);
        if matches!(self.estado, Estado::Pronto(_)) && self.agentes_em.is_none_or(|t| t.elapsed() >= INTERVALO) {
            self.agentes_em = Some(web_time::Instant::now());
            rede::agentes(&self.servidor, &ctx, self.envio.clone());
        }
        ctx.request_repaint_after(INTERVALO);
        let mut pedida = None;
        let ocupado = self.gravando.is_some();
        let pela_tecla = crate::menu::teclado(&ctx, &mut self.vista);
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| match &self.estado {
            Estado::Carregando => {
                ui.painter().rect_filled(ui.max_rect(), 0, tema::PAPEL);
                crate::planta::Texto::novo(tr("Carregando…"), 14.0).cor(tema::MEDIO).na_linha(ui.painter(), 64.0, 64.0, 22.4);
            }
            Estado::Erro(motivo) => {
                ui.painter().rect_filled(ui.max_rect(), 0, tema::PAPEL);
                crate::planta::Texto::novo(motivo.as_str(), 14.0).cor(tema::ERRO).caixa(ui.max_rect().width() - 128.0).em(ui.painter(), egui::pos2(64.0, 64.0));
                if crate::planta::botao(ui, egui::pos2(64.0, 120.0), &crate::planta::Botao::entrada(tr("Tentar de novo"))).clicked() {
                    pedida = Some(Acao::Atualizar);
                }
            }
            Estado::Pronto(doc) => {
                pedida = telas::desenhar(ui, &mut self.vista, doc, &self.servidor, ocupado);
                pedida = pedida.take().or_else(|| telas::dialogos(ui.ctx(), &mut self.vista, doc, ocupado));
            }
        });
        if matches!(self.estado, Estado::Pronto(_)) {
            let escolhido = crate::menu::passo(&ctx, &mut self.vista, ocupado).or(pela_tecla);
            if let Some(comando) = escolhido {
                pedida = crate::menu::executar(comando, &ctx, &mut self.vista, &self.servidor).or(pedida);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        if pedida.is_none() && self.gravando.is_none() {
            if let (Some(roteiro), Estado::Pronto(doc)) = (&mut self.roteiro, &self.estado) {
                pedida = roteiro.proximo(doc, &mut self.vista);
                ctx.request_repaint_after(std::time::Duration::from_millis(400));
            }
        }
        if let Some(acao) = pedida {
            self.aplicar(&ctx, acao);
        }
    }
}

/// A folha foi desenhada para 960 pontos de largura ou mais. Numa janela mais estreita (o gerenciador de janelas divide a tela em colunas), em vez de cortar e rolar de lado,
/// a interface inteira encolhe para caber, como «ajustar à largura»: tudo continua à vista e clicável.
fn ajustar_a_largura(ctx: &egui::Context) {
    const LARGURA_MINIMA: f32 = 960.0;
    let zoom = ctx.zoom_factor();
    let fisica = ctx.content_rect().width() * zoom;
    let alvo = (fisica / LARGURA_MINIMA).clamp(0.45, 1.0);
    if (alvo - zoom).abs() > 0.01 {
        ctx.set_zoom_factor(alvo);
    }
}
