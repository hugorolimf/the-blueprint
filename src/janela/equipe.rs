//! A janela da equipe: o convite que chega pelo link, aberto no app instalado antes da versão web.
//! O instalador registra `blueprint://`; a página `/convite` tenta `blueprint://convite?u=<https://endereço>&s=<segredo>` e, se o app não
//! responde, segue na web. Aqui o link vira uma segunda janela (WebView2, o mesmo perfil) apontada ao servidor da equipe, onde o convidado
//! entra pelo nome e liga o próprio Claude (`conectar`, pelo botão da tela Minha chave). Qualquer site pode disparar `blueprint://`, por isso
//! nada abre nem muda sem uma pergunta nativa com o endereço.
use super::{criar_webview, Evento};
use crate::registrar::conectar::{executar, validar_endereco};
use tao::event_loop::{EventLoopProxy, EventLoopWindowTarget};
use tao::window::{Window, WindowBuilder};
use wry::{WebContext, WebView};
use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, IDYES, MB_ICONQUESTION, MB_OK, MB_SETFOREGROUND, MB_YESNO};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Convite {
    /// `https://host[:porta]`, sem caminho
    pub origem: String,
    /// a página de convite do servidor, com o segredo no fragmento
    pub url: String,
}

/// `Some` se `arg` é um link `blueprint://convite?u=…&s=…` com endereço e segredo no formato que o servidor gera (o mesmo segredo que a página aceita).
pub fn convite_do_link(arg: &str) -> Option<Convite> {
    let consulta = arg.strip_prefix("blueprint://convite")?.trim_start_matches('/').strip_prefix('?')?;
    let (mut origem, mut segredo) = ("", "");
    for par in consulta.split('&') {
        match par.split_once('=') {
            Some(("u", v)) => origem = v,
            Some(("s", v)) => segredo = v,
            _ => {}
        }
    }
    let segredo_ok = (20..=200).contains(&segredo.len()) && segredo.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_');
    let origem = validar_endereco(origem, false).ok().filter(|_| segredo_ok)?;
    Some(Convite { url: format!("{origem}/convite#{segredo}"), origem })
}

/// `Some` se `arg` é o link interno `blueprint://equipe?u=<origem>` que a rota /api/equipes/abrir entrega (equipes.rs): abre a equipe já lembrada, na raiz.
pub fn equipe_do_link(arg: &str) -> Option<Convite> {
    let origem = validar_endereco(arg.strip_prefix(crate::equipes::PREFIXO_LINK)?, false).ok()?;
    Some(Convite { url: format!("{origem}/"), origem })
}

fn host(origem: &str) -> &str {
    origem.split_once("://").map_or(origem, |(_, h)| h)
}

fn largo(texto: &str) -> Vec<u16> {
    texto.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Caixa de diálogo do Windows; bloqueia a thread que chama (nunca chame no laço da janela).
pub(super) fn perguntar(texto: &str) -> bool {
    let (t, c) = (largo(texto), largo("The Blueprint (beta)"));
    // SAFETY: as duas strings terminam em zero e vivem durante a chamada; sem janela dona (ponteiro nulo).
    unsafe { MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_YESNO | MB_ICONQUESTION | MB_SETFOREGROUND) == IDYES }
}

pub(super) fn avisar(texto: &str) {
    let (t, c) = (largo(texto), largo("The Blueprint (beta)"));
    // SAFETY: igual a `perguntar`.
    unsafe { MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_OK | MB_SETFOREGROUND) };
}

/// Pergunta se abre o convite e só então manda o laço da janela abrir (recusou: só traz a janela principal).
pub fn pedir_abertura(proxy: EventLoopProxy<Evento>, convite: Convite) {
    std::thread::spawn(move || {
        let host = host(&convite.origem);
        let abrir = perguntar(&crate::tr!("Open the invite from {host} in The Blueprint?", "Abrir o convite de {host} no The Blueprint?"));
        let _ = proxy.send_event(if abrir { Evento::Equipe(convite) } else { Evento::Mostrar });
    });
}

