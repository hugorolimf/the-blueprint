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
- **What guests never reach:** the terminal, "Start with Claude", the Claude Code hooks and sessions, the folder links, the updater, the
  sharing controls, the whole-data import (`PUT /api/dados`), the window, other workspaces, and the text you send to your own agent
  sessions. Those routes answer 404 on the guest port.
- **Cloudflare sits in the middle.** HTTPS ends at Cloudflare, which then carries the traffic to the app through the tunnel. Share only
  what you are comfortable sending through Cloudflare.
- **Third-party binary.** `cloudflared` runs with a fixed version and SHA-256; a file that does not match is deleted and refused. It runs
  with `--no-autoupdate`, without a console window, and dies with the app.

## A fixed address later

- **Cloudflare with your own domain:** a named tunnel (free Cloudflare account and a domain on Cloudflare) gives a stable address and
  streams live updates normally.
- **Tailscale Funnel:** a stable `*.ts.net` address, if you already use Tailscale.
- **A team server:** `blueprint serve --hospedar` on a small machine that stays on ([hosting guide](hospedar.md)).

## For developers

| Route (local port only, loopback Host and Origin, JSON) | Answer |
| --- | --- |
| `GET /api/compartilhar` | `{"estado": "desligado" \| "baixando" \| "ligando" \| "ligado" \| "erro", "url"?, "workspaceId"?, "erro"?, "porta"?}` |
| `POST /api/compartilhar/ligar` `{"workspaceId"}` | 202 and the state; 400 `falta-workspace`, 404 `workspace-inexistente`, 409 `ja-compartilhando` |
| `POST /api/compartilhar/desligar` `{}` | 200 and the state (`desligado`) |

Every state change also goes to the page as the `compartilhar` event on `/api/mcp/eventos` (local page only). Invites and people use the
routes of the hosted mode (`/api/convites`, `/api/pessoas`), called by the local page while sharing; links come out with the tunnel address.
`BLUEPRINT_TUNEL_COMANDO=<program>` replaces `cloudflared` with a fake that prints a `https://<words-with-dashes>.trycloudflare.com` line
on its error output (tests and end-to-end checks, no network). Code: `src/compartilhar/`.
