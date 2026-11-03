# Planejar: the sheet is the plan

[Back to the README](../README.md) · [Português](pt-BR/planejar.md)

An agent writes the plan piece by piece into a fixed format, and you watch it arrive on the blueprint's sheet. Nothing it sends changes your approved content until you decide. Ticking a plan starts an agent for it, in its own copy of the repository, and you integrate the result when it is ready.

**Where it exists:** only in the local app with the native window (Linux). On Windows, in the browser, under `blueprint serve`, on the guest port and in hosted mode the routes and tools are absent or refused, and `propor_blueprint` remains the way to plan.

## The format

A plan is a folder with a `plano.json` in the format `blueprint.plano/1`, plus its figures:

```json
{
  "formato": "blueprint.plano/1",
  "blueprint": "Energy monitor",
  "projetos": [{ "acoes": [{
    "id": "1.1", "nome": "Read the meters", "de": "2026-11-02", "ate": "2026-11-09",
    "planos": [
      { "id": "1.1.1", "texto": "Read the meters over the local network", "area": "collector" },
      { "id": "1.1.2", "texto": "Store the readings", "area": "database", "depende_de": ["1.1.1"] }
    ]
  }]}]
}
```

Only the blueprint's name and its actions are required. Actions can also carry `texto`, `checkpoints`, `tabelas` and `detalhes` (figures in SVG, PNG, JPG or WebP); the plan as a whole can carry `decisoes` and `perguntas`. Full examples: `src/plano/fixtures/`. Fields such as `aprovado`, `estado` or `feito` in the JSON have no effect.

- **`id`** links one version of the plan to the next. Keep it stable.
- **`area`** says what part of the project a plan touches. Two plans whose areas overlap (`app` and `app/bar`) never run at the same time, even in different blueprints of the workspace.
- **`depende_de`** holds a plan until those plans are done.
- **`plano`**, on a detail, table, checkpoint, decision or question, places it inside that plan of the action. You review plan by plan; without `plano`, the piece belongs to the whole action.

## The agent's tools

| Tool | What it does |
| --- | --- |
| `abrir_plano`, `escrever_acao`, `fechar_plano` | write piece by piece: the actions appear at once as "Waiting for the plan" and fill in as they are written. Calling `abrir_plano` again resumes the open writing |
| `salvar_plano` | sends a whole folder at once |
| `ler_plano`, `ler_respostas`, `ler_workspace` | read the plan, your reviews and answers, and the state of the workspace. Reading has no side effects |
| `falar_com_voce` | leaves you a message |

No tool approves a change, ticks a plan, integrates a branch or turns on an agent, in any profile. Those are gestures of the app window only.

## What you decide

- **New actions.** Written piece by piece, they enter the sheet when the writing closes. From `salvar_plano`, they wait for **Place** on the "What arrived" screen.
- **Changed actions.** Your approved content stays on the sheet, with the change as a word-level diff inside the action. New screens, details and tables appear in the action too, marked with `+`, before you decide. **Approve** applies it, **Edit** changes the text first, **Reject** leaves everything as it was. A plan changed while you were looking (409) asks you to review again.
- **Decisions and details** that ask for approval get their own control. Rewriting one drops its approval.
- **A plan that is done** and whose text, instructions or area changed goes back to not done when you approve; the diff warns you on that line.
- **While an agent runs** on a changed or removed plan, approval is refused until you cancel it or it finishes.

## Running agents

1. Click the plan's checkbox, or **Start the N**. Only this gesture creates work.
2. The app starts the plans that are free, one per tick, up to the limit (4 by default, 1 to 16). The rest wait for a slot, a dependency or an area.
3. Each agent opens in an external terminal, in a copy of the repository outside your folder (`git worktree`), on the branch `agente/<id>`. A plan that follows a finished, not yet integrated plan starts from its branch. A folder without git has no copy: one agent per folder.
4. The agent finishes with `marcar_concluido` and evidence, which is written on the server. Only the agent holding the secret of that attempt can do it.
5. **Integrate** on the agents screen merges the branch into the main folder with `--no-ff`, then removes the copy and the branch. It refuses if the main folder moved to another branch or has a merge in progress. A conflict is aborted and reported. Your local changes are never discarded.

An agent with no signal for 120 seconds appears as gone, and **Resume** brings it back in the same copy. **Cancel** stops it at its next signal and keeps the branch and the copy.

The **orchestrator** is an optional agent of the workspace. It stays off until you turn it on or send it a message, since waking it spends tokens. It has no Bash, sees the state and leaves messages; it does not approve, start or complete anything.

## What this does not guarantee

- A copy is not a sandbox: an agent can reach the main folder by absolute path, and `.git/hooks` is shared. "No conflict between agents" relies on the areas being declared honestly.
- A hostile process running as your user can read the database and the app's memory, or edit SQLite. An agent with Bash can reach the agent routes without the gesture token, though not the gesture routes.
- A `plano.json` written by someone else can try to induce the agent. Its text goes to the agent marked as data, which helps but is not immunity.
- With the native window open, the old web page cannot save: its writes require the token of the window.
- An agent stopped on a permission or folder trust prompt waits in its terminal. Whether it asks depends on your own Claude Code settings.
- Backups copy the database. Figures on disk are not copied, as with prototypes today.
