//! O que todas as rotas compartilham: configuração, camada de dados, MCP e arquivos estáticos.
use super::config::Config;
use super::estaticos::Assets;
use crate::atualizador::Atualizador;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::erro::Resultado;
use crate::mcp::Mcp;
use serde_json::{json, Value};
use std::sync::Mutex;

pub struct Estado {
    /// o que traz a janela à frente: só existe no app com janela (o servidor puro não tem)
    pub ao_mostrar: Mutex<Option<Box<dyn Fn() + Send>>>,
    /// o que encerra o app depois de o instalador da atualização ser lançado (só a janela o define)
    pub ao_sair: Mutex<Option<Box<dyn Fn() + Send>>>,
    /// acha, baixa e confere a versão nova; só instala quando o usuário clica (src/atualizador)
    pub atualizador: Atualizador,
    pub config: Config,
    pub dados: Mutex<Dados>,
    /// o data.json já serializado, para GET /api/dados não refazer o texto a cada pedido; limpo a cada gravação
    pub dados_json: Mutex<Option<Vec<u8>>>,
    pub mcp: Mcp,
    pub assets: Assets,
}

/// Abre o banco; se estiver vazio, copia o data.json (sem alterá-lo) ou começa com um arquivo vazio.
pub fn abrir_dados(config: &Config) -> Resultado<Dados> {
    if let Some(pasta) = config.banco.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(pasta)?;
    }
    let mut dados = Dados::abrir(&config.banco)?;
    let ctx = Contexto::local();
    if dados.contagens(&ctx)?.workspaces == 0 {
        let inicial: Value = match config.importar_de.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(texto) => serde_json::from_str(&texto)?,
            None => json!({"blueprints": []}),
        };
        dados.gravar_tudo(&ctx, &inicial)?;
    }
    Ok(dados)
}
