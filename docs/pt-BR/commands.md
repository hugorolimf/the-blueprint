# Servidor e comandos

[Voltar ao README](../../README.pt-BR.md) · [English](../commands.md)

```
cargo build --release
./target/release/blueprint serve        # http://127.0.0.1:3000
```

Variáveis do `serve`: `HOST` (padrão 127.0.0.1), `PORT` (3000), `BLUEPRINT_DB` (banco), `DADOS_ARQ` (JSON importado na primeira execução com o banco vazio), `PROTOTIPOS_DIR`, `BLUEPRINT_ASSETS_DIR` (serve `web/` direto da pasta, sem recompilar) e `MCP_REDE=1` (veja [SECURITY.md](../../SECURITY.md) antes de usar).

Comandos: `blueprint` (janela), `blueprint --version`, `blueprint doctor` (confere app, porta, banco, plugin do Claude Code, MCP do Codex e pasta ligada, e diz o que fazer; só lê, veja [Solução de problemas](troubleshooting.md)), `blueprint atualizar [--verificar|--instalar]`, `blueprint serve`, `blueprint --mcp` (MCP por stdio), `blueprint claude [args]` (abre uma sessão Claude com o plugin e a status line; ativa o agente quando a pasta está ligada), `blueprint importar <json> <banco>`, `blueprint exportar <banco> <json>`, `blueprint registrar [--remover]` (instala ou remove o plugin no Claude Code e o MCP no Codex), `blueprint registrar --status-line [--remover]` (status line global opcional), `blueprint conectar <endereço> <chave> [--remover]` (liga o seu Claude Code e o Codex a um workspace compartilhado com você; veja [Compartilhar](compartilhar.md#o-claude-do-convidado)) e `blueprint hook sessao|statusline|status|statusline-config` (usados pelo plugin; veja abaixo).

## Completar comandos

`blueprint completions powershell` e `blueprint completions bash` imprimem um script que completa os subcomandos. No PowerShell, ponha `blueprint completions powershell | Out-String | Invoke-Expression` no seu perfil; no Bash, `source <(blueprint completions bash)` no `~/.bashrc`.

## Exportar como Markdown

`blueprint exportar <banco.db> <pasta> --markdown` grava uma pasta por blueprint, com um índice `README.md` e um arquivo por ação (datas, estado, planos com a evidência, checkpoints). É só leitura: o banco continua sendo a fonte e nada é importado de volta. Faça commit da pasta junto do código se quiser o plano versionado.
