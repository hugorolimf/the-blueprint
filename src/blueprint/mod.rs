//! Validação da proposta que o agente envia pelo MCP: porte do antigo `plano.mjs` (mesmas regras e mesmas mensagens de erro).
//! Nada aqui altera dados: devolve erros, avisos e a proposta normalizada para a tela de revisão.
mod datas;
mod itens;
mod mover;
mod prazos;
mod remocoes;
mod resumos;
mod texto;
mod validar;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests_prazos;

pub use datas::{dia_valido, dias_entre, ms_do_iso};
pub use texto::verdade;
pub use resumos::{contexto_acao, contexto_blueprint, estrutura_blueprint, resumo_blueprint};
pub use validar::{validar_proposta, Validacao};
