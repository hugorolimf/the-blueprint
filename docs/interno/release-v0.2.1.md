# The Blueprint v0.2.1 (beta)

[Português](../pt-BR/interno/release-v0.2.1.md)

## What is new

- **A guest's Claude connects with one command.** On the My key screen, a guest copies `blueprint conectar <address> <key>` (The Blueprint must be installed on their computer). Their Claude Code then reads and proposes in the shared workspace with their role, and their hooks reach your app: you see "Claude · name" working on your sheet. When the sharing address changes, they run the same command again and it applies at once, even to open sessions. The key stays in a file of the app, never in process arguments; `blueprint conectar --remover` undoes it. See [Share from the app](../compartilhar.md).
- **Several workspaces can be shared at the same time** through one address, and the Share dialog always talks about the workspace you have open.
- **Leaner `ler_blueprint` for agents:** on large blueprints, finished and distant planned actions become one line each (about 89k to 20k characters for 100 actions), with optional `incluir` and `projeto` parameters.
- **More from the Claude Code plugin:** the folder link follows git worktrees to the main folder; new hooks (permission requests and denials, stop failures, compaction) turn into agent states without ever blocking Claude; subagents get the plan context when they start; three role agents ship with the plugin (explorer, builder, reviewer); status lines show context, usage limits, worktree and agent, and the subagent status line feeds focus mode.
- A subagent that finishes leaves the sheet after 10 s; only the main Claude stays as Done.
- **Decisions on a blueprint:** a card in Edit records open and settled decisions (title, choice, reason), and agents can propose them.

## How to install

Download `Blueprint-0.2.1-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
