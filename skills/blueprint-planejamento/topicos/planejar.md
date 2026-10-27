# Plan a blueprint: actions, plans and checkpoints

## Actions

- `nome`: short and specific, not "Action 1".
- `inicio` and `fim`: a sequence with no gaps or conflicts. New actions start **today or later**, unless the user asks for past dates.
- `status`: `planejado` or `andamento` (andamento only if the work has already started).
- `observacao` (in the app: **Action instructions**): the **complete outline of the action** (up to 20000 characters): goal, deliverable, ordered steps, **dependencies**, decisions and assumptions, risks and the definition of done. The plans are the tickable items; the instructions tell how and why.
- As many plans as the action needs, and 1 to 3 checkpoints per action.
- `projeto` (optional): the action's group, by name or id; declare them in `blueprint.projetos` (`[{ "nome": "Interface" }]`, also valid on an existing blueprint). Only for 2 to 5 distinct lines of work; without it the action stays in "Geral" (reserved). Details: topic `editar`.
- **Decisions** (stack, scope, rules, chosen or open): `blueprint.decisoes` = `[{ "titulo", "escolha", "motivo", "estado": "aberta|decidida" }]`, valid on any blueprint (read them in `ler_blueprint`); not as text in `observacao`. With `id` it updates (omitted fields stay), without it is new; `decidida` needs `escolha`; `acoes: []` is valid with only decisions.

## Plans

**Title (`texto`)**: actionable, starting with a **verb in the infinitive form**.

**Description**: required on a new plan, concrete, with three parts:

```
Scope: what goes in (and what does not).
Expected result: what will exist when it is done.
Validation: how to prove it is done, preferably a command or test, or an observable behavior.
```

**Size**: **one work session** (a few hours). If it needs "and then...", split it; if too trivial, merge it.

**Order**: in the order they must be done; dependencies between actions go in the action's instructions.

Cite files or modules **only** if you have seen them in the folder. Never invent paths, APIs or business facts: propose something reasonable and label it an assumption.

## Checkpoints

- **Verifiable checkpoints**: a deliverable, a decision or a demonstration ("End-to-end flow validated"), not an activity.
- `data` within the action's period, usually near the end or near an important risk.
- A short name that someone can answer with yes or no.

## Deadlines at AI pace

The one executing is the AI: short deadlines, in sequence, starting today, with no gaps of more than 2 days.

Before estimating, use the blueprint's `tempoReal` and the `tempoReal.inicioEm`/`feitoEm` of comparable completed plans (elapsed time, pauses included). If measurements are absent or thin, say so; do not invent precision.

- **Plan:** estimate positive `horas` based on the work; consider splitting unusually large plans when it helps the user.
- **Action:** 1 to 3 calendar days. More only with a written reason in the instructions (waiting on a person, CI, publishing, a third party).
- **The user's time** (reviewing, approving, testing on a real machine) becomes a dated checkpoint, never hidden slack in the deadline.

Detail: lean, standard (default) or detailed.

## New blueprint

Without `blueprint_id`, also send `blueprint: { nome, descricao }` (a short name, a description of one or two sentences). Propose the name; do not require the user to choose.

## Before sending, check

- [ ] Titles with a verb in the infinitive; none repeated within the action.
- [ ] Every new plan has a description (scope, result, validation).
- [ ] Every new plan has `horas`; actions of up to 3 days (or with a reason) and no gaps between them.
- [ ] Dates `yyyy-mm-dd`, `fim` after `inicio`, checkpoints inside the action, no past dates without a request.
- [ ] Nothing marked done; ids only the ones you read in `ler_blueprint`.
- [ ] Invented facts? If so, turn them into declared assumptions.

## After sending

With the **Apply automatically** switch on (default in the local app) a proposal enters by itself, with a notice and a 60 s Undo; otherwise it waits in the Plan screen. If the user already approved it in the conversation, say "sent; it enters by itself if automatic apply is on" and check with `ler_blueprint` before saying it is in. `remover`, `mover` and new blueprints always wait for the user's click.
