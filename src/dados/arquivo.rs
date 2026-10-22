//! Importar e exportar o data.json de/para o banco. O arquivo original só é lido (copiado para dentro do banco), nunca movido nem alterado.
use super::{Contagens, Dados};
use crate::contexto::Contexto;
use crate::erro::Resultado;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn importar_arquivo(json: &Path, banco: &Path) -> Resultado<Contagens> {
    let valor: Value = serde_json::from_str(&fs::read_to_string(json)?)?;
    Dados::abrir(banco)?.gravar_tudo(&Contexto::local(), &valor)
}

/// Escreve o JSON num arquivo temporário e troca pelo destino, para nunca deixar um arquivo meio escrito.
pub fn exportar_arquivo(banco: &Path, saida: &Path) -> Resultado<()> {
    let mut dados = Dados::abrir(banco)?;
    let texto = serde_json::to_string_pretty(dados.ler_tudo(&Contexto::local())?)?;
    let temporario = saida.with_extension("json.tmp");
    fs::write(&temporario, texto)?;
    fs::rename(&temporario, saida)?;
    Ok(())
}
