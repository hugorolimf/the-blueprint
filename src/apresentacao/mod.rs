//! Apresentações geradas pela IA (contrato em docs/interno/apresentacao-ia.md): o agente manda o conteúdo (JSON) e o código de cada figura; aqui se valida,
//! se guarda em arquivo (`<pasta de dados>/apresentacoes/<id>.json`, nunca no Git) e se monta a página sozinha, com a biblioteca embutida, para o visor isolado.
//! Os metadados (`apresentacoes` no `ler_blueprint`) saem do próprio arquivo: nada aqui altera o banco nem o data.json.
//! shortcut: sem índice, listar lê a pasta inteira; com algumas dezenas de apresentações por máquina isso é instantâneo, e um índice vale quando passarem de centenas.
pub mod armazenar;
mod ia;
pub mod montar;
pub mod validar;

pub use ia::arquivo;

#[cfg(test)]
mod tests;
