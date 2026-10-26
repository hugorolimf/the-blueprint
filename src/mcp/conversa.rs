//! Enviar comentário do plano ao agente e conversar com o plano (contrato em docs/claude-na-folha.md, "Enviar comentário do plano e conversar com o plano").
//! Ida: `POST /api/mcp/conversa` (só loopback, por clique do usuário) leva o texto ao canal da sessão escolhida, pelo mesmo caminho do iniciar (`entregar_ao_canal`).
//! Volta: a ferramenta `responder_conversa` → `POST /api/mcp/conversa/resposta`; a página busca `GET /api/mcp/conversa` ao receber o evento SSE `conversa`.
//! A conversa é só memória (some ao reiniciar o app) e nada aqui grava nos dados: ajuste da IA vira proposta na fila, que o usuário aplica.
use super::acoes;
use super::hooks::{texto, travado};
use super::iniciar::limpo;
use super::status::lista;
use super::Mcp;
use crate::blueprint::numeros_das_acoes;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;

/// O texto do usuário: até 800 caracteres (o canal corta a mensagem inteira em 1.200, e o cabeçalho com os nomes ocupa o resto).
pub(super) const MAX_TEXTO: usize = 800;
pub(super) const MAX_RESPOSTA: usize = 4000;
const MAX_MENSAGENS: usize = 40;
/// passou disso, esquece todas (como as sessões): é memória de trabalho, não histórico
const MAX_CONVERSAS: usize = 32;
/// a linha `data:` do SSE que o canal lê tem teto de 2.048 bytes (`watch.rs`)
const MAX_EVENTO: usize = 1900;

pub(super) type Conversas = HashMap<String, VecDeque<Value>>;

fn chave(ws: &str, bp: &str, acao: &str) -> String {
    format!("{ws}\u{1f}{bp}\u{1f}{acao}")
}

fn id_valido(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= 100 && !id.chars().any(char::is_control)
}

/// Controle (quebra de linha inclusive) vira espaço: a mensagem do canal é uma linha só.
fn sem_controle(t: &str) -> String {
    t.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>().trim().to_string()
}

fn recusa(status: u16, erro: &str, codigo: &str) -> (u16, Value) {
    (status, json!({"erro": erro, "codigo": codigo}))
}

impl Mcp {
    fn guardar_mensagem(&self, ws: &str, bp: &str, acao: &str, mensagem: Value) {
        {
            let mut c = self.conversas.lock().unwrap_or_else(|e| e.into_inner());
            let k = chave(ws, bp, acao);
            if c.len() >= MAX_CONVERSAS && !c.contains_key(&k) {
                c.clear();
            }
            let lista = c.entry(k).or_default();
            lista.push_back(mensagem);
            while lista.len() > MAX_MENSAGENS {
                lista.pop_front();
            }
        }
        self.notificar("conversa", Some(json!({"workspaceId": ws, "blueprintId": bp, "acaoId": acao}))); // sem texto: a página busca a lista
    }

    /// As mensagens da conversa de uma ação, da mais antiga à mais nova.
    pub(super) fn conversa(&self, ws: &str, bp: &str, acao: &str) -> Value {
        let c = self.conversas.lock().unwrap_or_else(|e| e.into_inner());
        json!({"mensagens": c.get(&chave(ws, bp, acao)).map(|l| l.iter().cloned().collect::<Vec<_>>()).unwrap_or_default()})
    }
}

/// A ação (e o plano) nos dados: o cabeçalho que diz ao agente de onde vem o texto. 404 se não existir.
fn cabecalho(dados: &Mutex<Dados>, ws: &str, bp: &str, acao_id: &str, plano_id: Option<&str>) -> Result<String, ErroHttp> {
    let mut d = travado(dados);
    let tudo = d.ler_tudo(&Contexto::local())?;
    let n = acoes::numero_do_blueprint(tudo, ws, bp);
    let estrutura = acoes::blueprint(tudo, ws, bp, None)?;
    let p = &estrutura["blueprint"];
    let lista_acoes = lista(&p["acoes"]);
    let i = lista_acoes.iter().position(|a| texto(a, "id") == acao_id).ok_or_else(|| falha(404, "Ação não encontrada neste blueprint."))?;
    let acao = &lista_acoes[i];
    let ids = format!("workspace_id {ws}, blueprint_id {bp}, acao_id {acao_id}");
    let da_acao = format!("{} «{}» do blueprint «{}»", numeros_das_acoes(p, n)[i], limpo(texto(acao, "nome")), limpo(texto(p, "nome")));
    Ok(match plano_id {
        Some(pl) => {
            let plano = lista(&acao["planos"]).iter().find(|x| texto(x, "id") == pl).ok_or_else(|| falha(404, "Plano não encontrado nesta ação."))?;
            format!("Comentário no plano «{}» (plano_id {pl}) da ação {da_acao} ({ids})", limpo(texto(plano, "texto")))
        }
        None => format!("Ação {da_acao} ({ids})"),
    })
}

