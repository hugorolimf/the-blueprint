# Develop and test

[Back to the README](../README.md) · [Português](pt-BR/development.md)

```
cargo dev                                    # the app in dev mode, with automatic reload (needs cargo-watch)
```

`cargo dev` opens the app's window with your data (`%APPDATA%\Blueprint`), so close the app first. Saved `web/`, `kit/` or `blueprint.html`: the window reloads (CSS swaps without losing the screen; F12 opens the dev tools). Saved `src/`, `skills/` or `Cargo.toml`: `cargo-watch` (`cargo install cargo-watch`, once) rebuilds and reopens. Without it: `cargo run -- dev`. To work in the browser with sample data, without Node in the window: `node dev.mjs` (http://127.0.0.1:3917; options `--porta N`, `--limpar`, `--abrir`).

```
cargo test                                   # more than 230 tests: data, validation, MCP, HTTP
node emblemas.test.mjs                       # marks and figures (the Anatomy kit)
node agentes-figura.test.mjs
node i18n.test.mjs                           # the language layer
node scripts/i18n-completude.mjs             # every key used in the code has a translation
node scripts/migrar-vocabulario.test.mjs
```

End-to-end checks (they need `target/release/blueprint.exe` and, for the screen ones, Microsoft Edge): `scripts/contrato-http.mjs`, `fumaca-rust.mjs`, `robo-e2e.mjs`, `painel-agentes-e2e.mjs`, `idioma-e2e.mjs`, `roadmap-e2e.mjs`, `sincronia-e2e.mjs`, `capturas.mjs --comparar`, `mutacoes.mjs` (it breaks the security rules on purpose and checks that some test fails) and `testar-instalador.mjs`. See [CONTRIBUTING.md](../CONTRIBUTING.md).

## Languages

The interface follows the Windows language and falls back to English. In the code, every user-visible text goes through `tr("texto em português")`; the Portuguese source text is the key, so text without a translation falls back to Portuguese and never breaks. Catalogs live in `web/i18n/en-<area>.js`. `node scripts/i18n-varrer.mjs` opens every screen in English and lists what is still in Portuguese; `node scripts/i18n-inventario.mjs` lists candidates to convert.

## Documentation languages

The documentation is written in English first, with Brazilian Portuguese as the second language:

- The file with the plain name is the English one (`docs/mcp.md`); its translation lives in `docs/pt-BR/` with the same name (`docs/pt-BR/mcp.md`). `CONTRIBUTING.md`, `SECURITY.md` and `AGENTS.md` stay at the root, with their translations in `docs/pt-BR/`. The READMEs and the roadmap use the `.pt-BR.md` suffix next to the English file (`README.pt-BR.md`, `ROADMAP.pt-BR.md`).
- Each file opens with a language selector below the title: `[Português](pt-BR/<name>.md)` in the English file, `[English](../<name>.md)` in the Portuguese one.
- Both versions keep the same headings, in the same order and at the same levels; the text can differ in wording, never in content.
- File names are in English. When a file is renamed, the old name stays as a short page that links to the new one, so outside links keep working.
- Maintainer-only notes (`docs/interno/`) follow the same rule. A few older technical references still say "Portuguese only for now" at the top; they are translated as they are touched.

`node scripts/conferir-links.mjs` (it runs in CI) checks every relative link and, for every file in `docs/pt-BR/`, that the English file exists, that each one links to the other and that the headings match.

Glossary (use these names in English; the code, the database and the MCP fields keep the Portuguese names):

| English | Portuguese | In the code and the MCP |
| --- | --- | --- |
| Workspace | Workspace | `workspace_id`, `workspaces[]` |
| Blueprint | Blueprint | `blueprint_id`, `blueprints[]` |
| Project | Projeto | `projetos[]`, `projetoId`, `projeto` |
| Action | Ação | `acao_id`, `acoes[]` |
| Plan | Plano | `plano_id`, `planos[]` |
| Checkpoint | Checkpoint | `checkpoint_id`, `checkpoints[]` |
| Proposal | Proposta | `propor_blueprint` |
| Record (work done outside the plan) | Registro | `registrar_trabalho` |
| Evidence | Evidência | `evidencia` |

Do not use "phase", "task" or "milestone" for these ideas. The one exception is the agent notice on screen, which says "Task completed" for a finished plan.
