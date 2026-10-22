//! O app é do subsistema "windows" (abrir pelo atalho não mostra uma janela de console). Para `blueprint serve`, `importar` e `exportar`
//! digitados num terminal, este trecho reencaixa a saída no console de quem chamou. Se a saída já foi redirecionada (um agente chamando
//! `--mcp`, um `> arquivo`), não mexe em nada.
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING};
use windows_sys::Win32::System::Console::{AttachConsole, GetStdHandle, SetStdHandle, ATTACH_PARENT_PROCESS, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn anexar_ao_terminal() {
    // SAFETY: só chamadas do Windows com argumentos válidos; os nomes CONOUT$/CONIN$ terminam em zero e vivem durante cada chamada.
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            return; // sem terminal por trás (aberto pelo atalho): nada a fazer
        }
        for (canal, nome, acesso) in [(STD_OUTPUT_HANDLE, "CONOUT$", GENERIC_WRITE), (STD_ERROR_HANDLE, "CONOUT$", GENERIC_WRITE), (STD_INPUT_HANDLE, "CONIN$", GENERIC_READ)] {
            let atual = GetStdHandle(canal);
            if atual.is_null() || atual == INVALID_HANDLE_VALUE {
                let h = CreateFileW(wide(nome).as_ptr(), acesso, FILE_SHARE_READ | FILE_SHARE_WRITE, null(), OPEN_EXISTING, 0, null_mut());
                if h != INVALID_HANDLE_VALUE {
                    SetStdHandle(canal, h);
                }
            }
        }
    }
}
