//! As sessões do Claude Code abertas em pastas de blueprint (contrato em docs/claude-na-folha.md, "Escolher a sessão ao iniciar a ação"). O SessionStart registra
//! (`hook/sessao`), o SessionEnd tira (`hook/fim`) e `GET /api/mcp/sessoes` lista as de um blueprint para o diálogo "Iniciar a ação". Só memória, nada grava nos dados.
//! Canal: um `blueprint --mcp` com `BLUEPRINT_CHANNEL=1` abre o SSE com `?canal=<id>&cwd=<pasta>`; enquanto a conexão vive ele é um destinatário, e o pedido de
//! iniciar uma ação vai a ele pelo evento `iniciar-acao` (ver `watch.rs`).
//! shortcut: sem pid, não dá para juntar a sessão do hook com o processo do canal. A lista mostra cada canal da pasta como entrega possível e, no máximo, as
//! sessões da pasta que sobram como "sem canal" (só copiar o prompt); uma sessão que cai sem SessionEnd fica até o app reiniciar. Trocar por pid se o uso pedir.
use super::hooks::{caminho_valido, texto};
use super::Mcp;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Quantas sessões lembrar; passou disso, esquece todas (o próximo SessionStart de cada uma a registra de novo).
pub(super) const MAX_ABERTAS: usize = 64;

pub(super) struct Sessao {
    cwd: String,
    workspace_id: String,
    blueprint_id: String,
}

pub(super) type Abertas = HashMap<String, Sessao>;
/// canal (id da sessão com canal) → pasta
pub(super) type Canais = Arc<Mutex<HashMap<String, String>>>;

/// A pasta como o banco a guarda: minúsculas e `/`.
fn normal(c: &str) -> String {
    c.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Tira o canal da lista quando a conexão SSE dele fecha.
pub(super) struct Guarda {
    canais: Canais,
    id: String,
}

impl Drop for Guarda {
    fn drop(&mut self) {
        self.canais.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.id);
    }
}

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

    /// Registra o canal de uma sessão enquanto a guarda viver. Id fora do formato (1 a 32 letras e números) ou pasta inválida: não registra.
    pub(super) fn abrir_canal(&self, id: &str, cwd: &str) -> Option<Guarda> {
        if id.is_empty() || id.len() > 32 || !id.bytes().all(|b| b.is_ascii_alphanumeric()) || !caminho_valido(cwd) {
            return None;
        }
        self.canais.lock().unwrap_or_else(|e| e.into_inner()).insert(id.into(), cwd.into());
        Some(Guarda { canais: self.canais.clone(), id: id.into() })
    }

    /// As sessões abertas do blueprint: cada canal da pasta (`canal: true`, dá para entregar) e as sessões que sobram sem canal (`canal: false`).
    pub(super) fn sessoes_do_blueprint(&self, workspace_id: &str, blueprint_id: &str) -> Value {
        let a = self.abertas.lock().unwrap_or_else(|e| e.into_inner());
        let mut do_bp: Vec<(&String, &Sessao)> = a.iter().filter(|(_, s)| s.workspace_id == workspace_id && s.blueprint_id == blueprint_id).collect();
        do_bp.sort_by(|x, y| x.0.cmp(y.0));
        let mut canais: Vec<(String, String)> = self.canais.lock().unwrap_or_else(|e| e.into_inner()).iter().map(|(i, c)| (i.clone(), c.clone())).collect();
        canais.sort();
        let mut pastas: Vec<String> = do_bp.iter().map(|(_, s)| normal(&s.cwd)).collect();
        pastas.sort();
        pastas.dedup();
        let mut saida = Vec::new();
        for pasta in pastas {
            let sessoes: Vec<_> = do_bp.iter().filter(|(_, s)| normal(&s.cwd) == pasta).collect();
            let canais_da_pasta: Vec<_> = canais.iter().filter(|(_, c)| normal(c) == pasta).collect();
            saida.extend(canais_da_pasta.iter().map(|(id, c)| json!({"id": id, "pasta": c, "canal": true})));
            saida.extend(sessoes.iter().take(sessoes.len().saturating_sub(canais_da_pasta.len())).map(|(id, s)| json!({"id": id, "pasta": s.cwd, "canal": false})));
        }
        json!({"sessoes": saida})
    }

    /// O pedido de iniciar a ação vai à sessão `canal_id`, que precisa estar conectada e ser de uma pasta deste blueprint. Erro: (status, código, frase).
    pub(super) fn entregar_pedido(&self, workspace_id: &str, blueprint_id: &str, canal_id: &str, pedido: &str) -> Result<(), (u16, &'static str, &'static str)> {
        let Some(cwd) = self.canais.lock().unwrap_or_else(|e| e.into_inner()).get(canal_id).cloned() else {
            return Err((410, "sessao-caiu", "Essa sessão não está mais conectada ao app."));
        };
        let da_pasta = self.abertas.lock().unwrap_or_else(|e| e.into_inner()).values().any(|s| s.workspace_id == workspace_id && s.blueprint_id == blueprint_id && normal(&s.cwd) == normal(&cwd));
        if !da_pasta {
            return Err((409, "sessao-de-outra-pasta", "Essa sessão não é de uma pasta deste blueprint."));
        }
        self.notificar("iniciar-acao", Some(json!({"workspaceId": workspace_id, "blueprintId": blueprint_id, "canalId": canal_id, "texto": pedido})));
        Ok(())
    }
}

/// `hook/fim`: o SessionEnd tira a sessão da lista. Sem `session_id`, não faz nada.
pub(super) fn fim(mcp: &Mcp, corpo: &Value) -> Value {
    mcp.fechar_sessao(texto(corpo, "session_id"));
    json!({})
}
