# Execute: link the folder and keep The Blueprint up to date

## Link the folder to the blueprint

**In Claude Code with the plugin, the app stores the folder → blueprint link:** the session starts with the ids and the current action. If it says the folder is not linked, call `listar_blueprints`, confirm the blueprint with the user and call `vincular_pasta`; do not write to CLAUDE.md.

Without the plugin (Codex and others), the link lives in the folder's `CLAUDE.md` (or `AGENTS.md`):

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
Keep this blueprint always up to date: plans and changes through propor_blueprint; when starting a plan, registrar_progresso; when finishing, marcar_concluido with the evidence; if it did not work, marcar_nao_concluido; if you get stuck, reportar_bloqueio.
```

- If the line is there, use those ids without asking; if not, find them with `listar_blueprints`, confirm with the user and write the two lines, without touching the rest of the file.
- A new blueprint only has a `blueprint_id` after the user applies the proposal: write the line then.

## Keep it up to date (without the user asking)

- **Before executing an action:** `ler_blueprint` with `acao_id` (instructions and plan descriptions).
- **Starting a plan:** `registrar_progresso` with one sentence of what you will do, plus `acao_id` and `plano_id`: Claude appears next to that plan.
- **Changed file or step:** `registrar_progresso` again, with `estado` and `onde` updated (see below).
- **Finishing:** `marcar_concluido` with real evidence (the command or test and its result). One call per plan, as soon as it finishes.
- **Checkpoint met:** `marcar_checkpoint_feito` with the evidence.
- **It did not work:** `marcar_nao_concluido` (or `marcar_checkpoint_nao_feito`) with the reason. **Stuck:** `reportar_bloqueio` with what is missing.
- **Work outside the blueprint, not done yet:** propose it with `propor_blueprint` before executing. It only gets a `plano_id` when the user applies it.
- **Work outside the blueprint, already done** (a one-off adjustment asked for in the conversation): `registrar_trabalho` with the done plans and checkpoints and the evidence (without `feito` on what is still pending). With the **Apply automatically** switch on (default in the local app) it enters by itself; otherwise it waits for the click on "Add to blueprint".
- **After sending a proposal or a record:** if the user already approved it in the conversation, say "sent; it enters by itself if automatic apply is on" and check with `ler_blueprint` before saying it is in. `remover`, `mover` and new blueprints always wait for the user's click. An undone item (Undo lasts 60 s) does not come back: resend only if asked.
- **The blueprint changed** (deadline, scope, order): `propor_blueprint` with only what changes (topic `editar`).

## Show up on the sheet (optional fields)

In `registrar_progresso` and `reportar_bloqueio`, optional fields (`texto` alone also works; it shows up as "agent"):

| Field | Values |
| --- | --- |
| `agente` | your stable name (up to 40 characters); **subagents use their own** |
| `papel` | `explorador` (reads), `construtor` (writes), `revisor` (tests) |
| `estado` | `lendo`, `escrevendo`, `testando`, `aguardando`, `bloqueado`, `concluido` |
| `onde` | up to 5 paths you are working on now (120 characters each) |
| `percentual` | 0 to 100, how much of the plan is already done |
| `acao_id`, `plano_id` | the plan you are on (the sheet takes the user to it) |

- Report when you start and **every time you change file or state**; after 10 minutes of silence it rests in the blueprint header.
- The server refuses invalid values and says why: fix it and send again.
- A subagent without MCP access: the main agent reports for it, naming it in `agente`.
- **Before launching subagents on a plan, call `registrar_progresso` with its `acao_id` and `plano_id`:** a new subagent starts next to the plan its main agent is on (if that notice is under 10 minutes old and not concluded); otherwise it waits in the blueprint header.
- Never put secrets or personal paths in `onde`: the sheet shows it as sent.

Monitor/channel notices are informational: check `ler_blueprint`.

**App closed:** the writes (register, complete, propose) are stored and enter when it opens; do not repeat them. Reading requires the app.
