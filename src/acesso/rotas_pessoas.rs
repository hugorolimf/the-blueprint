//! Pessoas, convites e atividade de um workspace (docs/colaboracao.md, seções 3 e 5). Dono administra; supervisor lista pessoas e lê a
//! atividade; colaborador e visitante não veem nada daqui. Nunca remover nem rebaixar o último dono (`ultimo-dono`).
use super::convites::{self, NovoConvite};
use super::rotas::{corpo, proibido, texto, travar, R};
use crate::contexto::{Acao, Contexto, Papel};
use crate::dados::atividade::{self, Registro};
use crate::dados::convites as dconvites;
use crate::dados::membros;
use crate::http::pedido::{falha, ErroHttp, Pedido, Resposta};
use crate::http::Estado;
use crate::tempo::iso_utc;
use serde_json::{json, Value};

/// Dono do workspace inteiro (ou de todos), e não visitante.
fn exige_dono(ctx: &Contexto, ws: &str) -> Result<(), ErroHttp> {
    if ws.is_empty() || ws == "*" || ctx.visitante || !ctx.pode(Acao::Administrar, ws, None) { Err(proibido()) } else { Ok(()) }
}

/// Dono ou supervisor do workspace inteiro (quem acompanha).
fn exige_acompanha(ctx: &Contexto, ws: &str) -> Result<(), ErroHttp> {
    match ctx.papel_em(ws, None) {
        Some(Papel::Dono | Papel::Supervisor) if !ctx.visitante && !ws.is_empty() && ws != "*" => Ok(()),
        _ => Err(proibido()),
    }
}

fn nao_encontrado() -> ErroHttp {
    falha(404, crate::tr!("not found", "não encontrado"))
}

fn papel_invalido() -> ErroHttp {
    falha(400, crate::tr!("invalid role", "papel inválido")).codigo("papel-invalido")
}

pub fn rota(estado: &Estado, pedido: &Pedido, ctx: &Contexto, url_publica: &str) -> R {
    let ws_consulta = pedido.parametro("workspace").unwrap_or_default();
    match (pedido.metodo.as_str(), pedido.caminho.as_str()) {
        ("GET", "/api/pessoas") => {
            exige_acompanha(ctx, &ws_consulta)?;
            let mut lista = membros::do_workspace(travar(estado).conexao(), &ws_consulta)?;
            lista.iter_mut().for_each(|m| m["voce"] = json!(m["id"] == ctx.usuario.as_str()));
            Ok(Resposta::json(200, &json!({"membros": lista})))
        }
        ("GET", "/api/convites") => {
            exige_dono(ctx, &ws_consulta)?;
            Ok(Resposta::json(200, &json!({"convites": dconvites::do_workspace(travar(estado).conexao(), &ws_consulta)?})))
        }
        ("GET", "/api/atividade") => {
            exige_acompanha(ctx, &ws_consulta)?;
            let limite = pedido.parametro("limite").and_then(|l| l.parse::<i64>().ok()).unwrap_or(100).clamp(1, 500);
            Ok(Resposta::json(200, &json!({"itens": atividade::listar(travar(estado).conexao(), &ws_consulta, limite)?})))
        }
        ("POST", "/api/pessoas/papel") => mudar_papel(estado, &corpo(pedido)?, ctx),
        ("POST", "/api/pessoas/remover") => remover(estado, &corpo(pedido)?, ctx),
        ("POST", "/api/convites") => criar(estado, &corpo(pedido)?, ctx, url_publica),
        ("POST", "/api/convites/revogar") => revogar(estado, &corpo(pedido)?, ctx, None),
        ("POST", "/api/convites/trocar") => revogar(estado, &corpo(pedido)?, ctx, Some(url_publica)),
        ("GET" | "POST", _) => Err(nao_encontrado()),
        _ => Err(falha(405, crate::tr!("method not allowed", "método não permitido"))),
    }
}