/// O botão "Conectar meu Claude" da janela da equipe: depois da pergunta, instala o plugin do convidado com a chave dele (o mesmo que `blueprint conectar`).
pub fn pedir_conexao(origem: String, chave: String) {
    std::thread::spawn(move || {
        let host = host(&origem);
        if perguntar(&crate::tr!("Connect your Claude Code and Codex on this computer to {host}? They will read and update the shared blueprints with your role.", "Conectar o seu Claude Code e o Codex deste computador a {host}? Eles vão ler e atualizar os blueprints compartilhados com o seu papel.")) {
            avisar(&executar(&origem, &chave).0);
        }
    });
}

/// A segunda janela, com o WebView da equipe. O WebView vem primeiro na tupla para ser destruído antes da janela.
pub fn abrir_janela(alvo: &EventLoopWindowTarget<Evento>, contexto: &mut WebContext, convite: &Convite, proxy: EventLoopProxy<Evento>) -> Result<(WebView, Window), wry::Error> {
    let window = WindowBuilder::new()
        .with_title(format!("The Blueprint (beta) · {}", host(&convite.origem)))
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 800.0))
        .with_min_inner_size(tao::dpi::LogicalSize::new(800.0, 500.0))
        .with_window_icon(super::icone::janela().ok())
        .build(alvo)
        .map_err(|e| wry::Error::Io(std::io::Error::other(e.to_string())))?;
    let origem = format!("{}/", convite.origem); // a barra final: `https://host.outro.site` não passa por `https://host`
    let webview = criar_webview(&window, contexto, &convite.url, &origem, false, || {}, proxy)?;
    Ok((webview, window))
}

#[cfg(test)]
mod tests {
    use super::*;

    const S: &str = "Ab3_dE-fGhIjKlMnOpQrStUvWxYz0123456789abcd";

    #[test]
    fn link_valido_vira_a_pagina_de_convite() {
        let c = convite_do_link(&format!("blueprint://convite?u=https://abc-def.trycloudflare.com&s={S}")).unwrap();
        assert_eq!(c.origem, "https://abc-def.trycloudflare.com");
        assert_eq!(c.url, format!("https://abc-def.trycloudflare.com/convite#{S}"));
        // o Windows/navegador pode acrescentar a barra depois do host; a porta vale
        assert!(convite_do_link(&format!("blueprint://convite/?u=https://equipe.exemplo.com:8443&s={S}")).is_some());
    }

    #[test]
    fn link_de_equipe_lembrada_abre_na_raiz() {
        let c = equipe_do_link("blueprint://equipe?u=https://abc-def.trycloudflare.com").unwrap();
        assert_eq!((c.origem.as_str(), c.url.as_str()), ("https://abc-def.trycloudflare.com", "https://abc-def.trycloudflare.com/"));
        assert_eq!(equipe_do_link("blueprint://equipe?u=http://abc.com"), None);
        assert_eq!(equipe_do_link("blueprint://equipe?u=https://abc.com/x"), None);
        assert_eq!(equipe_do_link(&format!("blueprint://convite?u=https://abc.com&s={S}")), None);
    }

    #[test]
    fn recusa_o_que_nao_e_convite() {
        let ruins = [
            format!("blueprint://convite?u=http://equipe.exemplo.com&s={S}"), // sem https
            format!("blueprint://convite?u=https://equipe.exemplo.com/caminho&s={S}"),
            format!("blueprint://convite?u=https://a.com@evil.com&s={S}"),
            "blueprint://convite?u=https://equipe.exemplo.com&s=curto".to_string(),
            format!("blueprint://convite?u=https://equipe.exemplo.com&s={S}<x>"),
            format!("blueprint://convite?s={S}"),
            format!("blueprint://outro?u=https://equipe.exemplo.com&s={S}"),
            format!("https://equipe.exemplo.com/convite#{S}"),
        ];
        for r in ruins {
            assert_eq!(convite_do_link(&r), None, "{r}");
        }
    }
}
