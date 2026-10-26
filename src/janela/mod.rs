//! O app como aplicativo do Windows: janela nativa (WebView2, via `wry` e `tao`) apontada para o servidor local, ícone na bandeja,
//! instância única e dados em %APPDATA%\Blueprint\. Só existe no Windows. A janela é só a tela: tudo (dados, MCP, protótipos) é do servidor,
//! que continua de pé com a janela fechada, e por isso o agente segue falando com o app.
pub mod atualizando;
pub mod bandeja;
pub mod console;
pub mod dev;
pub mod equipe;
pub mod icone;
pub mod instancia;
pub mod projeto_local;

use crate::erro::{Erro, Resultado};
use crate::http::config::pasta_do_app;
use crate::http::{self, Config, Servidor};
use instancia::Instancia;
use std::time::Duration;
use tao::dpi::LogicalSize;
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::window::{Fullscreen, Window, WindowBuilder};
use wry::{NewWindowResponse, PageLoadEvent, WebContext, WebView, WebViewBuilder};

/// O que o laço da janela recebe de fora (bandeja, servidor, carregamento da página).
#[derive(Debug, Clone)]
pub enum Evento {
    /// o convite confirmado pelo usuário: abre a janela da equipe (equipe.rs)
    Equipe(equipe::Convite),
    /// trazer a janela à frente (clique na bandeja, segunda execução)
    Mostrar,
    /// a página terminou de carregar: pode mostrar a janela
    Carregou,
    Sair,
    /// o estado da atualização mudou (baixando, pronta…): a bandeja o reflete
    Atualizacao,
    /// o clique em "Atualizar para X…" na bandeja
    Atualizar,
    VerificarAtualizacao,
    AlternarAutomatica,
    /// o botão "Tela cheia" da página (IPC): liga/desliga a tela cheia do Windows
    TelaCheia,
    /// só no modo dev: um arquivo da interface mudou (`so_css`: basta trocar o estilo)
    Recarregar { so_css: bool },
}

fn falha(msg: impl std::fmt::Display) -> Erro {
    Erro::Io(std::io::Error::other(msg.to_string()))
}

/// Abre o link num navegador de verdade (a janela do app só mostra o próprio app).
fn abrir_no_navegador(url: &str) {
    use std::os::windows::process::CommandExt;
    if url.starts_with("http://") || url.starts_with("https://") {
        let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).creation_flags(0x0800_0000).spawn(); // sem janela de console
    }
}

/// Um WebView novo na janela. Chamado na abertura e toda vez que a janela volta da bandeja (ao sair, o WebView é destruído).
/// `origem` é até onde a página pode navegar (o servidor local, ou o da equipe); só a janela da equipe (https) aceita o IPC `conectar`.
fn criar_webview(window: &Window, contexto: &mut WebContext, url: &str, origem: &str, dev: bool, avisa: impl Fn() + 'static, proxy: EventLoopProxy<Evento>) -> Result<WebView, wry::Error> {
    let (origem, origem_ipc) = (origem.to_string(), origem.to_string());
    WebViewBuilder::new_with_web_context(contexto)
        .with_devtools(dev) // F12 só no modo dev
        .with_url(url)
        .with_background_color(icone::PAPEL) // o fundo do tema desde o primeiro quadro: sem flash branco
        .with_on_page_load_handler(move |evento, _| {
            if matches!(evento, PageLoadEvent::Finished) {
                avisa();
            }
        })
        .with_ipc_handler(move |req| {
            if req.body() == "tela-cheia" {
                let _ = proxy.send_event(Evento::TelaCheia);
            } else if let Some(chave) = req.body().strip_prefix("conectar ").filter(|_| origem_ipc.starts_with("https://")) {
                equipe::pedir_conexao(origem_ipc.trim_end_matches('/').to_string(), chave.trim().to_string());
            } else if let Some(corpo) = req.body().strip_prefix("projeto ").filter(|_| origem_ipc.starts_with("https://")) {
                // a janela da equipe pede ao app instalado o que roda nesta máquina (configurar a pasta, abrir o Claude): validado e perguntado em projeto_local.rs
                projeto_local::pedir(origem_ipc.trim_end_matches('/').to_string(), corpo.to_string());
            } else if let Some(detalhes) = req.body().strip_prefix("equipe ").filter(|_| origem_ipc.starts_with("https://")).and_then(crate::equipes::detalhes_do_ipc) {
                // a página da equipe diz quem a pessoa é e que workspaces vê lá: a tela de workspaces do app mostra isso
                crate::equipes::lembrar(&pasta_do_app(), origem_ipc.trim_end_matches('/'), Some(detalhes));
            }
        })
        .with_navigation_handler(move |destino| destino.starts_with(&origem) || destino.starts_with("about:") || destino.starts_with("data:") || destino.starts_with("blob:"))
        .with_new_window_req_handler(|destino, _| {
            abrir_no_navegador(&destino);
            NewWindowResponse::Deny
        })
        .build(window)
}

