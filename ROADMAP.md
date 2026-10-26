# The Blueprint roadmap

[Leia em português](ROADMAP.pt-BR.md) · Updated: 2026-10-26

Where The Blueprint is and where it is going. States: shipped, doing now, next, later. No dates are promised; the order is the point. Ideas and votes are welcome in Issues.

## How to read it

✅ Shipped · 🔨 Doing now · ⏭️ Next · 💭 Later

## Planning

- ✅ **Workspace → Blueprint → Project → Action → Plan/Checkpoint**: A planning model with a blueprint-style sheet, projects as numbered sections and per-action numbering.
- ✅ **Deadlines at AI pace and measured real time**: Plans of 0.5–4 h, actions of 1–3 days; the app measures how long each plan really took.
- ✅ **Propose, apply, prove**: The agent only proposes; you apply. Marking a plan done requires evidence (the command or test and its result).
- ⏭️ **Presentation and print of a blueprint**: Turn a blueprint into a project proposal, a schedule or a progress report for clients and companies, printable to A4 and exportable as one HTML file. The AI also builds interactive Anatomy figures that explain the project being built, shown in a sandbox and replaced by a static snapshot when printed.
- 💭 **Import and export Markdown and GitHub Issues**: Move plans between the Blueprint, Markdown files in a repository and issue trackers.
- 💭 **One Git worktree per action**: Start an action in its own worktree so parallel agents do not step on each other (an idea seen in ORCA and Vibe Kanban).

## Agent integration

- ✅ **MCP server for any agent**: Tools to list, read, propose, report progress, mark done with evidence, attach prototypes and register work done outside the plan.
- ✅ **Claude Code plugin**: Skill, agent, hooks, status line and /blueprint:status; the Claude appears on the sheet, on the plan it works on, with its subagents.
- ✅ **Start an action from the app**: A confirmed click opens a new Claude Code session, or sends the request to a session already open in the folder.
- ⏭️ **Claude Code inside the app**: The official claude command running in an integrated, themed terminal where the agents work, organized by action, with every Claude Code capability. Architecture to be decided with a measured prototype (pseudoconsole vs. stream-json); light and lazy-loaded.
- ⏭️ **Native Codex integration**: Bring to Codex what the Claude Code plugin does: hooks, skills, status and sessions, after a documented parity study. Today Codex gets the MCP server and the AGENTS.md line only.
- 💭 **Guides for other MCP agents**: Tested setup guides for other agents that speak MCP.

## Interface

- ✅ **Sheet, context menu, animations and notifications**: The blueprint sheet with expand and collapse, a context menu, agent notifications and the animated Claude.
- 🔨 **This roadmap inside the app**: An animated, interactive Roadmap screen fed by the same file as this page.
- ⏭️ **First five minutes**: An empty-state with next steps, a sample blueprint inside the app and a blueprint doctor command to diagnose the setup.

## Data and security

- ✅ **Local-first, small and private**: SQLite database on your machine, a ~2.5 MB binary, no account, no telemetry; MCP routes answer only on loopback; prototypes run in an opaque-origin sandbox.
- ✅ **Windows stay in sync**: Versioned writes with a 409 on stale data, a three-way merge by id and a live update on every save, so two windows never overwrite each other.
- ✅ **Updates on your click**: Downloads in the background, verifies the SHA-256 and installs only when you click, after copying your database.
- ⏭️ **Data that survives anything**: Persist the agent queue, rotating automatic backups and a blueprint restore command.
- ⏭️ **Signed releases**: Sign the release checksums so the app can prove an installer is the author's, not only intact.
- 💭 **Performance guards in CI**: Fail the build when the binary, the API latency or the idle memory regress.

## Platforms

- ✅ **Windows 10/11**: Native window, tray, single instance and an installer; blueprint serve runs the same app as a local web server anywhere.
- 💭 **Linux and macOS**: A feasibility study first: what is Windows-only today and what a build on Ubuntu hits.
- 💭 **iOS and Android app**: Follow the agents, review proposals and show a presentation from the phone. The desktop stays the source of data.

## Collaboration and sharing

- 💭 **Collaborative mode**: A server mode with members, roles (owner, collaborator, supervisor, visitor) and invite links, live between people. Design first; the sync fix shipped now is its first building block.
- 💭 **Public read-only link**: Share a proposal or progress with a client through a revocable link.

## Community

- ✅ **Open source from day one**: MIT license, CONTRIBUTING, SECURITY, CI on Windows and Ubuntu, releases with checksums.
- 🔨 **A README, demo videos and guides worth reading**: A new README modeled on the best projects of the genre, recorded demos, step-by-step guides and a troubleshooting FAQ.
- ⏭️ **winget and Scoop**: Install with a single command from the Windows package managers.

---

Have an idea or a vote? [Open an issue](https://github.com/hugorolimf/the-blueprint/issues). Source of this page: `roadmap.json` (the app's Roadmap screen reads the same file).
