//! Testes da camada de dados, um arquivo por assunto. `exemplo()` cobre os casos difíceis da ida e volta:
//! lista ausente x vazia, null, número inteiro x fracionário, campo desconhecido e campo mapeado com tipo inesperado.
mod ida_e_volta;
mod itens;
mod migracoes;
mod pastas;
mod projetos;
mod queda;

use serde_json::Value;

pub fn exemplo() -> Value {
    serde_json::from_str(include_str!("exemplo.json")).expect("exemplo.json é um JSON válido")
}
