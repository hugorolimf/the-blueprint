//! A conversa com o servidor: ler os dados, gravar o documento inteiro e buscar o exemplo. O mesmo `ehttp` serve o desktop (numa thread)
//! e o navegador (`fetch`), então não há código por sistema aqui, só o endereço. Cada resposta volta por um canal e acorda a tela.

use crate::documento::Documento;
use serde_json::Value;
use std::sync::mpsc::Sender;
use std::sync::RwLock;

/// O cabeçalho que prova que o pedido veio da janela (o mesmo nome em `src/mcp/planejar/mod.rs`).
pub const CABECALHO_GESTO: &str = "X-Blueprint-Gesto";

/// O token do gesto do usuário. Só o programa `blueprint` o define, por chamada de função; nunca vem de env nem de arquivo e nunca vai em GET nem em log.
static GESTO: RwLock<Option<String>> = RwLock::new(None);

/// A janela ganhou o token do servidor que está no mesmo processo (`nativa.rs`).
pub fn definir_gesto(token: String) {
    *GESTO.write().unwrap_or_else(|e| e.into_inner()) = Some(token);
}

/// O token, se esta janela tem um (no navegador e com `cargo run` contra um servidor à parte, não tem: fica só leitura).
pub fn gesto() -> Option<String> {
    GESTO.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Por que uma gravação não entrou.
pub enum Falha {
    /// outra janela gravou no meio (409): a tela relê os dados
    Desatualizado,
    Outra(String),
}

pub enum Msg {
    Dados(Result<Documento, String>),
    Gravado(Result<u64, Falha>),
    Exemplo(Result<Value, String>),
    /// os agentes ativos e a vida deles em ms
    Agentes(Vec<crate::agentes::Agente>, i64),
}

/// O endereço do servidor: no navegador, a própria origem da página (`/api` passa pelo mesmo servidor, sem CORS); no desktop,
/// `BLUEPRINT_URL` ou o app local na porta 3000 (o mesmo padrão do `blueprint doctor`).
pub fn servidor() -> String {
    #[cfg(target_arch = "wasm32")]
    return String::new();
    #[cfg(not(target_arch = "wasm32"))]
    std::env::var("BLUEPRINT_URL").ok().filter(|v| !v.is_empty()).map_or_else(|| "http://127.0.0.1:3000".to_string(), |v| v.trim_end_matches('/').to_string())
}

fn sem_conexao(base: &str) -> String {
    format!("Não consegui falar com o The Blueprint em {}. Abra o app ou rode `blueprint serve`.", if base.is_empty() { "esta página" } else { base })
}

fn entregar(ctx: &egui::Context, envio: &Sender<Msg>, msg: Msg) {
    let _ = envio.send(msg);
    ctx.request_repaint();
}

fn versao_de(r: &ehttp::Response) -> Option<u64> {
    r.headers.get("x-dados-versao")?.trim().parse().ok()
}

pub fn buscar(base: &str, ctx: &egui::Context, envio: Sender<Msg>) {
    let (ctx, base) = (ctx.clone(), base.to_string());
    ehttp::fetch(ehttp::Request::get(format!("{base}/api/dados")), move |resposta| {
        let resultado = match resposta {
            Err(_) => Err(sem_conexao(&base)),
            Ok(r) if r.status != 200 => Err(format!("O servidor respondeu {}.", r.status)),
            Ok(r) => serde_json::from_slice::<Value>(&r.bytes).map_err(|e| format!("Os dados vieram num formato que não entendi: {e}")).map(|raw| Documento { raw, versao: versao_de(&r).unwrap_or(0) }),
        };
        entregar(&ctx, &envio, Msg::Dados(resultado));
    });
}

fn pedido_gravar(base: &str, corpo: Vec<u8>, versao: u64, token: Option<&str>) -> ehttp::Request {
    let mut pedido = ehttp::Request::post(format!("{base}/api/dados"), corpo);
    pedido.method = ehttp::Method::PUT;
    // todos de uma vez: o `post` já traz um Content-Type de texto, e o servidor só aceita application/json
    pedido.headers = ehttp::Headers::new(&[("Content-Type", "application/json"), ("If-Match", &versao.to_string())]);
    if let Some(token) = token {
        pedido.headers.insert(CABECALHO_GESTO, token);
    }
    pedido
}

/// `PUT /api/dados` com a versão lida em `If-Match`: se outra janela gravou antes, o servidor recusa (409) em vez de pisar nela.
pub fn gravar(base: &str, ctx: &egui::Context, envio: Sender<Msg>, doc: &Documento) {
    let (ctx, base) = (ctx.clone(), base.to_string());
    let Ok(corpo) = serde_json::to_vec(&doc.raw) else {
        return entregar(&ctx, &envio, Msg::Gravado(Err(Falha::Outra("Não consegui montar os dados para gravar.".into()))));
    };
    ehttp::fetch(pedido_gravar(&base, corpo, doc.versao, gesto().as_deref()), move |resposta| {
        let resultado = match resposta {
            Err(_) => Err(Falha::Outra(sem_conexao(&base))),
            Ok(r) if r.status == 409 => Err(Falha::Desatualizado),
            Ok(r) if r.status != 200 => Err(Falha::Outra(format!("O servidor respondeu {}.", r.status))),
            Ok(r) => Ok(versao_de(&r).unwrap_or(0)),
        };
        entregar(&ctx, &envio, Msg::Gravado(resultado));
    });
}

/// O modelo de exemplo (`GET /api/exemplo`), no idioma do servidor.
pub fn exemplo(base: &str, ctx: &egui::Context, envio: Sender<Msg>) {
    let (ctx, base) = (ctx.clone(), base.to_string());
    ehttp::fetch(ehttp::Request::get(format!("{base}/api/exemplo")), move |resposta| {
        let resultado = match resposta {
            Err(_) => Err(sem_conexao(&base)),
            Ok(r) if r.status != 200 => Err(format!("O servidor respondeu {}.", r.status)),
            Ok(r) => serde_json::from_slice(&r.bytes).map_err(|e| e.to_string()),
        };
        entregar(&ctx, &envio, Msg::Exemplo(resultado));
    });
}

/// `GET /api/mcp/agentes`: quem está trabalhando. Falha em silêncio (servidor sem a lista: sem figuras).
pub fn agentes(base: &str, ctx: &egui::Context, envio: Sender<Msg>) {
    let (ctx, base) = (ctx.clone(), base.to_string());
    ehttp::fetch(ehttp::Request::get(format!("{base}/api/mcp/agentes")), move |resposta| {
        let Ok(r) = resposta else { return };
        let Ok(j) = serde_json::from_slice::<Value>(&r.bytes) else { return };
        let lista = serde_json::from_value(j.get("agentes").cloned().unwrap_or_default()).unwrap_or_default();
        entregar(&ctx, &envio, Msg::Agentes(lista, j.get("vidaMs").and_then(Value::as_i64).unwrap_or(600_000)));
    });
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::io::{Read, Write};
    use std::time::Duration;

    /// De ponta a ponta, num servidor de verdade: o token definido sai no `PUT`, junto do `If-Match`.
    #[test]
    fn gravar_leva_gesto() {
        let servidor = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", servidor.local_addr().unwrap());
        let leitor = std::thread::spawn(move || {
            let (mut conexao, _) = servidor.accept().unwrap();
            conexao.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
            let (mut lido, mut bloco) = (Vec::new(), [0u8; 4096]);
            while !String::from_utf8_lossy(&lido).contains("{\"a\":1}") {
                match conexao.read(&mut bloco) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => lido.extend_from_slice(&bloco[..n]),
                }
            }
            conexao.write_all(b"HTTP/1.1 200 OK\r\nx-dados-versao: 8\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            String::from_utf8_lossy(&lido).to_lowercase()
        });
        definir_gesto("token-de-teste".into());
        let (envio, recebe) = std::sync::mpsc::channel();
        let doc = Documento { raw: serde_json::json!({"a": 1}), versao: 7 };
        gravar(&base, &egui::Context::default(), envio, &doc);
        let pedido = leitor.join().unwrap();
        assert!(pedido.starts_with("put /api/dados "), "{pedido}");
        assert!(pedido.contains("x-blueprint-gesto: token-de-teste"), "{pedido}");
        assert!(pedido.contains("if-match: 7"), "{pedido}");
        assert!(pedido.contains("content-type: application/json"), "{pedido}");
        assert!(matches!(recebe.recv_timeout(Duration::from_secs(5)), Ok(Msg::Gravado(Ok(8)))));
        assert_eq!(gesto().as_deref(), Some("token-de-teste"));
    }

    #[test]
    fn gravar_sem_token_nao_inventa_cabecalho() {
        let pedido = pedido_gravar("", vec![], 1, None);
        assert_eq!(pedido.headers.get(CABECALHO_GESTO), None);
        assert_eq!(pedido.headers.get("If-Match"), Some("1"));
    }
}