/// Liga o servidor; se a porta (3000) estiver ocupada, deixa o sistema escolher outra.
fn iniciar_servidor(mut config: Config) -> Resultado<Servidor> {
    match http::iniciar(config.clone()) {
        Ok(s) => Ok(s),
        Err(_) if config.porta != 0 => {
            config.porta = 0;
            http::iniciar(config)
        }
        Err(e) => Err(e),
    }
}

/// O app com janela. Só volta (com código de saída) se a janela nem chegou a abrir; ao sair pela bandeja o processo termina.
/// `dev` (`blueprint dev`): o mesmo app, mas com a interface lida da pasta do projeto, F12 e recarga ao salvar; fechar a janela encerra tudo.
/// `link`: o `blueprint://convite?…` que o navegador entregou (equipe.rs); um link que não vale abre o app como sem link.
pub fn abrir(dev: bool, link: Option<String>) -> i32 {
    let convite = link.as_deref().and_then(equipe::convite_do_link);
    if instancia::garantir() == Instancia::JaAberta {
        if dev {
            // mesmo banco e mesma porta: dois processos teriam filas do MCP separadas e a janela ficaria com dados velhos
            eprintln!("O The Blueprint já está aberto (veja a bandeja). O modo dev usa o mesmo banco e a mesma porta: feche o app (bandeja → Sair) e rode de novo.");
            return 1;
        }
        instancia::pedir_que_mostre(link.as_deref().filter(|_| convite.is_some()));
        return 0;
    }
    match preparar(dev, convite) {
        Ok(()) => 0,
        Err(e) => {
            mensagem_de_erro(&format!("O The Blueprint (beta) não conseguiu abrir: {e}"));
            1
        }
    }
}

/// Sem console (o app é do subsistema "windows"), o erro precisa aparecer de outro jeito: fica num arquivo de registro.
fn mensagem_de_erro(texto: &str) {
    let _ = std::fs::create_dir_all(pasta_do_app());
    let _ = std::fs::write(pasta_do_app().join("erro.txt"), texto);
}

