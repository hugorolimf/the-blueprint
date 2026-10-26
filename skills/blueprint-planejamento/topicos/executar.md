# Execute: link the folder and keep The Blueprint up to date

## Link the folder to the blueprint

**In Claude Code with the plugin, the app stores the folder → blueprint link:** the session already starts with the ids and the current action. If it says the folder is not linked, call `listar_blueprints`, confirm the blueprint with the user and call `vincular_pasta`; do not write anything to CLAUDE.md.

Without the plugin (Codex and others), the link lives in the folder's `CLAUDE.md` (or `AGENTS.md`):

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
Keep this blueprint always up to date: plans and changes through propor_blueprint; when starting a plan, registrar_progresso; when finishing, marcar_concluido with the evidence; if it did not work, marcar_nao_concluido; if you get stuck, reportar_bloqueio.
```

- If the line is already there, use those ids and do not ask.
- If it is not, find out with `listar_blueprints`, confirm with the user and write the two lines, without touching the rest of the file.
- A new blueprint only has a `blueprint_id` after the user applies the proposal: write the line at that moment.

## Keep it up to date (without the user asking)

- **Before executing an action:** `ler_blueprint` with `acao_id` to get the instructions and the plan descriptions.
- **Starting a plan:** `registrar_progresso` with one sentence about what you are going to do, plus `acao_id` and `plano_id`: Claude appears next to that plan on the sheet.
- **Changed file or step:** `registrar_progresso` again, with `estado` and `onde` updated (see below).
- **Finishing:** `marcar_concluido` with real evidence (the command or test that was run and its result). One call per plan, as soon as it finishes.
- **Checkpoint met:** `marcar_checkpoint_feito` with the evidence; it is usually the last item of the action.
- **It did not work:** `marcar_nao_concluido` (or `marcar_checkpoint_nao_feito`) with the reason. **Stuck:** `reportar_bloqueio` with what is missing.
- **Work outside the blueprint, not done yet:** propose it with `propor_blueprint` before executing. It only gets a `plano_id` when the user applies it.
- **Work outside the blueprint, already done** (a one-off adjustment asked for in the conversation): `registrar_trabalho` with the done plans and checkpoints and the evidence (without `feito` on what is still pending). Then ask the user whether they want to add it to the blueprint. It only enters if they click "Add to blueprint" (Plan screen).
- **The blueprint changed** (deadline, scope, order): `propor_blueprint` with only what changes (topic `editar`). Do not leave the change only in the terminal.

## Show up on the sheet (optional fields)

In `registrar_progresso` and `reportar_bloqueio`, optional fields (`texto` alone also works; it shows up as "agent"):

| Field | Values |
| --- | --- |
| `agente` | your name, stable during the work (up to 40 characters, no line breaks); **subagents use their own name** |
| `papel` | `explorador` (reads), `construtor` (writes), `revisor` (tests) |
| `estado` | `lendo`, `escrevendo`, `testando`, `aguardando`, `bloqueado`, `concluido` |
| `onde` | up to 5 paths (files or folders, up to 120 characters each) you are working on now |
| `percentual` | 0 to 100, how much of the plan is already done |
| `acao_id`, `plano_id` | the plan you are on (the sheet takes the user to it) |

- Report when you start and **every time you change file or state**; with no signal for 10 minutes it rests in the blueprint header.
- The server refuses values outside these (an invented state or role, an `onde` that is too large, a name with control characters) and says why: fix it and send again.
- A subagent without access to the MCP: the main agent reports for it, with the subagent's name in `agente`.
- Never put secrets, passwords or personal paths in `onde`: the sheet shows the text as it came.

Monitor/channel notices are informational: check `ler_blueprint`. `BLUEPRINT_CHANNEL=1` swaps the monitor for the channel.

**App closed:** the writes (register, complete, propose) are stored and enter when it opens; do not repeat them. Reading requires the app.
