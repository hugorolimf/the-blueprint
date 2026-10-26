//! A chave de MCP de cada membro: `bpk_` + 43 caracteres, mostrada uma vez; o banco guarda o sha256 e um prefixo para a tela reconhecer.
//! O agente a manda em `Authorization: Bearer` e age com o papel do membro (só em /api/mcp/*).
use super::{segredo, Acesso};
use crate::contexto::Contexto;
use crate::dados::{membros, sessoes, Dados};
use crate::erro::Resultado;
use crate::tempo::iso_utc;

pub const PREFIXO: &str = "bpk_";

/// A chave de `Authorization: Bearer bpk_...`, se o cabeçalho tiver essa forma.
pub fn do_cabecalho(valor: &str) -> Option<&str> {
    let chave = valor.strip_prefix("Bearer ").or_else(|| valor.strip_prefix("bearer "))?.trim();
    chave.strip_prefix(PREFIXO).filter(|s| segredo::forma_valida(s)).map(|_| chave)
}

/// O contexto do membro dono da chave; None = chave que não existe ou de membro removido.
pub fn contexto(dados: &mut Dados, acesso: &Acesso, chave: &str, agora_ms: i64) -> Resultado<Option<Contexto>> {
    let hash = segredo::hash(chave);
    let Some(membro) = sessoes::membro_da_chave(dados.conexao(), &hash)? else { return Ok(None) };
    let Some(nome) = membros::nome_ativo(dados.conexao(), &membro)? else { return Ok(None) };
    let ctx = Contexto::membro(&membro, &nome, membros::acessos(dados.conexao(), &membro)?);
    if acesso.anotar_visto(&hash, agora_ms) {
        dados.transacao(|t| sessoes::chave_usada(t, &hash, &iso_utc(agora_ms)))?;
    }
    Ok(Some(ctx))
}

/// Gera a chave do membro (trocando a anterior) e devolve (chave, prefixo). A chave só existe nesta resposta.
pub fn gerar(conn: &rusqlite::Connection, membro_id: &str) -> Resultado<(String, String)> {
    let corpo = segredo::gerar();
    let chave = format!("{PREFIXO}{corpo}");
    let prefixo = format!("{PREFIXO}{}", &corpo[..4]);
    sessoes::gravar_chave(conn, &segredo::hash(&chave), membro_id, &prefixo)?;
    Ok((chave, prefixo))
}

/// Os comandos que a tela mostra para ligar o agente ao servidor (a chave vai por variável de ambiente, nunca nos argumentos do curl).
pub fn comandos(url_publica: &str, chave: &str) -> serde_json::Value {
    let env = format!("--env BLUEPRINT_URL={url_publica} --env BLUEPRINT_CHAVE={chave}");
    serde_json::json!({"claude": format!("claude mcp add blueprint-equipe {env} -- blueprint --mcp"), "codex": format!("codex mcp add blueprint-equipe {env} -- blueprint --mcp")})
}
