//! O MCP do The Blueprint: a fila em memória que o agente do usuário alimenta, as rotas /api/mcp (porte de mcp.mjs) e o modo `--mcp` por stdio
//! (porte de mcp-roadmap.mjs). Aqui nada grava no banco: a página aplica o que chega e só então algo muda.
pub mod acoes;
pub mod atividade;
pub(crate) mod cliente;
mod eventos;
mod fila;
mod permissao;
pub mod rotas;
pub mod stdio;

#[cfg(test)]
mod tests;

pub use eventos::FluxoSse;

use atividade::Atividade;
use fila::Fila;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;

pub struct Mcp {
    /// o endereço em que o servidor escuta (para conferir o Host do pedido)
    pub host: String,
    pub rede: bool,
    /// o que a página mostra para o usuário registrar no agente
    pub arquivo: String,
    pub url: String,
    pub prototipos: PathBuf,
    fila: Mutex<Fila>,
    /// quem está trabalhando agora: presença dos agentes (só memória, some ao reiniciar)
    atividade: Mutex<Atividade>,
    assinantes: Mutex<Vec<Sender<&'static str>>>,
}

impl Mcp {
    pub fn novo(host: &str, rede: bool, arquivo: &str, url: &str, prototipos: PathBuf) -> Mcp {
        Mcp { host: host.into(), rede, arquivo: arquivo.into(), url: url.into(), prototipos, fila: Mutex::new(Fila::default()), atividade: Mutex::new(Atividade::default()), assinantes: Mutex::new(Vec::new()) }
    }

    /// Avisa quem acompanha por SSE: "fila" (item novo) ou "agentes" (a presença mudou).
    fn avisar(&self, evento: &'static str) {
        self.assinantes.lock().unwrap_or_else(|e| e.into_inner()).retain(|s| s.send(evento).is_ok());
    }

    /// Põe um item no fim da fila, com id e hora, e avisa quem acompanha por SSE.
    pub fn empurra(&self, item: Map<String, Value>) -> Value {
        let novo = self.fila.lock().unwrap_or_else(|e| e.into_inner()).empurra(item, None);
        self.avisar("fila");
        novo
    }

    /// Como `empurra`, mas tira antes a proposta pendente do mesmo workspace e blueprint: reenviar substitui.
    pub fn empurra_proposta(&self, item: Map<String, Value>, workspace_id: &str, blueprint_id: &str) -> Value {
        let novo = self.fila.lock().unwrap_or_else(|e| e.into_inner()).empurra(item, Some((workspace_id, blueprint_id)));
        self.avisar("fila");
        novo
    }

    /// O agente `nome` reportou que está trabalhando (ver `atividade`).
    pub fn registrar_atividade(&self, nome: &str, dados: Map<String, Value>) {
        self.atividade.lock().unwrap_or_else(|e| e.into_inner()).registrar(crate::tempo::agora_ms(), nome, dados);
        self.avisar("agentes");
    }

    /// Os agentes ativos agora (os sem sinal há `VIDA_MS` não entram), do mais recente ao mais antigo.
    pub fn agentes(&self) -> Value {
        let lista = self.atividade.lock().unwrap_or_else(|e| e.into_inner()).lista(crate::tempo::agora_ms());
        serde_json::json!({"agentes": lista, "vidaMs": atividade::VIDA_MS})
    }

    pub fn itens(&self) -> Vec<Value> {
        self.fila.lock().unwrap_or_else(|e| e.into_inner()).itens.clone()
    }

    pub fn ack(&self, ids: &HashSet<i64>) {
        self.fila.lock().unwrap_or_else(|e| e.into_inner()).ack(ids);
    }

    /// Cada chamada é um acompanhante novo (uma conexão SSE); recebe o nome do evento ("fila" ou "agentes").
    pub fn assinar(&self) -> Receiver<&'static str> {
        let (tx, rx) = mpsc::channel();
        self.assinantes.lock().unwrap_or_else(|e| e.into_inner()).push(tx);
        rx
    }
}
