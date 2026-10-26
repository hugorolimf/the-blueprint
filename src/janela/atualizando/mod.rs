//! A janela de progresso da atualização. O app não pode ficar aberto enquanto o instalador troca o .exe, e o instalador roda sem janela: sem isto o usuário veria o app
//! sumir e voltar sem saber o que houve. Quem mostra o progresso é um processo à parte, `blueprint atualizando <instalador> <exe-instalado> [args…]`, que roda de uma
//! CÓPIA do .exe em %TEMP% (o instalador troca o original e encerra os blueprint.exe da pasta de instalação), lança o instalador, espera, espera o app novo subir e fecha.
//! Na falha (instalador com código diferente de 0) mostra o motivo e reabre o app anterior, que continua instalado.
mod passos;

use crate::erro::Resultado;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use tao::dpi::{LogicalSize, PhysicalPosition};
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::{WebContext, WebViewBuilder};

const PREFIXO: &str = "blueprint-atualizando-";
const LARGURA: f64 = 440.0;
const ALTURA: f64 = 268.0;

#[derive(Debug, Clone)]
enum Evento {
    Carregou,
    Etapa(u8),
    Pronto,
    Erro(String),
    Arrastar,
    Fechar,
}

/// Chamado pelo app no clique em Atualizar: copia o .exe para %TEMP% e abre a janela de progresso com o instalador. `Err` se algo falhar (quem chama roda o instalador
/// direto, como antes: o progresso é conforto, nunca condição para atualizar).
pub fn lancar(instalador: &Path, args: &[String]) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let base = std::env::temp_dir();
    limpar_antigas(&base);
    let pasta = base.join(format!("{PREFIXO}{}", std::process::id()));
    std::fs::create_dir_all(&pasta).map_err(|e| e.to_string())?;
    let copia = pasta.join("blueprint-atualizando.exe");
    std::fs::copy(&exe, &copia).map_err(|e| e.to_string())?;
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;
    Command::new(&copia)
        .arg("atualizando")
        .arg(instalador)
        .arg(&exe)
        .args(args)
        // o app aberto pelo atalho soltou o console (FreeConsole): herdar o stdio passaria handles mortos ou reaproveitados, e o spawn falhava às vezes (erro 50)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(0x0000_0008 | 0x0000_0200) // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP: sobrevive ao app, que sai logo depois
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Apaga as cópias de atualizações anteriores (a corrente fica; as outras já acabaram). Falhar não importa.
fn limpar_antigas(base: &Path) {
    let Ok(lista) = std::fs::read_dir(base) else { return };
    for e in lista.flatten() {
        if e.file_name().to_string_lossy().starts_with(PREFIXO) {
            let _ = std::fs::remove_dir_all(e.path());
        }
    }
}

fn porta_aceita(porta: &Path, desde: SystemTime) -> bool {
    let nova = std::fs::metadata(porta).and_then(|m| m.modified()).is_ok_and(|m| m >= desde);
    nova && std::fs::read_to_string(porta).ok().and_then(|t| t.trim().parse::<u16>().ok()).is_some_and(|p| {
        std::net::TcpStream::connect_timeout(&std::net::SocketAddr::from(([127, 0, 0, 1], p)), Duration::from_millis(300)).is_ok()
    })
}

/// O trabalho de fundo: roda o instalador, espera o app novo e avisa a janela de cada etapa.
fn trabalhar(instalador: PathBuf, exe_antigo: PathBuf, args: Vec<String>, enviar: impl Fn(Evento)) {
    let inicio = SystemTime::now();
    let saida = Command::new(&instalador).args(&args).status().map(|s| s.code());
    match passos::resultado(saida) {
        Ok(()) => {
            enviar(Evento::Etapa(3));
            let porta = crate::http::config::pasta_do_app().join("porta");
            // se o app não subir em 40 s a janela fecha mesmo assim: o instalador acabou bem
            passos::esperar(Duration::from_secs(40), Duration::from_millis(250), || porta_aceita(&porta, inicio));
            enviar(Evento::Pronto);
            std::thread::sleep(Duration::from_millis(1100));
            enviar(Evento::Fechar);
        }
        Err(motivo) => {
            let _ = Command::new(&exe_antigo).spawn(); // a versão anterior segue instalada: reabre
            enviar(Evento::Erro(motivo));
        }
    }
}

