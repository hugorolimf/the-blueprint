//! O MCP do The Blueprint: a fila em memória que o agente do usuário alimenta, as rotas /api/mcp (porte de mcp.mjs) e o modo `--mcp` por stdio
//! (porte de mcp-roadmap.mjs). Aqui nada grava no banco: a página aplica o que chega e só então algo muda.
pub mod acoes;
mod ajuste;
pub mod atividade;
pub(crate) mod cliente;
mod eventos;
mod fila;
mod hooks;
mod iniciar;
mod lembrete;
mod registro;
pub mod pendentes;
pub(crate) mod permissao;
pub mod rotas;
mod status;
pub mod stdio;

#[cfg(test)]
mod ajuste_tests;
#[cfg(test)]
mod hooks_tests;
#[cfg(test)]
mod iniciar_tests;
#[cfg(test)]
mod lembrete_tests;
#[cfg(test)]
mod pendentes_tests;
#[cfg(test)]
mod projetos_tests;
#[cfg(test)]
mod registro_tests;
#[cfg(test)]
mod status_tests;
#[cfg(test)]
mod tests;

pub use eventos::{Evento, FluxoSse};

use atividade::Atividade;
use fila::Fila;
use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
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
    assinantes: Mutex<Vec<SyncSender<Evento>>>,
    pub(super) pastas_avisadas: Mutex<HashSet<String>>,
    /// session_id do Claude Code → (workspace_id, blueprint_id), lembrado pelo hook de sessão (só memória; o próximo SessionStart refaz)
    sessoes: Mutex<HashMap<String, (String, String)>>,
    /// sessões com edição sem plano ainda não registrada (o Stop devolve o lembrete; ver `lembrete`)
    sem_registro: Mutex<lembrete::Pendentes>,
}

/// Quantas sessões lembrar; passou disso, esquece todas (cada hook seguinte volta a achar a pasta no banco).
const MAX_SESSOES: usize = 64;

impl Mcp {
    pub fn novo(host: &str, rede: bool, arquivo: &str, url: &str, prototipos: PathBuf) -> Mcp {
        Mcp { host: host.into(), rede, arquivo: arquivo.into(), url: url.into(), prototipos, fila: Mutex::new(Fila::default()), atividade: Mutex::new(Atividade::default()), assinantes: Mutex::new(Vec::new()), pastas_avisadas: Mutex::new(HashSet::new()), sessoes: Mutex::new(HashMap::new()), sem_registro: Mutex::new(HashMap::new()) }
    }

    pub fn lembrar_sessao(&self, session_id: &str, workspace_id: &str, blueprint_id: &str) {
        if session_id.is_empty() {
            return;
        }
        let mut s = self.sessoes.lock().unwrap_or_else(|e| e.into_inner());
        if s.len() >= MAX_SESSOES {
            s.clear();
        }
        s.insert(session_id.into(), (workspace_id.into(), blueprint_id.into()));
    }

    pub fn sessao(&self, session_id: &str) -> Option<(String, String)> {
        self.sessoes.lock().unwrap_or_else(|e| e.into_inner()).get(session_id).cloned()
    }

    /// Avisa quem acompanha por SSE: "fila" (item novo) ou "agentes" (a presença mudou).
    fn avisar(&self, evento: &'static str) {
        self.notificar(evento, None);
    }

    pub fn notificar(&self, nome: &'static str, dados: Option<Value>) {
        let evento = Evento { nome, dados };
        self.assinantes.lock().unwrap_or_else(|e| e.into_inner()).retain(|s| !matches!(s.try_send(evento.clone()), Err(TrySendError::Disconnected(_))));
    }

    /// Põe um item no fim da fila, com id e hora, e avisa quem acompanha por SSE.
    pub fn empurra(&self, item: Map<String, Value>) -> Value {
        let novo = self.fila.lock().unwrap_or_else(|e| e.into_inner()).empurra(item);
        self.avisar("fila");
        novo
    }

