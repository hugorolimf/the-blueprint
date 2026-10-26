# Plan a blueprint: actions, plans and checkpoints

## Actions

- `nome`: short and specific (e.g. "Real terminal for the agent"), not "Action 1".
- `inicio` and `fim`: a sequence with no gaps or conflicts. New actions start **today or later**, unless the user asks for past dates.
- `status`: `planejado` or `andamento` (andamento only if the work has already started).
- `observacao` (in the app: **Action instructions**): the **complete outline of the action**. It can be long (up to 20000 characters) and should carry: goal, deliverable, step by step in order, **dependencies** ("depends on action X"), decisions and assumptions, risks and the definition of done. The plans are the tickable items; the instructions tell how and why.
- 3 to 8 plans and 1 to 3 checkpoints per action.
- `projeto` (optional): the action's group in the blueprint, by name or id; declare them in `blueprint.projetos` (`[{ "nome": "Interface" }]`, also valid on an existing blueprint). Only with distinct lines of work (2 to 5); without the field the action stays in "Geral" (a reserved name). Details: topic `editar`.
- **Decisions** (stack, scope, rules chosen or still open): `blueprint.decisoes` = `[{ "titulo", "escolha", "motivo", "estado": "aberta|decidida" }]`, valid on any blueprint (existing ones come in `ler_blueprint`). Put them here, not as text in an action's `observacao`. With `id` it updates (what you omit stays), without it is new; `decidida` needs `escolha`. Only decisions is valid (`acoes: []`).

## Plans

**Title (`texto`)**: up to **45 characters**, starting with a **verb in the infinitive form**, actionable.

| Good | Bad |
| --- | --- |
| Validate the folder and the Git root | Folder |
| Create a route to stop the agent | Do the server part |
| Cover errors with tests | Tests and other assorted things |

**Description**: required on a new plan, concrete, with three parts:

```
Scope: what goes in (and what does not).
Expected result: what will exist when it is done.
Validation: how to prove it is done — preferably a command or test ("node x.test.mjs passes"), or an observable behavior.
```

**Size**: fits in **one work session** (a few hours). If it needs "and then...", split it; if it is too trivial, merge it.

**Order**: in the order they must be done; dependencies between actions go in the action's instructions.

Cite files or modules **only** if you have seen them in the folder. Never invent paths, APIs or business facts: when information is missing, propose something reasonable and say it is an assumption.

## Checkpoints

- **Verifiable checkpoints**: a deliverable, a decision or a demonstration ("End-to-end flow validated"), not an activity ("Work on the flow").
- `data` within the action's period, usually near the end or near an important risk.
- A short name that someone can answer with yes or no.

## Deadlines at AI pace

The one executing is the AI: short deadlines, in sequence, starting today, with no gaps of more than 2 days.

- **Plan:** 0.5 to 4 h of agent work (above 8 h, split it). Give `horas` (0.5 to 16) on every new plan.
- **Action:** 1 to 3 calendar days. More only with a written reason in the instructions (waiting on a person, CI, publishing, a third party).
- **The user's time** (reviewing, approving, testing on a real machine) becomes a dated checkpoint, never hidden slack in the deadline.
- Good: a 2-day action with 4 plans of 1 to 3 h. Bad: a 3-week action with 5 plans of 1 day.

## Level of detail

**Lean** (few plans, one-line descriptions), **standard** (as described above) or **detailed** (more small plans, command validation on all of them, risks in the instructions). Without a request, use standard.

## New blueprint

Without `blueprint_id`, also send `blueprint: { nome, descricao }` (a short name, a description of one or two sentences). Propose the name; do not require the user to choose.

## Before sending, check

- [ ] Titles with a verb in the infinitive and up to 45 characters; none repeated within the action.
- [ ] Every new plan has a description (scope, result, validation).
- [ ] Every new plan has `horas`; actions of up to 3 days (or with a reason) and no gaps between them.
- [ ] Dates `yyyy-mm-dd`, `fim` after `inicio`, checkpoints inside the action, no past dates without a request.
- [ ] Nothing marked done; ids only the ones you read in `ler_blueprint`.
- [ ] Invented facts? If so, turn them into declared assumptions.
