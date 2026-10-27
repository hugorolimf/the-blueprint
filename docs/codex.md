# Codex and other MCP agents

[Back to the README](../README.md) · [Português](pt-BR/codex.md)

Claude Code gets a plugin ([guide](claude-code.md)). Codex, and any other agent that speaks MCP, gets only the MCP server: the same tools, without hooks, skill or status line.

Research and design of the native Codex integration (hooks, plugin, parity table): [codex-parity.md](codex-parity.md).

## Codex

**Connect.** The installer (the *connect* task) and `blueprint registrar` already run this for the `codex` they find on the PATH. By hand, with the app open:

```
codex mcp add blueprint -- <path>\blueprint.exe --mcp
codex mcp list
```

`codex mcp list` shows `blueprint`. `blueprint registrar --remover` (or `codex mcp remove blueprint`) undoes it. If the app is not at `http://127.0.0.1:3000`, add `--env BLUEPRINT_URL=http://127.0.0.1:<port>` before the `--`.

**Link the folder.** Codex has no session hook, so it does not learn the folder's blueprint by itself. Write this line in the folder's `AGENTS.md` (the agent finds the ids with `listar_blueprints`):

```
Blueprint: workspace_id <id>, blueprint_id <id> (MCP blueprint).
```

**What Codex sees and what it does not:**

| | Claude Code with the plugin | Codex |
| --- | --- | --- |
| MCP tools (propose, progress, mark done with evidence) | yes | yes |
| Blueprint and current action injected when the session opens | yes (hook) | no: it reads the `AGENTS.md` line |
| Figure on the sheet follows every file edit | yes (hook) | only when it calls `registrar_progresso` |
| Planning guide | skill, loaded by itself | `guia_de_planejamento` tool |
| Status line, `/blueprint:status` | yes | no |

The rule to keep the blueprint up to date and the tool names sit in the first 512 characters of the MCP instructions, which is what Codex reads.

## Other MCP agents (not tested)

Any agent that runs a stdio MCP server can use the same command: `<path>\blueprint.exe --mcp`, plus the `Blueprint:` line in the instructions file that agent reads. Only Claude Code and Codex are tested; tell us in [Issues](https://github.com/hugorolimf/the-blueprint/issues) if another one works.

Tools, offline queue and language: [MCP and other agents](mcp.md).
