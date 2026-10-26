# Sharing a workspace from the app

[Back to the README](../README.md) · [Português](pt-BR/compartilhar.md)

The desktop app can give other people access to **one workspace** without a server: the app you have open answers them, through a free
HTTPS address from Cloudflare (a "quick tunnel", no account needed). People open the link in a browser, type their name and work on the same
blueprints with you, live. Nothing is copied: there is still one database, yours. For a team server that stays on without your computer,
see [Hosting The Blueprint for a team](hospedar.md). Design and contract: [colaboracao.md](colaboracao.md#compartilhar-pelo-app).

## How to turn it on

1. Open the workspace and click **Share**. Choose the workspace and turn sharing on.
2. The first time, the app downloads `cloudflared` (about 53 MB) from Cloudflare's official releases on GitHub into
   `%APPDATA%\Blueprint\bin\`. The version and its SHA-256 are fixed in the app; the file is checked before every run.
3. In a few seconds the dialog shows the address, like `https://some-random-words.trycloudflare.com`.
4. Create an invite (role, optional blueprint, validity) and send the link. Each link works once by default; a public read-only link is a
   visitor invite for one blueprint. Roles are the same as in hosted mode: owner, collaborator, supervisor, visitor.
5. The person opens the link, types a name and lands on the workspace. Their agent can connect too, with a personal MCP key from the
   **My key** screen (the command already points at the sharing address).

To stop, click **Turn off** in the same dialog: the tunnel goes down and the guest port closes. Quitting the app also stops it (the tunnel
process lives in a Job Object and dies with the app). Hiding the window in the tray does **not** stop it.

## Opening the invite in the installed app

The installer registers the `blueprint://` protocol. When a guest opens an invite link, the page first tries to open **the installed
app**: the browser asks once ("Open The Blueprint?"), the app asks which address it is opening, and a window of its own shows the shared
workspace (same sign-in by name, same screens, separate from your own local window). If the app is not installed, or does not answer in
2.5 s, the web version continues in the same tab (a "Continue in the browser" button is always there; `?web` after `/convite` skips the
attempt). Only a valid `https://` address and an invite code in the format the server generates are accepted; anything else just opens
the app normally. In that window, **My key → Generate key** also shows **Connect my Claude on this computer**: after a native question
with the address, it does what `blueprint conectar` does (next section), with no command to copy.

## The guest's Claude

A guest can connect their own Claude Code (and Codex) to the shared workspace with one command. They need The Blueprint installed on
their computer (the installer from the release; the app does not have to be open there).

1. In the browser, on the shared workspace, the guest opens **My key** and clicks **Generate key**.
2. The screen shows the command, ready to copy: `blueprint conectar https://<address> bpk_...`. They run it in a terminal.
3. They open a new Claude Code session in a folder whose `CLAUDE.md` (or `AGENTS.md`) has the line
   `Blueprint: workspace_id <id>, blueprint_id <id>` of a blueprint in the shared workspace.

What the command does: it installs a plugin of its own, `blueprint-equipe` (marketplace `blueprint-equipe-local`, in
`%APPDATA%\Blueprint\claude-plugin-equipe`), so it never clashes with the local `blueprint` plugin, and registers the MCP `blueprint-equipe`
in Codex. Claude then reads and proposes in the shared workspace with the guest's role, and its hooks (progress on edits, subagents, the
reminder at the end of a turn) reach the owner's app: the owner sees the guest's Claude on the sheet as **Claude · <name>**.

- **The address changes?** Run `blueprint conectar` again with the new address (and a new key if the old one was revoked). Open sessions
  pick up the new address on their next call.
- **Undo:** `blueprint conectar --remover` removes the plugin, the MCPs and the key from the computer.
- **Where the key lives:** in `%APPDATA%\Blueprint\conexao-equipe.json` (your user folder only), read at each request by
  `blueprint --mcp --equipe` and `blueprint hook <event> --equipe`. It is never in the arguments of the MCP or hook processes, nor in the
  plugin copy that Claude Code keeps; the hooks send it as `Authorization: Bearer`, like the MCP. It does appear once in the
  `blueprint conectar` command itself (and in your shell history), and Codex keeps only the command, not the key.
- **What the guest's hooks reach:** only the blueprints the role can write; the folder links, the session list, the status line and
  "Start with Claude" stay on the owner's machine only. If the owner's app is closed or the address is old, the hooks fail silently
  (3 to 5 s at most) and never block Claude.

## Limits

- **The app must stay open**, and the computer awake and online. When it closes, guests lose access until you share again.
- **The free address changes every time** you turn sharing on. Old links stop working and the guests' sign-in belongs to the old address,
  so send new invites each time. A fixed address is on the roadmap (below).
- **No guarantee from Cloudflare.** Quick tunnels are meant for testing: no uptime guarantee, Cloudflare's terms apply, and there is a
  limit of about 200 requests in flight. Good for a few people, not for a big team.