/// `blueprint atualizando <instalador> <exe-instalado> [args do instalador…]`. Devolve o código de saída.
pub fn executar(args: &[String]) -> i32 {
    let [instalador, exe_antigo, resto @ ..] = args else { return 2 };
    match abrir(PathBuf::from(instalador), PathBuf::from(exe_antigo), resto.to_vec()) {
        Ok(()) => 0,
        Err(e) => {
            // sem a janela o instalador ainda precisa rodar: a atualização não depende do conforto
            eprintln!("erro: {e}");
            let _ = Command::new(instalador).args(resto).status();
            1
        }
    }
}

fn abrir(instalador: PathBuf, exe_antigo: PathBuf, args: Vec<String>) -> Resultado<()> {
    let falha = |m: &dyn std::fmt::Display| crate::erro::Erro::Io(std::io::Error::other(m.to_string()));
    let versao = instalador.file_name().and_then(|n| n.to_str()).and_then(passos::versao_do_instalador);
    let event_loop = EventLoopBuilder::<Evento>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let mut janela = WindowBuilder::new()
        .with_title("The Blueprint (beta)")
        .with_inner_size(LogicalSize::new(LARGURA, ALTURA))
        .with_decorations(false)
        .with_resizable(false)
        .with_always_on_top(true)
        .with_window_icon(super::icone::janela().ok());
    if let Some(m) = event_loop.primary_monitor() {
        let (t, p, f) = (m.size(), m.position(), m.scale_factor());
        janela = janela.with_position(PhysicalPosition::new(
            p.x + (t.width as i32 - (LARGURA * f) as i32) / 2,
            p.y + (t.height as i32 - (ALTURA * f) as i32) / 2,
        ));
    }
    let window = janela.build(&event_loop).map_err(|e| falha(&e))?;
    let pasta = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)).unwrap_or_else(std::env::temp_dir);
    let mut contexto = WebContext::new(Some(pasta.join("webview"))); // junto da cópia: não compartilha a pasta do app, que ainda pode estar fechando
    let ipc = proxy.clone();
    let webview = WebViewBuilder::new_with_web_context(&mut contexto)
        .with_html(passos::pagina(versao.as_deref()))
        .with_background_color(super::icone::PAPEL)
        .with_ipc_handler(move |req| {
            let _ = ipc.send_event(match req.body().as_str() {
                "pronta" => Evento::Carregou, // a página avisa quando o script dela já definiu as funções
                "arrastar" => Evento::Arrastar,
                _ => Evento::Fechar,
            });
        })
        .build(&window)
        .map_err(|e| falha(&e))?;
    let envio = proxy.clone();
    std::thread::spawn(move || trabalhar(instalador, exe_antigo, args, move |e| drop(envio.send_event(e))));
    // o instalador pode acabar antes de a página carregar: os avisos esperam na fila e entram na ordem
    let (mut pronta, mut fila): (bool, Vec<String>) = (false, Vec::new());
    event_loop.run(move |evento, _, fluxo| {
        *fluxo = ControlFlow::Wait;
        let script = match evento {
            Event::UserEvent(Evento::Carregou) => {
                pronta = true;
                for s in fila.drain(..) {
                    drop(webview.evaluate_script(&s));
                }
                None
            }
            Event::UserEvent(Evento::Etapa(n)) => Some(format!("etapa({n})")),
            Event::UserEvent(Evento::Pronto) => Some("pronto()".to_string()),
            Event::UserEvent(Evento::Erro(m)) => Some(format!("erro({})", serde_json::to_string(&m).unwrap_or_default())),
            Event::UserEvent(Evento::Arrastar) => {
                drop(window.drag_window());
                None
            }
            Event::UserEvent(Evento::Fechar) | Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *fluxo = ControlFlow::Exit;
                None
            }
            _ => None,
        };
        if let Some(s) = script {
            if pronta {
                drop(webview.evaluate_script(&s));
            } else {
                fila.push(s);
            }
        }
    })
}