fn preparar(dev: bool, convite: Option<equipe::Convite>) -> Resultado<()> {
    let event_loop = EventLoopBuilder::<Evento>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let mut config = Config::do_app();
    if dev {
        let raiz = dev::raiz_do_projeto().ok_or_else(|| falha("o modo dev precisa do binário compilado na pasta do projeto (cargo run -- dev)"))?;
        println!("dev: interface lida de {} · salve web/, kit/ ou blueprint.html para recarregar a janela; F12 abre as ferramentas", raiz.display());
        let mut vigia = dev::Vigia::novo(raiz.clone());
        let p = proxy.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(300));
            if let Some(so_css) = vigia.mudou() {
                if p.send_event(Evento::Recarregar { so_css }).is_err() {
                    break; // o laço da janela acabou
                }
            }
        });
        config.assets_dir.get_or_insert(raiz);
    }
    let servidor = iniciar_servidor(config)?;
    instancia::registrar_porta(servidor.porta);
    let (a, sair) = (proxy.clone(), proxy.clone());
    servidor.estado.atualizador.ouvir(move |_| {
        let _ = a.send_event(Evento::Atualizacao);
    });
    *servidor.estado.ao_sair.lock().unwrap_or_else(|e| e.into_inner()) = Some(Box::new(move || {
        let _ = sair.send_event(Evento::Sair); // o instalador já foi lançado: o app sai para soltar o .exe e a porta
    }));
    if !dev {
        servidor.estado.atualizador.ao_abrir(); // recupera o instalador já baixado e verifica em segundo plano (no máximo uma vez por dia)
    }
    let mostrar = proxy.clone();
    // a segunda execução manda o link do convite junto (rota /api/app/mostrar): confirmado, abre a janela da equipe; sem link, traz a janela
    // o link `blueprint://equipe` é interno: a rota /api/equipes/abrir só o monta para uma equipe já lembrada, depois do clique do usuário no app
    *servidor.estado.ao_mostrar.lock().unwrap_or_else(|e| e.into_inner()) = Some(Box::new(move |link| match link.as_deref().and_then(equipe::convite_do_link) {
        Some(convite) => equipe::pedir_abertura(mostrar.clone(), convite),
        None => {
            let _ = mostrar.send_event(match link.as_deref().and_then(equipe::equipe_do_link) {
                Some(equipe) => Evento::Equipe(equipe),
                None => Evento::Mostrar,
            });
        }
    }));
    let so_equipe = convite.is_some(); // aberto pelo link: a janela principal fica na bandeja até alguém pedir
    if let Some(convite) = convite {
        equipe::pedir_abertura(proxy.clone(), convite);
    }

    let icone = icone::janela().map_err(falha)?;
    let window = WindowBuilder::new()
        .with_title(if dev { "The Blueprint (beta) · dev" } else { "The Blueprint (beta)" })
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(LogicalSize::new(800.0, 500.0))
        .with_window_icon(Some(icone))
        .with_visible(false) // só aparece com a página pronta
        .build(&event_loop)
        .map_err(falha)?;
    let mut contexto = WebContext::new(Some(pasta_do_app().join("webview")));
    let url = servidor.url.clone();
    let avisa = {
        let p = proxy.clone();
        move || {
            let _ = p.send_event(Evento::Carregou);
        }
    };
    let mut webview = Some(criar_webview(&window, &mut contexto, &url, &url, dev, avisa.clone(), proxy.clone()).map_err(falha)?);
    let mut equipe_janela = None;
    {
        // se a página demorar ou falhar, a janela aparece mesmo assim em 2 s
        let p = proxy.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            let _ = p.send_event(Evento::Carregou);
        });
    }
    let mut bandeja = None;

    event_loop.run(move |evento, alvo, fluxo| {
        *fluxo = ControlFlow::Wait; // zero trabalho em repouso: só acorda com um evento
        match evento {
            Event::NewEvents(StartCause::Init) => bandeja = bandeja::criar(proxy.clone(), &servidor.estado.atualizador.json()).ok(),
            Event::UserEvent(Evento::Atualizacao) => {
                if let Some(b) = &bandeja {
                    b.refletir(&servidor.estado.atualizador.json());
                }
            }
            Event::UserEvent(Evento::Atualizar) => {
                // o clique na bandeja é a autorização; se for recusado (fila do MCP cheia), a janela abre e mostra o motivo
                if crate::http::atualizacao::instalar_agora(&servidor.estado).is_err() {
                    let _ = proxy.send_event(Evento::Mostrar);
                }
            }
            Event::UserEvent(Evento::VerificarAtualizacao) => servidor.estado.atualizador.verificar_em_segundo_plano(true),
            Event::UserEvent(Evento::AlternarAutomatica) => {
                let a = &servidor.estado.atualizador;
                a.definir_automatica(!a.automatica());
            }
            Event::UserEvent(Evento::Carregou) => {
                if !window.is_visible() && !so_equipe {
                    window.set_visible(true);
                    window.set_focus();
                }
            }
            Event::UserEvent(Evento::Mostrar) => {
                if webview.is_none() {
                    webview = criar_webview(&window, &mut contexto, &url, &url, dev, avisa.clone(), proxy.clone()).ok(); // volta da bandeja: o Carregou mostra a janela
                }
                window.set_minimized(false);
                window.set_visible(true);
                window.set_focus();
            }
            Event::UserEvent(Evento::Equipe(convite)) => {
                equipe_janela = None; // um convite novo troca a janela da equipe
                equipe_janela = equipe::abrir_janela(alvo, &mut contexto, &convite, proxy.clone()).ok();
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id, .. } if equipe_janela.as_ref().is_some_and(|(_, w): &(WebView, Window)| w.id() == window_id) => {
                equipe_janela = None; // fechar a janela da equipe a destrói; o app e a janela principal seguem
            }
            Event::UserEvent(Evento::TelaCheia) => {
                let cheia = window.fullscreen().is_none();
                window.set_fullscreen(cheia.then_some(Fullscreen::Borderless(None)));
                if let Some(w) = &webview {
                    let _ = w.evaluate_script(&format!("telaCheiaNativa({cheia})"));
                }
            }
            Event::UserEvent(Evento::Recarregar { so_css }) => {
                if let Some(w) = &webview {
                    let _ = if so_css { w.evaluate_script(dev::TROCA_CSS) } else { w.reload() };
                }
            }
            Event::UserEvent(Evento::Sair) => {
                servidor.parar();
                bandeja = None;
                *fluxo = ControlFlow::Exit;
            }
            // fechar a janela manda o app para a bandeja: o WebView é destruído (é a maior fatia da memória) e só o servidor fica
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } if dev => {
                servidor.parar(); // no modo dev fechar a janela encerra o processo: libera a porta e o banco para o próximo `cargo run`
                bandeja = None;
                *fluxo = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                webview = None;
                window.set_visible(false);
            }
            _ => {}
        }
        let _ = &bandeja; // a bandeja precisa viver tanto quanto o laço
    })
}
