<p align="center">
  <img src="recursos/icone/icone.svg" width="120" alt="The Blueprint">
</p>

<h1 align="center">The Blueprint</h1>

<p align="center"><strong>Local-first roadmap for AI coding agents: they propose, you approve, completion needs evidence.</strong></p>

<p align="center">
  <a href="https://github.com/hugorolimf/the-blueprint/releases"><img src="https://img.shields.io/github/v/release/hugorolimf/the-blueprint?include_prereleases" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/hugorolimf/the-blueprint" alt="MIT license"></a>
  <a href="https://github.com/hugorolimf/the-blueprint/actions/workflows/test.yml"><img src="https://img.shields.io/github/actions/workflow/status/hugorolimf/the-blueprint/test.yml?label=tests" alt="Tests"></a>
</p>

<p align="center">
  Beta · Windows 10/11 · MIT · <a href="README.pt-BR.md">Português</a> · <a href="ROADMAP.md">Roadmap</a> · <a href="docs/comparison.md">How it compares</a>
</p>

![A blueprint sheet with projects, actions, plans and checkpoints](docs/img/blueprint.png)

## Why

Your coding agent (Claude Code, Codex) plans and builds fast. The cost is a plan you never reviewed and a "done" you cannot check. The Blueprint is a small desktop app that holds your whole project as one sheet, and your own agent talks to it over [MCP](https://modelcontextprotocol.io):

- **It proposes, you apply.** Everything the agent sends is a numbered proposal. Nothing changes in your data until you apply it on the *Planejar* (Plan) screen.
- **Done needs proof.** A plan is only marked done with the command or test that was run and its result, which becomes a comment on the plan.
- **You see the work.** Claude appears on the sheet, on the plan it is working on, with its subagents.
- **Small and private.** One ~2.5 MB executable, a SQLite file on your machine, no account, no telemetry.

## Quick start (about 5 minutes, Windows)

1. Download `Blueprint-<version>-instalador.exe` from [Releases](https://github.com/hugorolimf/the-blueprint/releases) and run it. It installs for your user only (no administrator) and, when you keep the *connect* task checked, adds the Claude Code plugin and registers the MCP in Codex.
2. Open **The Blueprint** from the Start menu. A window and a tray icon appear.
3. Open Claude Code (or Codex) in your project folder as you always do.
4. Ask: *"Plan this project in The Blueprint."* The proposal arrives in the app; review it on **Planejar** and apply it.
5. Ask: *"Start the first action."* As the agent works, the Claude walks to the plan on the sheet, and each plan is checked with its evidence.

Not on Windows, or no installer? Build it with [Rust](https://rustup.rs): `cargo build --release`, then `blueprint serve` runs the same app as a local web server at `http://127.0.0.1:3000` ([details](docs/commands.md)).

## How it works

```
Workspace
└─ Blueprint            the scope
   └─ Project           groups actions (optional)
      └─ Action         start, end, status, instructions
         ├─ Plan        title, description, hours, done + evidence
         └─ Checkpoint  name, date, done
```

The agent reads the structure, proposes changes, reports progress and marks plans done through MCP tools (`listar_blueprints`, `ler_blueprint`, `propor_blueprint`, `registrar_progresso`, `marcar_concluido` and more; [full list](docs/mcp.md)). You start an action yourself, with a confirmed click or the `blueprint claude` command, so the app never sends work to an agent on its own. Plans are sized for AI pace (0.5 to 4 hours) and the app measures how long each one really took. [The model in detail](docs/model.md).

## Documentation

The guides are English first; each has a Portuguese version in [`docs/pt-BR/`](docs/pt-BR/).

| | |
| --- | --- |
| [Install from source, performance](docs/install.md) | installer details, data folder, measured numbers |
| [Claude Code plugin](docs/claude-code.md) | skill, agent, hooks, status line, `/blueprint:status`, the terminal panel |
| [MCP and other agents](docs/mcp.md) | tools, Codex, offline queue |
| [Updates](docs/updates.md) | background download, SHA-256 check, install only on your click |
| [Server and commands](docs/commands.md) | `blueprint serve`, environment variables, every subcommand |
| [Development](docs/development.md) | `cargo dev`, tests, end-to-end checks |
| [How it compares](docs/comparison.md) | Backlog.md, Task Master, Vibe Kanban, Plane |

## FAQ

**Does the app run my agent or spend tokens by itself?** No. You start an action, and only then Claude Code runs, as usual.

**Does it send my data anywhere?** No account and no telemetry. The only network call is a `GET` to GitHub Releases to look for updates, which you can turn off ([how](docs/updates.md)).

**Where is my data?** In `%APPDATA%\Blueprint\blueprint.db`. `blueprint exportar` writes it to JSON.

**Windows shows a SmartScreen or Defender warning.** The installer is not code-signed yet, so that can happen on the first run. Compare its SHA-256 with the `SHA256SUMS` file in the same release; signed releases are on the [roadmap](ROADMAP.md).

**macOS and Linux?** Not yet; see the [roadmap](ROADMAP.md). The server mode works anywhere you can build it.

**Codex?** The MCP works today; a native integration (hooks, skills, status) is planned.

## Roadmap, contributing, security

- [ROADMAP.md](ROADMAP.md): what exists, what is being built, what comes next.
- [CONTRIBUTING.md](CONTRIBUTING.md): ideas and good first issues are welcome in [Issues](https://github.com/hugorolimf/the-blueprint/issues).
- [SECURITY.md](SECURITY.md): the MCP routes answer only on loopback; prototypes run in an opaque-origin sandbox.

## License

[MIT](LICENSE). The isometric kit descends from [Anatomy](https://github.com/wheresryan22/anatomy) (MIT); see [NOTICE.md](NOTICE.md).
