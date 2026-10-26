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
    /// `blueprint serve --hospedar`: o servidor de equipe atrás de um proxy HTTPS (None = o app local de sempre)
    pub hospedagem: Option<Hospedagem>,
}

/// O modo hospedado (docs/colaboracao.md, seção 4). TLS fica com o proxy reverso; o app confia nos cabeçalhos dele só vindos do próprio proxy.
#[derive(Clone, Debug, PartialEq)]
pub struct Hospedagem {
    /// `https://equipe.exemplo.com`, sem barra no fim: também é a única Origin aceita
    pub url_publica: String,
    /// o host da URL pública (com a porta, se houver): o único `Host` aceito
    pub host_publico: String,
    /// BLUEPRINT_PROXY: o IP do proxy quando ele não está nesta máquina
    pub proxy: Option<String>,
    /// BLUEPRINT_HSTS=1
    pub hsts: bool,
    /// `--novo-link-dono`: revoga o link do dono pendente e imprime outro
    pub novo_link_dono: bool,
}

impl Hospedagem {
    /// Confere a configuração; o texto de erro diz o que falta (nunca contém segredo: não há segredo aqui).
    pub fn de(url: &str, host: &str, proxy: Option<&str>, hsts: bool) -> Result<Hospedagem, String> {
        let resto = url.strip_prefix("https://").ok_or_else(|| crate::tr!("BLUEPRINT_URL_PUBLICA must start with https:// (TLS is done by the reverse proxy)", "BLUEPRINT_URL_PUBLICA precisa começar com https:// (o TLS fica com o proxy reverso)"))?;
        if resto.is_empty() || resto.contains(['/', '?', '#', '@', ' ']) {
            return Err(crate::tr!("BLUEPRINT_URL_PUBLICA must be only the origin, like https://team.example.com (no path and no trailing slash)", "BLUEPRINT_URL_PUBLICA precisa ser só a origem, como https://equipe.exemplo.com (sem caminho e sem barra no fim)"));
        }
        let proxy = proxy.map(str::trim).filter(|p| !p.is_empty());
        if let Some(p) = proxy {
            p.parse::<std::net::IpAddr>().map_err(|_| crate::tr!("BLUEPRINT_PROXY must be an IP address", "BLUEPRINT_PROXY precisa ser um endereço IP"))?;
        }
        let loopback = matches!(host, "127.0.0.1" | "::1" | "localhost");
        if !loopback && proxy.is_none() {
            return Err(crate::tr!("HOST={host} listens outside this machine: set BLUEPRINT_PROXY to the proxy IP (or use HOST=127.0.0.1 with the proxy on this machine)", "HOST={host} escuta fora desta máquina: defina BLUEPRINT_PROXY com o IP do proxy (ou use HOST=127.0.0.1 com o proxy nesta máquina)"));
        }
        Ok(Hospedagem { url_publica: url.to_string(), host_publico: resto.to_ascii_lowercase(), proxy: proxy.map(String::from), hsts, novo_link_dono: false })
    }
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
            hospedagem: None,
        }
    }

    /// `blueprint serve --hospedar`: o ambiente de `do_ambiente` mais BLUEPRINT_URL_PUBLICA (obrigatória), BLUEPRINT_PROXY e BLUEPRINT_HSTS.
    /// O MCP sem token por loopback e MCP_REDE não valem aqui: a sessão ou a chave substituem o loopback.
    pub fn hospedado(novo_link_dono: bool) -> Result<Config, String> {
        let mut c = Config::do_ambiente();
        let url = env::var("BLUEPRINT_URL_PUBLICA").unwrap_or_default();
        if url.is_empty() {
            return Err(crate::tr!("set BLUEPRINT_URL_PUBLICA (for example https://team.example.com) to host The Blueprint", "defina BLUEPRINT_URL_PUBLICA (por exemplo https://equipe.exemplo.com) para hospedar o The Blueprint"));
        }
        let mut h = Hospedagem::de(&url, &c.host, env::var("BLUEPRINT_PROXY").ok().as_deref(), env::var("BLUEPRINT_HSTS").as_deref() == Ok("1"))?;
        h.novo_link_dono = novo_link_dono;
        // só importa um data.json pedido com DADOS_ARQ (nunca um que estiver por acaso na pasta atual)
        let importar = env::var("DADOS_ARQ").ok().filter(|v| !v.is_empty()).map(PathBuf::from);
        (c.hospedagem, c.rede_mcp, c.importar_de) = (Some(h), false, importar);
        Ok(c)
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
