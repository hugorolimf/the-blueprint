# AGENTS.md — guide for agents working in this repository

[Português](docs/pt-BR/AGENTS.md)

The Blueprint (beta), a local-first roadmap in Rust (`blueprint.exe`) with a web interface. Fixed vocabulary: **Workspace → Blueprint → Project (optional) → Action → Plan / Checkpoint** (no "phase", "task", "milestone"; **the only exception, requested by the user on 2026-10-23:** the on-screen notice that the agent completed a plan is called "Tarefa concluída" ("Tarefa não concluída") in Portuguese and "Task completed" in English, because "plan" is also the proposal the agent sends and the notice was confusing; "project" is only the group of actions inside the blueprint, never the name of the blueprint). The code, the database and the MCP field names are in Portuguese; the interface and the documentation are English first.

## Map

| Where | What |
| --- | --- |
| `src/dados/` | the only layer that talks to SQLite (migrations in `src/dados/migracoes/NNN_*.sql`; `formato.rs` maps the JSON to tables) |
| `src/blueprint/` | validation of the agent's proposals (`validar_proposta`), deadlines, summaries for `ler_blueprint` |
| `src/mcp/` | the queue (read from memory, saved in the database), MCP actions, agent activity, SSE, `ferramentas.json` and `instrucoes.txt` (the source of truth of the MCP, written in English) |
| `src/http/` | the synchronous HTTP server (`tiny_http`), pure routes, static files with ETag and gzip, the versioned `/api/dados` |
| `src/janela/` | the WebView2 window, tray and single instance (Windows only) |
| `web/`, `blueprint.html` | the interface: CSS in `web/css`, classic scripts in `web/js` in the order listed in `blueprint.html`, language catalogs in `web/i18n` |
| `kit/`, `emblemas.mjs`, `agentes-figura.mjs`, `agentes-claude.mjs` | the isometric kit (Anatomy) and figures; they run in the browser |
| `skills/blueprint-planejamento/` | the guide the agent reads through the MCP (`guia_de_planejamento`), in English |
| `scripts/` | end-to-end checks (HTTP contract, Edge e2e, mutations, measurements, installer, the video recorder) |
| `roadmap.json` | the single source of the public roadmap (generates `ROADMAP.md` and the app's Roadmap screen) |

## Commands

```
cargo test                        # Rust tests (one tests.rs per module)
cargo build --release             # target/release/blueprint.exe
node emblemas.test.mjs && node agentes-figura.test.mjs && node agentes-claude.test.mjs && node scripts/migrar-vocabulario.test.mjs
node i18n.test.mjs && node scripts/i18n-completude.mjs
node scripts/contrato-http.mjs    # the API against referencias/api-snapshots.json
node scripts/capturas.mjs --comparar
```

## Rules

- Run tests and builds only when they can finish in under one minute; stop any that reach the limit and report what was not verified.

- Files of at most 350 lines (ceiling 500 with the reason at the top). One file, one responsibility.
- Few dependencies; no framework or bundler in the interface.
- **No emojis. Figures are drawn with Anatomy. The app's theme applies everywhere** (screens, prototypes, videos, documents).
- Every user-visible text goes through `tr()` and has an English catalog entry (`web/i18n`); documentation is English first, with Portuguese in `docs/pt-BR/`.
- Security: `/api/mcp/*` loopback only, `Host` and `Origin` checked, every POST `application/json`; prototypes in an opaque-origin `iframe sandbox="allow-scripts"` (never `allow-same-origin`). What the agent sends is a proposal; completing requires evidence. Writes to `/api/dados` carry `If-Match`.
- Changed the MCP? Update `src/mcp/ferramentas.json`, `instrucoes.txt` (the rule and the tool names stay in the first 512 characters: Codex only reads that), the guide in `skills/` and the tests in `src/mcp/tests.rs`.
- Do not put real data in the repository: use `data.exemplo.json` or `data.exemplo.en.json`.

## Use The Blueprint while you work

If the app is running and the `blueprint` MCP is registered (`claude mcp add blueprint -- <path>\blueprint.exe --mcp`), keep the blueprint up to date: plan with `propor_blueprint`, say what you are doing with `registrar_progresso` and finish with `marcar_concluido` + evidence. The line that links a folder to the blueprint is:

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
```
