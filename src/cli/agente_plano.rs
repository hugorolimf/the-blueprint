//! `blueprint agente-plano <execucao_id>`: o invólucro que o motor do Planejar abre no terminal, com só um id hexadecimal na linha de comando (contrato, D6 e seção 7).
//! Todo o resto vem do app por HTTP (`execucao/partida`), então o texto do plano nunca passa pelo terminal nem por shell. Daqui para a frente o agente é um filho deste
//! processo: um sinal a cada 30 s diz ao app que ele vive (e traz o pedido de parar), e `execucao/fim` leva o código de saída.
use crate::instancia::url_do_app;
use crate::mcp::cliente::requisitar;
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

const INTERVALO_SINAL: Duration = if cfg!(test) { Duration::from_millis(100) } else { Duration::from_secs(30) };
const ESPERA_TENTATIVA: Duration = if cfg!(test) { Duration::from_millis(50) } else { Duration::from_secs(1) };
const OLHAR_FILHO: Duration = if cfg!(test) { Duration::from_millis(20) } else { Duration::from_millis(200) };
/// O motor abriu o terminal há pouco, mas o app pode estar reabrindo em outra porta: cada tentativa relê a porta.
const TENTATIVAS_PARTIDA: u32 = 20;
const TENTATIVAS_FIM: u32 = 3;
const CODIGO_SEM_COMANDO: i32 = 127;
/// O código que um processo encerrado por SIGTERM teria: é o que `fim` leva quando o app mandou parar.
const CODIGO_PARADO: i32 = 143;

struct Partida {
    segredo: String,
    prompt: String,
    comando: Vec<String>,
    claude: bool,
    nome_agente: String,
    pasta: String,
}

