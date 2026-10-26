# The Blueprint v0.2.0 (beta)

[Português](pt-BR/release-v0.2.0.md)

The Blueprint stops being single-user: you can now work on the same blueprint with other people, live.

## What is new

### Working together

- **Share a workspace from the app.** A Share button on the sheet opens one workspace (or several) to the people you invite, straight from your own app, through an HTTPS tunnel from Cloudflare. No server and no account: the app downloads `cloudflared` the first time (fixed version, checked by SHA-256). Guests use the browser and only see the workspace of their invite; your other workspaces, the terminal, Start with Claude and the updater stay closed to them. Limits: it works while your app is open, and the free address changes every time you turn sharing on. See [Share from the app](compartilhar.md).
- **Team server.** `blueprint serve --hospedar`, behind an HTTPS reverse proxy such as Caddy, hosts workspaces for a team that is always on. See [Hosting](hospedar.md).
- **Invites, roles and people.** The link is the invite (no passwords): owner, collaborator, supervisor and visitor, per workspace or per blueprint, with expiry and revoke. The secret travels after the `#`, so it never reaches proxy logs, and only its hash is stored. People panel, invite links shown once, a Follow panel for supervisors and a public read-only link that opens a blueprint or its presentation.
- **Live changes and conflicts.** The page saves only what changed. Changes reach everyone in under a second, presence shows who is looking, and the item someone else is editing gets an outline. When two people change the same field, the last one wins and both see who changed it; a per-field history is kept.
- **Each person's own agent.** Every member gets an MCP key, so their Claude or Codex reads and proposes with their role (`BLUEPRINT_URL` and `BLUEPRINT_CHAVE`).

### Claude, inside and around the app

- **Claude Code inside the app:** a Terminal panel with tabs runs the official `claude` (all features), and Start with Claude can open in it.
- **Talk to the plan:** a panel on each action sends your messages to your Claude session and shows its answers; any change it suggests arrives as a proposal you apply. Plans also get a Send to agent button.
- **Agent timeline per action**, stored in the database (last 50 events, 30 days).
- **Focus mode** (one card per working agent) and **native search** in the bar, the workspace and the open blueprint.
- **Presentation and printing** of a blueprint (full screen, A4, single-file export).

### Also

- English first: the interface follows the Windows language, with a selector on the entry screen; server messages, hooks and the status line speak your language, and what the agent reads is in English.
- Roadmap screen inside the app, `blueprint exportar --markdown`, shell completions for PowerShell and Bash.
- Two windows of the same person no longer overwrite each other.
- Fixes: the channel of sessions opened by the button now reaches them (`@inline`), sessions that close badly leave the list after 24 h, and a development build no longer takes over your Claude plugin.

## Security notes

Sharing exposes your app to the internet through the tunnel. Guests arrive on a separate local port where nothing counts as you, every route checks the guest's role, and local-only routes do not exist there. Read [Share from the app](compartilhar.md) and the [security review](seguranca-servidor.md) before sharing with people you do not trust. HTTPS on a real hosted server was tested through a test proxy, not with a real certificate.

## How to install

Download `Blueprint-0.2.0-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
