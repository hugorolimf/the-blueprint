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

### Fifth round, 26 Oct 2026

Task Master lets you choose which MCP tools load with the `TASK_MASTER_TOOLS` environment variable (`all`, `standard`, `core` or a custom list; the default is a lean set of seven), per its [MCP documentation](https://docs.task-master.dev/capabilities/mcp.md). When it was added could not be confirmed. This supports the already planned action "MCP tool profiles", so no new action was created. Focalboard: only a community MCP server turned up and nothing from 2026.

Round decision: the blueprint already holds 77 actions, so this round only logs the source and moves on to execute.

### Sixth round, 26 Oct 2026

No Claude Code changelog entry for October 2026 turned up. Third-party guides ([claudelog](https://www.claudelog.com/faqs/what-is-plugins-system-in-claude-code/), [fast.io](https://fast.io/resources/claude-code-plugins-development-guide.md)) say Claude Code loads MCP tools lazily ("MCP Tool Search", up to 95% less context) and lists `http` and `mcp_tool` among its hook types; both claims are unverified against the official documentation. If the lazy loading is real, the planned "MCP tool profiles" matter mostly for agents that do not have it (Codex among them), so the action should start by measuring the context cost in each agent. No new action.

### Seventh round, 26 Oct 2026

Backlog.md: the search returned nothing about the project, so there is no new source. Plane's [changelog](https://plane.so/changelog/commercial) lists Releases and Collections in Wiki in self-hosted v2.6.0 (18 May 2026), a "Catch Up" widget on the mobile home screen with Plane AI Autopilot and Skills in mobile v2.4.0 (31 Jul 2026), and sub-item progress that counts cancelled items (v3.3.2, 7 Oct 2026); these come from a search summary, not from reading each entry. The "Catch Up" idea is what the planned "progress summary for a period" already covers, and cancelled items counting toward progress does not apply (Blueprint has no cancelled state). No new action; the round goes back to executing.

### Eighth round, 26 Oct 2026

Claude Code, October 2026 (third-party only: [Havoptic releases](https://www.havoptic.com/releases/claude-code/2026/10) and an [October cheat sheet](https://angelo-lima.fr/en/claude-code-cheatsheet-october-2026-state-of-play-en/); not checked against code.claude.com): v2.1.287 to v2.1.296 between 1 and 9 Oct, with hooks that can block an action when they fail, plugin install straight from marketplaces, a setting for how hard subagents work, and 33 hook events. Blueprint's hooks are indicators on purpose (they never block the agent), and the plugin is already a local marketplace, so nothing applies. Backlog.md (1.48.0) and Task Master (0.43.1): versions from undated index pages, no October 2026 release notes found. No new action; the round goes back to executing the "Native Claude integration" project.

### Ninth round, 26 Oct 2026

Collaboration, since the "Real-time collaboration" project started today. Plane ([changelog](https://plane.so/changelog/cloud)): live edit sync with member cursors in Pages since Sep 2024 ([entry](https://plane.so/changelog/cloud-sep-20-2024)), version history with per-person diffs in Pages (Feb 2026, [entry](https://plane.so/changelog/2026-02-01-product-tour-try-section-and-plane-ai-updates)), views shared with specific people (Aug 2026, from a search summary). A [forum request](https://forum.plane.so/t/formal-request-real-time-collaboration-websockets-should-be-prioritized-over-new-features/276) (undated) says teammates' work-item changes only show after a manual refresh, which confirms that live updates per item (planned action "Presence and live changes") matter more than live text editing. Focalboard: only hosting pages; the first user to sign up becomes admin ([Railway](https://railway.com/deploy/focalboard)), where Blueprint prints a one-time owner link instead. Linear and Vibe Kanban: nothing found. Idea kept for the collaboration design, not a new action: per-item history with who changed what, on top of the planned activity log. No new action.

### Tenth round, 26 Oct 2026

Linear (secondary sources only; no October 2026 entry found on linear.app/changelog): Linear Agent left beta in June 2026 ([PricingSaaS, week 26](https://pricingsaas.com/companies/linear/diffs/2026W26)), and its Agent API lets outside agents such as Claude Code appear as workspace members that are assigned issues and post updates ([BuildBetter guide](https://blog.buildbetter.ai/linear-ai-agents-2026-guide-5-alternatives-for-engineering-teams/), vendor-adjacent). Vibe Kanban: v0.0.164 (29 Jan 2026) shows subagent conversation logs in the workspace UI ([newreleases.io](https://newreleases.io/project/github/BloopAI/vibe-kanban/release/v0.0.164-20260129121853)); Blueprint already shows subagents on the sheet and keeps a per-action timeline. Task Master: no 2026 release notes found.

Idea added to the blueprint App (planned, not started): an assignee on each plan, a person or their agent, with a "My plans" filter and the MCP telling each agent which plans are its own. It belongs to the real-time collaboration project and depends on members.

Also added to `roadmap.json`: `responsavel-plano`.

### Eleventh round, 26 Oct 2026

Tunnels, for the Share button being built today (secondary sources; Cloudflare's own docs not read). Cloudflare quick tunnels need no account and give a random trycloudflare.com address until the process stops, are labelled a demo tool with no uptime guarantee, and one guide reports a cap of about 200 in-flight requests and no Server-Sent Events ([Flavio Copes](https://flaviocopes.com/cloudflare-quick-tunnels/)); named tunnels are free with a Cloudflare account and a domain on Cloudflare and keep a stable hostname ([same author](https://flaviocopes.com/cloudflare-tunnel/)). Tailscale Funnel exposes one local service to the public internet over HTTPS through a relay that does not decrypt traffic; visitors need no Tailscale ([Tailscale docs](https://tailscale.com/kb/1223/tailscale-funnel)). The SSE claim matters: live changes use SSE, so the builder was asked to measure it with the real cloudflared and add a fallback if needed.

Idea added to the blueprint App (planned, not started): a fixed address for sharing (named tunnel or Tailscale Funnel). Also added to `roadmap.json`: `compartilhar-workspace` (now).

### Twelfth round, 26 Oct 2026

Claude Code and remote MCP servers (secondary sources; the official docs were not read). `claude mcp add --scope user --transport http --header "Authorization: Bearer <token>" <name> <url>` adds a remote MCP server with a static header, and `${ENV_VAR}` can stand for the value ([Hashrocket TIL](https://til.hashrocket.com/posts/z9uvwfc3w1-setting-up-mcp-in-claude-code-with-auth-headers), [TrueFoundry](https://truefoundry.com/blog/mcp-authentication-in-claude-code)). Open reports: the connectivity preflight omits the configured headers and treats the 401 as fatal (2.1.220, July 2026, [claudeissues.com](https://claudeissues.com/issue/82534-http-mcp-transport-connectivity-preflight-omits-configured-headers-treats-result)); a server that also advertises OAuth makes Claude Code skip the header ([claudeissues.com](https://claudeissues.com/issue/2831-bug-mcp-http-transport-bypasses-authorization-header-attempts-oauth2-registratio)); `claude mcp add` echoes the header value ([claudeissues.com](https://claudeissues.com/issue/63150-claude-mcp-add-echoes-authorization-header-value-verbatim-to-stdout-leaks-bearer)).

Idea added to the blueprint App (planned, not started): remote MCP over HTTP for guests, so a guest connects their Claude without installing The Blueprint (no OAuth advertised; the preflight without a header gets the server description). Also added to `roadmap.json`: `mcp-http-convidados`.
