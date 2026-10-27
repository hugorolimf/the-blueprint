//! As apresentações geradas pela IA no MCP: `POST /api/mcp/apresentacao` (a ferramenta `anexar_apresentacao`), `apresentacao/remover` e
//! `GET /api/mcp/apresentacoes` (contrato em docs/interno/apresentacao-ia.md). Tudo chega como JSON (nenhum arquivo do disco é lido) e fica em
//! `<pasta de dados>/apresentacoes/<id>.json`, fora do banco e do Git; nada aqui altera o blueprint. O visor abre a página montada em `/apresentacao/<id>`.
use super::acoes::{novo_id_prototipo, texto_js};
use super::Mcp;
use crate::apresentacao::{armazenar, editar as edicao, validar::validar};
use crate::contexto::{Acao, Contexto};
use crate::http::pedido::{falha, ErroHttp};
use serde_json::{json, Value};

type R<T> = Result<T, ErroHttp>;

fn avisar(mcp: &Mcp, ws: &str, bp: &str, id: &str) {
    mcp.notificar("apresentacao", Some(json!({"workspaceId": ws, "blueprintId": bp, "apresentacaoId": id})));
}

/// Guarda (ou, com `apresentacaoId`, substitui) uma apresentação do blueprint. `dados` é o arquivo como o contexto o lê (um blueprint que ele não lê dá 404).
pub fn anexar(mcp: &Mcp, dados: &Value, corpo: &Value) -> R<Value> {
    let (ws, bp) = (texto_js(corpo.get("workspaceId")), texto_js(corpo.get("blueprintId")));
    if bp.is_empty() {
        return Err(falha(400, crate::tr!("Send blueprint_id.", "Informe o blueprint_id.")).codigo("falta-blueprint"));
    }
    super::acoes::localizar(dados, &ws, &bp)?;
    let validada = validar(&corpo["conteudo"], &corpo["figuras"])?;
    let pasta = mcp.pasta_apresentacoes();
    let atualiza = texto_js(corpo.get("apresentacaoId"));
    let id = if atualiza.is_empty() {
        novo_id_prototipo()
    } else {
        // só substitui uma que este blueprint já tem
        let existe = armazenar::ler(&pasta, &atualiza).is_some_and(|r| armazenar::alvo(&r) == (ws.as_str(), bp.as_str()));
        if !existe {
            return Err(falha(404, crate::tr!("apresentacao_id does not exist in this blueprint (see ler_blueprint)", "apresentacao_id não existe neste blueprint (veja ler_blueprint)")).codigo("apresentacao-nao-encontrada"));
        }
        atualiza.clone()
    };
    let figuras = if corpo["figuras"].is_null() { json!([]) } else { corpo["figuras"].clone() };
    let registro = json!({"id": id, "workspaceId": ws, "blueprintId": bp, "titulo": validada.titulo, "publico": validada.publico, "nivel": validada.nivel, "em": crate::tempo::iso_utc_agora(), "conteudo": corpo["conteudo"], "figuras": figuras});
    armazenar::gravar(&pasta, &id, &registro).map_err(|e| falha(500, crate::tr!("could not save the presentation: {e}", "não consegui gravar a apresentação: {e}")))?;
    avisar(mcp, &ws, &bp, &id);
    Ok(json!({"ok": true, "apresentacaoId": id, "titulo": validada.titulo, "caminho": format!("/apresentacao/{id}"), "atualizado": !atualiza.is_empty(), "avisos": validada.avisos}))
}

/// Chamado pela página ao excluir. Só mexe em arquivo de id válido; no modo hospedado exige gravar no blueprint dono dele. Idempotente.
pub fn remover(mcp: &Mcp, corpo: &Value, ctx: &Contexto) -> R<Value> {
    let id = corpo.get("apresentacaoId").and_then(Value::as_str).filter(|i| armazenar::id_valido(i)).ok_or_else(|| falha(400, crate::tr!("invalid apresentacaoId", "apresentacaoId inválido")).codigo("apresentacao-invalida"))?;
    let pasta = mcp.pasta_apresentacoes();
    if let Some(registro) = armazenar::ler(&pasta, id) {
        let (ws, bp) = armazenar::alvo(&registro);
        if !ctx.local && !ctx.pode(Acao::Gravar, ws, Some(bp)) {
            return Err(falha(403, crate::tr!("you do not have permission for this", "você não tem permissão para isso")));
        }
        armazenar::remover(&pasta, id);
        avisar(mcp, ws, bp, id);
    }
    Ok(json!({"ok": true}))
}

