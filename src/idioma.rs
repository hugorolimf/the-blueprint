//! O idioma do texto que a PESSOA lê e que sai do backend: erros das rotas, avisos dos hooks, status line e o que o monitor imprime.
//! O texto para o AGENTE (instruções do MCP, ferramentas, guia, respostas das ferramentas) é sempre em inglês: o `--mcp` fixa `En`.
//! Como chega: num pedido HTTP, `Accept-Language` (o WebView2 manda o idioma do Windows; o `--mcp` manda `en`; o `blueprint hook` manda o do
//! usuário); sem o cabeçalho, o idioma do usuário: `BLUEPRINT_IDIOMA` (ex.: `pt-BR`, `en`) > idioma do Windows (ou `LANG` fora dele) > inglês.
//! Só há catálogo para inglês e português; qualquer outro idioma cai no inglês, como a interface (web/js/i18n.js).
//! O idioma vale para a thread inteira enquanto o pedido é atendido (`com`), para as mensagens não precisarem recebê-lo por parâmetro.
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Idioma {
    En,
    Pt,
}

impl Idioma {
    /// `pt`, `pt-BR`, `pt_PT.UTF-8` → Pt; `en`, `en-US` → En; o resto → None.
    pub fn de_texto(t: &str) -> Option<Idioma> {
        let t = t.trim().to_ascii_lowercase();
        if t.starts_with("pt") { Some(Idioma::Pt) } else if t.starts_with("en") { Some(Idioma::En) } else { None }
    }

    /// A primeira etiqueta de `Accept-Language` que tem catálogo (a ordem já é a de preferência); nenhuma → inglês.
    pub fn de_accept_language(v: &str) -> Idioma {
        v.split(',').find_map(|parte| Idioma::de_texto(parte.split(';').next().unwrap_or(""))).unwrap_or(Idioma::En)
    }

    /// O valor para `Accept-Language` e para `BLUEPRINT_IDIOMA`.
    pub fn codigo(self) -> &'static str {
        match self {
            Idioma::En => "en",
            Idioma::Pt => "pt-BR",
        }
    }
}

/// A escolha do usuário, pura (testável): a variável manda; senão o sistema; senão inglês.
pub fn escolher(variavel: Option<&str>, sistema: Option<Idioma>) -> Idioma {
    variavel.and_then(Idioma::de_texto).or(sistema).unwrap_or(Idioma::En)
}

#[cfg(windows)]
fn do_sistema() -> Option<Idioma> {
    // LANGID: os 10 bits baixos são o idioma primário (0x09 inglês, 0x16 português)
    match unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() } & 0x3ff {
        0x16 => Some(Idioma::Pt),
        0x09 => Some(Idioma::En),
        _ => None,
    }
}

#[cfg(not(windows))]
fn do_sistema() -> Option<Idioma> {
    ["LC_ALL", "LC_MESSAGES", "LANG"].iter().find_map(|v| std::env::var(v).ok().filter(|s| !s.is_empty())).and_then(|s| Idioma::de_texto(&s))
}

/// O idioma do usuário (sem pedido HTTP). Nos testes é sempre português, para não depender da máquina; os testes de inglês pedem `En` com `com`.
pub fn do_usuario() -> Idioma {
    if cfg!(test) {
        return Idioma::Pt;
    }
    static USUARIO: std::sync::OnceLock<Idioma> = std::sync::OnceLock::new();
    *USUARIO.get_or_init(|| escolher(std::env::var("BLUEPRINT_IDIOMA").ok().as_deref(), do_sistema()))
}

/// O idioma de um pedido: `Accept-Language`, senão o do usuário.
pub fn do_pedido(p: &crate::http::pedido::Pedido) -> Idioma {
    p.cabecalho("accept-language").filter(|v| !v.trim().is_empty()).map_or_else(do_usuario, Idioma::de_accept_language)
}

thread_local! {
    static ATUAL: Cell<Option<Idioma>> = const { Cell::new(None) };
}

/// O idioma em uso nesta thread.
pub fn atual() -> Idioma {
    ATUAL.with(Cell::get).unwrap_or_else(do_usuario)
}

pub fn ingles() -> bool {
    atual() == Idioma::En
}

/// Um de dois textos fixos, sem alocar (para quem guarda `&'static str`).
pub fn ou(en: &'static str, pt: &'static str) -> &'static str {
    if ingles() { en } else { pt }
}

/// Roda `f` com `idioma` nesta thread e devolve o anterior ao sair.
pub fn com<T>(idioma: Idioma, f: impl FnOnce() -> T) -> T {
    let antes = ATUAL.with(|a| a.replace(Some(idioma)));
    let r = f();
    ATUAL.with(|a| a.set(antes));
    r
}

/// Fixa o idioma da thread até ela terminar (o `--mcp` fala só inglês com o agente).
pub fn fixar(idioma: Idioma) {
    ATUAL.with(|a| a.set(Some(idioma)));
}

/// `tr!("Plan {n} not found", "Plano {n} não encontrado")`: o texto no idioma em uso (os dois aceitam as mesmas variáveis de `format!`).
#[macro_export]
macro_rules! tr {
    ($en:literal, $pt:literal $(, $a:expr)* $(,)?) => {
        if $crate::idioma::ingles() { format!($en $(, $a)*) } else { format!($pt $(, $a)*) }
    };
}

#[cfg(test)]
#[path = "idioma_tests.rs"]
mod tests;
