# The Blueprint roadmap

[Leia em português](ROADMAP.pt-BR.md) · Updated: 2026-10-26

Where The Blueprint is and where it is going. States: shipped, doing now, next, later. No dates are promised; the order is the point. Ideas and votes are welcome in Issues.

## How to read it

Shipped · Doing now · Next · Later

## Planning

- *Shipped* · **Workspace → Blueprint → Project → Action → Plan/Checkpoint**: A planning model with a blueprint-style sheet, projects as numbered sections and per-action numbering.
- *Shipped* · **Deadlines at AI pace and measured real time**: Plans of 0.5–4 h, actions of 1–3 days; the app measures how long each plan really took.
- *Shipped* · **Propose, apply, prove**: The agent only proposes; you apply. Marking a plan done requires evidence (the command or test and its result).
- *Next* · **Presentation and print of a blueprint**: Turn a blueprint into a project proposal, a schedule or a progress report for clients and companies, printable to A4 and exportable as one HTML file. The AI also builds interactive Anatomy figures that explain the project being built, shown in a sandbox and replaced by a static snapshot when printed.
- *Next* · **Export a blueprint as Markdown files**: One file per action plus an index, readable on GitHub and versioned with the code. Read-only: the database stays the source.
- *Later* · **Import and export Markdown and GitHub Issues**: Move plans between the Blueprint, Markdown files in a repository and issue trackers.
- *Later* · **One Git worktree per action**: Start an action in its own worktree so parallel agents do not step on each other (an idea seen in ORCA and Vibe Kanban).
- *Later* · **Import GitHub issues as a proposal**: Open issues become actions and plans in a numbered proposal that you review and apply. Nothing is saved before your click.

## Agent integration

- *Shipped* · **MCP server for any agent**: Tools to list, read, propose, report progress, mark done with evidence, attach prototypes and register work done outside the plan.
- *Shipped* · **Claude Code plugin**: Skill, agent, hooks, status line and /blueprint:status; the Claude appears on the sheet, on the plan it works on, with its subagents.
- *Shipped* · **Start an action from the app**: A confirmed click opens a new Claude Code session, or sends the request to a session already open in the folder.
- *Next* · **Claude Code inside the app**: The official claude command running in an integrated, themed terminal where the agents work, organized by action, with every Claude Code capability. Architecture to be decided with a measured prototype (pseudoconsole vs. stream-json); light and lazy-loaded.
- *Next* · **Native Codex integration**: Bring to Codex what the Claude Code plugin does: hooks, skills, status and sessions, after a documented parity study. Today Codex gets the MCP server and the AGENTS.md line only.
- *Later* · **Guides for other MCP agents**: Tested setup guides for other agents that speak MCP.
- *Later* · **MCP tool profiles for a smaller context**: An environment variable limits which tools the agent sees (executor, planner or full), so the tool descriptions take less of its context.
- *Later* · **Agent run timeline on each action**: A read-only list of what the agent did on an action and when: started, changed file, finished a plan, blocked.
- *Later* · **Send a plan comment to the agent**: A button on a plan comment sends it to the Claude session you pick, only on your click and only on your machine.
- *Later* · **Codex hooks for automatic progress**: Once the hook format is confirmed in the official source, Codex gets what Claude Code already has: the right blueprint at session start and edits recorded as progress.

## Interface

- *Shipped* · **Sheet, context menu, animations and notifications**: The blueprint sheet with expand and collapse, a context menu, agent notifications and the animated Claude.
- *Doing now* · **This roadmap inside the app**: An animated, interactive Roadmap screen fed by the same file as this page.
- *Next* · **First five minutes**: An empty-state with next steps, a sample blueprint inside the app and a blueprint doctor command to diagnose the setup.

## Data and security

- *Shipped* · **Local-first, small and private**: SQLite database on your machine, a ~2.5 MB binary, no account, no telemetry; MCP routes answer only on loopback; prototypes run in an opaque-origin sandbox.
- *Shipped* · **Windows stay in sync**: Versioned writes with a 409 on stale data, a three-way merge by id and a live update on every save, so two windows never overwrite each other.
- *Shipped* · **Updates on your click**: Downloads in the background, verifies the SHA-256 and installs only when you click, after copying your database.
- *Next* · **Data that survives anything**: Persist the agent queue, rotating automatic backups and a blueprint restore command.
- *Next* · **Signed releases**: Sign the release checksums so the app can prove an installer is the author's, not only intact.
- *Later* · **Performance guards in CI**: Fail the build when the binary, the API latency or the idle memory regress.

## Platforms

- *Shipped* · **Windows 10/11**: Native window, tray, single instance and an installer; blueprint serve runs the same app as a local web server anywhere.
- *Later* · **Linux and macOS**: A feasibility study first: what is Windows-only today and what a build on Ubuntu hits.
- *Later* · **iOS and Android app**: Follow the agents, review proposals and show a presentation from the phone. The desktop stays the source of data.

## Collaboration and sharing

- *Later* · **Collaborative mode**: A server mode with members, roles (owner, collaborator, supervisor, visitor) and invite links, live between people. Design first; the sync fix shipped now is its first building block.
- *Later* · **Public read-only link**: Share a proposal or progress with a client through a revocable link.
- *Later* · **Progress summary for a period**: One click copies a Markdown summary of what was finished, what is in progress and what is blocked, with the evidence. Computed from your data, no AI and no network.

## Community

- *Shipped* · **Open source from day one**: MIT license, CONTRIBUTING, SECURITY, CI on Windows and Ubuntu, releases with checksums.
- *Doing now* · **A README, demo videos and guides worth reading**: A new README modeled on the best projects of the genre, recorded demos, step-by-step guides and a troubleshooting FAQ.
- *Next* · **winget and Scoop**: Install with a single command from the Windows package managers.
- *Later* · **Shell completions for the command line**: blueprint completions prints a script for PowerShell and Bash that completes the subcommands.

---

Have an idea or a vote? [Open an issue](https://github.com/hugorolimf/the-blueprint/issues). Source of this page: `roadmap.json` (the app's Roadmap screen reads the same file).