/// `GET /api/mcp/apresentacao?id=`: a apresentação guardada, com o `codigo` das figuras (quem edita precisa dele). Quem não lê o blueprint dela recebe o mesmo 404 de quem pede um id que não existe.
pub fn ler(mcp: &Mcp, id: &str, ctx: &Contexto) -> R<Value> {
    let nao_achada = || falha(404, crate::tr!("presentation not found in this blueprint (see ler_blueprint)", "apresentação não encontrada neste blueprint (veja ler_blueprint)")).codigo("apresentacao-nao-encontrada");
    let r = armazenar::ler(&mcp.pasta_apresentacoes(), id).ok_or_else(nao_achada)?;
    let (ws, bp) = armazenar::alvo(&r);
    if !ctx.local && !ctx.pode(Acao::Ler, ws, Some(bp)) {
        return Err(nao_achada());
    }
    let mut v = json!({"id": r["id"], "titulo": r["titulo"], "publico": r["publico"], "nivel": r["nivel"], "em": r["em"], "conteudo": r["conteudo"], "figuras": r["figuras"]});
    for k in ["editadoEm", "editadoPor"] {
        if let Some(x) = r.get(k) {
            v[k] = x.clone();
        }
    }
    Ok(v)
}

/// `POST /api/mcp/apresentacao/editar`: operações pequenas (ou o conteúdo e as figuras inteiros) sobre uma apresentação que já existe; tudo ou nada, validado como o `anexar`.
/// Quem grava o blueprint dono dela edita; o dono está no arquivo, não nos dados (como em `remover`).
pub fn editar(mcp: &Mcp, corpo: &Value, ctx: &Contexto) -> R<Value> {
    let id = corpo.get("apresentacaoId").and_then(Value::as_str).filter(|i| armazenar::id_valido(i)).ok_or_else(|| falha(400, crate::tr!("invalid apresentacaoId", "apresentacaoId inválido")).codigo("apresentacao-invalida"))?;
    let pasta = mcp.pasta_apresentacoes();
    let registro = armazenar::ler(&pasta, id).ok_or_else(|| falha(404, crate::tr!("presentation not found in this blueprint (see ler_blueprint)", "apresentação não encontrada neste blueprint (veja ler_blueprint)")).codigo("apresentacao-nao-encontrada"))?;
    let (ws, bp) = armazenar::alvo(&registro);
    if !ctx.local && !ctx.pode(Acao::Gravar, ws, Some(bp)) {
        return Err(falha(403, crate::tr!("you do not have permission for this", "você não tem permissão para isso")));
    }
    let agora = crate::tempo::iso_utc_agora();
    let (novo, avisos) = edicao::editar_registro(&registro, corpo, &agora)?;
    armazenar::gravar(&pasta, id, &novo).map_err(|e| falha(500, crate::tr!("could not save the presentation: {e}", "não consegui gravar a apresentação: {e}")))?;
    avisar(mcp, ws, bp, id);
    Ok(json!({"ok": true, "apresentacaoId": id, "editadoEm": agora, "avisos": avisos}))
}

/// `GET /api/mcp/apresentacoes?workspace=&blueprint=`: os metadados, da mais nova para a mais antiga. Quem não lê o blueprint recebe a lista vazia.
pub fn listar(mcp: &Mcp, workspace: &str, blueprint: &str, ctx: &Contexto) -> Value {
    if !ctx.local && !ctx.pode(Acao::Ler, workspace, Some(blueprint)) {
        return json!({"apresentacoes": []});
    }
    json!({"apresentacoes": armazenar::listar(&mcp.pasta_apresentacoes(), workspace, blueprint)})
}
