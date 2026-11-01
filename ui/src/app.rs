//! A janela: o cabeçalho, o estado da conexão e a tela do momento. Quem lê e grava no servidor é o `app`: as telas só pedem uma `Acao`,
//! ele aplica no documento e manda. Se a gravação falhar, o documento volta ao que era antes (nada fica pela metade na tela).

use crate::documento::{self, Documento};
use crate::i18n::tr;
use crate::rede::{self, Falha, Msg};
use crate::telas::{self, Acao, Tela, Vista};
use crate::{fonte, tema};
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
    envio: Sender<Msg>,
    recebe: Receiver<Msg>,
    /// alguém pediu a janela à frente (`Mostrar`)
    mostrar: Arc<AtomicBool>,
    /// o clique de teste em curso: posição e se já é a hora de soltar
    clique: Option<(egui::Pos2, bool)>,
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
            envio,
            recebe,
            mostrar,
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
        rede::buscar(&self.servidor, ctx, self.envio.clone());
    }

    /// Trata o que chegou do servidor desde o último quadro.
    fn receber(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.recebe.try_recv() {
            match msg {
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

    /// Muda o documento e grava. Recusado (`Err`), nada muda e o motivo aparece na tela.
    fn mutar(&mut self, ctx: &egui::Context, f: impl FnOnce(&mut Documento) -> Result<Option<Tela>, String>) {
        let Estado::Pronto(doc) = &mut self.estado else { return };
        let antes = doc.raw.clone();
        match f(doc) {
            Err(motivo) => {
                doc.raw = antes;
                self.vista.erro = motivo;
            }
            Ok(depois) => {
                self.vista.erro.clear();
                self.vista.aviso = tr("Salvando…").to_string();
                self.vista.depois = depois;
                self.gravando = Some(antes);
                rede::gravar(&self.servidor, ctx, self.envio.clone(), doc);
            }
        }
    }

    fn aplicar(&mut self, ctx: &egui::Context, acao: Acao) {
        if self.gravando.is_some() {
            return;
        }
        match acao {
            Acao::AbrirNota(ws, bp, acao) => {
                if let Estado::Pronto(doc) = &self.estado {
                    self.vista.nota_texto = doc.obs(&ws, &bp, acao.as_deref());
                }
                self.vista.nota = Some((ws, bp, acao));
            }
            Acao::SalvarObs(ws, bp, acao, novo) => self.mutar(ctx, |d| if d.definir_obs(&ws, &bp, acao.as_deref(), &novo) { Ok(None) } else { Err(tr("Não foi possível salvar.").to_string()) }),
            Acao::Atualizar => self.recarregar(ctx),
            Acao::DefinirNome(nome) => self.mutar(ctx, |d| {
                d.definir_usuario(&nome);
                Ok(None)
            }),
            Acao::CriarWorkspace { nome, descricao, icone } => {
                self.mutar(ctx, |d| d.criar_workspace(&nome, &descricao, &icone).map(|id| Some(Tela::Workspace(id))));
                if self.vista.erro.is_empty() {
                    self.vista.novo = None;
                } else if let Some(n) = self.vista.novo.as_mut() {
                    n.erro = std::mem::take(&mut self.vista.erro);
                }
            }
            Acao::Excluir(id) => self.mutar(ctx, |d| if d.excluir_workspace(&id) { Ok(None) } else { Err(tr("Não foi possível excluir o workspace.").to_string()) }),
            Acao::Mover(id, delta) => self.mutar(ctx, |d| {
                d.mover_workspace(&id, delta);
                Ok(None)
            }),
            Acao::Marcar(ws, bp, acao, tipo, item) => self.mutar(ctx, |d| if d.marcar_item(&ws, &bp, &acao, tipo, &item) { Ok(None) } else { Err(tr("Não foi possível marcar o item.").to_string()) }),
            Acao::MoverPara(id, para) => self.mutar(ctx, |d| {
                d.mover_para(&id, para);
                Ok(None)
            }),
            Acao::AbrirExemplo => {
                self.vista.aviso = tr("Abrindo o exemplo…").to_string();
                rede::exemplo(&self.servidor, ctx, self.envio.clone());
            }
            Acao::ExemploPronto(modelo) => self.mutar(ctx, |d| Ok(Some(Tela::Workspace(d.adicionar_exemplo(&modelo, &documento::hoje()))))),
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
        if let Some((pos, solta)) = self.clique.take() {
            if solta {
                raw.events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() });
            } else {
                raw.events.push(egui::Event::PointerMoved(pos));
                raw.events.push(egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
                self.clique = Some((pos, true));
                ctx.request_repaint();
            }
        } else if let Some(pos) = self.vista.clique_pedido.take() {
            raw.events.push(egui::Event::PointerMoved(pos));
            self.clique = Some((pos, false));
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
        if self.mostrar.swap(false, Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        self.receber(&ctx);
        const INTERVALO: std::time::Duration = std::time::Duration::from_secs(3);
        if matches!(self.estado, Estado::Pronto(_)) && self.agentes_em.is_none_or(|t| t.elapsed() >= INTERVALO) {
            self.agentes_em = Some(web_time::Instant::now());
            rede::agentes(&self.servidor, &ctx, self.envio.clone());
        }
        ctx.request_repaint_after(INTERVALO);
        let mut pedida = None;
        let ocupado = self.gravando.is_some();
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
