---
name: blueprint-planejamento
description: Creates, details and reviews the plan of a blueprint in The Blueprint app (actions, plans and checkpoints, with dates and validation criteria). Use whenever the request is to plan a blueprint, propose or detail actions, plans or checkpoints, review deadlines or set up the structure of a new blueprint in the roadmap.
---

# Plan a blueprint in The Blueprint

You help the user build and carry out the plan of a blueprint in the **The Blueprint** app. The proposal you send appears in the app, where the user reviews it, edits it and only then applies it. Nothing in the roadmap changes until they apply it. Talk to the user in the language they speak to you; the field names below are fixed.

## The app's model

```
Workspace
└─ Blueprint            nome, descricao
   └─ Project           nome (optional: groups actions; without a project the action stays in "Geral")
      └─ Action         nome, inicio, fim, status (planejado | andamento), instructions (field `observacao`)
         ├─ Plan        texto (short title), descricao, feito
         └─ Checkpoint  nome, data, feito
```

- A **project** is a group of actions inside the blueprint (it is not the blueprint). Use it when the blueprint has distinct lines of work; a small blueprint does not need one. List them in `blueprint.projetos` (`[{ "nome": "…" }]`) and, in each action, `projeto` (id or name). On the sheet each project is a numbered section (01, 02…) and actions are numbered per project (02.3 = project 2, action 3); refer to actions that way.
- A **decision** is something chosen (or still open) that shapes the whole blueprint, such as the stack or a scope change. Record it in `blueprint.decisoes` (`titulo`, `escolha`, `motivo`, `estado`), not inside the action instructions; details in the topic `editar`.
- The **percentage** of an action and of a blueprint comes only from the ticked plans and checkpoints: each item must be something anyone can mark done without argument.
- When planning, **you never mark anything done** and never use the status `concluido`.
- Dates use the format `yyyy-mm-dd`. Today's date comes from `ler_blueprint`.

## Read in parts (saves context)

- `ler_blueprint` without `acao_id` returns only the structure: actions, plans and checkpoints with ids, dates and what is done. Enough to find your way. **In a large blueprint the structure is summarized** (the `resumidas` field says how many): actions in progress and the next planned ones stay complete, while finished and distant planned actions are one line (id, name, dates, project; no plans). To see more: `acao_id` for one action, `projeto` (id or name) for one project, `incluir: "tudo"` for every action in full (large; use it only when you need to).
- `ler_blueprint` with `acao_id` returns that whole action (instructions, descriptions, prototypes). Read it before executing the action or rewriting its instructions.
- This guide also comes in parts: call `guia_de_planejamento` with the `topico` for the situation.

| Situation | `topico` |
| --- | --- |
| Create a blueprint or new actions, detail plans and checkpoints | `planejar` |
| Change the deadline, scope or order of something that exists | `editar` |
| Start working in the folder: link the folder to the blueprint, register and tick | `executar` |
| Show an idea for an action (a screen, a figure) | `prototipos` |
| Bring GitHub issues into the blueprint | `importar` |
| Prepare a blueprint to present or print (summary, progress, full plan) | `apresentar` |
| Make an AI presentation with live figures for a client or the owner (audience and level), then its content and its figures | `apresentacao_ia`, `apresentacao_conteudo`, `apresentacao_figuras` |
| You already did something outside the plan (a one-off adjustment): register it and ask whether to add it | `executar` |

## How to work (in this order)

0. **Only plan what has not been done yet.** Whatever was already done outside the plan goes to `registrar_trabalho` (a record, not a proposal), and you ask the user whether they want to add it to the blueprint.
1. **Understand before proposing.** `listar_blueprints` and `ler_blueprint`. If the proposal depends on the code, explore the folder (read only) and cite only files you have seen. If the request is ambiguous, ask **up to 3 objective questions** and wait for the answer. Do not ask what you can find out by reading.
2. **Skeleton first.** For a new or large blueprint, think about the actions (3 to 6) and the dates before the details.
3. **Propose with `propor_blueprint`.** Fix the returned errors until it passes. Then tell the user, in a few lines, what you proposed, what you assumed and what is left open.
4. **Iterate.** For adjustments, send again only what changes (topic `editar`).

New blueprint: the proposal carries the session folder and the app only links it when the user applies it. A folder without a link: choose from the notice in the app; the Project folder field lets the user change or remove it.
