//! `blueprint doctor`: confere o app (porta e instância única), a versão, o banco, o plugin do Claude Code, o MCP do Codex e o vínculo
//! da pasta atual, e diz o que fazer em cada item. Só lê: nunca grava nem conserta (nem pega a trava da instância única, nem renomeia a
//! pasta antiga como `pasta_do_app` faz). A saída é feita para colar numa issue: a pasta do usuário aparece como %USERPROFILE%.
mod agentes;
mod app;
mod banco;

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Estado {
    Ok,
    Aviso,
    Falha,
}

#[derive(Debug)]
pub struct Item {
    pub nome: String,
    pub estado: Estado,
    /// O que achou ou, se não está certo, o que fazer.
    pub texto: String,
}

impl Item {
    fn novo(nome: String, estado: Estado, texto: String) -> Item {
        Item { nome, estado, texto }
    }
}

/// Onde olhar. Vem das variáveis de ambiente em `do_processo`; os testes montam com pastas temporárias.
pub struct Ambiente {
    /// BLUEPRINT_URL ou http://127.0.0.1:3000 (onde o agente procura o app)
    pub base: String,
    /// %APPDATA%\Blueprint
    pub pasta_app: PathBuf,
    /// CLAUDE_CONFIG_DIR ou ~/.claude
    pub claude: PathBuf,
    /// onde o `claude mcp add --scope user` grava: ~/.claude.json (ou CLAUDE_CONFIG_DIR/.claude.json)
    pub claude_json: PathBuf,
    /// CODEX_HOME ou ~/.codex
    pub codex: PathBuf,
    pub cwd: PathBuf,
    /// o mutex da instância única existe (o app com janela está aberto)
    pub instancia: bool,
}

fn var(nome: &str) -> Option<PathBuf> {
    std::env::var_os(nome).filter(|v| !v.is_empty()).map(PathBuf::from)
}

impl Ambiente {
    pub fn do_processo() -> Ambiente {
        let home = var("USERPROFILE").or_else(|| var("HOME")).unwrap_or_default();
        let config = var("CLAUDE_CONFIG_DIR");
        Ambiente {
            base: crate::mcp::stdio::base(),
            pasta_app: var("APPDATA").map_or_else(|| PathBuf::from("."), |a| a.join("Blueprint")),
            claude_json: config.as_ref().map_or_else(|| home.join(".claude.json"), |c| c.join(".claude.json")),
            claude: config.unwrap_or_else(|| home.join(".claude")),
            codex: var("CODEX_HOME").unwrap_or_else(|| home.join(".codex")),
            cwd: std::env::current_dir().unwrap_or_default(),
            #[cfg(windows)]
            instancia: crate::janela::instancia::aberta(),
            #[cfg(not(windows))]
            instancia: false,
        }
    }
}

/// Todas as conferências, na ordem em que aparecem.
pub fn conferir(amb: &Ambiente) -> Vec<Item> {
    let versao = app::versao_do_app(&amb.base);
    let porta = std::fs::read_to_string(amb.pasta_app.join("porta")).ok().and_then(|t| t.trim().parse().ok());
    let (item_banco, arquivo) = banco::banco(&amb.pasta_app);
    vec![
        app::app(&amb.base, versao.is_some(), porta, amb.instancia),
        app::versao(versao.as_deref()),
        item_banco,
        agentes::claude(&amb.claude, &amb.claude_json),
        agentes::codex(&amb.codex),
        banco::pasta(arquivo.as_deref(), &amb.cwd),
    ]
}

/// O relatório: uma linha por item, com o rótulo do estado alinhado.
pub fn texto(itens: &[Item]) -> String {
    let mut linhas = vec![crate::tr!("The Blueprint doctor (blueprint {})", "Diagnóstico do The Blueprint (blueprint {})", crate::atualizador::VERSAO)];
    for i in itens {
        let rotulo = match i.estado {
            Estado::Ok => "OK".to_string(),
            Estado::Aviso => crate::tr!("WARN", "AVISO"),
            Estado::Falha => crate::tr!("FAIL", "FALHA"),
        };
        linhas.push(format!("  {rotulo:<6}{}: {}", i.nome, i.texto));
    }
    let falhas = itens.iter().filter(|i| i.estado == Estado::Falha).count();
    linhas.push(if falhas == 0 { crate::tr!("Nothing to fix.", "Nada a corrigir.") } else { crate::tr!("{falhas} item(s) to fix; the line says how.", "{falhas} item(ns) a corrigir; a linha diz como.") });
    linhas.join("\n")
}

/// Sai com 1 se algum item falhou (para scripts), 0 se só há OK e avisos.
pub fn executar() -> i32 {
    let itens = conferir(&Ambiente::do_processo());
    let mut saida = texto(&itens);
    if let Some(home) = var("USERPROFILE").map(|h| h.display().to_string()).filter(|h| h.len() > 3) {
        saida = saida.replace(&home, "%USERPROFILE%");
    }
    println!("{saida}");
    i32::from(itens.iter().any(|i| i.estado == Estado::Falha))
}

#[cfg(test)]
mod tests;
