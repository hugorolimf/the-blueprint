//! Convites: criar (o segredo só existe na resposta), conferir, entrar e o link do dono impresso pelo servidor. O segredo viaja no fragmento
//! da URL (`/convite#<segredo>`) e no corpo dos POST; nunca em caminho, `Location`, log ou mensagem de erro.
use super::{segredo, sessoes};
use crate::contexto::{Contexto, Papel};
use crate::dados::atividade::{self, Registro};
use crate::dados::convites::{self, Convite};
use crate::dados::{membros, Dados};
use crate::erro::Resultado;
use crate::http::pedido::{falha, ErroHttp};
use crate::tempo::iso_utc;
use rusqlite::Connection;

pub const NOME_MAX: usize = 60;
const HORA_MS: i64 = 3_600_000;

pub struct NovoConvite<'a> {
    pub papel: Papel,
    pub workspace_id: &'a str,
    pub blueprint_id: &'a str,
    pub abre: &'a str,
    pub validade_horas: i64,
    pub usos_max: Option<i64>,
}

/// O que `criar` devolve: o convite gravado e o segredo (que vai para o link e some).
pub fn criar(conn: &Connection, autor: Option<(&str, &str)>, n: &NovoConvite, agora_ms: i64) -> Resultado<(Convite, String)> {
    let valor = segredo::gerar();
    let c = Convite {
        id: segredo::id(),
        papel: n.papel.texto().into(),
        workspace_id: n.workspace_id.into(),
        blueprint_id: n.blueprint_id.into(),
        abre: n.abre.into(),
        criado_por: autor.map(|(id, _)| id.to_string()),
        criado_em: iso_utc(agora_ms),
        expira_em: iso_utc(agora_ms + n.validade_horas * HORA_MS),
        usos_max: n.usos_max,
        usos: 0,
        revogado_em: None,
    };
    convites::criar(conn, &c, &segredo::hash(&valor))?;
    let tipo = if autor.is_some() { "convite-criado" } else { "link-dono-impresso" };
    let (membro_id, nome) = autor.map_or((None, ""), |(id, nome)| (Some(id), nome));
    atividade::gravar(conn, agora_ms, &Registro { membro_id, nome, tipo, workspace_id: n.workspace_id, blueprint_id: n.blueprint_id, alvo: &c.id })?;
    Ok((c, valor))
}

pub fn link(url_publica: &str, segredo: &str) -> String {
    format!("{url_publica}/convite#{segredo}")
}

/// O convite válido desse segredo. Inexistente, vencido, revogado, esgotado ou malformado: None (a rota responde igual em todos os casos).
pub fn conferir(dados: &Dados, segredo_dado: &str, agora_ms: i64) -> Resultado<Option<Convite>> {
    if !segredo::forma_valida(segredo_dado) {
        return Ok(None);
    }
    Ok(convites::por_hash(dados.conexao(), &segredo::hash(segredo_dado))?.filter(|c| c.valido(&iso_utc(agora_ms))))
}

/// O nome é só um rótulo: 1 a 60 caracteres, sem caractere de controle.
pub fn nome_valido(nome: &str) -> Result<String, ErroHttp> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(falha(400, crate::tr!("tell us your name", "informe o seu nome")).codigo("falta-nome"));
    }
    if nome.chars().count() > NOME_MAX || nome.chars().any(char::is_control) {
        return Err(falha(400, crate::tr!("the name must have up to {NOME_MAX} characters, without control characters", "o nome precisa ter até {NOME_MAX} caracteres, sem caracteres de controle")).codigo("nome-invalido").params(serde_json::json!({"max": NOME_MAX})));
    }
    Ok(nome.to_string())
}

/// O resultado de entrar: o convite usado, o valor do cookie, a validade dele (segundos) e para onde a página vai.
pub struct Entrada {
    pub convite: Convite,
    pub cookie: String,
    pub max_age: i64,
    pub destino: String,
}

