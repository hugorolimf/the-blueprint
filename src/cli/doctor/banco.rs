//! Doctor: o banco abre e está íntegro? A pasta atual é de algum blueprint? Abre o SQLite só para leitura (sem migrar, sem criar).
use super::{Estado, Item};
use rusqlite::{Connection, OpenFlags};
use std::path::{Path, PathBuf};

/// Só leitura. Sem o `-wal` ao lado (ninguém com o banco aberto), abre como imutável: senão o SQLite criaria o `-wal` e o `-shm` de um banco em WAL.
fn abrir(arquivo: &Path) -> rusqlite::Result<Connection> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX | OpenFlags::SQLITE_OPEN_URI;
    if PathBuf::from(format!("{}-wal", arquivo.display())).exists() {
        return Connection::open_with_flags(arquivo, flags);
    }
    let caminho = arquivo.display().to_string().replace('\\', "/").replace('%', "%25").replace(' ', "%20").replace('?', "%3f").replace('#', "%23");
    let raiz = match (arquivo.is_absolute(), caminho.starts_with('/')) {
        (false, _) => "file:",
        (true, true) => "file://",
        (true, false) => "file:///", // C:/...
    };
    Connection::open_with_flags(format!("{raiz}{caminho}?immutable=1"), flags)
}

/// O banco do app em `pasta_app` (o mesmo que `Config::do_app` escolhe) e, se abriu, o caminho para a conferência da pasta.
pub fn banco(pasta_app: &Path) -> (Item, Option<PathBuf>) {
    let nome = crate::tr!("Database", "Banco");
    let Some(arquivo) = ["blueprint.db", "roadmap.db"].iter().map(|n| pasta_app.join(n)).find(|p| p.exists()) else {
        let texto = crate::tr!(
            "no database in {} yet: open the app once and it creates one.",
            "ainda não há banco em {}: abra o app uma vez e ele cria.",
            pasta_app.display()
        );
        return (Item::novo(nome, Estado::Falha, texto), None);
    };
    let conferido = abrir(&arquivo).and_then(|c| c.query_row("PRAGMA quick_check", [], |l| l.get::<_, String>(0)));
    match conferido {
        Ok(r) if r == "ok" => (Item::novo(nome, Estado::Ok, crate::tr!("{} opens and passes the integrity check", "{} abre e passa na conferência de integridade", arquivo.display())), Some(arquivo)),
        Ok(r) => (Item::novo(nome, Estado::Falha, crate::tr!(
            "{} is damaged ({r}). Close the app and go back to a backup: blueprint restaurar lists them.",
            "{} está danificado ({r}). Feche o app e volte a uma cópia: blueprint restaurar lista as cópias.",
            arquivo.display()
        )), None),
        Err(e) => (Item::novo(nome, Estado::Falha, crate::tr!(
            "{} does not open ({e}). Close the app and go back to a backup: blueprint restaurar lists them.",
            "{} não abre ({e}). Feche o app e volte a uma cópia: blueprint restaurar lista as cópias.",
            arquivo.display()
        )), None),
    }
}

/// A linha `Blueprint: workspace_id …` do CLAUDE.md ou AGENTS.md da pasta (o caminho do Codex e dos agentes sem plugin).
fn linha_blueprint(cwd: &Path) -> Option<String> {
    ["CLAUDE.md", "AGENTS.md"].iter().find_map(|n| {
        let texto = std::fs::read_to_string(cwd.join(n)).ok()?;
        texto.lines().map(str::trim).find(|l| l.contains("Blueprint: workspace_id")).map(|l| format!("{n}: {l}"))
    })
}

/// O vínculo da pasta atual: primeiro a tabela `pastas` do app (a pasta ou a mãe mais próxima), depois a linha do CLAUDE.md/AGENTS.md.
pub fn pasta(banco: Option<&Path>, cwd: &Path) -> Item {
    let nome = crate::tr!("Folder link", "Pasta ligada");
    let no_app = banco.and_then(|b| abrir(b).ok()).and_then(|c| crate::dados::pastas::procurar(&c, &cwd.display().to_string()).ok().flatten());
    if let Some((ws, bp)) = no_app {
        return Item::novo(nome, Estado::Ok, crate::tr!("{} is linked to workspace_id {ws}, blueprint_id {bp}", "{} está ligada ao workspace_id {ws}, blueprint_id {bp}", cwd.display()));
    }
    if let Some(linha) = linha_blueprint(cwd) {
        return Item::novo(nome, Estado::Ok, crate::tr!("{} has the line in {linha}", "{} tem a linha em {linha}", cwd.display()));
    }
    Item::novo(nome, Estado::Aviso, crate::tr!(
        "{} is not linked to a blueprint. If it belongs to one, ask the agent to link it (vincular_pasta) or pick it in Edit > Blueprint data > Project folder.",
        "{} não está ligada a um blueprint. Se for de um, peça ao agente para ligar (vincular_pasta) ou escolha em Editar > Dados do blueprint > Pasta do projeto.",
        cwd.display()
    ))
}

#[cfg(test)]
#[path = "banco_tests.rs"]
mod tests;
