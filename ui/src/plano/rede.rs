//! O laço do Planejar com o servidor: pulso a cada 1 s (D5), folha quando o pulso muda e a cada 30 s, gestos com o token.
//! Canal próprio: nada aqui passa pelo `Msg` do app. A interface só desenha o que a última resposta trouxe; rodada velha é descartada, para resposta atrasada não pisar na nova.

use super::modelo::{Agentes, Chegou, Config, Folha, Pulso};
use super::Estado;
use crate::i18n::{ingles, tr, tr_com};
use crate::rede::{self as rede_app, CABECALHO_GESTO};
use serde_json::Value;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use web_time::Instant;

pub const TEXTOS: &[(&str, &str)] = &[
    ("Abra pelo app para fazer isso.", "Open the app to do this."),
    ("O plano mudou enquanto você olhava. Atualizei a folha: confira de novo.", "The plan changed while you were looking. I refreshed the sheet: check it again."),
    ("Não consegui falar com o The Blueprint.", "Could not reach The Blueprint."),
    ("O servidor respondeu {n}.", "The server answered {n}."),
];

const PULSO: Duration = Duration::from_secs(1);
const FOLHA: Duration = Duration::from_secs(30);

pub enum Msg {
    Pulso(u64, Option<Pulso>),
    Folha(u64, Result<Folha, String>),
    Agentes(u64, Result<Agentes, String>),
    Chegou(u64, Result<Chegou, String>),
    Config(u64, Result<Config, String>),
    Gesto(RespostaGesto),
}

/// O que um gesto deu, já traduzido no que a tela faz.
#[derive(Debug, PartialEq)]
pub enum RespostaGesto {
    Feito(Vec<String>),
    /// 403 `gesto-negado`: esta janela não tem o token
    SemToken,
    /// 409 `plano-mudou`: refaz a folha e avisa
    PlanoMudou,
    Recusado(String),
}

pub struct Canal {
    envio: Sender<Msg>,
    recebe: Receiver<Msg>,
    alvo: Option<(String, String)>,
    /// a última rodada pedida de folha, agentes e o que chegou; só ela vale
    rodada: u64,
    seq: u64,
    /// o pulso a caminho
    pulso_em_voo: Option<u64>,
    pulso_em: Option<Instant>,
    folha_em: Option<Instant>,
    visto: Option<(u64, u64)>,
    pedido: bool,
}

impl Default for Canal {
    fn default() -> Canal {
        let (envio, recebe) = channel();
        Canal { envio, recebe, alvo: None, rodada: 0, seq: 0, pulso_em_voo: None, pulso_em: None, folha_em: None, visto: None, pedido: false }
    }
}

impl Canal {
    pub fn pedir(&mut self) {
        self.pedido = true;
    }

    fn proximo(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }
}

/// O que um pulso novo pede: (refazer a folha, recarregar o documento). O primeiro só registra os números e confere a folha.
fn comparar(visto: Option<(u64, u64)>, novo: (u64, u64)) -> (bool, bool) {
    match visto {
        None => (true, false),
        Some(v) => (v != novo, v.0 != novo.0),
    }
}

fn devido(ultimo: Option<Instant>, agora: Instant, intervalo: Duration) -> bool {
    ultimo.is_none_or(|t| agora.duration_since(t) >= intervalo)
}

/// Id de workspace e blueprint na URL: só o que não precisa de escape passa.
fn codificar(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

fn idioma() -> &'static str {
    if ingles() {
        "en"
    } else {
        "pt-BR"
    }
}

/// Um GET do Planejar. Nunca leva o token (D7).
pub fn pedido_ler(base: &str, caminho: &str, ws: &str, bp: Option<&str>) -> ehttp::Request {
    let mut url = format!("{base}/api/mcp/planejar/{caminho}");
    if !ws.is_empty() {
        url.push_str(&format!("?workspace={}", codificar(ws)));
    }
    if let Some(bp) = bp {
        url.push_str(&format!("&blueprint={}", codificar(bp)));
    }
    let mut pedido = ehttp::Request::get(url);
    pedido.headers.insert("Accept-Language", idioma());
    pedido
}

/// O POST de um gesto: `application/json`, com o token se esta janela tem um.
pub fn pedido_gesto(base: &str, rota: &str, corpo: &Value, token: Option<&str>) -> ehttp::Request {
    let mut pedido = ehttp::Request::post(format!("{base}/api/mcp/gesto/{rota}"), serde_json::to_vec(corpo).unwrap_or_default());
    pedido.headers = ehttp::Headers::new(&[("Content-Type", "application/json"), ("Accept-Language", idioma())]);
    if let Some(token) = token {
        pedido.headers.insert(CABECALHO_GESTO, token);
    }
    pedido
}

