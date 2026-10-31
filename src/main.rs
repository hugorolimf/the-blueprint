//! Ponto de entrada. Sem argumentos abre o app com janela; `serve`, `--mcp`, `importar` e `exportar` são da linha de comando (veja `cli.rs`).
//! O executável é de console para o shell esperar `blueprint claude`; sem argumentos, `janela::console` solta o console que o atalho criou.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(windows)]
    if args.is_empty() {
        blueprint::janela::console::soltar_console_proprio();
    }
    std::process::exit(blueprint::cli::executar(args));
}
