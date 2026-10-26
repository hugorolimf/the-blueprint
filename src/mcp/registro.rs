//! `registrar_trabalho`: o agente relata o que já fez fora do plano (planos e checkpoints já marcados, com evidência, e o que ficou pendente). Vira um item
//! "registro" da fila, com número próprio ("Registro nº N"): NÃO é proposta. Nada é salvo aqui; o usuário decide, na tela, se adiciona ao blueprint.
use super::acoes::{localizar, texto_js};
use super::Mcp;
use crate::blueprint::validar_registro;
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Map, Value};

/// Valida e guarda. O blueprint tem de existir (um registro nunca cria blueprint).
pub fn registro(mcp: &Mcp, dados: &Value, chamada: &Value) -> Result<(u16, Value), ErroHttp> {
    let (workspace_id, blueprint_id) = (texto_js(chamada.get("workspaceId")), texto_js(chamada.get("blueprintId")));
    let (_, blueprint) = localizar(dados, &workspace_id, &blueprint_id)?;
    let Some(blueprint) = blueprint else { return Err(falha(400, crate::tr!("registrar_trabalho needs blueprint_id (a record is about a blueprint that already exists; see listar_blueprints).", "registrar_trabalho precisa de blueprint_id (um registro é sobre um blueprint que já existe; veja listar_blueprints).")).codigo("falta-blueprint")) };
    let r = validar_registro(chamada, blueprint, &crate::tempo::hoje_local());
    if !r.erros.is_empty() {
        return Ok((422, json!({"erros": r.erros, "avisos": r.avisos})));
    }
    let mut item = Map::new();
    for (k, v) in [("tipo", json!("registro")), ("workspaceId", json!(workspace_id)), ("blueprintId", json!(blueprint_id)), ("registro", r.proposta.unwrap_or(Value::Null)), ("avisos", json!(r.avisos))] {
        item.insert(k.into(), v);
    }
    let guardado = mcp.empurra_registro(item);
    Ok((200, json!({"ok": true, "avisos": r.avisos, "numero": guardado["numero"]})))
}
