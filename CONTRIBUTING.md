# How to contribute

[Português](docs/pt-BR/CONTRIBUTING.md)

Thank you! The project is small on purpose. Before a big change, open an issue so we can talk about it. What is planned and what is being built is in the [ROADMAP](ROADMAP.md).

## Vocabulary (important)

The model is **Workspace → Blueprint → Project (optional) → Action → Plan / Checkpoint**. "Project" is the group of actions inside a blueprint (the fields `projetos`, `projetoId` and `projeto`); do not use it as the name of the blueprint. Do not use "phase", "task" or "milestone" in the code, the interface, the MCP or the documents (they exist only in the migration script and in the SQL migration `002_vocabulario.sql`). A "plan" is the tickable item; the agent's whole submission is a **proposal** (`propor_blueprint`). The code, the database and the MCP field names are in Portuguese (`acao`, `plano`, `feito`…); the interface and the documentation are in English first.

## Code rules

- **Small files:** at most 350 lines; the absolute ceiling is 500 and only with the reason written at the top of the file. One file, one responsibility; no generic `utils`.
- **One test per module** (`tests.rs` or `tests/` next to the Rust module). Changed a security or data rule? Write the test that fails without the change.
- **Few dependencies.** Rust: only crates worth their weight (no `tokio` if a synchronous server does the job). Browser: no framework and no bundler; classic scripts loaded in the order of `blueprint.html` (`inicio.js` last).
- **The isometric kit** (`kit/`, `emblemas.mjs`) runs in the browser and does not go to Rust; changed the drawing? Run `node emblemas.test.mjs` and `node agentes-figura.test.mjs` and only re-record the references on purpose.
- **Prototypes** never get `allow-same-origin`. The `/api/mcp/*` routes stay loopback-only.
- `blueprint.html` uses CRLF line endings; edit it with care, without converting the whole file.
- **No emojis** anywhere in the project. If you need a figure, draw it with Anatomy. Everything visual uses the app's own theme (`web/css/tokens.css`).
- **Every text the user reads goes through `tr()`** and needs an English entry in `web/i18n/` (see [Develop and test](docs/development.md#languages)); `node scripts/i18n-completude.mjs` fails otherwise.
- **Documentation is written in English first**; the Portuguese version lives in `docs/pt-BR/` with the same file name (rule and glossary in [Documentation languages](docs/development.md#documentation-languages)).

## Develop

`cargo dev` opens the app in dev mode (a native window, with your data; close the app first): saving `web/` or `kit/` reloads the window; saving `src/` rebuilds and reopens (needs `cargo install cargo-watch`, once; without it, `cargo run -- dev`). `node dev.mjs` is the browser alternative, with a sample database in `.dev/`. Neither runs tests.

## Run the tests

```
cargo test
node emblemas.test.mjs
node agentes-figura.test.mjs
node i18n.test.mjs
node scripts/i18n-completude.mjs
node scripts/migrar-vocabulario.test.mjs
cargo build --release
node scripts/contrato-http.mjs       # the API's answers against referencias/api-snapshots.json
node scripts/fumaca-rust.mjs
node scripts/idioma-e2e.mjs          # needs Microsoft Edge
node scripts/roadmap-e2e.mjs         # needs Microsoft Edge
node scripts/capturas.mjs --comparar # the 7 screens, pixel by pixel (hash)
```

Changed an API answer or the look of a screen on purpose? Re-record the reference (`contrato-http.mjs --gravar`, `capturas.mjs --gravar`), check the difference and say in the pull request what changed and why.

## Pull requests

1. One change per PR, with the why in the description.
2. `cargo test` and the `node` tests green (CI runs the same ones on Linux and Windows).
3. Update the documentation and the agent guide (`skills/blueprint-planejamento/`) if the MCP changed.

Good first issues carry the **good first issue** label.
