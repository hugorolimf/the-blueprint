# The Blueprint v0.1.0 (beta)

[Português](../pt-BR/interno/release-v0.1.0.md)

First public version, in beta (it may change between versions): a local-first roadmap, in Rust, that **your AI agent** (Claude Code, Codex…) reads and updates over MCP and that **you** review on a blueprint sheet.

## What it does

- The model **Workspace → Blueprint → Project → Action → Plan / Checkpoint**, with dates, a calculated percentage and a plotting sheet; each project is a numbered section of the sheet.
- The agent **proposes** (`propor_blueprint`) and you **apply** on the Plan screen (one proposal at a time, numbered); completing a plan or checkpoint requires **evidence**.
- Deadlines at AI pace: hours per plan, warnings for long or loose deadlines and measurement of real time.
- **Claude Code plugin** (skill, agent, MCP and hooks), terminal mod and status line: the session starts on the right blueprint and action, and edits become progress.
- **Claude** (and the subagents) appears on the sheet, walks to the plan it works on and rests when it stops; the **Start with Claude** button opens a new session on an action.
- With the app **closed**, the agent keeps registering: what it did is kept and enters the app when it opens.
- **Automatic update on your click**: the app downloads the new version in the background, checks the SHA-256 and only installs when you click Update (window, tray or `blueprint atualizar --instalar`).
- HTML action prototypes run in an isolated viewer (sandbox).
- A Windows app (WebView2 window, tray, a 3 MB installer, data in `%APPDATA%\Blueprint`) and `blueprint serve` for any system.

## How to install

Download `Blueprint-0.1.0-instalador.exe` (it installs for your user only) or build with `cargo build --release`. The installer connects the plugin and the MCP to the Claude Code and Codex it finds. By hand: `claude mcp add blueprint -- <path>\blueprint.exe --mcp`. Details in the [README](../../README.md). This release's `SHA256SUMS` carries the installer's hash.

## Limitations

- The window, tray and installer are Windows only; on other systems use `blueprint serve` in the browser.
- No login or roles: it is a local app (the MCP routes only answer on loopback). The collaborative server mode does not exist yet.
- The app does not talk to agents: it only shows what your agent reports (and opens the Claude session when you click).
- The installer is not signed (Windows may warn on the first run) and the `SHA256SUMS` comes from the same release: it checks the download, not the author. Signing is the next step.
