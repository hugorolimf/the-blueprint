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
