//! O app é do subsistema "console": digitado num terminal (`blueprint claude`, `serve`, `importar`…), o shell espera e o filho usa o mesmo console.
//! Aberto pelo atalho (sem argumentos), o Windows cria um console só para ele; este trecho o solta na hora para não ficar uma janela preta atrás do app.
use windows_sys::Win32::System::Console::{FreeConsole, GetConsoleProcessList};

pub fn soltar_console_proprio() {
    let mut processos = [0u32; 2];
    // SAFETY: `processos` tem 2 posições, como o tamanho informado.
    unsafe {
        // shortcut: se o app foi aberto de dentro de um terminal sem argumentos, o console é compartilhado (2+ processos) e fica como está.
        if GetConsoleProcessList(processos.as_mut_ptr(), 2) == 1 {
            FreeConsole();
        }
    }
}
