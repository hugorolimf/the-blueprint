# Server and commands

[Back to the README](../README.md) · [Português](pt-BR/commands.md)

```
cargo build --release
./target/release/blueprint serve        # http://127.0.0.1:3000
```

Variables of `serve`: `HOST` (default 127.0.0.1), `PORT` (3000), `BLUEPRINT_DB` (the database), `DADOS_ARQ` (a JSON imported on the first run when the database is empty), `PROTOTIPOS_DIR`, `BLUEPRINT_ASSETS_DIR` (serves `web/` straight from the folder, with no rebuild) and `MCP_REDE=1` (read [SECURITY.md](../SECURITY.md) before using it).

Commands: `blueprint` (the window), `blueprint --version`, `blueprint doctor` (checks the app, port, database, Claude Code plugin, Codex MCP and folder link, and says what to do; it only reads, see [Troubleshooting](troubleshooting.md)), `blueprint atualizar [--verificar|--instalar]`, `blueprint serve`, `blueprint --mcp` (MCP over stdio), `blueprint claude [args]` (opens a Claude session with the plugin and the status line; activates the agent when the folder is linked), `blueprint importar <json> <database>`, `blueprint exportar <database> <json>`, `blueprint registrar [--remover]` (installs or removes the plugin in Claude Code and the MCP in Codex), `blueprint registrar --status-line [--remover]` (an optional global status line), `blueprint conectar <address> <key> [--remover]` (connects your Claude Code and Codex to a workspace shared with you; see [Sharing](compartilhar.md#the-guests-claude)) and `blueprint hook sessao|statusline|status|statusline-config` (used by the plugin; see [the plugin guide](claude-code.md)).

The subcommand names (`atualizar`, `importar`, `exportar`, `registrar`) are in Portuguese.

## Shell completions

`blueprint completions powershell` and `blueprint completions bash` print a script that completes the subcommands. In PowerShell, add `blueprint completions powershell | Out-String | Invoke-Expression` to your profile; in Bash, add `source <(blueprint completions bash)` to `~/.bashrc`.

## Export as Markdown

`blueprint exportar <database.db> <folder> --markdown` writes one folder per blueprint, with a `README.md` index and one file per action (dates, status, plans with their evidence, checkpoints). It is read-only: the database stays the source, and nothing is imported back. Commit the folder next to your code if you want the plan versioned.
