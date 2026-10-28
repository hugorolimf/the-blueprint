# The Blueprint v0.4.0 (beta)

[Português](../pt-BR/interno/release-v0.4.0.md)

## What is new

- **Nothing is lost when the app closes.** Proposals, work records and the marks your agent sends now wait in the database instead of in memory, and come back the same after a restart.
- **Daily backups and `blueprint restaurar`.** When the app opens it copies the database at most once a day and keeps the last 7. `blueprint restaurar` lists the copies and `blueprint restaurar <number>` goes back to one (with the app closed; the current database is kept first).
- **`blueprint doctor`.** One command checks whether the app answers where agents look, the version, the database, the Claude Code plugin, the Codex MCP and whether the current folder is linked to a blueprint, and says what to do for each problem.
- **First use.** An empty app shows the next step and can open an example blueprint, so you see the cycle before planning your own.
- **MCP tool profiles.** `BLUEPRINT_MCP_PERFIL=executor` or `planejador` gives an agent only the tools it needs (up to 50% less context).
- **Top bar of the sheet.** The buttons no longer cover the blueprint title; in a narrow window they turn into icons, with the name on hover.
- **Docs.** Troubleshooting guide, guide for Codex and other MCP agents, and a demo of the cycle in the README.

## Known limits

- The installer is still not signed (the signing proposal is in the repository and waits for a key).

## How to install

Download `Blueprint-0.4.0-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