fn id_valido(id: &str) -> bool {
    id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

fn ler_partida(v: &Value) -> Option<Partida> {
    let texto = |campo: &str| v.get(campo)?.as_str().map(str::to_string);
    let comando: Vec<String> = v.get("comando")?.as_array()?.iter().map(|a| a.as_str().map(str::to_string)).collect::<Option<_>>()?;
    let claude = v.get("claude").and_then(Value::as_bool).unwrap_or(false);
    let segredo = texto("segredo").filter(|s| !s.is_empty())?;
    let pasta = texto("pasta").filter(|p| !p.is_empty())?;
    if comando.is_empty() && !claude {
        return None;
    }
    Some(Partida { segredo, prompt: texto("prompt")?, comando, claude, nome_agente: texto("nomeAgente").unwrap_or_default(), pasta })
}

/// O motivo que o app deu, ou o corpo cru se ele não veio no formato `{erro}`.
fn motivo_da_recusa(status: u16, corpo: &str) -> String {
    let erro = serde_json::from_str::<Value>(corpo).ok().and_then(|v| v.get("erro")?.as_str().map(str::to_string));
    format!("{status}: {}", erro.unwrap_or_else(|| corpo.to_string()))
}

/// Pede a partida. O app que respondeu recusando é resposta final; só sem resposta se tenta de novo.
fn buscar_partida(id: &str) -> Result<(String, Partida), String> {
    let corpo = json!({ "id": id }).to_string();
    let mut ultimo = crate::tr!("The Blueprint app was not found: it is closed and BLUEPRINT_URL is not set.", "O app The Blueprint não foi encontrado: está fechado e BLUEPRINT_URL não está definida.");
    for tentativa in 0..TENTATIVAS_PARTIDA {
        if tentativa > 0 {
            sleep(ESPERA_TENTATIVA);
        }
        let Some(url) = url_do_app() else { continue };
        match requisitar(&url, "POST", "execucao/partida", Some(&corpo)) {
            Ok((200, texto)) => {
                let partida = serde_json::from_str::<Value>(&texto).ok().and_then(|v| ler_partida(&v));
                return partida.map(|p| (url, p)).ok_or_else(|| crate::tr!("the app sent an invalid start", "o app mandou uma partida inválida"));
            }
            Ok((status, texto)) => return Err(motivo_da_recusa(status, &texto)),
            Err(e) => ultimo = e,
        }
    }
    Err(ultimo)
}

/// `claude::montar_comando` consulta o app e grava os hooks pelo `BLUEPRINT_URL` deste processo, que aqui pode nem existir (o app foi achado pelo arquivo `porta`).
/// A variável só vale durante a montagem, porque depois o sinal relê a porta a cada tentativa; ainda não há outra thread neste ponto.
fn com_url_no_ambiente<T>(url: &str, montar: impl FnOnce() -> T) -> T {
    let antes: Option<OsString> = std::env::var_os("BLUEPRINT_URL");
    std::env::set_var("BLUEPRINT_URL", url);
    let resultado = montar();
    match antes {
        Some(valor) => std::env::set_var("BLUEPRINT_URL", valor),
        None => std::env::remove_var("BLUEPRINT_URL"),
    }
    resultado
}

/// Sem shell e sem canal: o prompt é o último argumento do filho, nunca da linha do terminal.
fn preparar_comando(id: &str, url: &str, p: &Partida) -> Result<Command, i32> {
    let resto = p.comando.get(1..).unwrap_or(&[]);
    let mut comando = if p.claude {
        let extras: Vec<String> = resto.iter().chain(std::iter::once(&p.prompt)).cloned().collect();
        com_url_no_ambiente(url, || super::claude::montar_comando(&extras))?
    } else {
        let Some(cli) = crate::registrar::procurar(&p.comando[0]) else {
            eprintln!("{}", crate::tr!("error: command not found: {}", "erro: comando não encontrado: {}", p.comando[0]));
            return Err(CODIGO_SEM_COMANDO);
        };
        let mut comando = Command::new(cli);
        comando.args(resto).arg(&p.prompt);
        comando
    };
    comando
        .env("BLUEPRINT_EXECUCAO", id)
        .env("BLUEPRINT_EXECUCAO_SEGREDO", &p.segredo)
        .env("BLUEPRINT_AGENTE_NOME", &p.nome_agente)
        .env("BLUEPRINT_MCP_PERFIL", "executor")
        .env("BLUEPRINT_URL", url);
    Ok(comando)
}

/// Roda até o processo acabar. Falha de sinal só tenta de novo: app fechado não derruba o agente.
fn sinalizar(id: String, segredo: String, parar: Arc<AtomicBool>) {
    let corpo = json!({ "id": id, "segredo": segredo }).to_string();
    loop {
        let resposta = url_do_app().and_then(|url| requisitar(&url, "POST", "execucao/sinal", Some(&corpo)).ok());
        if let Some((200, texto)) = resposta {
            if serde_json::from_str::<Value>(&texto).is_ok_and(|v| v["parar"] == true) {
                parar.store(true, Ordering::SeqCst);
                return;
            }
        }
        sleep(INTERVALO_SINAL);
    }
}

/// Relê a porta a cada tentativa. Se o app não responder, ele percebe sozinho pela falta de sinal (`sumiu`).
fn avisar_fim(id: &str, segredo: &str, codigo: i32) {
    let corpo = json!({ "id": id, "segredo": segredo, "codigo": codigo }).to_string();
    for tentativa in 0..TENTATIVAS_FIM {
        if tentativa > 0 {
            sleep(ESPERA_TENTATIVA);
        }
        if url_do_app().is_some_and(|url| requisitar(&url, "POST", "execucao/fim", Some(&corpo)).is_ok()) {
            return;
        }
    }
    eprintln!("{}", crate::tr!("warning: could not tell the app that the agent ended", "aviso: não consegui avisar o app de que o agente terminou"));
}

/// Líder do grupo (o caso do terminal): SIGTERM no grupo inteiro, que leva os netos e este processo junto. O filho não vai para grupo próprio: TUI em grupo de fundo
/// não lê o terminal. Fora disso (o grupo é de quem nos chamou), só o filho.
fn encerrar_filho(filho: &mut Child) {
    #[cfg(unix)]
    // SAFETY: chamadas sem ponteiros; `killpg(0, ..)` vale para o grupo deste processo, que acabamos de conferir ser o nosso.
    unsafe {
        if libc::getpgrp() == libc::getpid() {
            libc::killpg(0, libc::SIGTERM);
        }
    }
    let _ = filho.kill();
    let _ = filho.wait();
}

/// Espera o filho acabar ou o app mandar parar, e avisa o fim. Avisa antes de matar, porque no caso do líder este processo morre junto.
fn acompanhar(id: &str, segredo: &str, filho: &mut Child, parar: &AtomicBool) -> i32 {
    loop {
        match filho.try_wait() {
            Ok(Some(status)) => {
                let codigo = status.code().unwrap_or(1);
                avisar_fim(id, segredo, codigo);
                return codigo;
            }
            Ok(None) if parar.load(Ordering::SeqCst) => {
                avisar_fim(id, segredo, CODIGO_PARADO);
                encerrar_filho(filho);
                return CODIGO_PARADO;
            }
            Ok(None) => sleep(OLHAR_FILHO),
            Err(e) => {
                eprintln!("{}", crate::tr!("error: lost track of the agent: {}", "erro: perdi o agente de vista: {}", e));
                avisar_fim(id, segredo, 1);
                return 1;
            }
        }
    }
}

fn rodar(id: &str, url: &str, p: &Partida) -> i32 {
    let desistir = |codigo: i32| {
        avisar_fim(id, &p.segredo, codigo);
        codigo
    };
    if let Err(e) = std::env::set_current_dir(Path::new(&p.pasta)) {
        eprintln!("{}", crate::tr!("error: could not enter {}: {}", "erro: não consegui entrar em {}: {}", p.pasta, e));
        return desistir(1);
    }
    let mut comando = match preparar_comando(id, url, p) {
        Ok(comando) => comando,
        Err(codigo) => return desistir(codigo),
    };
    let mut filho = match comando.spawn() {
        Ok(filho) => filho,
        Err(e) => {
            eprintln!("{}", crate::tr!("error: could not start the agent: {}", "erro: não consegui iniciar o agente: {}", e));
            return desistir(CODIGO_SEM_COMANDO);
        }
    };
    let parar = Arc::new(AtomicBool::new(false));
    let (id_sinal, segredo_sinal, parar_sinal) = (id.to_string(), p.segredo.clone(), Arc::clone(&parar));
    std::thread::spawn(move || sinalizar(id_sinal, segredo_sinal, parar_sinal));
    acompanhar(id, &p.segredo, &mut filho, &parar)
}

/// Devolve o código de saída: o do agente, 2 com id inválido, 1 se não houve partida (sem `fim`: sem partida não há segredo, e o motor percebe pela falta de sinal).
pub fn executar(id: &str) -> i32 {
    if !id_valido(id) {
        eprintln!("{}", crate::tr!("agente-plano: the id must have 16 hexadecimal digits", "agente-plano: o id precisa ter 16 dígitos hexadecimais"));
        return 2;
    }
    match buscar_partida(id) {
        Ok((url, partida)) => rodar(id, &url, &partida),
        Err(e) => {
            eprintln!("{}", crate::tr!("agente-plano: could not start: {}", "agente-plano: não consegui partir: {}", e));
            1
        }
    }
}

#[cfg(all(test, unix))]
#[path = "agente_plano_tests.rs"]
mod tests;
