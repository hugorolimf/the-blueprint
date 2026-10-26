# Security

[Português](docs/pt-BR/SECURITY.md)

## Threat model

The app is **local-first**: it runs on your machine and trusts whoever can already reach your machine.

- **`/api/mcp/*` routes** only answer on loopback (`127.0.0.1` or `localhost`), check the `Host` header and, when present, `Origin`, and require `content-type: application/json` on every `POST`. They have **no token**: whoever reaches loopback already reaches `/api/dados`.
- **`MCP_REDE=1`** opens those routes to the network. Use it only on a trusted network and at your own risk: without a token, any machine that reaches the port reads and proposes changes. To host it, put a reverse proxy with TLS and authentication in front.
- **What the agent sends never changes your data by itself:** plans are proposals you apply on screen; removing is a request you approve item by item; completing a plan or checkpoint requires evidence, which is kept as a comment.
- **Action prototypes** are third-party HTML (your agent generates it). They are served with `Content-Security-Policy: sandbox allow-scripts; default-src 'none'…` and shown in an `iframe sandbox="allow-scripts"` with an **opaque origin**, no network and no access to the app. `allow-same-origin` is never added. Each file has a 16-character hexadecimal id and the server refuses any other path.
- **Static files:** the server delivers only a fixed list (`blueprint.html`, `emblemas.mjs`, `agentes-figura.mjs`, `web/**`, `kit/**`); `data.json`, the database and the source code never leave over HTTP.
- **Writes:** the SQLite database (WAL) is written in a transaction; an invalid write leaves everything as it was. Each request body has a size limit. Writes to `/api/dados` are versioned (`If-Match`, a `409` on stale data), so two windows cannot overwrite each other.
- **The stdio MCP** (`blueprint --mcp`) is a thin HTTP client of the local server; it opens no ports.
- **Updates** only make a `GET` to GitHub Releases and verify the installer's SHA-256 against the release's `SHA256SUMS`. That proves integrity, not authenticity: signing the release is on the [roadmap](ROADMAP.md).

These rules have tests (`cargo test`) and the script `scripts/mutacoes.mjs` breaks each one on purpose to prove that some test fails.

## Reporting a vulnerability

Do not open a public issue. Use **GitHub → Security → Report a vulnerability** (private vulnerability reporting) on this repository and describe what you saw, how to reproduce it and the impact. We will answer as soon as we can; fixes ship in a new version and are named in the release notes.

## Supported versions

Only the latest version receives security fixes.