pub fn classificar(status: u16, corpo: &[u8]) -> RespostaGesto {
    let j: Value = serde_json::from_slice(corpo).unwrap_or_default();
    let lista = |chave: &str| -> Vec<String> { j.get(chave).and_then(Value::as_array).map_or_else(Vec::new, |l| l.iter().filter_map(Value::as_str).map(str::to_string).collect()) };
    if (200..300).contains(&status) {
        return RespostaGesto::Feito(lista("avisos"));
    }
    match (status, j.get("codigo").and_then(Value::as_str)) {
        (403, Some("gesto-negado")) => RespostaGesto::SemToken,
        (409, Some("plano-mudou")) => RespostaGesto::PlanoMudou,
        _ => {
            let erros = lista("erros");
            let motivo = if erros.is_empty() { j.get("erro").and_then(Value::as_str).map_or_else(|| tr_com("O servidor respondeu {n}.", &[("n", &status.to_string())]), str::to_string) } else { erros.join(" ") };
            RespostaGesto::Recusado(motivo)
        }
    }
}

pub fn gesto(ctx: &egui::Context, canal: &Canal, servidor: &str, rota: &str, corpo: &Value) {
    let (ctx, envio) = (ctx.clone(), canal.envio.clone());
    ehttp::fetch(pedido_gesto(servidor, rota, corpo, rede_app::gesto().as_deref()), move |resposta| {
        let msg = match resposta {
            Ok(r) => classificar(r.status, &r.bytes),
            Err(_) => RespostaGesto::Recusado(tr("Não consegui falar com o The Blueprint.").to_string()),
        };
        let _ = envio.send(Msg::Gesto(msg));
        ctx.request_repaint();
    });
}

/// Busca um JSON e entrega o resultado no canal.
fn buscar<T: serde::de::DeserializeOwned + 'static>(ctx: &egui::Context, canal: &Canal, pedido: ehttp::Request, msg: impl FnOnce(Result<T, String>) -> Msg + Send + 'static) {
    let (ctx, envio) = (ctx.clone(), canal.envio.clone());
    ehttp::fetch(pedido, move |resposta| {
        let lido = match resposta {
            Err(_) => Err(tr("Não consegui falar com o The Blueprint.").to_string()),
            Ok(r) if r.status != 200 => Err(tr_com("O servidor respondeu {n}.", &[("n", &r.status.to_string())])),
            Ok(r) => serde_json::from_slice(&r.bytes).map_err(|e| e.to_string()),
        };
        let _ = envio.send(msg(lido));
        ctx.request_repaint();
    });
}

fn pedir_rodada(ctx: &egui::Context, estado: &mut Estado, ws: &str, bp: &str, servidor: &str) {
    let rodada = estado.canal.proximo();
    let canal = &mut estado.canal;
    canal.rodada = rodada;
    canal.pedido = false;
    canal.folha_em = Some(Instant::now());
    buscar(ctx, canal, pedido_ler(servidor, "folha", ws, Some(bp)), move |r| Msg::Folha(rodada, r));
    buscar(ctx, canal, pedido_ler(servidor, "agentes", ws, None), move |r| Msg::Agentes(rodada, r));
    buscar(ctx, canal, pedido_ler(servidor, "chegou", ws, Some(bp)), move |r| Msg::Chegou(rodada, r));
    buscar(ctx, canal, pedido_ler(servidor, "config", "", None), move |r| Msg::Config(rodada, r));
}

/// Aplica uma resposta ao estado. Sem rede nem tempo: é o que os testes exercitam.
pub(super) fn tratar(estado: &mut Estado, msg: Msg) {
    let rodada = estado.canal.rodada;
    match msg {
        Msg::Pulso(seq, pulso) => {
            if estado.canal.pulso_em_voo != Some(seq) {
                return;
            }
            estado.canal.pulso_em_voo = None;
            let Some(p) = pulso else { return };
            let novo = (p.versao_dados, p.revisao);
            let (folha, doc) = comparar(estado.canal.visto, novo);
            estado.canal.visto = Some(novo);
            estado.canal.pedido |= folha;
            estado.recarregar_doc |= doc;
        }
        Msg::Folha(n, _) | Msg::Agentes(n, _) | Msg::Chegou(n, _) | Msg::Config(n, _) if n != rodada => {}
        Msg::Folha(_, Ok(mut folha)) => {
            // a mesma folha de novo (a cada 30 s ela volta igual) fica com a geração que já tem: o que a tela mediu dela continua valendo
            if let Some(antes) = estado.folha.as_ref() {
                folha.geracao = antes.geracao;
            }
            if estado.folha.as_ref() != Some(&folha) {
                folha.renovar();
            }
            estado.folha = Some(folha);
            estado.folha_falhou = false;
        }
        Msg::Folha(_, Err(_)) => estado.folha_falhou = estado.folha.is_none(),
        Msg::Agentes(_, r) => estado.agentes = r.ok().or(estado.agentes.take()),
        Msg::Chegou(_, r) => estado.chegou = r.ok().or(estado.chegou.take()),
        Msg::Config(_, r) => estado.config = r.ok().or(estado.config.take()),
        Msg::Gesto(RespostaGesto::Feito(avisos)) => {
            estado.aviso = avisos.join(" ");
            estado.canal.pedido = true;
        }
        Msg::Gesto(RespostaGesto::SemToken) => estado.aviso = tr("Abra pelo app para fazer isso.").to_string(),
        Msg::Gesto(RespostaGesto::PlanoMudou) => {
            estado.aviso = tr("O plano mudou enquanto você olhava. Atualizei a folha: confira de novo.").to_string();
            estado.canal.pedido = true;
        }
        Msg::Gesto(RespostaGesto::Recusado(motivo)) => estado.aviso = motivo,
    }
}

