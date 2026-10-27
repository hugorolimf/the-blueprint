//! Validação da proposta que o agente envia pelo MCP: porte do antigo `plano.mjs` (mesmas regras e mesmas mensagens de erro).
//! Nada aqui altera dados: devolve erros, avisos e a proposta normalizada para a tela de revisão.
mod datas;
mod decisoes;
mod enxuta;
mod impressao;
mod itens;
mod mover;
mod mover_acao;
mod prazos;
mod projetos;
mod registro;
mod remocoes;
mod resumos;
mod texto;
mod validar;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_mover_acao;

#[cfg(test)]
mod tests_decisoes;

#[cfg(test)]
mod tests_enxuta;

#[cfg(test)]
mod tests_projetos;

#[cfg(test)]
mod tests_impressao;

#[cfg(test)]
mod tests_numeros;

#[cfg(test)]
mod tests_prazos;

#[cfg(test)]
mod tests_registro;

pub use enxuta::estrutura_lida;
pub use datas::{dia_valido, dias_entre, ms_do_iso};
pub use mover_acao::mover_acoes;
pub use texto::verdade;
pub use projetos::numeros_das_acoes;
pub use registro::validar_registro;
pub use resumos::{contexto_acao, contexto_blueprint, estrutura_blueprint, resumo_blueprint};
pub use validar::{validar_proposta, Validacao};
