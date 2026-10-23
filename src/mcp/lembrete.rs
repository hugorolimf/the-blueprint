//! Lembrete de registro: uma edição sem plano (o hook de edição não herdou plano) deixa a sessão "sem registro"; no Stop do turno o app devolve
//! um lembrete único para o agente chamar `registrar_trabalho` e perguntar ao usuário. Só memória, como as sessões; nada aqui grava nos dados.
use super::hooks::{texto, travado};
use super::Mcp;
use serde_json::{json, Value};

/// Sessão → (workspace_id, blueprint_id) das sessões com edição sem plano ainda não registrada.
pub(super) type Pendentes = std::collections::HashMap<String, (String, String)>;

/// Uma edição que não herdou plano: a sessão fica pendente de registro (no blueprint da pasta).
pub fn editou_sem_plano(mcp: &Mcp, session_id: &str, ws: &str, bp: &str) {
    if !session_id.is_empty() {
        travado(&mcp.sem_registro).insert(session_id.into(), (ws.into(), bp.into()));
    }
}

/// Um registro (ou um plano marcado) do blueprint: as sessões dele deixam de estar pendentes.
pub fn registrado(mcp: &Mcp, ws: &str, bp: &str) {
    travado(&mcp.sem_registro).retain(|_, alvo| alvo.0 != ws || alvo.1 != bp);
}

/// `hook/parada` (Stop, http): devolve o lembrete uma vez e o tira da lista. Com `stop_hook_active` (o Claude já está continuando por causa de
/// um Stop anterior) não faz nada: assim não entra em loop e não repete no mesmo turno.
pub fn parada(mcp: &Mcp, corpo: &Value) -> Value {
    if corpo["stop_hook_active"] == true {
        return json!({});
    }
    let Some((ws, bp)) = travado(&mcp.sem_registro).remove(texto(corpo, "session_id")) else { return json!({}) };
    let aviso = format!(
        "The Blueprint: houve edições fora de um plano nesta sessão. Antes de encerrar, chame registrar_trabalho (workspace_id {ws}, blueprint_id {bp}) com o que já foi feito, \
         e pergunte ao usuário se ele quer adicionar isso ao blueprint."
    );
    json!({"hookSpecificOutput": {"hookEventName": "Stop", "additionalContext": aviso}})
}
