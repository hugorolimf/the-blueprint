# Hosting The Blueprint for a team

[Back to the README](../README.md) · [Português](pt-BR/hospedar.md)

The same `blueprint` binary that runs the desktop app also runs a team server: `blueprint serve --hospedar`. People join through invite links
(no passwords: the URL is the invite), each one with a role per workspace, and each person's agent connects with a personal MCP key. The local
app does not change: without `--hospedar` nothing on this page applies. Design and contract: [colaboracao.md](colaboracao.md).

## What you need

- A machine that stays on (a small Linux VPS is enough) and a domain name pointing to it, for example `team.example.com`.
- The binary: on Windows, the installed `blueprint.exe`; on Linux, build it with `cargo build --release` (the result is `target/release/blueprint`).
- A reverse proxy that terminates TLS. The app never speaks HTTPS itself: [Caddy](https://caddyserver.com) gets and renews the certificate on its own.

## Configuration

| Variable | Required | What it does |
| --- | --- | --- |
| `BLUEPRINT_URL_PUBLICA` | yes | The public origin, `https://team.example.com`, with no path and no trailing slash. It is also the only accepted `Origin` and `Host`. |
| `HOST` | no | Where the app listens. Default `127.0.0.1` (the proxy on the same machine). |
| `PORT` | no | Default `3000`. |
| `BLUEPRINT_DB` | no | The SQLite database. Default `blueprint.db` in the current folder. |
| `PROTOTIPOS_DIR` | no | Where action prototypes are stored. Default `prototipos`. |
| `BLUEPRINT_PROXY` | when the proxy is on another machine | The proxy's IP. The app refuses to start with `HOST` outside loopback without it. |
| `BLUEPRINT_HSTS` | no | `1` adds `Strict-Transport-Security: max-age=31536000`. Turn it on once HTTPS works. |
| `DADOS_ARQ` | no | A `data.json` copied into an empty database on the first run (the file is not changed). |

`BLUEPRINT_HOSPEDAR=1 blueprint serve` is the same as `blueprint serve --hospedar`. `MCP_REDE` has no effect here: in hosted mode the session
cookie or the member's key replaces the loopback rule.

The app trusts `X-Forwarded-Proto` and `X-Forwarded-For` only from the proxy (a loopback peer or `BLUEPRINT_PROXY`). A request without
`X-Forwarded-Proto: https` gets `403 https-obrigatorio`, even for the page; a different `Host` gets `403 host-invalido`; a browser request that
changes something must carry the public `Origin` (`403 origem-invalida`). Every response carries `Referrer-Policy: no-referrer`,
`X-Content-Type-Options: nosniff` and `X-Frame-Options: DENY`.

## Caddy

A minimal `Caddyfile` (Caddy forwards the original `Host` and sets `X-Forwarded-Proto` and `X-Forwarded-For` by default):

```
team.example.com {
    reverse_proxy 127.0.0.1:3000
}
```

With nginx, forward the same headers yourself: `proxy_set_header Host $host;`, `proxy_set_header X-Forwarded-Proto $scheme;`,
`proxy_set_header X-Forwarded-For $remote_addr;`, plus `proxy_buffering off;` so live updates (server-sent events) are not held back.

## Running it as a service

A systemd unit (`/etc/systemd/system/blueprint.service`):

```
[Unit]
Description=The Blueprint (team server)
After=network.target

[Service]
User=blueprint
WorkingDirectory=/var/lib/blueprint
Environment=BLUEPRINT_URL_PUBLICA=https://team.example.com
Environment=BLUEPRINT_DB=/var/lib/blueprint/blueprint.db
Environment=PROTOTIPOS_DIR=/var/lib/blueprint/prototipos
Environment=BLUEPRINT_HSTS=1
ExecStart=/usr/local/bin/blueprint serve --hospedar
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Then `systemctl enable --now blueprint`. On Windows, run the same command from Task Scheduler at startup, with the variables set for that user.

## The owner link

On the first start, with no owner yet, the server prints **once** a link like `https://team.example.com/convite#...`, valid for 24 hours and for
a single use. Open it, type your name, and you are the owner of every workspace. The secret is in the part after `#`, which the browser never
sends to the server or to the proxy log. The system journal keeps that printed line, which is why the link works only once and expires.

Lost it before using it? Stop the service and run `blueprint serve --hospedar --novo-link-dono` once: it revokes the pending link and prints a
new one. Whoever has the server's terminal is the authority.

## Invites, roles and agents

From the app, an owner invites people to a workspace as **owner**, **collaborator** (edits), **supervisor** (reads and follows progress) or
**visitor** (a public, read-only link to one blueprint). Links expire (1 to 720 hours, 168 by default), can be limited in uses, revoked or
swapped. Only the SHA-256 of each secret is stored; ten wrong attempts from one address in 15 minutes get `429 muitas-tentativas`.

Each member generates a personal MCP key on their own screen and connects their agent with the command it shows:

```
claude mcp add blueprint-equipe --env BLUEPRINT_URL=https://team.example.com --env BLUEPRINT_CHAVE=bpk_... -- blueprint --mcp
```

The agent then acts with that member's role (a supervisor's agent can read but not write). For `https://` the `--mcp` uses the system `curl`
and passes the key through its standard input, never on the command line. Generating a new key replaces the old one; revoking it gives
`401 chave-invalida`. Known limit: the Claude Code plugin hooks (session, edits) talk to the local app and do not follow the team server, and the
routes that open processes on the local machine (terminal, "Start with Claude", conversation, updates) do not exist in hosted mode.

## Backup

The database is one SQLite file in WAL mode. For a consistent copy while the server runs, use SQLite's online backup:

```
sqlite3 /var/lib/blueprint/blueprint.db ".backup '/var/backups/blueprint-$(date +%F).db'"
```

Or stop the service and copy `blueprint.db` together with `blueprint.db-wal` and `blueprint.db-shm`. Back up `PROTOTIPOS_DIR` too.
`blueprint exportar <database> <file.json>` writes the plans as JSON (people, invites and sessions stay out of it).

## Updating

The team server has no automatic updater. Make a backup, replace the binary, restart the service. New database migrations run on start and
never change one that was already applied.

## Testing locally

`node scripts/servidor-e2e.mjs` and `node scripts/convites-e2e.mjs` start a hosted server with a temporary database behind
`scripts/proxy-teste.mjs`, a test proxy that plays Caddy's part without TLS.
