//! Configuração do servidor, lida do ambiente com os mesmos nomes do servidor Node (HOST, PORT, MCP_REDE, PROTOTIPOS_DIR).
use std::env;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    /// 0 = o sistema escolhe uma porta livre (testes e janela nativa)
    pub porta: u16,
    /// MCP_REDE=1: libera /api/mcp para a rede; por padrão só loopback
    pub rede_mcp: bool,
    /// o banco SQLite (BLUEPRINT_DB); é a fonte da verdade
    pub banco: PathBuf,
    /// data.json copiado para o banco só na primeira execução, com o banco vazio (DADOS_ARQ)
    pub importar_de: Option<PathBuf>,
    pub prototipos: PathBuf,
    /// BLUEPRINT_ASSETS_DIR: serve a interface desta pasta em vez da embutida (desenvolvimento, sem recompilar)
    pub assets_dir: Option<PathBuf>,
    /// o executável, que o agente registra com `--mcp`
    pub exe: PathBuf,
}

fn pasta_de(var: &str, padrao: &str) -> PathBuf {
    env::var(var).ok().filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(padrao))
}

impl Config {
    pub fn do_ambiente() -> Config {
        Config {
            host: env::var("HOST").ok().filter(|v| !v.is_empty()).unwrap_or_else(|| "127.0.0.1".into()),
            porta: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(3000),
            rede_mcp: env::var("MCP_REDE").as_deref() == Ok("1"),
            banco: pasta_de("BLUEPRINT_DB", "blueprint.db"),
            importar_de: Some(pasta_de("DADOS_ARQ", "data.json")),
            prototipos: pasta_de("PROTOTIPOS_DIR", "prototipos"),
            assets_dir: env::var("BLUEPRINT_ASSETS_DIR").ok().filter(|v| !v.is_empty()).map(PathBuf::from),
            exe: env::current_exe().unwrap_or_else(|_| PathBuf::from("blueprint")),
        }
    }

    /// O app instalado: dados em %APPDATA%\Blueprint\ (banco e protótipos), porta 3000 e, na primeira execução com o banco vazio, uma cópia do
    /// data.json que estiver ao lado do executável ou na pasta atual. As variáveis de ambiente ainda mandam mais que isto.
    pub fn do_app() -> Config {
        let mut c = Config::do_ambiente();
        let base = pasta_do_app();
        if env::var_os("BLUEPRINT_DB").is_none_or(|v| v.is_empty()) {
            c.banco = ["blueprint.db", "roadmap.db"].iter().map(|n| base.join(n)).find(|p| p.exists()).unwrap_or_else(|| base.join("blueprint.db"));
        }
        if env::var_os("PROTOTIPOS_DIR").is_none_or(|v| v.is_empty()) {
            c.prototipos = base.join("prototipos");
        }
        let ao_lado = c.exe.parent().map(|p| p.join("data.json"));
        let candidatos = [env::var("DADOS_ARQ").ok().filter(|v| !v.is_empty()).map(PathBuf::from), ao_lado, Some(PathBuf::from("data.json")), Some(base.join("data.json"))];
        c.importar_de = candidatos.into_iter().flatten().find(|p| p.is_file());
        c
    }

    /// O endereço para o agente e para a janela: 0.0.0.0 vira 127.0.0.1.
    pub fn url(&self, porta: u16) -> String {
        let host = if self.host == "0.0.0.0" { "127.0.0.1" } else { self.host.as_str() };
        format!("http://{host}:{porta}")
    }
}

/// %APPDATA%\Blueprint (ou, sem APPDATA, a pasta atual). O app se chamava Roadmap: se só existir a pasta antiga, ela é renomeada junto com o banco.
/// shortcut: se a pasta antiga estiver em uso (servidor antigo rodando), segue nela sem mover nada; o banco antigo é achado em `do_app`.
pub fn pasta_do_app() -> PathBuf {
    let Some(appdata) = env::var_os("APPDATA").filter(|v| !v.is_empty()) else { return PathBuf::from(".") };
    let (nova, antiga) = (Path::new(&appdata).join("Blueprint"), Path::new(&appdata).join("Roadmap"));
    if nova.exists() || !antiga.exists() {
        return nova;
    }
    if std::fs::rename(&antiga, &nova).is_err() {
        return antiga;
    }
    for sufixo in ["", "-wal", "-shm"] {
        let _ = std::fs::rename(nova.join(format!("roadmap.db{sufixo}")), nova.join(format!("blueprint.db{sufixo}")));
    }
    nova
}