/// Trocou de blueprint: nada do anterior fica na tela, e o que ainda vier dele é descartado.
fn trocar_alvo(estado: &mut Estado, ws: &str, bp: &str) {
    let (antigo, recarregar_doc) = (std::mem::take(&mut estado.canal), estado.recarregar_doc);
    let seq = antigo.seq + 1;
    // o mesmo canal: o que ainda vier do alvo anterior chega e é descartado pela rodada
    let canal = Canal { envio: antigo.envio, recebe: antigo.recebe, alvo: Some((ws.to_string(), bp.to_string())), rodada: seq, seq, pedido: true, pulso_em_voo: None, pulso_em: None, folha_em: None, visto: None };
    *estado = Estado { canal, recarregar_doc, ..Estado::default() };
}

/// Se o Planejar já é deste blueprint; se não for, o próximo `passo` o zera.
pub fn e_o_alvo(estado: &Estado, ws: &str, bp: &str) -> bool {
    estado.canal.alvo.as_ref().is_some_and(|(w, b)| w == ws && b == bp)
}

/// Passa o Planejar para este blueprint já, para o que vier depois não ser zerado pelo próximo `passo`.
pub fn entrar(estado: &mut Estado, ws: &str, bp: &str) {
    if !e_o_alvo(estado, ws, bp) {
        trocar_alvo(estado, ws, bp);
    }
}

/// Um quadro do Planejar: trata o que chegou, manda o pulso e a rodada que estiverem na hora.
pub fn passo(ctx: &egui::Context, estado: &mut Estado, ws: &str, bp: &str, servidor: &str) {
    entrar(estado, ws, bp);
    while let Ok(msg) = estado.canal.recebe.try_recv() {
        tratar(estado, msg);
    }
    let agora = Instant::now();
    if estado.canal.pedido || devido(estado.canal.folha_em, agora, FOLHA) {
        pedir_rodada(ctx, estado, ws, bp, servidor);
    }
    if estado.canal.pulso_em_voo.is_none() && devido(estado.canal.pulso_em, agora, PULSO) {
        let seq = estado.canal.proximo();
        estado.canal.pulso_em_voo = Some(seq);
        estado.canal.pulso_em = Some(agora);
        buscar(ctx, &estado.canal, pedido_ler(servidor, "pulso", ws, Some(bp)), move |r: Result<Pulso, String>| Msg::Pulso(seq, r.ok()));
    }
    // o laço do app só acorda a cada 3 s; aqui a tela pede 1 s enquanto está aberta
    ctx.request_repaint_after(PULSO);
}

/// Fora de um blueprint (workspaces e lista de blueprints) só o pulso: um plano que um agente criou agora tem de aparecer sem reabrir o app.
pub fn vigiar(ctx: &egui::Context, estado: &mut Estado, servidor: &str) {
    while let Ok(msg) = estado.canal.recebe.try_recv() {
        tratar(estado, msg);
    }
    let agora = Instant::now();
    if estado.canal.pulso_em_voo.is_none() && devido(estado.canal.pulso_em, agora, PULSO) {
        let seq = estado.canal.proximo();
        estado.canal.pulso_em_voo = Some(seq);
        estado.canal.pulso_em = Some(agora);
        buscar(ctx, &estado.canal, pedido_ler(servidor, "pulso", "", None), move |r: Result<Pulso, String>| Msg::Pulso(seq, r.ok()));
    }
    ctx.request_repaint_after(PULSO);
}

#[cfg(test)]
#[path = "rede_testes.rs"]
mod testes;