    /// Põe a proposta no fim da fila do alvo, com o próximo número, sem apagar nenhuma outra. Com `numero` (ajuste ou substituição), toma o lugar
    /// da proposta desse número. `entrada` é o que o agente mandou (com os ids provisórios), guardado para os próximos ajustes.
    pub fn empurra_proposta(&self, item: Map<String, Value>, entrada: Value, numero: Option<i64>) -> Value {
        let novo = self.fila.lock().unwrap_or_else(|e| e.into_inner()).empurra_proposta(item, entrada, numero);
        self.avisar("fila");
        novo
    }

    /// Põe o registro do trabalho já feito no fim da fila do alvo, com o próximo número de registro (a contagem é própria, separada da das propostas).
    pub fn empurra_registro(&self, item: Map<String, Value>) -> Value {
        let novo = self.fila.lock().unwrap_or_else(|e| e.into_inner()).empurra_registro(item);
        self.avisar("fila");
        novo
    }

    /// A entrada guardada da proposta pendente `numero` do alvo; se não existir, os números que estão na fila dele.
    pub fn entrada_da_proposta(&self, workspace_id: &str, blueprint_id: &str, numero: i64) -> Result<Value, Vec<i64>> {
        let f = self.fila.lock().unwrap_or_else(|e| e.into_inner());
        f.entrada(workspace_id, blueprint_id, numero).ok_or_else(|| f.pendentes(workspace_id, blueprint_id))
    }

    /// O agente `nome` reportou que está trabalhando (ver `atividade`).
    pub fn registrar_atividade(&self, nome: &str, dados: Map<String, Value>) {
        self.atividade.lock().unwrap_or_else(|e| e.into_inner()).registrar(crate::tempo::agora_ms(), nome, dados);
        self.avisar("agentes");
    }

    /// Um plano foi marcado como concluído: o agente que trabalhava nele passa a concluído (a página o mantém na folha até o usuário clicar).
    pub fn concluir_plano(&self, blueprint_id: &str, plano_id: &str, mensagem: &Value) {
        if self.atividade.lock().unwrap_or_else(|e| e.into_inner()).concluir_plano(crate::tempo::agora_ms(), blueprint_id, plano_id, mensagem) {
            self.avisar("agentes");
        }
    }

    /// Os agentes conhecidos (a página põe em repouso os sem sinal há `vidaMs`), do mais recente ao mais antigo.
    pub fn agentes(&self) -> Value {
        let lista = self.atividade.lock().unwrap_or_else(|e| e.into_inner()).lista();
        serde_json::json!({"agentes": lista, "vidaMs": atividade::VIDA_MS})
    }

    pub fn itens(&self) -> Vec<Value> {
        self.fila.lock().unwrap_or_else(|e| e.into_inner()).itens.clone()
    }

    pub fn ack(&self, ids: &HashSet<i64>) {
        self.fila.lock().unwrap_or_else(|e| e.into_inner()).ack(ids);
    }

    /// Tira da fila o registro que a página adicionou ou descartou; `registro` = {workspaceId, blueprintId, numero, criadaEm}.
    pub fn ack_registro(&self, registro: &Value) {
        let texto = |k: &str| registro[k].as_str().unwrap_or("").to_string();
        if let Some(numero) = registro["numero"].as_i64() {
            self.fila.lock().unwrap_or_else(|e| e.into_inner()).ack_registro(&texto("workspaceId"), &texto("blueprintId"), numero, &texto("criadaEm"));
        }
    }

    /// Tira da fila a proposta que a página aplicou ou descartou; `proposta` = {workspaceId, blueprintId, numero, criadaEm}.
    pub fn ack_proposta(&self, proposta: &Value) {
        let texto = |k: &str| proposta[k].as_str().unwrap_or("").to_string();
        if let Some(numero) = proposta["numero"].as_i64() {
            self.fila.lock().unwrap_or_else(|e| e.into_inner()).ack_proposta(&texto("workspaceId"), &texto("blueprintId"), numero, &texto("criadaEm"));
        }
    }

    /// Cada chamada é um acompanhante novo (uma conexão SSE); recebe o nome do evento ("fila" ou "agentes").
    pub fn assinar(&self) -> Receiver<Evento> {
        let (tx, rx) = mpsc::sync_channel(256);
        self.assinantes.lock().unwrap_or_else(|e| e.into_inner()).push(tx);
        rx
    }
}
