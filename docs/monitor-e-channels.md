# Notices from the app to Claude Code

[Português](pt-BR/monitor-e-channels.md)

`blueprint watch` follows `/api/mcp/eventos` over SSE, at `http://127.0.0.1:3000` or `BLUEPRINT_URL`. The plugin's monitor starts it with `when: always`; it needs neither `userConfig` nor a channel flag. The installer writes the absolute path of the executable into the command. The manifest uses `experimental.monitors`.

The target comes from the folder's link in the database, read through `hook/sessao` on every reconnection. So the `Blueprint:` line in the instructions file does not need to exist for the filter to work. Folders without a link stay silent. The process sleeps between connection attempts (1, 2, 4, 8, 16 and 30 seconds); an open connection blocks waiting for data. There is no polling at rest.

The page sends `/api/mcp/aviso` after a successful `PUT /api/dados`: a proposal applied or a checkpoint marked by the user. The queue ack only confirms receipt and does not raise an "applied" notice. The server checks that the blueprint exists and, for a checkpoint, that it is marked in the database. Events carry only ids; the monitor prints fixed messages, never names, comments, HTML, commands or proposal content. Each message is under 200 bytes; at most 20 per minute. SSE lines/events larger than 2048 bytes close the connection and trigger the wait. Each subscriber has a queue capped at 256 events; the excess is dropped. The notices are informational, with no history and no delivery guarantee while disconnected.

Monitors run only in interactive sessions and with compatible providers. Disabling the plugin in the middle of a session does not stop a monitor that already started; close and reopen the session. These limits come from the [official components reference](https://code.claude.com/docs/en/plugins/components) and the [monitors manifest](https://code.claude.com/docs/en/plugins/manifest-reference#monitors).

## Reproducible validation

```text
cargo test watch --lib
cargo test aviso_so --lib
cargo test o_sse --lib
cargo build
node scripts/testar-monitor.mjs
claude plugin validate claude-plugin/plugins/blueprint
```

On 2026-10-22: the tests of the reader, the 100-event cap, the filter, the fixed messages and a fake server with three events and a reconnection passed. The test with the real server, a temporary database and the `watch` process confirmed delivery, filtering and reconnection after closing and reopening the app; it measured 15.625 ms of CPU over four seconds with the app closed. `plugin validate` passed, with an icon warning for the catalog. A real interactive session opened by `blueprint claude` showed "1 monitor"; after an "applied proposal" notice on the temporary target `sm/integracao`, it displayed «Monitor event: The Blueprint: o usuário aplicou uma proposta…» (the message was in Portuguese then; it now follows the user's language) and the model called `ler_blueprint`.

## Comparison with Channels

User decision on 2026-10-22: **keep Channels as an experimental option**. The monitor stays the default.

The Blueprint's own channel was also validated in an isolated interactive session: MCP `server:blueprint`, `BLUEPRINT_CHANNEL=1` and the development flag. After a `proposta-aplicada` notice on the temporary server, the terminal displayed `← blueprint: The Blueprint: o usuário aplicou uma proposta…` and Claude called the Blueprint tools. The channel shares the monitor's filter, rate limit and fixed text. Turning it on is optional; when it is on, the monitor does not print the same notice again.

| Aspect | Monitor | Own channel |
| --- | --- | --- |
| Activation | Plugin and interactive session | MCP over stdio and an explicit session option |
| Flag | No channel flag | `--dangerously-load-development-channels server:blueprint` |
| Confirmation | The plugin's normal trust flow | Development confirmation in every session |
| Contract | `experimental.monitors`; may change | Research preview; specific capability and notification |
| Content | stdout lines, limited fixed text | `notifications/claude/channel`, filtered content |
| Cost | Context only when there is a notice; idle process | Context when there is a notice; persistent MCP process |
| History | Open session only | Open session only |

Local pre-assessment on 2026-10-22: Claude Code 2.1.309 signed in through claude.ai, firstParty provider, Pro subscription. The global Bun shim had no version configured (`proto::detect::failed`), but a temporary Bun 1.4.4 executable made it possible to test the official fakechat. An isolated interactive session received the message over a local WebSocket and answered `BLUEPRINT_CHANNEL_OK`: the delivery prerequisite was met, without changing the global configuration.

The MCP declares `capabilities.experimental['claude/channel']`, with no permission relay capability and no reply tool. Emitting is opt-in with `BLUEPRINT_CHANNEL=1`: after `notifications/initialized`, it follows the same filtered events as the monitor and sends `notifications/claude/channel` as indivisible JSON-RPC lines. Without the opt-in it opens no extra connection. When the variable is set, the monitor exits silently to avoid duplicates. The permanent decision to keep or drop it is still the user's.

To try it in PowerShell with the plugin installed:

```powershell
$env:BLUEPRINT_CHANNEL = '1'
blueprint claude --dangerously-load-development-channels plugin:blueprint@blueprint-local
# In the next normal session, go back to the monitor:
Remove-Item Env:BLUEPRINT_CHANNEL
```

For an MCP registered directly, use `server:blueprint` instead of `plugin:blueprint@blueprint-local`. The choice depends on how it was installed, not on the tool's displayed name.

The [official Channels documentation](https://code.claude.com/docs/en/channels) lists the compatible providers and accounts. The [development reference](https://code.claude.com/docs/en/channels-reference#test-during-the-research-preview) requires an interactive session for the development flag: with `-p`/SDK it is ignored; organization policies still apply. This integration offers no reply over the channel and no permission relay. Since 2026-10-23 the channel also carries ONE request from the user: the click on Start for an open session (event `iniciar-acao`, only to the addressed channel; see [claude-na-folha.md](claude-na-folha.md), "Choosing the session"). The other notices stay informational.
