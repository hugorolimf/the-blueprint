//! `blueprint restaurar [<número>|<arquivo>]`: sem argumento lista as cópias do banco (automáticas e de antes de atualizar); com um, volta a ela.
//! Só com o app fechado: o app guarda uma cópia dos dados em memória e gravaria por cima do banco restaurado.
use crate::dados::backups::{listar, pasta_backups, restaurar};
use crate::http::Config;
use std::path::PathBuf;

pub fn executar(escolha: Option<&str>) -> i32 {
    let banco = Config::do_app().banco;
    let copias = listar(&banco);
    let Some(escolha) = escolha else {
        if copias.is_empty() {
            println!("{}", crate::tr!("No backups in {}.", "Nenhuma cópia em {}.", pasta_backups(&banco).display()));
        }
        for (i, c) in copias.iter().enumerate() {
            println!("{:>3}  {}", i + 1, c.display());
        }
        if !copias.is_empty() {
            println!("{}", crate::tr!("To go back to one (close the app first): blueprint restaurar <number>", "Para voltar a uma (feche o app antes): blueprint restaurar <número>"));
        }
        return 0;
    };
    if app_aberto() {
        eprintln!("{}", crate::tr!("error: close The Blueprint first (tray > Quit); restoring with the app open would be overwritten.", "erro: feche o The Blueprint antes (bandeja > Sair); restaurar com o app aberto seria sobrescrito."));
        return 1;
    }
    let copia = match escolha.parse::<usize>() {
        Ok(n) if (1..=copias.len()).contains(&n) => copias[n - 1].clone(),
        Ok(_) => {
            eprintln!("{}", crate::tr!("error: there is no backup number {escolha}; run blueprint restaurar to list them.", "erro: não há a cópia número {escolha}; rode blueprint restaurar para listar."));
            return 1;
        }
        Err(_) => PathBuf::from(escolha),
    };
    match restaurar(&banco, &copia, crate::tempo::agora_ms()) {
        Ok(guardado) => {
            println!("{}", crate::tr!("Database restored from {}.", "Banco restaurado de {}.", copia.display()));
            if let Some(g) = guardado {
                println!("{}", crate::tr!("The previous database was kept in {}.", "O banco de antes ficou em {}.", g.display()));
            }
            0
        }
        Err(e) => {
            eprintln!("erro: {e}");
            1
        }
    }
}

/// O app (mutex da instância única) ou um `blueprint serve` (responde em BLUEPRINT_URL, padrão a porta 3000).
fn app_aberto() -> bool {
    // a trava fica com este processo até ele sair: o app também não abre no meio da restauração
    #[cfg(windows)]
    if crate::janela::instancia::garantir() == crate::janela::instancia::Instancia::JaAberta {
        return true;
    }
    crate::mcp::cliente::requisitar_em(&crate::mcp::stdio::base(), "GET", "/api/atualizacao", None).is_ok()
}
