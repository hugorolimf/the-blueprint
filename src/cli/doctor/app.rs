//! Doctor: o app responde onde o agente procura? Uma só instância? A mesma versão deste `blueprint`?
use super::{Estado, Item};
use crate::atualizador::VERSAO;

/// A versão que o app em `base` informa (`GET /api/atualizacao`, só leitura), ou `None` se nada respondeu.
pub fn versao_do_app(base: &str) -> Option<String> {
    let (200, corpo) = crate::mcp::cliente::requisitar_em(base, "GET", "/api/atualizacao", None).ok()? else { return None };
    serde_json::from_str::<serde_json::Value>(&corpo).ok()?["atual"].as_str().map(String::from)
}

fn porta_de(base: &str) -> Option<u16> {
    base.trim_end_matches('/').rsplit(':').next()?.parse().ok()
}

/// `respondeu`: o app (ou um `blueprint serve`) atendeu em `base`; `porta`: o que o app anotou em %APPDATA%\Blueprint\porta;
/// `instancia`: o app com janela está aberto (mutex da instância única).
pub fn app(base: &str, respondeu: bool, porta: Option<u16>, instancia: bool) -> Item {
    let nome = crate::tr!("App and port", "App e porta");
    // o arquivo só vale com o app aberto (ele não é apagado ao sair)
    let outra = porta.filter(|p| instancia && Some(*p) != porta_de(base));
    let (estado, texto) = match (respondeu, instancia, outra) {
        (true, true, Some(p)) => (Estado::Falha, crate::tr!(
            "two processes: the app window listens on port {p}, but another process answers at {base}, so the window does not see what the agent sends. Quit the app (tray > Quit), stop the other blueprint process (Task Manager) and open the app again.",
            "dois processos: a janela do app escuta na porta {p}, mas outro processo responde em {base}, e a janela não vê o que o agente manda. Feche o app (bandeja > Sair), encerre o outro processo blueprint (Gerenciador de Tarefas) e abra o app de novo."
        )),
        (true, true, None) => (Estado::Ok, crate::tr!("the app is open and answers at {base} (single instance)", "o app está aberto e responde em {base} (instância única)")),
        (true, false, _) => (Estado::Ok, crate::tr!("a blueprint server answers at {base} (blueprint serve, not the app window)", "um servidor blueprint responde em {base} (blueprint serve, não a janela do app)")),
        (false, true, Some(p)) => (Estado::Falha, crate::tr!(
            "the app is open on port {p}, but agents look at {base}. Quit the app (tray > Quit) and open it again to take that port, or set BLUEPRINT_URL=http://127.0.0.1:{p}.",
            "o app está aberto na porta {p}, mas os agentes procuram em {base}. Feche o app (bandeja > Sair) e abra de novo para ele pegar essa porta, ou defina BLUEPRINT_URL=http://127.0.0.1:{p}."
        )),
        (false, true, None) => (Estado::Falha, crate::tr!(
            "the app is open but does not answer at {base}. Quit it (tray > Quit) and open it again.",
            "o app está aberto, mas não responde em {base}. Feche (bandeja > Sair) e abra de novo."
        )),
        (false, false, _) => (Estado::Falha, crate::tr!(
            "the app is closed (nothing answers at {base}). Open The Blueprint from the Start menu; what agents send meanwhile is kept in %APPDATA%\\Blueprint\\pendentes.",
            "o app está fechado (nada responde em {base}). Abra o The Blueprint pelo menu Iniciar; o que os agentes mandarem enquanto isso fica guardado em %APPDATA%\\Blueprint\\pendentes."
        )),
    };
    Item::novo(nome, estado, texto)
}

/// A versão deste `blueprint` contra a do app que respondeu (`None`: app fechado, nada a comparar).
pub fn versao(do_app: Option<&str>) -> Item {
    let nome = crate::tr!("Version", "Versão");
    let (estado, texto) = match do_app {
        None => (Estado::Ok, crate::tr!("blueprint {VERSAO} (the app's version is compared when it is open)", "blueprint {VERSAO} (a do app é comparada quando ele está aberto)")),
        Some(v) if v == VERSAO => (Estado::Ok, crate::tr!("blueprint {VERSAO}, the same as the app", "blueprint {VERSAO}, a mesma do app")),
        Some(v) => (Estado::Aviso, crate::tr!(
            "this blueprint is {VERSAO} and the app is {v}: another copy of blueprint.exe is running this command. Use the installed one (%LOCALAPPDATA%\\Programs\\Blueprint\\blueprint.exe) or update the app (blueprint atualizar).",
            "este blueprint é {VERSAO} e o app é {v}: outra cópia do blueprint.exe roda este comando. Use a instalada (%LOCALAPPDATA%\\Programs\\Blueprint\\blueprint.exe) ou atualize o app (blueprint atualizar)."
        )),
    };
    Item::novo(nome, estado, texto)
}

#[cfg(test)]
#[path = "app_tests.rs"]
mod tests;
