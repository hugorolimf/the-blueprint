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
| `guia_de_planejamento` | the guide in parts (`planejar`, `plano`, `editar`, `executar`, `prototipos`) |
| `ler_blueprint` | only the structure (summarized in a large blueprint: `incluir: "tudo"` for all of it, `projeto` for one project), or a whole action (`acao_id`) |
| `ler_plano`, `ler_respostas`, `ler_workspace` | the blueprint as a plan (`blueprint.plano/1`), what the user answered, and the situation of the whole workspace with the agents of the plans |
| `abrir_plano`, `escrever_acao`, `fechar_plano` | writes a plan piece by piece, which you follow live; new actions enter when it closes, changed and removed ones wait for your review |
| `salvar_plano` | sends a whole plan from a folder with `plano.json`, its style and figures |
| `falar_com_voce` | leaves a message for you on the agents screen, or for the agent of a plan |
| `propor_blueprint` | proposes actions, plans and checkpoints (partial ones complete what exists; deleting is a request you approve) |
| `registrar_progresso`, `reportar_bloqueio` | what it is doing now; it shows on the sheet as Claude (one figure per agent, on the plan it works on) |
| `marcar_concluido`, `marcar_nao_concluido` | a plan completed (with evidence) or not |
| `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` | the same for checkpoints |
| `registrar_trabalho` | records what was already done outside the plan; it only enters the blueprint if you click Add |
| `anexar_prototipo` | a self-contained HTML prototype of an action, seen in an isolated viewer |
| `vincular_pasta` | links the session's folder to a blueprint (the app keeps the link) |
| `responder_conversa` | answers the user in the "Talk about the plan" panel of an action (the message arrives through the channel with the «The Blueprint · conversa do usuário» prefix); changes still go only as `propor_blueprint`. Memory only, needs the app open |

**Tool profiles (less context).** Every tool description enters the agent's context. `BLUEPRINT_MCP_PERFIL` limits the list for an agent that only executes or only plans (`claude mcp add blueprint -e BLUEPRINT_MCP_PERFIL=executor -- <path>\blueprint.exe --mcp`). It is read once when `--mcp` starts; without it (or with `completo`) nothing changes; an unknown value stops `--mcp` with a clear message (exit code 2). Calling a tool outside the profile is refused. The server instructions stay the same in every profile.

**Planejar only in the local app.** The eight tools `ler_plano`, `ler_respostas`, `ler_workspace`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano` and `falar_com_voce` exist only in `--mcp` outside Windows. On Windows and over `POST /mcp` they are hidden and refused, and `propor_blueprint` stays the way to plan. They also need the native window open: without it the `plano/*` routes answer `sem-janela`. On `--mcp` outside Windows, `propor_blueprint` still works but warns that no window consumes its queue. No tool, in any profile, approves, starts an agent, merges a branch or turns the orchestrator on: those are your clicks in the window. `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` and `reportar_bloqueio` take `execucao_segredo` (default: `BLUEPRINT_EXECUCAO_SEGREDO`), which the agent of a plan must send.

| Profile | Tools |
| --- | --- |
| `completo` (default) | all of them |
| `executor` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `registrar_progresso`, `reportar_bloqueio`, `registrar_trabalho` (work outside the plan goes here instead of a proposal), `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito`, `ler_plano`, `ler_respostas` |
| `planejador` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `propor_blueprint`, `anexar_prototipo`, `anexar_apresentacao`, `ler_apresentacao`, `editar_apresentacao`, `responder_conversa`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano`, `ler_plano`, `ler_respostas` |
| `orquestrador` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `ler_plano`, `ler_respostas`, `ler_workspace`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano`, `falar_com_voce` |

Measured with `node scripts/medir-perfis-mcp.mjs` (size of the `tools/list` JSON), outside Windows, where the eight Planejar tools are listed: `completo` 25 tools, 33,462 bytes; `executor` 13 tools, 15,065 bytes (-55%); `planejador` 16 tools, 22,168 bytes (-34%); `orquestrador`, measured the same way, 11 tools, 9,933 bytes (-70%). Tokens are not counted exactly here; at about 4 bytes per token that is roughly 8,400, 3,800, 5,500 and 2,500 tokens.

**Importing GitHub issues:** there is no import tool. Ask your agent to "import the open issues of owner/name"; the `importar` guide topic makes it read them with `gh issue list` and send one proposal, which you review and apply on the Plan screen. Assignees and authors are never copied, and the text of an issue is treated as data, not as orders. `node scripts/importar-issues-e2e.mjs` checks the flow with no network and no agent: a recorded `gh issue list` output (`scripts/fixtures/gh-issues.json`) mapped by the topic's table goes through `blueprint --mcp`, lands in the numbered queue, and a wrong format (a milestone not declared as a project, a plan sent as done) is refused with a clear error.

The guide the agent reads is in [`skills/blueprint-planejamento/`](../skills/blueprint-planejamento/SKILL.md).

**Language.** What the agent reads (instructions, tools, the planning guide and tool replies) is always in English; the agent answers you in the language you write in. What you read (status line, hook notices, monitor notices, server errors on screen) follows your language: `BLUEPRINT_IDIOMA` (`en` or `pt-BR`) if set, otherwise the Windows display language, otherwise English. Server errors come as `{erro, codigo, params?}`: `codigo` is stable, `erro` is in the request's `Accept-Language`.