/// O membro alvo está no workspace por um acesso só dele (não pelo de todos, que só quem tem o terminal do servidor muda)?
fn alvo_no_workspace(conn: &rusqlite::Connection, membro: &str, ws: &str) -> Result<(), ErroHttp> {
    let acessos = membros::acessos(conn, membro)?;
    if membros::nome_ativo(conn, membro)?.is_none() || !acessos.iter().any(|a| a.workspace_id == ws || a.workspace_id == "*") {
        return Err(nao_encontrado());
    }
    if acessos.iter().any(|a| a.workspace_id == "*") {
        return Err(proibido());
    }
    Ok(())
}

fn ultimo_dono() -> ErroHttp {
    falha(409, crate::tr!("the workspace needs at least one owner", "o workspace precisa de pelo menos um dono")).codigo("ultimo-dono")
}

fn era_dono(conn: &rusqlite::Connection, membro: &str, ws: &str) -> Result<bool, ErroHttp> {
    Ok(membros::acessos(conn, membro)?.iter().any(|a| a.workspace_id == ws && a.blueprint_id.is_empty() && a.papel == Papel::Dono))
}

fn mudar_papel(estado: &Estado, c: &Value, ctx: &Contexto) -> R {
    let (ws, membro) = (texto(c, "workspaceId"), texto(c, "membroId"));
    exige_dono(ctx, ws)?;
    let papel = Papel::de_texto(texto(c, "papel")).filter(|p| *p != Papel::Visitante).ok_or_else(papel_invalido)?;
    let agora = crate::tempo::agora_ms();
    travar(estado).transacao(|t| {
        alvo_no_workspace(t, membro, ws)?;
        if papel != Papel::Dono && era_dono(t, membro, ws)? && membros::donos_do_workspace(t, ws, membro)? == 0 {
            return Err(ultimo_dono().into());
        }
        membros::definir_papel(t, membro, ws, papel)?;
        atividade::gravar(t, agora, &Registro { membro_id: Some(&ctx.usuario), nome: &ctx.nome, tipo: "papel-mudado", workspace_id: ws, blueprint_id: "", alvo: membro })?;
        Ok(())
    })?;
    Ok(Resposta::json(200, &json!({"ok": true})))
}

fn remover(estado: &Estado, c: &Value, ctx: &Contexto) -> R {
    let (ws, membro) = (texto(c, "workspaceId"), texto(c, "membroId"));
    exige_dono(ctx, ws)?;
    let agora = crate::tempo::agora_ms();
    let removido = travar(estado).transacao(|t| {
        alvo_no_workspace(t, membro, ws)?;
        if era_dono(t, membro, ws)? && membros::donos_do_workspace(t, ws, membro)? == 0 {
            return Err(ultimo_dono().into());
        }
        let de_vez = membros::remover_do_workspace(t, membro, ws, &iso_utc(agora))?;
        atividade::gravar(t, agora, &Registro { membro_id: Some(&ctx.usuario), nome: &ctx.nome, tipo: "membro-removido", workspace_id: ws, blueprint_id: "", alvo: membro })?;
        Ok(de_vez)
    })?;
    // os fluxos abertos dele fecham; se ainda tem outros workspaces, reabre com o acesso novo
    estado.acesso.encerrar(membro);
    let _ = removido;
    Ok(Resposta::json(200, &json!({"ok": true})))
}

