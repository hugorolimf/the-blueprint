//! O que `/api/mcp/planejar/*` devolve, já pronto para desenhar (contrato, seção 5.2). Os nomes seguem os campos do servidor, em snake_case aqui.
//! `default` em tudo: campo que falta não derruba a folha. Os testes conferem, com o JSON gravado pelo teste de ponta a ponta do servidor, que nenhum campo se perde.

use serde::Deserialize;
#[cfg(test)]
use serde::Serialize;
use serde_json::Value;

macro_rules! modelo {
    ($($(#[$m:meta])* pub struct $nome:ident { $($corpo:tt)* })*) => {$(
        $(#[$m])*
        #[derive(Clone, Debug, Default, Deserialize, PartialEq)]
        #[cfg_attr(test, derive(Serialize))]
        #[serde(rename_all = "camelCase", default)]
        pub struct $nome { $($corpo)* }
    )*};
}

/// A situação de um plano, a mesma do escalonador do servidor.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "lowercase")]
pub enum Situacao {
    #[default]
    Nao,
    Revisao,
    Trabalhando,
    Fila,
    Dependencia,
    Area,
    Parado,
    Bloqueado,
    Pronto,
    /// de um servidor mais novo: desenha sem oferecer clique
    #[serde(other)]
    Outra,
}

/// Como a ação está na escrita em curso.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "lowercase")]
pub enum EstadoEscrita {
    #[default]
    Pronta,
    Aguardando,
    Escrevendo,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "lowercase")]
pub enum TipoLinha {
    Prazo,
    Novo,
    Alterado,
    #[default]
    Igual,
    Removido,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "lowercase")]
pub enum TipoPalavra {
    #[default]
    Igual,
    Menos,
    Mais,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "lowercase")]
pub enum EstadoRevisao {
    Aprovada,
    #[default]
    Revisar,
    Reprovada,
    Respondida,
}

