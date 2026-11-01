//! A conversa com o servidor: ler os dados, gravar o documento inteiro e buscar o exemplo. O mesmo `ehttp` serve o desktop (numa thread)
//! e o navegador (`fetch`), então não há código por sistema aqui, só o endereço. Cada resposta volta por um canal e acorda a tela.

use crate::documento::Documento;
use serde_json::Value;
use std::sync::mpsc::Sender;

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

/// `PUT /api/dados` com a versão lida em `If-Match`: se outra janela gravou antes, o servidor recusa (409) em vez de pisar nela.
pub fn gravar(base: &str, ctx: &egui::Context, envio: Sender<Msg>, doc: &Documento) {
    let (ctx, base) = (ctx.clone(), base.to_string());
    let Ok(corpo) = serde_json::to_vec(&doc.raw) else {
        return entregar(&ctx, &envio, Msg::Gravado(Err(Falha::Outra("Não consegui montar os dados para gravar.".into()))));
    };
    let mut pedido = ehttp::Request::post(format!("{base}/api/dados"), corpo);
    pedido.method = ehttp::Method::PUT;
    // todos de uma vez: o `post` já traz um Content-Type de texto, e o servidor só aceita application/json
    pedido.headers = ehttp::Headers::new(&[("Content-Type", "application/json"), ("If-Match", &doc.versao.to_string())]);
    ehttp::fetch(pedido, move |resposta| {
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
