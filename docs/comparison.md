# The Blueprint and its neighbors

[Português](pt-BR/comparison.md)

An honest comparison, with what could be verified on the projects' own pages on 25 Oct 2026 (sources and numbers in [docs/interno/research.md](interno/research.md)). Where The Blueprint loses, it says so.

| | **The Blueprint** | [Backlog.md](https://github.com/MrLesk/Backlog.md) | [Task Master](https://github.com/eyaltoledano/claude-task-master) | [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | [Plane](https://github.com/makeplane/plane) |
| --- | --- | --- | --- | --- | --- |
| In one sentence | a local-first roadmap your agent reads and updates over MCP, on a blueprint-style sheet | a Markdown task manager and kanban board for any Git repository, for humans and agents | a task system for AI-driven development, to use in Cursor, Windsurf and others | a kanban board to coordinate coding agents | an open source alternative to Jira and Linear |
| License | MIT | MIT | "Other" (read the LICENSE) | Apache-2.0 | AGPL-3.0 |
| Stars | 3 | ~7k | ~28k | ~28k | ~61k |
| Latest release | v0.3.0 | v1.55.3 (23 Oct 2026) | 0.43.1 (31 Mar 2026) | v0.1.44 (24 Apr 2026) | v1.4.3-rc2 (23 Oct 2026) |
| Where the data lives | a SQLite database on your computer | Markdown files in the repository | see the repository | see the repository | your own server or the cloud |
| Systems | Windows (window, tray, installer); the server runs on any system | npm, bun, brew, nix | npm | npx | Docker or cloud |

## What is different about The Blueprint

- **The agent proposes, you apply.** Everything the agent sends is a proposal in a numbered queue; nothing changes in your data before you apply it on the **Plan** screen.
- **Completion needs evidence.** A plan is only marked done with the command or test that was run and its result, which becomes a comment on the plan.
- **A sheet, not just a list.** Workspace → Blueprint → Project → Action → Plan and Checkpoint, with deadlines at AI pace and measured real time.
- **You see the agent working.** Claude appears on the sheet, on the plan it is working on, with its subagents.
- **Small and local.** An executable of about 2.5 MB, no telemetry and no account.

## Where the others are better

- **Maturity and community.** The Blueprint is at v0.1 with 3 stars; all the others have years of use and a community.
- **Platforms.** The Blueprint is Windows in the window; Backlog.md, Task Master and Vibe Kanban install through npm on any system.
- **Teams.** Plane has users, roles, sprints and docs ready; The Blueprint's collaborative mode is still only a design (see the [ROADMAP](../ROADMAP.md)).
- **Data in the repository.** Backlog.md keeps tasks as Markdown in your own Git, which gives history and review by pull request; The Blueprint keeps them in a local database (and exports to JSON).

## Which one to choose

- You want tasks versioned next to the code, in Markdown: **Backlog.md**.
- You want to break a PRD into tasks inside your editor: **Task Master**.
- You want a board for several agents in parallel: **Vibe Kanban** (community-maintained since April 2026) or **ORCA**.
- You want to manage a whole team: **Plane**.
- You want to control what your agent plans, approve every change and demand proof of what was done, on a sheet that shows the whole project: **The Blueprint**.
