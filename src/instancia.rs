//! Instância única do app com janela (`janela` no Windows, `nativa` nos outros): a segunda execução não abre outro app, só pede à primeira que traga a janela à frente.
//! A trava é um mutex nomeado no Windows e um `flock` em `instancia.lock` nos outros sistemas (os dois somem sozinhos se o app cair); a porta do servidor da primeira
//! fica num arquivo `porta` na pasta do app, porque a porta pode não ser a 3000 (se estava ocupada, o app usa outra).
use crate::http::config::pasta_do_app;
use crate::mcp::cliente::requisitar_em;
use std::path::PathBuf;
#[cfg(windows)]
use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
#[cfg(windows)]
use windows_sys::Win32::System::Threading::CreateMutexW;

#[derive(Debug, PartialEq, Eq)]
pub enum Instancia {
    Primeira,
    JaAberta,
}

fn arquivo_da_porta() -> PathBuf {
    pasta_do_app().join("porta")
}

#[cfg(windows)]
fn nome_da_trava() -> Vec<u16> {
    // `BLUEPRINT_INSTANCIA_PARA_TESTE` (só testes): outra trava, para um app de teste abrir ao lado do app do usuário (o instalador de teste usa a mesma no AppMutex)
    let nome = std::env::var("BLUEPRINT_INSTANCIA_PARA_TESTE").map_or_else(|_| "Local\\Blueprint.InstanciaUnica".to_string(), |s| format!("Local\\Blueprint.Teste.{s}"));
    nome.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Só olha se o app está aberto, sem pegar a trava (o `blueprint doctor` não pode impedir o app de abrir).
#[cfg(windows)]
pub fn aberta() -> bool {
    let nome = nome_da_trava();
    // SAFETY: `nome` termina em zero e vive durante a chamada; o handle, se veio, é fechado logo em seguida.
    unsafe {
        let h = windows_sys::Win32::System::Threading::OpenMutexW(windows_sys::Win32::System::Threading::SYNCHRONIZATION_SYNCHRONIZE, 0, nome.as_ptr());
        if h.is_null() {
            return false;
        }
        windows_sys::Win32::Foundation::CloseHandle(h);
        true
    }
}

/// Tenta ser a primeira instância. O mutex fica aberto até o processo acabar (de propósito: é isso que mantém a trava).
#[cfg(windows)]
pub fn garantir() -> Instancia {
    let nome = nome_da_trava();
    // SAFETY: `nome` termina em zero e vive durante a chamada; sem atributos de segurança (ponteiro nulo é o padrão).
    let (_, ja) = unsafe {
        let h = CreateMutexW(std::ptr::null(), 0, nome.as_ptr());
        (h, GetLastError() == ERROR_ALREADY_EXISTS)
    };
    if ja { Instancia::JaAberta } else { Instancia::Primeira }
}

/// Tenta pegar a trava de arquivo: `Ok(Some)` pegou (vale enquanto o arquivo estiver aberto), `Ok(None)` outro processo a tem, `Err` não deu para abrir o arquivo.
#[cfg(not(windows))]
fn travar() -> std::io::Result<Option<std::fs::File>> {
    use std::os::fd::AsRawFd;
    // `BLUEPRINT_INSTANCIA_PARA_TESTE` (só testes): outra trava, para um app de teste abrir ao lado do app do usuário
    let nome = std::env::var("BLUEPRINT_INSTANCIA_PARA_TESTE").map_or_else(|_| "instancia.lock".to_string(), |s| format!("instancia.teste.{s}.lock"));
    std::fs::create_dir_all(pasta_do_app())?;
    let arquivo = std::fs::OpenOptions::new().create(true).truncate(false).write(true).open(pasta_do_app().join(nome))?;
    // SAFETY: o descritor é de um arquivo aberto que vive durante a chamada.
    Ok((unsafe { libc::flock(arquivo.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0).then_some(arquivo))
}

/// Só olha se o app está aberto: pega a trava e a solta na hora (o `blueprint doctor` não pode impedir o app de abrir).
#[cfg(not(windows))]
pub fn aberta() -> bool {
    matches!(travar(), Ok(None))
}

/// Tenta ser a primeira instância. O arquivo fica aberto até o processo acabar (de propósito: é isso que mantém a trava).
#[cfg(not(windows))]
pub fn garantir() -> Instancia {
    static TRAVA: std::sync::OnceLock<std::fs::File> = std::sync::OnceLock::new();
    match travar() {
        Ok(None) => Instancia::JaAberta,
        Ok(Some(arquivo)) => {
            let _ = TRAVA.set(arquivo);
            Instancia::Primeira
        }
        Err(_) => Instancia::Primeira,
    }
}

/// A primeira instância anota onde o servidor dela está escutando.
pub fn registrar_porta(porta: u16) {
    let _ = std::fs::create_dir_all(pasta_do_app());
    let _ = std::fs::write(arquivo_da_porta(), porta.to_string());
}

/// A porta que a primeira instância anotou. Sempre relê: o app pode ter reaberto em outra porta.
pub fn porta_registrada() -> Option<u16> {
    std::fs::read_to_string(arquivo_da_porta()).ok().and_then(|t| t.trim().parse().ok())
}

/// Onde achar o app: `BLUEPRINT_URL`, se definida; senão, a porta anotada. `arquivo` é o conteúdo do arquivo `porta`.
pub fn url_de(env: Option<&str>, arquivo: Option<&str>) -> Option<String> {
    if let Some(url) = env.map(str::trim).filter(|u| !u.is_empty()) {
        return Some(url.trim_end_matches('/').to_string());
    }
    arquivo.and_then(|t| t.trim().parse::<u16>().ok()).map(|porta| format!("http://127.0.0.1:{porta}"))
}

/// A URL do app agora; `None` se não há `BLUEPRINT_URL` nem porta anotada. Relê a cada chamada.
pub fn url_do_app() -> Option<String> {
    url_de(std::env::var("BLUEPRINT_URL").ok().as_deref(), std::fs::read_to_string(arquivo_da_porta()).ok().as_deref())
}

/// Pede à instância aberta que mostre a janela (ou, com `link`, o `blueprint://convite…` que o navegador entregou). Devolve se ela respondeu.
pub fn pedir_que_mostre(link: Option<&str>) -> bool {
    let Some(porta) = porta_registrada() else { return false };
    // esta execução acabou de ser lançada pelo usuário; sem isto o Windows deixa a janela da primeira instância atrás do navegador
    // SAFETY: sem ponteiros; só concede à primeira instância o direito de ir para a frente.
    #[cfg(windows)]
    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::AllowSetForegroundWindow(windows_sys::Win32::UI::WindowsAndMessaging::ASFW_ANY) };
    let corpo = serde_json::json!({ "url": link }).to_string();
    requisitar_em(&format!("http://127.0.0.1:{porta}"), "POST", "/api/app/mostrar", Some(&corpo)).is_ok_and(|(status, _)| status == 200)
}

#[cfg(test)]
mod tests {
    use super::url_de;

    #[test]
    fn url_de_le_a_porta() {
        assert_eq!(url_de(None, Some("4123\n")).as_deref(), Some("http://127.0.0.1:4123"));
        assert_eq!(url_de(Some("http://127.0.0.1:5000/"), Some("4123")).as_deref(), Some("http://127.0.0.1:5000"), "BLUEPRINT_URL vence");
        assert_eq!(url_de(Some("  "), Some("4123")).as_deref(), Some("http://127.0.0.1:4123"), "vazia é como ausente");
        assert_eq!(url_de(None, Some("não é porta")), None);
        assert_eq!(url_de(None, Some("70000")), None);
        assert_eq!(url_de(None, None), None);
    }
}
