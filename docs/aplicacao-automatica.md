# Automatic apply

[Português](pt-BR/aplicacao-automatica.md)

The contract that the code (`web/js/auto-aplicar*.js`), the MCP replies (`src/mcp/stdio.rs`) and the planning guide (`skills/blueprint-planejamento/topicos/`) follow. What you ask for or approve in the chat with your agent enters the blueprint without a second click on the Plan screen.

## Who authorizes

- **You, per blueprint.** The switch **Apply automatically: on/off** sits in the header of the Plan screen and is saved per workspace and blueprint in the browser of the window that applies (`blueprint.autoAplicar.<workspace>:<blueprint>`).
- **Local app:** on until you turn it off.
- **Team mode (`blueprint serve --hospedar`):** off by default; only the owner sees the switch and can turn it on. Collaborators, supervisors and visitors never apply automatically.
- **The agent never approves itself.** It only sends a proposal or a record to the queue, as always. The page is what writes, through the same path as the click (`aplicarAberta`, `registroAdiciona`), with the same validation, version control (`If-Match`) and acknowledgement to the server.

## What enters by itself

- A proposal (`propor_blueprint`) for a blueprint that already exists, in the workspace open in the window.
- A record of done work (`registrar_trabalho`) with at least one action.
- Not while you are typing in the Plan screen of that same blueprint: then it waits for your click.

## What never enters by itself

- A proposal with `remover` (deleting actions, plans or checkpoints) or `mover` (moving actions between blueprints). It waits in the Plan screen with the note "needs your confirmation".
- A proposal for a new blueprint.
- Anything while the switch is off, or while no window of the app is open (v1 applies only in the page; with the app in the tray the item waits in the queue).

## Undo

- Every automatic apply shows the notice **Applied automatically** with **Undo** for **60 seconds**, once.
- Undo reverses only the operations that apply made, and only where the value is still the one it left. What someone changed afterwards (another window, the agent) stays, and the page says so instead of guessing.
- An undone item does not go back to the queue: the agent resends it only if you ask.

## Record of each automatic apply

- The Plan screen lists the last automatic applies of the blueprint (time, text, Undo or "undone"). The page keeps the last 30 in the browser.
- The agent is told which item was applied and by whom: the SSE events `proposta-aplicada` and `registro-adicionado` carry `numero` and `origem` (`automatica` or `usuario`), never free text.
- No duplicate apply: each item is keyed by workspace, blueprint, type, number and `criadaEm`. If the server sends an item again (the acknowledgement was lost), the page only acknowledges it again.

## What the agent says

The replies of `propor_blueprint` and `registrar_trabalho` say the item enters by itself when the switch is on, and that removals, moves and new blueprints wait for the click. The agent checks with `ler_blueprint` before telling you it entered.

## Tests

`node auto-aplicar-logica.test.mjs` (rules and Undo), `cargo test mcp::tests` (MCP reply and the first 512 characters of the instructions) and `node scripts/auto-aplicar-e2e.mjs [exe]` (the agent sends through `--mcp`, the page applies, undoes, keeps removals waiting and does nothing with the switch off; pt-BR and en-US).
