//! Ponto de entrada. Sem argumentos abre o app com janela; `serve`, `--mcp`, `importar` e `exportar` são da linha de comando (veja `cli.rs`).
//! Na versão final o app é do subsistema "windows" (o atalho não abre uma janela de console); a linha de comando reencaixa a saída no terminal.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    if !args.is_empty() && args[0] != "--mcp" {
        blueprint::janela::console::anexar_ao_terminal();
    }
    std::process::exit(blueprint::cli::executar(args));
}
