//! As sessões do Claude Code abertas em pastas de blueprint (contrato em docs/claude-na-folha.md, "Escolher a sessão ao iniciar a ação"). O SessionStart registra
//! (`hook/sessao`), o SessionEnd tira (`hook/fim`) e `GET /api/mcp/sessoes` lista as de um blueprint para o diálogo "Iniciar a ação". Só memória, nada grava nos dados.
//! shortcut: sem pid, uma sessão que caiu sem SessionEnd (Claude morto) fica na lista até o app reiniciar ou passar de `MAX_ABERTAS`; trocar por checagem de pid
//! quando a junção com o processo do canal existir.
use super::hooks::{caminho_valido, texto};
use super::Mcp;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Quantas sessões lembrar; passou disso, esquece todas (o próximo SessionStart de cada uma a registra de novo).
pub(super) const MAX_ABERTAS: usize = 64;

pub(super) struct Sessao {
    cwd: String,
    workspace_id: String,
    blueprint_id: String,
}

pub(super) type Abertas = HashMap<String, Sessao>;

impl Mcp {
    /// Registra a sessão aberta; o mesmo `session_id` (resume, clear) só atualiza.
    pub(super) fn abrir_sessao(&self, session_id: &str, cwd: &str, workspace_id: &str, blueprint_id: &str) {
        if session_id.is_empty() || !caminho_valido(cwd) {
            return;
        }
        let mut a = self.abertas.lock().unwrap_or_else(|e| e.into_inner());
        if a.len() >= MAX_ABERTAS && !a.contains_key(session_id) {
            a.clear();
        }
        a.insert(session_id.into(), Sessao { cwd: cwd.into(), workspace_id: workspace_id.into(), blueprint_id: blueprint_id.into() });
    }

    pub(super) fn fechar_sessao(&self, session_id: &str) {
        self.abertas.lock().unwrap_or_else(|e| e.into_inner()).remove(session_id);
        self.sessoes.lock().unwrap_or_else(|e| e.into_inner()).remove(session_id);
    }

    /// As sessões abertas do blueprint, por id (ordem estável). `canal` é sempre false até o registro do canal existir.
    pub(super) fn sessoes_do_blueprint(&self, workspace_id: &str, blueprint_id: &str) -> Value {
        let a = self.abertas.lock().unwrap_or_else(|e| e.into_inner());
        let mut lista: Vec<(&String, &Sessao)> = a.iter().filter(|(_, s)| s.workspace_id == workspace_id && s.blueprint_id == blueprint_id).collect();
        lista.sort_by(|x, y| x.0.cmp(y.0));
        json!({"sessoes": lista.iter().map(|(id, s)| json!({"id": id, "pasta": s.cwd, "canal": false})).collect::<Vec<_>>()})
    }
}

/// `hook/fim`: o SessionEnd tira a sessão da lista. Sem `session_id`, não faz nada.
pub(super) fn fim(mcp: &Mcp, corpo: &Value) -> Value {
    mcp.fechar_sessao(texto(corpo, "session_id"));
    json!({})
}