fn criar(estado: &Estado, c: &Value, ctx: &Contexto, url_publica: &str) -> R {
    let ws = texto(c, "workspaceId");
    exige_dono(ctx, ws)?;
    let papel = Papel::de_texto(texto(c, "papel")).ok_or_else(papel_invalido)?;
    let bp = texto(c, "blueprintId");
    let abre = c.get("abre").and_then(Value::as_str).unwrap_or("folha");
    if !matches!(abre, "folha" | "apresentacao") {
        return Err(falha(400, crate::tr!("abre must be folha or apresentacao", "abre precisa ser folha ou apresentacao")).codigo("invalido"));
    }
    let validade = match c.get("validadeHoras") {
        None | Some(Value::Null) => 168,
        Some(v) => v.as_i64().filter(|h| (1..=720).contains(h)).ok_or_else(|| falha(400, crate::tr!("validadeHoras goes from 1 to 720", "validadeHoras vai de 1 a 720")).codigo("validade-invalida"))?,
    };
    let usos_max = match c.get("usosMax") {
        None => (papel != Papel::Visitante).then_some(1),
        Some(Value::Null) => None,
        Some(v) => Some(v.as_i64().filter(|n| (1..=1000).contains(n)).ok_or_else(|| falha(400, crate::tr!("usosMax goes from 1 to 1000 (or null)", "usosMax vai de 1 a 1000 (ou null)")).codigo("usos-invalidos"))?),
    };
    let mut dados = travar(estado);
    let arquivo = dados.ler_tudo(&Contexto::local())?;
    let existe = match bp {
        "" => super::filtro::blueprints_do(arquivo, ws).is_some(),
        _ => super::filtro::nome_do_blueprint(arquivo, ws, bp).is_some(),
    };
    if !existe {
        return Err(falha(404, crate::tr!("workspace or blueprint not found", "workspace ou blueprint não encontrado")));
    }
    if papel == Papel::Visitante && bp.is_empty() {
        return Err(falha(400, crate::tr!("a public link needs blueprintId", "o link público precisa de blueprintId")).codigo("invalido"));
    }
    let n = NovoConvite { papel, workspace_id: ws, blueprint_id: bp, abre, validade_horas: validade, usos_max };
    let agora = crate::tempo::agora_ms();
    let (convite, segredo) = dados.transacao(|t| convites::criar(t, Some((&ctx.usuario, &ctx.nome)), &n, agora))?;
    let criador = Some((ctx.usuario.clone(), ctx.nome.clone()));
    Ok(Resposta::json(201, &json!({"convite": convite.json(criador), "link": convites::link(url_publica, &segredo)})))
}

/// Revoga o convite; com `trocar` (a URL pública), cria outro com os mesmos parâmetros e devolve o link novo.
fn revogar(estado: &Estado, c: &Value, ctx: &Contexto, trocar: Option<&str>) -> R {
    let id = texto(c, "id");
    let mut dados = travar(estado);
    let convite = dconvites::por_id(dados.conexao(), id)?.ok_or_else(nao_encontrado)?;
    exige_dono(ctx, &convite.workspace_id)?;
    let agora = crate::tempo::agora_ms();
    let tipo = if trocar.is_some() { "convite-trocado" } else { "convite-revogado" };
    let novo = dados.transacao(|t| {
        dconvites::revogar(t, id, &iso_utc(agora))?;
        atividade::gravar(t, agora, &Registro { membro_id: Some(&ctx.usuario), nome: &ctx.nome, tipo, workspace_id: &convite.workspace_id, blueprint_id: &convite.blueprint_id, alvo: id })?;
        let Some(_) = trocar else { return Ok(None) };
        let papel = Papel::de_texto(&convite.papel).unwrap_or(Papel::Visitante);
        let horas = 168; // shortcut: a troca dá a validade padrão em vez da original; guardar a duração pedida se a tela precisar
        let n = NovoConvite { papel, workspace_id: &convite.workspace_id, blueprint_id: &convite.blueprint_id, abre: &convite.abre, validade_horas: horas, usos_max: convite.usos_max };
        convites::criar(t, Some((&ctx.usuario, &ctx.nome)), &n, agora).map(Some)
    })?;
    drop(dados);
    estado.acesso.encerrar(&format!("visitante:{id}"));
    Ok(match (novo, trocar) {
        (Some((convite, segredo)), Some(url)) => Resposta::json(201, &json!({"convite": convite.json(Some((ctx.usuario.clone(), ctx.nome.clone()))), "link": convites::link(url, &segredo)})),
        _ => Resposta::json(200, &json!({"ok": true})),
    })
}
