//! O que todas as rotas compartilham: configuração, camada de dados, MCP e arquivos estáticos.
use super::config::Config;
use super::estaticos::Assets;
use crate::atualizador::Atualizador;
use crate::contexto::Contexto;
use crate::dados::Dados;
use crate::erro::Resultado;
use crate::mcp::Mcp;
use serde_json::{json, Value};
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;

/// Versão inicial dos dados: a hora de abertura, para uma versão guardada por uma página não valer depois de o app reiniciar.
pub fn versao_inicial() -> AtomicU64 {
    AtomicU64::new(crate::tempo::agora_ms().max(0) as u64)
}

/// A versão inicial e a sincronia que começa nela (o buffer de lotes precisa saber de onde a contagem parte).
pub fn versao_e_sincronia(presenca: bool) -> (AtomicU64, crate::sincronia::Sincronia) {
    let versao = versao_inicial();
    let sincronia = crate::sincronia::Sincronia::nova(versao.load(std::sync::atomic::Ordering::SeqCst), presenca);
    (versao, sincronia)
}

pub struct Estado {
    /// o que traz a janela à frente: só existe no app com janela (o servidor puro não tem)
    pub ao_mostrar: Mutex<Option<Box<dyn Fn() + Send>>>,
    /// o que encerra o app depois de o instalador da atualização ser lançado (só a janela o define)
    pub ao_sair: Mutex<Option<Box<dyn Fn() + Send>>>,
    /// acha, baixa e confere a versão nova; só instala quando o usuário clica (src/atualizador)
    pub atualizador: Atualizador,
    pub config: Config,
    /// o acesso do modo hospedado: sessões encerradas e tentativas inválidas, só memória (src/acesso)
    pub acesso: crate::acesso::Acesso,
    /// o compartilhamento pelo app: porta dos convidados e túnel, só memória e só quando ligado (src/compartilhar)
    pub compartilhar: crate::compartilhar::Compartilhar,
    pub dados: Mutex<Dados>,
    /// o data.json já serializado, para GET /api/dados não refazer o texto a cada pedido; limpo a cada gravação
    pub dados_json: Mutex<Option<Vec<u8>>>,
    /// a versão do data.json: sobe a cada gravação e muda a cada abertura do app; a página a manda em `If-Match` e uma gravação feita sobre
    /// uma versão velha é recusada (409), em vez de apagar o que outra janela gravou
    pub versao_dados: AtomicU64,
    /// buffer dos últimos lotes, assinantes de /api/eventos e presença (src/sincronia, só memória)
    pub sincronia: crate::sincronia::Sincronia,
    pub mcp: Mcp,
    /// as sessões do terminal do app (pseudoconsoles com o claude), só na memória: ver src/terminal
    pub terminais: crate::terminal::Terminais,
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