modelo! {
    /// `GET planejar/pulso`
    pub struct Pulso { pub versao_dados: u64, pub revisao: u64 }

    /// `GET planejar/folha`
    pub struct Folha {
        pub versao_dados: u64,
        pub revisao: u64,
        pub hoje: String,
        pub blueprint: BlueprintInfo,
        pub escrita: Option<Escrita>,
        pub pendente: Option<Pendente>,
        pub sem_pasta: bool,
        pub limite: u32,
        pub projetos: Vec<Projeto>,
        pub decisoes: Vec<Decisao>,
        pub perguntas: Vec<Pergunta>,
        pub numeros: NumerosFolha,
    }

    pub struct BlueprintInfo { pub id: String, pub nome: String, pub resumo: String, pub links: Vec<Value>, pub tem_nota: bool }
    pub struct Escrita { pub escrita_id: String, pub agente: String, pub origem: String, pub parada_ms: i64, pub parada: bool }
    pub struct Pendente { pub numero: i64, pub agente: String }
    pub struct NumerosFolha { pub aprovar: u32, pub diffs: u32, pub novas: u32 }
    pub struct Projeto { pub id: String, pub nome: String, pub acoes: Vec<Acao> }

    pub struct Acao {
        pub id: String,
        #[serde(rename = "ref")]
        pub referencia: String,
        pub nome: String,
        pub de: String,
        pub ate: String,
        pub sem_data: bool,
        pub texto: String,
        pub status: String,
        pub links: Vec<Value>,
        pub tem_nota: bool,
        pub escrita: EstadoEscrita,
        pub nova: bool,
        pub agentes: u32,
        pub planos: Vec<Plano>,
        pub checkpoints: Vec<Checkpoint>,
        pub tabelas: Vec<Tabela>,
        pub detalhes: Vec<Detalhe>,
        pub diff: Option<Diff>,
        pub recados: Vec<Recado>,
        pub numeros: NumerosAcao,
    }

    pub struct NumerosAcao { pub planos: u32, pub prontos: u32, pub detalhes: u32, pub decisoes: u32, pub aprovar: u32 }

    pub struct Plano {
        pub id: String,
        #[serde(rename = "ref")]
        pub referencia: String,
        pub texto: String,
        pub instrucoes: String,
        pub area: String,
        pub depende_de: Vec<String>,
        pub feito: bool,
        pub situacao: Situacao,
        /// `ajuda`, `sumiu`, `falhou` ou vazio
        pub causa: String,
        pub por: String,
        pub sem_iniciar: Vec<String>,
        pub execucao_id: String,
        pub ramo: String,
        pub agente: String,
        pub motivo: String,
    }

    pub struct Checkpoint { pub id: String, pub nome: String, pub data: String, pub feito: bool }
    pub struct Tabela { pub titulo: String, pub colunas: Vec<String>, pub larguras: Vec<f64>, pub linhas: Vec<Vec<String>> }
    pub struct Recado { pub de: String, pub texto: String, pub em: i64 }
    pub struct Revisao { pub estado: EstadoRevisao, pub observacao: String, pub em: i64 }
    pub struct Figura { pub hash: String, pub tipo: String, pub largura: Option<u32>, pub altura: Option<u32> }

    pub struct Detalhe {
        pub id: String,
        pub titulo: String,
        pub legenda: String,
        pub grupo: String,
        pub figura: Option<Figura>,
        pub pede_aprovacao: bool,
        pub hash: String,
        pub revisao: Option<Revisao>,
    }

    pub struct Decisao {
        pub id: String,
        pub rotulo: String,
        pub texto: String,
        pub data: String,
        pub acoes: Vec<String>,
        pub pede_aprovacao: bool,
        pub hash: String,
        pub revisao: Option<Revisao>,
    }

    pub struct Pergunta { pub id: String, pub texto: String, pub opcoes: Vec<String>, pub recomendada: String, pub acoes: Vec<String>, pub resposta: String }

    pub struct Diff {
        pub numero: i64,
        pub agente: String,
        pub hash: String,
        pub resumo: Resumo,
        pub removida: bool,
        pub prazo: Option<Prazo>,
        pub linhas: Vec<LinhaDiff>,
        pub detalhes_novos: Vec<Detalhe>,
        pub outros: Vec<String>,
        pub revisao: Option<Revisao>,
    }

    pub struct Resumo { pub mais: u32, pub menos: u32, pub alterados: u32, pub detalhes: u32, pub outros: u32 }
    pub struct Prazo { pub antes: String, pub depois: String }

    pub struct LinhaDiff {
        pub tipo: TipoLinha,
        #[serde(rename = "ref")]
        pub referencia: String,
        pub texto: String,
        pub antes: String,
        pub palavras: Vec<Palavra>,
        pub campos: Vec<String>,
        /// `reabre`: plano feito que volta a não feito ao aprovar
        pub situacao: String,
    }

    pub struct Palavra { pub t: TipoPalavra, pub s: String }

    /// `GET planejar/chegou`
    pub struct Chegou {
        pub numero: i64,
        pub agente: String,
        pub origem: String,
        pub novas: Vec<ChegouNova>,
        pub alteradas: Vec<ChegouAlterada>,
        pub removidas: Vec<ChegouRemovida>,
        pub iguais: u32,
    }

    pub struct ChegouNova {
        #[serde(rename = "ref")]
        pub referencia: String,
        pub nome: String,
        pub planos: u32,
        pub detalhes: u32,
        pub checkpoints: u32,
        pub pode_colocar: bool,
    }

    pub struct ChegouAlterada {
        #[serde(rename = "ref")]
        pub referencia: String,
        pub nome: String,
        pub resumo: Resumo,
    }

    pub struct ChegouRemovida {
        #[serde(rename = "ref")]
        pub referencia: String,
        pub nome: String,
    }

    /// `GET planejar/agentes`
    pub struct Agentes { pub limite: u32, pub orquestrador: Orquestrador, pub numeros: NumerosAgentes, pub avisos: Vec<String>, pub planos: Vec<PlanoAgente> }
    pub struct Orquestrador { pub ligado: bool, pub acordado: bool, pub agora: Vec<String> }
    pub struct NumerosAgentes { pub trabalhando: u32, pub fila: u32, pub esperando: u32, pub parados: u32, pub prontos: u32 }

    pub struct PlanoAgente {
        pub blueprint_id: String,
        pub blueprint_nome: String,
        pub acao_id: String,
        pub id: String,
        #[serde(rename = "ref")]
        pub referencia: String,
        pub texto: String,
        pub situacao: Situacao,
        pub causa: String,
        pub por: String,
        pub depende_de: Vec<String>,
        pub area: String,
        pub ramo: String,
        pub destino: String,
        pub execucao_id: String,
        pub motivo: String,
        pub sem_sinal: bool,
        pub subagentes: u32,
        pub pode_integrar: bool,
        pub pode_retomar: bool,
        pub pode_cancelar: bool,
    }

    /// `GET planejar/config`
    pub struct Config { pub comando_agente: Vec<String>, pub comando_orquestrador: Vec<String> }
}

#[cfg(test)]
#[path = "modelo_testes.rs"]
mod testes;