/// Entra pelo convite numa transação: confere de novo, cria o membro (ou reaproveita o da sessão atual e só soma o acesso), cria a sessão,
/// soma 1 em `usos` e grava `entrou`. `Ok(None)` = convite inválido.
pub fn entrar(dados: &mut Dados, atual: &Contexto, segredo_dado: &str, nome: Option<&str>, agora_ms: i64) -> Result<Option<Entrada>, ErroHttp> {
    let Some(convite) = conferir(dados, segredo_dado, agora_ms)? else { return Ok(None) };
    let papel = sessoes::papel_de_membro(&convite);
    let reaproveita = papel.is_some() && atual.autenticado() && !atual.visitante && !atual.local;
    let nome = match (papel, reaproveita) {
        (None, _) => String::new(),
        (Some(_), true) => atual.nome.clone(),
        (Some(_), false) => nome_valido(nome.unwrap_or(""))?,
    };
    let entrada = dados.transacao(|t| {
        // o mesmo segredo usado em paralelo por duas pessoas: a transação confere outra vez antes de contar o uso
        if !convites::por_id(t, &convite.id)?.is_some_and(|c| c.valido(&iso_utc(agora_ms))) {
            return Ok(None);
        }
        let membro_id = match (papel, reaproveita) {
            (None, _) => None,
            (Some(_), true) => Some(atual.usuario.clone()),
            (Some(_), false) => {
                let id = segredo::id();
                membros::criar(t, &id, &nome)?;
                Some(id)
            }
        };
        if let (Some(m), Some(p)) = (&membro_id, papel) {
            membros::somar_acesso(t, m, &convite.workspace_id, &convite.blueprint_id, p, Some(&convite.id))?;
        }
        convites::usar(t, &convite.id)?;
        let (cookie, max_age) = sessoes::criar(t, membro_id.as_deref(), &convite, agora_ms)?;
        let r = Registro { membro_id: membro_id.as_deref(), nome: &nome, tipo: "entrou", workspace_id: &convite.workspace_id, blueprint_id: &convite.blueprint_id, alvo: &convite.id };
        atividade::gravar(t, agora_ms, &r)?;
        let destino = match (papel, convite.workspace_id.as_str()) {
            (None, _) => format!("/#/p/{}", convite.blueprint_id),
            (Some(_), "*") => "/".to_string(),
            (Some(_), ws) => format!("/#/c/{ws}"),
        };
        Ok(Some(Entrada { convite: convite.clone(), cookie, max_age, destino }))
    })?;
    Ok(entrada)
}

/// O que o servidor faz com o link do dono ao subir (`blueprint serve --hospedar`).
#[derive(Debug, PartialEq)]
pub enum LinkDono {
    /// imprimir uma vez: `<URL_PUBLICA>/convite#<segredo>`
    Novo(String),
    /// já há um link pendente (impresso antes): só avisar como pedir outro
    JaImpresso,
    /// já existe dono de todos: nada a imprimir
    TemDono,
}

/// Sem dono `'*'` nem convite de dono pendente, cria o convite do dono (24 h, uso único). `novo` (`--novo-link-dono`) revoga o pendente e cria outro.
pub fn link_dono(dados: &mut Dados, url_publica: &str, novo: bool, agora_ms: i64) -> Resultado<LinkDono> {
    let agora = iso_utc(agora_ms);
    dados.transacao(|t| {
        let pendente = convites::pendente_do_dono(t, &agora)?;
        if novo {
            if let Some(p) = &pendente {
                convites::revogar(t, &p.id, &agora)?;
            }
        } else if membros::existe_dono_geral(t)? {
            return Ok(LinkDono::TemDono);
        } else if pendente.is_some() {
            return Ok(LinkDono::JaImpresso);
        }
        let n = NovoConvite { papel: Papel::Dono, workspace_id: "*", blueprint_id: "", abre: "folha", validade_horas: 24, usos_max: Some(1) };
        let (_, valor) = criar(t, None, &n, agora_ms)?;
        Ok(LinkDono::Novo(link(url_publica, &valor)))
    })
}
