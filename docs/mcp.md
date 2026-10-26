# Connect other agents (MCP)

[Back to the README](../README.md) · [Português](pt-BR/mcp.md)

In **Codex** (and in Claude Code without the plugin), connect only the MCP. The installer and `blueprint registrar` already do this for any Codex they find on the PATH. By hand, with the app open (or `blueprint serve` running):

```
claude mcp add blueprint -- <path>\blueprint.exe --mcp
codex mcp add blueprint -- <path>\blueprint.exe --mcp
```

If the app is not at `http://127.0.0.1:3000`, pass `BLUEPRINT_URL` (`claude mcp add blueprint -e BLUEPRINT_URL=http://127.0.0.1:3917 -- …`). The **Plan → How to connect** screen shows the exact command. Without the Claude Code plugin, the agent does not know on its own which blueprint the folder belongs to: write this line in the folder's `CLAUDE.md` or `AGENTS.md`: `Blueprint: workspace_id `…`, blueprint_id `…` (MCP `blueprint`).` (the agent finds the ids with `listar_blueprints`).

**With the app closed** the agent keeps registering: `registrar_progresso`, `marcar_concluido`, `marcar_checkpoint_feito`, `propor_blueprint`, `anexar_prototipo` and `vincular_pasta` are kept in files under `%APPDATA%\Blueprint\pendentes\` and, when the app opens, they enter the queue, in order, with the same validation (the screen applies them when you open the workspace). What the app refuses on opening goes to `pendentes\recusados\` with the reason. The read tools (`ler_blueprint`, `listar_blueprints`) need the app to be open.

Tools (their names are fixed; their descriptions and the planning guide are written in English for the agent):

| Tool | What it is for |
| --- | --- |
| `listar_blueprints` | workspaces and blueprints, with ids |
| `guia_de_planejamento` | the guide in parts (`planejar`, `editar`, `executar`, `prototipos`) |
| `ler_blueprint` | only the structure, or a whole action (`acao_id`) |
| `propor_blueprint` | proposes actions, plans and checkpoints (partial ones complete what exists; deleting is a request you approve) |
| `registrar_progresso`, `reportar_bloqueio` | what it is doing now; it shows on the sheet as Claude (one figure per agent, on the plan it works on) |
| `marcar_concluido`, `marcar_nao_concluido` | a plan completed (with evidence) or not |
| `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` | the same for checkpoints |
| `registrar_trabalho` | records what was already done outside the plan; it only enters the blueprint if you click Add |
| `anexar_prototipo` | a self-contained HTML prototype of an action, seen in an isolated viewer |
| `vincular_pasta` | links the session's folder to a blueprint (the app keeps the link) |

The guide the agent reads is in [`skills/blueprint-planejamento/`](../skills/blueprint-planejamento/SKILL.md).
