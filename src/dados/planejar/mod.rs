//! As tabelas do Planejar (migração 009): fora da árvore e sem FK, porque `PUT /api/dados` apaga e recria a árvore. Um arquivo por tabela; cada um acrescenta métodos a `Dados`.
//! Só `conclusao.rs` escreve na árvore (D8). Quem muda alguma destas tabelas avisa `Mcp::planejar_mudou` (D5); aqui não há `Mcp`.
mod conclusao;
mod config;
mod execucoes;
mod figuras;
mod orquestrador;
mod recados;
mod revisoes;
mod versoes;

#[cfg(test)]
mod tests;

pub use execucoes::{ExecucaoLinha, Lancamento, Mudanca, NovaExecucao, Partida, SITUACOES_VIVAS};
pub use figuras::FiguraLinha;
pub use orquestrador::OrqLinha;
pub use recados::{NovoRecado, RecadoLinha};
pub use revisoes::RevisaoLinha;
pub use versoes::VersaoLinha;

use super::Dados;
use crate::erro::Resultado;

impl Dados {
    /// O caminho como ele é no disco, da pasta mais rasa ligada ao blueprint; `None` se ela não é mais pasta (ver `pastas::pasta_real_do_blueprint`).
    pub fn pasta_real_do_blueprint(&self, workspace_id: &str, blueprint_id: &str) -> Resultado<Option<String>> {
        super::pastas::pasta_real_do_blueprint(&self.conn, workspace_id, blueprint_id)
    }
}

/// `bytes` bytes do gerador do sistema, em hexadecimal.
fn hex_aleatorio(bytes: usize) -> String {
    let mut b = vec![0u8; bytes];
    getrandom::fill(&mut b).expect("o gerador aleatório do sistema falhou");
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// As listas de refs guardadas como JSON numa coluna de texto.
fn lista_de(texto: &str) -> Vec<String> {
    serde_json::from_str(texto).unwrap_or_default()
}
