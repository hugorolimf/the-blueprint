# Reference research (25 Oct 2026)

[Português](../pt-BR/interno/research.md)

A record of what was consulted for the positioning, the README and the roadmap. Everything came from GitHub's public API and from official pages on 25 Oct 2026; numbers change, so redo the lookup before quoting them.

## Projects in the genre (GitHub API, 25 Oct 2026)

| Project | Stars | License | Last push | Latest release | What it is (the repository's own description) |
| --- | --- | --- | --- | --- | --- |
| [stablyai/orca](https://github.com/stablyai/orca) | 105,958 | MIT | 26 Oct 2026 | v1.4.240 (25 Oct) | an app to work with agents in worktrees (the source of several ideas in The Blueprint) |
| [makeplane/plane](https://github.com/makeplane/plane) | 61,440 | AGPL-3.0 | 23 Oct 2026 | v1.4.3-rc2 (23 Oct) | "Open-source Jira, Linear, Monday, and ClickUp alternative" |
| [BloopAI/vibe-kanban](https://github.com/BloopAI/vibe-kanban) | 28,456 | Apache-2.0 | 19 Sep 2026 | v0.1.44 (24 Apr) | a kanban board to coordinate coding agents (the company shut down in April 2026, the project continues community-maintained) |
| [eyaltoledano/claude-task-master](https://github.com/eyaltoledano/claude-task-master) | 28,276 | "Other" (read the LICENSE file) | 28 Apr 2026 | 0.43.1 (31 Mar) | "An AI-powered task-management system you can drop into Cursor, Lovable, Windsurf, Roo, and others" |
| [mattermost-community/focalboard](https://github.com/mattermost-community/focalboard) | 26,526 | "Other" | 18 May 2026 | v8.0.0 (13 Jun 2024) | an open source, self-hosted alternative to Trello, Notion and Asana |
| [MrLesk/Backlog.md](https://github.com/MrLesk/Backlog.md) | 7,149 | MIT | 25 Oct 2026 | v1.55.3 (23 Oct) | "A tool for managing project collaboration between humans and AI Agents in a git ecosystem" |
| [openai/codex](https://github.com/openai/codex) | 130,902 | Apache-2.0 | 23 Oct 2026 | rust-v0.164.0 (23 Oct) | OpenAI's coding agent (the target of the native integration) |
| [anthropics/claude-code](https://github.com/anthropics/claude-code) | 153,369 | (no license declared) | 25 Oct 2026 | v2.1.310 (23 Oct) | Anthropic's coding agent |

The Blueprint, the same day: 3 stars, MIT, 4 releases (v0.1.0 to v0.1.3), GitHub community profile at 85%.

## What repeats in the READMEs that work

Measured on the READMEs of Task Master, Backlog.md, Plane, Vibe Kanban, Focalboard, Excalidraw and Ollama:

1. A logo, one sentence and badges (version, license, stars) on the first screen.
2. The install command appears without scrolling: `npm i -g backlog.md`, `npx vibe-kanban`, Ollama's `curl | sh`.
3. Plenty of images: Excalidraw 23, Task Master 16, Plane 14, Backlog.md 13, Vibe Kanban 6, Focalboard 5, Ollama 1; The Blueprint had 1.
4. A "why" or comparison section (Backlog.md "Why Backlog.md in the AI era"; Plane "alternative to Jira and Linear").
5. The detail lives in separate documentation: Plane's README has 0 code blocks and links to the docs.
6. A guide per agent or client (Task Master "Option 1: MCP / Option 2: Command Line"; Backlog.md "Client guides").
7. Community and security visible (Discord, contributors, SECURITY).
8. All in English.

## Two signs about sustainability

- **Bloop (Vibe Kanban) shut the company down in April 2026**: "the vast majority of users were free" and they did not find a business model; the project continues as community-maintained open source. Before that, the team plan cost US$ 30 a month for 2 to 49 users. Source: [vibekanban.com/shutdown](https://vibekanban.com/shutdown) and [the pricing page captured before](https://vibekanban.com/pricing).
- **Plane** sells two things on the same AGPL core: the cloud (Free up to 12 users; Pro at US$ 6 per seat per month; Business at US$ 13; Enterprise on request) and a self-hosted commercial edition with approval workflows, audit, SSO and LDAP. Sources: [plane.so/pricing](https://plane.so/pricing.md), [Plane's editions](https://plane.so/blog/plane-and-its-editions) (the vendor's own sources; prices may have changed).

## Codex (for the native integration)

Secondary sources from April and May 2026 and one official page: Codex has AGENTS.md, skills (a folder with SKILL.md), MCP, subagents, plugins with a marketplace and hooks (`.codex/hooks.json`, requires `codex_hooks = true`). Official source: [developers.openai.com/codex/concepts/customization](https://developers.openai.com/codex/concepts/customization). **Not verified:** the exact format of hook events, `notify` and plugin publishing; the action "Codex integration: research and design" must read the official documentation and the `openai/codex` code before stating anything.
