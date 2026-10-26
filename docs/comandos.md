# Servidor e comandos

[Voltar ao README](../README.pt-BR.md)

```
cargo build --release
./target/release/blueprint serve        # http://127.0.0.1:3000
```

Variáveis do `serve`: `HOST` (padrão 127.0.0.1), `PORT` (3000), `BLUEPRINT_DB` (banco), `DADOS_ARQ` (JSON importado na primeira execução com o banco vazio), `PROTOTIPOS_DIR`, `BLUEPRINT_ASSETS_DIR` (serve `web/` direto da pasta, sem recompilar) e `MCP_REDE=1` (veja [SECURITY.md](SECURITY.md) antes de usar).

Comandos: `blueprint` (janela), `blueprint --version`, `blueprint atualizar [--verificar|--instalar]`, `blueprint serve`, `blueprint --mcp` (MCP por stdio), `blueprint claude [args]` (abre uma sessão Claude com o plugin e a status line; ativa o agente quando a pasta está ligada), `blueprint importar <json> <banco>`, `blueprint exportar <banco> <json>`, `blueprint registrar [--remover]` (instala ou remove o plugin no Claude Code e o MCP no Codex), `blueprint registrar --status-line [--remover]` (status line global opcional) e `blueprint hook sessao|statusline|status|statusline-config` (usados pelo plugin; veja abaixo).
