# Edit a blueprint that already exists

Send **only what changes**. The server fills in the rest with what already exists, so a partial proposal never erases content.

- **Existing action:** the `id` and only the fields that change. Name, dates, status, instructions, plans and checkpoints you do not send stay as they are.
- **Existing plan or checkpoint:** the `id` and only what changes (e.g. `{ "id": "t2", "texto": "New title" }` keeps the description).
- **New item** (no `id`): complete, as in the `planejar` topic (a plan with `texto` and `descricao`; a checkpoint with `nome` and `data`).
- Actions you do not cite stay as they are. **Nothing is erased** by omission, and items already done stay done.
- **Move** a plan or checkpoint to another action: cite its `id` in the destination action's list (existing or new). The item leaves the source and goes with the same id and whatever was already done; for a checkpoint, send the new `data`, within the destination's period. The screen shows "← source action".
- **Move a whole action from ANOTHER blueprint in the same workspace** (to merge blueprints): the `mover` list in `propor_blueprint` with `{acao_id, motivo}`, on the destination (existing) blueprint. The action arrives with plans, checkpoints, done state and prototypes; the user reviews and applies. Do not recreate the action with the same text: that would lose what is already done.
- **Projects** (groups of actions inside the blueprint): `blueprint.projetos` lists the projects that must exist (`[{ "nome": "Interface" }]`; a name that already exists is the same project, and the existing ones come in `projetos` of `ler_blueprint`). In each action, `projeto` accepts the id or name of an existing project or one listed in the proposal; `""` or `"Geral"` takes the action out of its project; without the field it stays where it is. An unknown project is an error. To distribute existing actions, send only `{ "id", "projeto" }` for each. A proposal never deletes or renames projects (that is done on the Edit screen), and action numbering does not change.
- **Delete:** the `remover` list in `propor_blueprint`, with `{acao_id, plano_id | checkpoint_id, motivo}` (only `acao_id` deletes the whole action). It is a request: the screen shows the items unticked and only what the user ticks and applies disappears. Always give the `motivo`; for an item already done, only if the user asked.

- **Proposal queue and adjustment:** each `propor_blueprint` enters a blueprint **queue** with a number (1, 2, 3…) and **never erases another**; the user applies or discards them one at a time on the Plan screen. The reply carries the number and the proposal's **provisional ids**: `+a1` (new action), `+a1.p2` (plan), `+a1.c1` (checkpoint); on an existing action, `+<action id>.p1`. To **fix a proposal that is still in the queue** (a title, one more plan), send `ajustar: <number>` with only what changes: cite the action/plan/checkpoint by provisional id (or real id) and the new fields; an item without `id` goes at the end. Nothing disappears by omission (to remove something, use `substituir`), and the whole proposal is revalidated: on error, the queued one stays as it was. `substituir: <number>` replaces the whole proposal, only when the user asks to redo it. A number that is not in the queue (already applied, discarded or the app restarted) is an error: send a new proposal. The queue lives in the app's memory and disappears when it restarts.
- Never invent an `id`: use only those from `ler_blueprint` and the provisional ones returned by `propor_blueprint`.
- A finished or distant planned action shows as one line in the summarized structure of a large blueprint: read it with `ler_blueprint(acao_id)` before touching its plans (the proposal needs the plan ids). To **rewrite the instructions** of an action, read it first with `ler_blueprint(acao_id)` and send the whole new text in `observacao`.
- **Reviewing deadlines:** point out the conflicts (overlapping actions without a reason, a checkpoint outside its action) and adjust only what makes sense, explaining.

Example: postpone an action and add a plan, without resending the rest:

```json
{ "acoes": [{ "id": "a1", "fim": "2026-12-05", "planos": [{ "texto": "Cover errors with tests", "descricao": "Scope: … Expected result: … Validation: node x.test.mjs passes." }] }] }
```
