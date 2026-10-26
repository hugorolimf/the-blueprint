//! Instância única: a segunda execução não abre outro app, só pede à primeira que traga a janela à frente.
//! A trava é um mutex nomeado do Windows (some sozinho se o app cair); a porta do servidor da primeira fica num arquivo
//! em %APPDATA%\Blueprint\porta, porque a porta pode não ser a 3000 (se estava ocupada, o app usa outra).
use crate::http::config::pasta_do_app;
use crate::mcp::cliente::requisitar_em;
use std::path::PathBuf;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
use windows_sys::Win32::System::Threading::CreateMutexW;

#[derive(Debug, PartialEq, Eq)]
pub enum Instancia {
    Primeira,
    JaAberta,
}

fn arquivo_da_porta() -> PathBuf {
    pasta_do_app().join("porta")
}

/// Tenta ser a primeira instância. O mutex fica aberto até o processo acabar (de propósito: é isso que mantém a trava).
pub fn garantir() -> Instancia {
    let nome: Vec<u16> = "Local\\Blueprint.InstanciaUnica".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: `nome` termina em zero e vive durante a chamada; sem atributos de segurança (ponteiro nulo é o padrão).
    let (_, ja) = unsafe {
        let h = CreateMutexW(std::ptr::null(), 0, nome.as_ptr());
        (h, GetLastError() == ERROR_ALREADY_EXISTS)
    };
    if ja { Instancia::JaAberta } else { Instancia::Primeira }
}

/// A primeira instância anota onde o servidor dela está escutando.
pub fn registrar_porta(porta: u16) {
    let _ = std::fs::create_dir_all(pasta_do_app());
    let _ = std::fs::write(arquivo_da_porta(), porta.to_string());
}

/// Pede à instância aberta que mostre a janela (ou, com `link`, o `blueprint://convite…` que o navegador entregou). Devolve se ela respondeu.
pub fn pedir_que_mostre(link: Option<&str>) -> bool {
    let Some(porta) = std::fs::read_to_string(arquivo_da_porta()).ok().and_then(|t| t.trim().parse::<u16>().ok()) else { return false };
    // esta execução acabou de ser lançada pelo usuário; sem isto o Windows deixa a janela da primeira instância atrás do navegador
    // SAFETY: sem ponteiros; só concede à primeira instância o direito de ir para a frente.
    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY) };
    let corpo = serde_json::json!({ "url": link }).to_string();
    requisitar_em(&format!("http://127.0.0.1:{porta}"), "POST", "/api/app/mostrar", Some(&corpo)).is_ok_and(|(status, _)| status == 200)
}