- **Live updates use short requests.** The quick tunnel holds a streaming response until it ends (measured: 41 events sent one per second
  all arrived together after 42 s). So, for guests only, the app ends each live stream right after an event, and the browser reconnects
  by itself and gets whatever it missed. A change after a quiet moment arrives fast (measured: about 50 ms after the owner saved); a change
  right after another can wait up to 3 s for the reconnect. An idle guest tab makes about one request every 25 s per stream.
- **The new address can take a moment.** Cloudflare says it may take some time to be reachable. If a browser asks too early, the computer
  can remember "this name does not exist" for up to a minute; wait and try again.
- No presence avatars for guests yet (they exist only in hosted mode). Windows only for now. The **Update** button refuses while sharing.

## Security

- **A separate door.** Guests reach the app through a second port, on `127.0.0.1` only, that exists only while sharing is on; the tunnel
  points at it. Your own window keeps using port 3000 as before. The app tells guests and you apart by the port, never by a header, because
  the tunnel itself connects from this machine.
- **Nothing on the guest port is you.** Every request there needs a guest session (from an invite) or a personal MCP key, with the
  Host and Origin of the tunnel address, and it only reaches the shared workspace, with the guest's role.
- **What guests never reach:** the terminal, "Start with Claude", the Claude Code sessions and status line (the hooks only with the
  guest's MCP key, see above), the folder links, the updater, the
  sharing controls, the whole-data import (`PUT /api/dados`), the window, other workspaces, and the text you send to your own agent
  sessions. Those routes answer 404 on the guest port.
- **Cloudflare sits in the middle.** HTTPS ends at Cloudflare, which then carries the traffic to the app through the tunnel. Share only
  what you are comfortable sending through Cloudflare.
- **Third-party binary.** `cloudflared` runs with a fixed version and SHA-256; a file that does not match is deleted and refused. It runs
  with `--no-autoupdate`, without a console window, and dies with the app.

## A fixed address

In the Share dialog, **Address** has two choices (kept for next time; they can only change while sharing is off):

- **Temporary** (default): Cloudflare quick tunnel, no account. The address changes every time you turn sharing on.
- **Fixed**: Tailscale Funnel. The address is `https://<your-device>.<your-tailnet>.ts.net` and never changes, so invite links, sessions and the guests' Claude (`blueprint conectar`) keep working after turning sharing off and on. You need Tailscale installed and signed in, MagicDNS and HTTPS certificates on, and the `funnel` attribute allowed for your device in the tailnet policy ([Funnel requirements](https://tailscale.com/kb/1223/funnel)). The app keeps no password or key: it runs your own `tailscale`. It checks `tailscale status --json` (connected, and the `.ts.net` name), picks the first free Funnel port among 443, 8443 and 10000 from `tailscale serve status --json` (it never takes a port you already use), and keeps `tailscale funnel --https=<port> http://127.0.0.1:<guest port>` running in the foreground inside a Job Object, so closing the app closes the Funnel (a `--bg` Funnel would stay pointing at the port after a crash). The address the command prints must be this device's, or sharing stops with an error.

Changing from one type to the other changes the address: people who already joined need one new invite link (and to run `blueprint conectar` again). Browsers do not carry a session cookie to another address, so this cannot be avoided.

Checked on 2026-10-26:

| | Temporary (Cloudflare quick tunnel) | Fixed (Tailscale Funnel) | Named Cloudflare tunnel (not built in) |
| --- | --- | --- | --- |
| Account | none | Tailscale account | Cloudflare account and a domain on Cloudflare |
| Address | changes every time | fixed `*.ts.net` | your own hostname |
| Ports | any | 443, 8443, 10000 only | any |
| Limits | "for testing and development", no uptime guarantee, 200 requests in flight (more get 429), no Server-Sent Events (the app works around that, see below) | "non-configurable bandwidth limits", TLS only | Free-plan terms restrict serving large files and video |
| Source | [Cloudflare quick tunnels](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/) | [Tailscale Funnel](https://tailscale.com/kb/1223/funnel), [funnel command](https://tailscale.com/kb/1311/tailscale-funnel) | [Create a remote tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-remote-tunnel/), [Tunnel FAQ](https://developers.cloudflare.com/cloudflare-one/faq/cloudflare-tunnels-faq/) |

The quick tunnel does not carry Server-Sent Events as a live stream, so on the guest port each live-update connection closes after its first event and the browser reconnects with `Last-Event-ID` (measured: 41 events arrived together after 42 s). The Tailscale Funnel does not have this limit. The named Cloudflare tunnel stays out of the app on purpose: it needs a domain, a dashboard setup and a token, which is more than "turn it on". If you use one, run it yourself pointing at `blueprint serve --hospedar` ([hosting guide](hospedar.md)). Code: `src/compartilhar/tailscale.rs`, `provedor.rs`; tests: `cargo test compartilhar` and `node scripts/endereco-fixo-e2e.mjs`.

- **A team server:** `blueprint serve --hospedar` on a small machine that stays on ([hosting guide](hospedar.md)).

## For developers

| Route (local port only, loopback Host and Origin, JSON) | Answer |
| --- | --- |
| `GET /api/compartilhar` | `{"estado": "desligado" \| "baixando" \| "ligando" \| "ligado" \| "erro", "url"?, "workspaces"?, "workspaceId"? (the first), "erro"?, "porta"?}` |
| `POST /api/compartilhar/ligar` `{"workspaceId"}` | 202 and the state; 400 `falta-workspace`, 404 `workspace-inexistente`. With the tunnel already on, it adds the workspace to the same address |
| `POST /api/compartilhar/desligar` `{"workspaceId"?}` | 200 and the state; with `workspaceId` it removes only that workspace (the tunnel stops when none is left), without it everything stops |

Every state change also goes to the page as the `compartilhar` event on `/api/mcp/eventos` (local page only). Invites and people use the
routes of the hosted mode (`/api/convites`, `/api/pessoas`), called by the local page while sharing; links come out with the tunnel address.
`BLUEPRINT_TUNEL_COMANDO=<program>` replaces `cloudflared` with a fake that prints a `https://<words-with-dashes>.trycloudflare.com` line
on its error output (tests and end-to-end checks, no network). Code: `src/compartilhar/`.

## Coming back to the team after closing the app (guest)

The guest's session is the team window's cookie (30 days, persistent WebView2 profile), but an invite link is single-use: with no local record there was no way to reopen the team window after closing the app. Now the team page tells the app (IPC `equipe`) the person's name and the workspaces they see, and the app keeps only that in `%APPDATA%\Blueprint\equipes.json` (`src/equipes.rs`; never the invite or the cookie). The workspaces screen shows one "Team" card per remembered team; clicking it calls `POST /api/equipes/abrir`, which only accepts an already-remembered origin and opens the team window, where the session carries on. "Remove from list" forgets the team (`/api/equipes/esquecer`). Local-only routes; test: `node scripts/equipes-locais-e2e.mjs`.

Limit: the quick-tunnel address changes every time the host turns sharing back on, and the cookie only works for the old address. In that case the card opens a "Sign in through an invite link" screen and the guest needs a new invite; a fixed address is the next action.

The guest also works from the installed app: see `docs/app-no-projeto-compartilhado.md` (set the repository folder up and start Claude on an action from the team window, with a native confirmation; the folder link lives in `CLAUDE.local.md`, out of Git).

## Remote MCP over HTTP (no app installed)

A guest can connect Claude Code or Codex **without installing The Blueprint**: the server (hosted mode and the guest port of Sharing) answers `POST /mcp`, the MCP "Streamable HTTP" transport, with the member's key as `Authorization: Bearer bpk_...` and the same tools and instructions as `blueprint --mcp`. The My key screen shows the commands.

Decisions (checked on 2026-10-26 against the sources below):

| Point | Decision | Source |
| --- | --- | --- |
| Endpoint | one path, `/mcp`, POST only; GET and DELETE answer 405 (the spec allows a server with no SSE stream and no session termination) | [MCP transports, 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports) |
| Replies | requests get one `application/json` object; notifications get `202` with no body; no batches | same |
| Session | none (`Mcp-Session-Id` is optional); each request carries the key | same |
| Version | `MCP-Protocol-Version`, if sent, must be 2025-06-18, 2025-03-26 or 2024-11-05, else 400 | same |
| Origin | a request with an `Origin` header (a browser) is refused with 403, per the DNS-rebinding warning | same |
| Auth | `Bearer` key; no OAuth and no `WWW-Authenticate`, so clients use the header. `initialize`, `ping` and `tools/list` answer without a key (the public server description: Claude Code's connection pre-test sends no header and treats 401 as fatal); `tools/call` without a key is 401; an invalid or revoked key is 401 `chave-invalida` | [Claude Code MCP](https://code.claude.com/docs/en/mcp) |
| Claude Code | `claude mcp add --transport http --scope user blueprint-equipe <url>/mcp --header 'Authorization: Bearer ${BLUEPRINT_CHAVE}'` (single quotes keep `${...}` literal; Claude Code expands it when it connects, so the key is not in the command it prints) | same |
| Codex | `[mcp_servers.blueprint-equipe]` with `url = "<url>/mcp"` and `bearer_token_env_var = "BLUEPRINT_CHAVE"` in `~/.codex/config.toml` | [Codex MCP](https://developers.openai.com/codex/mcp) |
| HTTPS | the tunnel and the hosted mode are HTTPS; the key never travels over plain HTTP outside loopback | hosted mode (docs/hospedar.md) |

Differences from `--mcp`: `vincular_pasta` is refused (the server cannot see your folder: write the `Blueprint:` line in `CLAUDE.local.md`, or use `blueprint conectar` with the installed app), `propor_blueprint` never sends the server's own folder, and there is no channel or hooks (progress shows only what the agent reports with `registrar_progresso`). It does not replace the installed plugin. Code: `src/mcp/http.rs`; tests: `cargo test mcp_http` and `node scripts/mcp-http-e2e.mjs`.
