# Hourly research log

[Português](../pt-BR/interno/pesquisa-horaria.md)

One entry per round. Sources are web search results, some of them secondary; check the official page before quoting anything.

## 26 Oct 2026

Sources:
- Backlog.md (MrLesk/Backlog.md): tasks as Markdown files in Git, terminal and web board, shell completions for Zsh, Bash, Fish and PowerShell, MCP for agents. Via the [project README](https://cdn.jsdelivr.net/gh/MrLesk/Backlog.md@main/README.md) and an [AI Engineer talk page](https://ai.engineer/talks/backlog-md-terminal-kanban-board-for-managing-tasks-with-ai-agents-alex-gavrilescu-funstage). The version number (1.48.0) came from an index entry and is unverified.
- Linear and Claude Code: the search found no official changelog entry linking the two. Secondary sources (a June 2026 Firecrawl comparison, a community changelog) only confirm that Claude Code now reads AGENTS.md when there is no CLAUDE.md. Nothing to act on.

Ideas added to the blueprint App (planned, not started):
1. Export a blueprint as Markdown files (one per action plus an index), read-only.
2. Import GitHub issues as a numbered proposal through the existing queue, so nothing is saved before the click.
3. Shell completions for PowerShell and Bash.

Also added to `roadmap.json`: `exportar-markdown`, `importar-issues`, `completar-comandos`.

### Second round, 26 Oct 2026

Sources:
- Plane Agents, launched 18 Sep 2026: agents get instructions, skills, triggers (assigned, mentioned, item changed, schedule), memory and MCP tools, and a run panel shows the plan and the tools called. Via [Plane's announcement](https://plane.so/blog/agents-are-now-live-in-plane) and [changelog of 30 Sep 2026](https://plane.so/changelog/2026-09-30-plane-agents-propel-new-ai-models). The credits and plan limits come from one aggregator and are unverified.
- A community MCP server for Plane limits its tools with `PLANE_MCP_PROFILE=worker|planner|full` ([pkg.go.dev](https://pkg.go.dev/github.com/ItsJennyFiggy/plane-mcp@v1.2.0), June 2026). Community-maintained, not official.
- Task Master: the search found no release notes, so nothing was taken from it.

Ideas added to the blueprint App (planned, not started):
1. MCP tool profiles (`BLUEPRINT_MCP_PERFIL`), so the agent loads fewer tool descriptions.
2. Agent run timeline on each action, read-only, with a retention limit.

Also added to `roadmap.json`: `perfis-mcp`, `linha-do-tempo-agente`.

### Third round, 26 Oct 2026

Sources:
- Linear: [Linear Agent](https://linear.app/changelog/2026-03-24-introducing-linear-agent) (Mar 2026) and an "Agent assisted project updates" entry of 18 Jun 2026; coding sessions and loops (recurring agent workflows) are reported by a third-party tracker only, so their dates are unverified. Check [linear.app/changelog](https://linear.app/changelog/timeline).
- Vibe Kanban: each workspace gets its own git worktree and branch, and inline diff comments go straight back to the agent ([Virtus Lab](https://virtuslab.com/blog/ai/vibe-kanban/), [fork README](https://github.com/MrEggs/vibe-kanban-fork)). The company shut down in April 2026 and the project is community-maintained; the fork's date and activity are unconfirmed.

Ideas added to the blueprint App (planned, not started):
1. Progress summary for a period, in Markdown, computed from existing data (no AI, no network).
2. Send a plan comment to the agent, reusing the click-only session channel (no wider exception).

Also added to `roadmap.json`: `resumo-progresso`, `comentario-ao-agente`.

### Fourth round, 26 Oct 2026

Sources (all secondary; the official `openai/codex` releases were not read):
- Codex CLI: hooks reported stable since v0.124.0 on 23 Apr 2026, reading a `hooks.json` and observing MCP tools and `apply_patch` ([Daniel Vaughan](https://codex.danielvaughan.com/2026/04/23/codex-cli-hooks-graduate-stable-v0124-mcp-observation-inline-config/)); `codex plugin ... --json` flags in v0.137 and v0.138 alpha and `codex agents` in v0.148 alpha ([Tech Dev Notes](https://techdevnotes.com/releases/codex-cli/0.137.0), [Gradually changelog](https://www.gradually.ai/en/changelogs/codex-cli/)).
- ORCA v1.4.235 (20 Oct 2026, via [Releasebot](https://releasebot.io/updates/stablyai/orca)): chat rewind, per-chat settings, full agent output stored, phone chats resume after reconnecting. Nothing new to take: The Blueprint does not run agents, and its page already re-reads the queue and the data when the event stream reconnects (`web/js/mcp.js`).

Idea added to the blueprint App (planned, not started): Codex hooks for automatic progress. It depends on the Codex research action confirming the format first.

Also added to `roadmap.json`: `hooks-codex`.
