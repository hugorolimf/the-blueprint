//! O apoio dos testes do invólucro: o app falso, o agente em script e o invólucro em processo próprio.
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use std::fs;

pub(super) const ID: &str = "0123456789abcdef";
pub(super) const SEGREDO: &str = "segredo-da-tentativa";

pub(super) struct Servidor {
    pub(super) porta: u16,
    pedidos: Arc<Mutex<Vec<(String, Value)>>>,
    pub(super) parar: Arc<AtomicBool>,
}

impl Servidor {
    pub(super) fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.porta)
    }
    pub(super) fn de(&self, rota: &str) -> Vec<Value> {
        self.pedidos.lock().unwrap().iter().filter(|(r, _)| r == &format!("/api/mcp/{rota}")).map(|(_, c)| c.clone()).collect()
    }
}

fn ler_pedido(conexao: &mut std::net::TcpStream) -> Option<(String, Value)> {
    let mut bruto = Vec::new();
    let mut pedaco = [0u8; 4096];
    loop {
        if let Some(fim) = bruto.windows(4).position(|w| w == b"\r\n\r\n") {
            let cabecalho = String::from_utf8_lossy(&bruto[..fim]).to_lowercase();
            let tamanho: usize = cabecalho.lines().find_map(|l| l.strip_prefix("content-length: ")?.trim().parse().ok()).unwrap_or(0);
            if bruto.len() >= fim + 4 + tamanho {
                let rota = String::from_utf8_lossy(&bruto[..fim]).split_whitespace().nth(1)?.to_string();
                return Some((rota, serde_json::from_slice(&bruto[fim + 4..fim + 4 + tamanho]).unwrap_or(Value::Null)));
            }
        }
        let n = conexao.read(&mut pedaco).ok()?;
        if n == 0 {
            return None;
        }
        bruto.extend_from_slice(&pedaco[..n]);
    }
}

/// `partida`: a resposta de `execucao/partida`, com o status. Fica no ar até o fim do binário de teste.
pub(super) fn servidor(status: u16, partida: Value) -> Servidor {
    let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
    let porta = ouvinte.local_addr().unwrap().port();
    let pedidos = Arc::new(Mutex::new(Vec::new()));
    let parar = Arc::new(AtomicBool::new(false));
    let (registro, pedido_de_parar) = (Arc::clone(&pedidos), Arc::clone(&parar));
    thread::spawn(move || {
        for conexao in ouvinte.incoming().flatten() {
            let mut conexao = conexao;
            let Some((rota, corpo)) = ler_pedido(&mut conexao) else { continue };
            let (codigo, resposta) = match rota.as_str() {
                "/api/mcp/execucao/partida" => (status, partida.clone()),
                "/api/mcp/execucao/sinal" => (200, json!({ "parar": pedido_de_parar.load(Ordering::SeqCst) })),
                "/api/mcp/hook/status" => (200, json!({ "vinculado": true })),
                _ => (200, json!({})),
            };
            registro.lock().unwrap().push((rota, corpo));
            let texto = resposta.to_string();
            let _ = write!(conexao, "HTTP/1.1 {codigo} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{texto}", texto.len());
        }
    });
    Servidor { porta, pedidos, parar }
}

pub(super) fn pasta_temporaria(nome: &str) -> PathBuf {
    let carimbo = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let pasta = std::env::temp_dir().join(format!("blueprint-involucro-{nome}-{}-{carimbo}", std::process::id()));
    fs::create_dir_all(&pasta).unwrap();
    pasta
}

pub(super) fn script(pasta: &Path, nome: &str, corpo: &str) -> String {
    let caminho = pasta.join(nome);
    fs::write(&caminho, format!("#!/bin/sh\n{corpo}\n")).unwrap();
    fs::set_permissions(&caminho, fs::Permissions::from_mode(0o755)).unwrap();
    caminho.display().to_string()
}

pub(super) fn partida(comando: &[&str], pasta: &Path, prompt: &str) -> Value {
    json!({ "tentativa": 1, "segredo": SEGREDO, "prompt": prompt, "comando": comando, "claude": false, "nomeAgente": "plano 1.2", "pasta": pasta.display().to_string() })
}

/// O invólucro em processo próprio, sem nada do ambiente de quem roda o teste apontando para um app de verdade.
pub(super) fn involucro(id: &str, ambiente: &[(&str, &str)]) -> Command {
    let mut comando = Command::new(std::env::current_exe().unwrap());
    comando.args(["--exact", "cli::agente_plano::tests::auxiliar", "--nocapture"]).env("BLUEPRINT_TESTE_INVOLUCRO", id).env_remove("BLUEPRINT_URL").env_remove("BLUEPRINT_CHAVE");
    comando.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    comando.envs(ambiente.iter().copied());
    comando
}

pub(super) fn esperar_ate(condicao: impl Fn() -> bool) -> bool {
    let inicio = Instant::now();
    while inicio.elapsed() < Duration::from_secs(10) {
        if condicao() {
            return true;
        }
        sleep(Duration::from_millis(10));
    }
    false
}

pub(super) fn terminar(filho: &mut Child) -> ExitStatus {
    let inicio = Instant::now();
    loop {
        if let Some(status) = filho.try_wait().unwrap() {
            return status;
        }
        if inicio.elapsed() > Duration::from_secs(15) {
            let _ = filho.kill();
            panic!("o invólucro não terminou");
        }
        sleep(Duration::from_millis(10));
    }
}

pub(super) fn rodar_ate_o_fim(s: &Servidor) -> ExitStatus {
    terminar(&mut involucro(ID, &[("BLUEPRINT_URL", &s.url())]).spawn().unwrap())
}

/// Processo já acabou (ou é zumbi de outro pai, que não roda mais)?
pub(super) fn morto(pid: i32) -> bool {
    let estado = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    // SAFETY: sinal 0 só confere se o processo existe.
    let existe = unsafe { libc::kill(pid, 0) } == 0;
    !existe || estado.rsplit(") ").next().is_some_and(|resto| resto.starts_with('Z'))
}

/// A porta anotada, como o app faz (`instancia::registrar_porta`), numa pasta do app só deste teste.
pub(super) fn anotar_porta(dados: &Path, porta: u16) {
    fs::create_dir_all(dados.join("Blueprint")).unwrap();
    fs::write(dados.join("Blueprint/porta"), porta.to_string()).unwrap();
}

/// A 1 nunca escuta nada aqui. Uma porta livre pedida ao sistema pode cair num servidor de outro teste, que roda ao mesmo tempo.
pub(super) const PORTA_FECHADA: u16 = 1;