/// `POST /api/mcp/conversa`: o usuário clicou em Enviar (comentário do plano ou mensagem da conversa). Entrega ao canal da sessão escolhida ou, com
/// `semCanal: true`, só devolve o `prompt` para a página copiar.
pub(super) fn enviar(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<(u16, Value), ErroHttp> {
    let (ws, bp, acao, sessao) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"), texto(corpo, "acaoId"), texto(corpo, "sessaoId"));
    let copiar = corpo["semCanal"] == true; // só copiar: não precisa de sessão
    if ![ws, bp, acao].iter().all(|i| id_valido(i)) || (!copiar && !id_valido(sessao)) {
        return Err(falha(400, "Informe workspaceId, blueprintId, acaoId e sessaoId."));
    }
    let (evento, plano) = match texto(corpo, "tipo") {
        "comentario" => {
            let pl = texto(corpo, "planoId");
            if !id_valido(pl) {
                return Err(falha(400, "Informe o planoId do comentário."));
            }
            ("comentario-plano", Some(pl))
        }
        "conversa" => ("conversa-usuario", None),
        _ => return Err(falha(400, "tipo deve ser comentario ou conversa.")),
    };
    let msg = sem_controle(texto(corpo, "texto"));
    if msg.is_empty() {
        return Err(falha(400, "Escreva o texto."));
    }
    if msg.chars().count() > MAX_TEXTO {
        return Ok(recusa(413, "Texto longo demais (até 800 caracteres).", "texto-longo"));
    }
    let prompt = format!("{}: {msg}", cabecalho(dados, ws, bp, acao, plano)?);
    if !copiar {
        let tamanho = json!({"workspaceId": ws, "blueprintId": bp, "canalId": sessao, "texto": prompt}).to_string().len();
        if tamanho > MAX_EVENTO {
            return Ok(recusa(413, "Texto longo demais para o canal; encurte.", "texto-longo"));
        }
        if let Err((s, codigo, frase)) = mcp.entregar_ao_canal(evento, ws, bp, sessao, &prompt) {
            return Ok(recusa(s, frase, codigo));
        }
    }
    if plano.is_none() {
        mcp.guardar_mensagem(ws, bp, acao, json!({"de": "usuario", "texto": msg, "em": crate::tempo::agora_ms()}));
    }
    Ok((200, json!({"ok": true, "entregue": !copiar, "prompt": prompt})))
}

/// `POST /api/mcp/conversa/resposta`: a IA responde pela ferramenta `responder_conversa`. `proposta`, se vier, é uma proposta pendente na fila do blueprint.
pub(super) fn responder(mcp: &Mcp, dados: &Mutex<Dados>, corpo: &Value) -> Result<(u16, Value), ErroHttp> {
    let (ws, bp, acao) = (texto(corpo, "workspaceId"), texto(corpo, "blueprintId"), texto(corpo, "acaoId"));
    if ![ws, bp, acao].iter().all(|i| id_valido(i)) {
        return Err(falha(400, "Informe workspace_id, blueprint_id e acao_id."));
    }
    let resposta = texto(corpo, "texto").trim();
    if resposta.is_empty() || resposta.chars().count() > MAX_RESPOSTA {
        return Err(falha(400, "texto é obrigatório e vai até 4000 caracteres."));
    }
    cabecalho(dados, ws, bp, acao, None)?; // a ação existe
    let mut mensagem = json!({"de": "agente", "texto": resposta, "em": crate::tempo::agora_ms()});
    if let Some(p) = corpo.get("proposta").filter(|v| !v.is_null()) {
        let n = p.as_i64().ok_or_else(|| falha(400, "proposta é o número de uma proposta da fila."))?;
        if let Err(pendentes) = mcp.entrada_da_proposta(ws, bp, n) {
            return Err(falha(409, format!("A proposta nº {n} não está na fila deste blueprint (pendentes: {pendentes:?}).")));
        }
        mensagem["proposta"] = json!(n);
    }
    mcp.guardar_mensagem(ws, bp, acao, mensagem);
    Ok((200, json!({"ok": true})))
}

/// A ferramenta MCP `responder_conversa` (stdio): monta o corpo e chama a rota pelo `servidor` do stdio.
pub(super) fn ferramenta(a: &Value, servidor: &dyn Fn(&str, &str, Option<&Value>) -> Result<Value, String>) -> Result<String, String> {
    let campo = |k: &str| a.get(k).and_then(Value::as_str).unwrap_or("").trim().to_string();
    if campo("texto").is_empty() {
        return Err("texto é obrigatório.".into());
    }
    let corpo = json!({"workspaceId": campo("workspace_id"), "blueprintId": campo("blueprint_id"), "acaoId": campo("acao_id"), "texto": campo("texto"), "proposta": a.get("proposta").cloned().unwrap_or(Value::Null)});
    servidor("conversa/resposta", "POST", Some(&corpo))?;
    Ok("Resposta mostrada ao usuário na conversa da ação. Lembre: mudanças no blueprint só por propor_blueprint; o usuário aplica.".into())
}
