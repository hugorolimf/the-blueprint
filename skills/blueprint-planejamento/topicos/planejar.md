# Plan a blueprint: actions, plans and checkpoints

## Actions

- `nome`: short and specific, not "Action 1".
- Order: actions run in the order they are written, so write them in the order the work must happen. The app has no dates or durations.
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
- Put it where it proves the action is done, or right after an important risk.
- A short name that someone can answer with yes or no.

## The user's time

Reviewing, approving and testing on a real machine are the user's: make each one a checkpoint.

Detail: lean, standard (default) or detailed.

## New blueprint

Without `blueprint_id`, also send `blueprint: { nome, descricao }` (a short name, a description of one or two sentences). Propose the name; do not require the user to choose.

## Before sending, check

- [ ] Titles with a verb in the infinitive; none repeated within the action.
- [ ] Every new plan has a description (scope, result, validation).
- [ ] Actions in the order the work must happen; no date or estimate in any field.
- [ ] Nothing marked done; ids only the ones you read in `ler_blueprint`.
- [ ] Invented facts? If so, turn them into declared assumptions.

## After sending

With the **Apply automatically** switch on (default in the local app) a proposal enters by itself, with a notice and a 60 s Undo; otherwise it waits in the Plan screen. If the user already approved it in the conversation, say "sent; it enters by itself if automatic apply is on" and check with `ler_blueprint` before saying it is in. `remover`, `mover` and new blueprints always wait for the user's click.
