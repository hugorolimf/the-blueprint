//! Sessões do navegador: o cookie `bp_sessao` (HttpOnly, SameSite=Lax, Secure) carrega um segredo de 43 caracteres; o banco guarda só o
//! sha256. Resolve o `Contexto` do membro (ou do visitante, preso ao convite) a cada pedido.
use super::{segredo, Acesso};
use crate::contexto::{Contexto, Papel};
use crate::dados::convites::{self, Convite};
use crate::dados::sessoes::{self, Sessao};
use crate::dados::{membros, Dados};
use crate::erro::Resultado;
use crate::tempo::iso_utc;
use rusqlite::Connection;

pub const COOKIE: &str = "bp_sessao";
pub const MEMBRO_MS: i64 = 30 * 86_400_000;
pub const VISITANTE_MS: i64 = 7 * 86_400_000;
/// `visto_em` e `usada_em` são gravados no máximo uma vez a cada 10 min por sessão (sem gravação por pedido)
pub const VISTO_A_CADA_MS: i64 = 10 * 60_000;

/// O contexto de quem tem uma sessão (hash do cookie) ainda válida; None = sessão vencida, apagada, de membro removido ou de convite revogado.
pub fn contexto(dados: &mut Dados, acesso: &Acesso, hash: &str, agora_ms: i64) -> Resultado<Option<Contexto>> {
    let agora = iso_utc(agora_ms);
    let Some(s) = sessoes::buscar(dados.conexao(), hash, &agora)? else { return Ok(None) };
    let ctx = match (&s.membro_id, &s.convite_id) {
        (Some(m), _) => match membros::nome_ativo(dados.conexao(), m)? {
            Some(nome) => Contexto::membro(m, &nome, membros::acessos(dados.conexao(), m)?),
            None => return Ok(None),
        },
        (None, Some(c)) => match convites::por_id(dados.conexao(), c)? {
            // o visitante já foi contado ao entrar: aqui vale só não estar revogado nem vencido
            Some(cv) if cv.revogado_em.is_none() && cv.expira_em > agora => Contexto::visitante(&cv.id, &cv.workspace_id, &cv.blueprint_id, &cv.abre),
            _ => return Ok(None),
        },
        (None, None) => return Ok(None),
    };
    if acesso.anotar_visto(hash, agora_ms) {
        dados.transacao(|t| sessoes::ver(t, hash, &agora))?;
    }
    Ok(Some(ctx))
}

/// Cria a sessão dentro da transação de entrada. Devolve o valor do cookie (o segredo, que só vai ao navegador) e a validade em segundos.
pub fn criar(conn: &Connection, membro_id: Option<&str>, convite: &Convite, agora_ms: i64) -> Resultado<(String, i64)> {
    let valor = segredo::gerar();
    let vida = if membro_id.is_some() { MEMBRO_MS } else { VISITANTE_MS };
    let mut fim = iso_utc(agora_ms + vida);
    if membro_id.is_none() && convite.expira_em < fim {
        fim = convite.expira_em.clone(); // visitante: 7 dias ou o fim do convite, o que vier antes (o mesmo formato ISO compara como texto)
    }
    let s = Sessao { membro_id: membro_id.map(String::from), convite_id: Some(convite.id.clone()), expira_em: fim };
    sessoes::criar(conn, &segredo::hash(&valor), &s, &iso_utc(agora_ms))?;
    Ok((valor, vida / 1000))
}

pub fn cookie(valor: &str, max_age_s: i64) -> String {
    format!("{COOKIE}={valor}; HttpOnly; SameSite=Lax; Path=/; Max-Age={max_age_s}; Secure")
}

pub fn cookie_vencido() -> String {
    format!("{COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0; Secure")
}

/// Papel de membro gravável a partir de um convite (o de visitante não vira acesso).
pub fn papel_de_membro(c: &Convite) -> Option<Papel> {
    Papel::de_texto(&c.papel).filter(|p| *p != Papel::Visitante)
}
