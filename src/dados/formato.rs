//! Como o data.json vira linhas: o nome de cada lista no JSON e as colunas "de primeira classe" de cada tabela.
//! O resto de cada objeto vai inteiro na coluna `extra` (JSON), então um campo novo nunca se perde na ida e volta.
//! O vocabulário do JSON é o novo (workspaces, blueprints, acoes, planos, checkpoints); se mudar de novo, só este arquivo (e uma migração das `listas`) muda.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Texto,
    Bool,
    Numero,
    /// qualquer valor JSON, guardado como texto
    Json,
}

pub struct Coluna {
    pub chave: &'static str,
    pub coluna: &'static str,
    pub tipo: Tipo,
}

pub struct Tabela {
    pub nome: &'static str,
    /// coluna da chave estrangeira para a tabela de cima
    pub pai: Option<&'static str>,
    /// chave da lista, no JSON do pai, que contém estes itens
    pub lista: &'static str,
    pub colunas: &'static [Coluna],
    /// os itens não têm id no JSON (os comentários): a linha ganha um número do banco
    pub sem_id: bool,
}

const fn c(chave: &'static str, coluna: &'static str, tipo: Tipo) -> Coluna {
    Coluna { chave, coluna, tipo }
}
use Tipo::{Bool, Json, Numero, Texto};

/// O workspace raiz é o próprio objeto do arquivo (sem `id` no JSON); o id fixo dele no banco é este.
pub const RAIZ_ID: &str = "sm";

pub const WORKSPACES: Tabela = Tabela { nome: "workspaces", pai: None, lista: "workspaces", sem_id: false, colunas: &[c("nome", "nome", Texto), c("descricao", "descricao", Texto)] };
pub const BLUEPRINTS: Tabela = Tabela { nome: "blueprints", pai: Some("workspace_id"), lista: "blueprints", sem_id: false, colunas: &[c("nome", "nome", Texto), c("descricao", "descricao", Texto), c("obs", "obs", Texto)] };
pub const ACOES: Tabela = Tabela {
    nome: "acoes", pai: Some("blueprint_id"), lista: "acoes", sem_id: false,
    colunas: &[c("nome", "nome", Texto), c("inicio", "inicio", Texto), c("fim", "fim", Texto), c("obs", "obs", Texto), c("status", "status", Texto), c("progresso", "progresso", Numero)],
};
pub const PLANOS: Tabela = Tabela {
    nome: "planos", pai: Some("acao_id"), lista: "planos", sem_id: false,
    colunas: &[c("texto", "texto", Texto), c("descricao", "descricao", Texto), c("feito", "feito", Bool), c("agente", "agente", Json)],
};
pub const CHECKPOINTS: Tabela = Tabela {
    nome: "checkpoints", pai: Some("acao_id"), lista: "checkpoints", sem_id: false,
    colunas: &[c("nome", "nome", Texto), c("data", "data", Texto), c("evidencia", "evidencia", Texto), c("feitoEm", "feito_em", Texto), c("feito", "feito", Bool)],
};
pub const COMENTARIOS: Tabela = Tabela {
    nome: "comentarios", pai: Some("plano_id"), lista: "comentarios", sem_id: true,
    colunas: &[c("texto", "texto", Texto), c("em", "em", Texto), c("tipo", "tipo", Texto), c("sessaoId", "sessao_id", Texto)],
};
pub const PROTOTIPOS: Tabela = Tabela { nome: "prototipos", pai: Some("acao_id"), lista: "prototipos", sem_id: false, colunas: &[c("titulo", "titulo", Texto), c("em", "em", Texto)] };

/// Marca, em `listas` do plano, que `agente.comentarios` existia como lista no JSON (os itens ficam na tabela `comentarios`).
pub const LISTA_COMENTARIOS: &str = "agente.comentarios";

/// Listas filhas de cada tabela (as que viram linhas em outra tabela).
pub fn filhas(t: &Tabela) -> &'static [&'static Tabela] {
    match t.nome {
        "workspaces" => &[&BLUEPRINTS],
        "blueprints" => &[&ACOES],
        "acoes" => &[&PLANOS, &CHECKPOINTS, &PROTOTIPOS],
        _ => &[],
    }
}
