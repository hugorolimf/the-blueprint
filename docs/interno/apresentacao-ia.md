# AI presentations: contract (internal)

Status: design contract, 2026-10-26. Blueprint App, project "Apresentacao", actions "Niveis e publicos das apresentacoes", "Gerar as apresentacoes do Atendimento" and "Gerar com IA na tela e no terminal". This file is the single source the three work streams (library, Rust, web) follow. Change it first, then the code.

## What it is

The user picks an **audience** (`cliente` | `interno`) and a **level** (`rapido` | `padrao` | `completo`; `epico` only when asked by name), in the Apresentar dialog or in a terminal conversation (`/blueprint:apresentar`). The agent writes a **content JSON** and one **figure program** per figure; the app assembles a standalone, sandboxed HTML page from them with an embedded library. The page is a real presentation interface in the app's own look (blue plotting paper, mono type, boxed headers, section navigation), with live anatomy figures as part of the interface.

Why data plus code, and not a full HTML from the agent: a dense figure is 0.3 to 1 MB of SVG, too much for a tool call; the figure program is 5 to 30 KB because the browser builds the SVG with the kit. No Node is needed on the user's machine, and a project whose content did not change keeps its figure (see `impressao`).

## Audience

- `cliente`: a PROJECT PLAN. Shows what the client receives, stages with dates, what is expected from the client, open decisions, risks, in plain language. Never shows technology, routes, files, hours per plan, or the internal instructions of actions. Nothing that the blueprint does not say (no prices, features or dates invented).
- `interno`: the full technical view for the owner.

`apresentacao-ia/termos-tecnicos.json` lists the terms that must not appear in a `cliente` presentation (lowercase, matched as whole words or word prefixes). The library (`auditarCliente`) and the MCP tool both use it; a hit is a warning in the tool reply, never silent.

## Levels

| Level | Figures | Anatomy size | Estimate (announced before starting) |
| --- | --- | --- | --- |
| `rapido` | 1 (the cover) | Figure, 40 to 60 parts | up to about 1 h |
| `padrao` | 1 per stage group, all different objects | several Figures | about 1 to 3 h |
| `completo` | the `padrao` set plus one dense machine of the whole system | Hero, 80 to 150 parts | about 2 to 4 h |
| `epico` | only on request by name | Epic | 6 h or more |

## Data model

`conteudo` (JSON, at most 128 KB):

```json
{
  "versao": 1,
  "publico": "cliente",
  "nivel": "padrao",
  "idioma": "pt-BR",
  "titulo": "string",
  "subtitulo": "string?",
  "cliente": "string?",
  "data": "yyyy-mm-dd",
  "resumo": ["short sentence", "..."],
  "entregas": [{ "titulo": "string", "texto": "string" }],
  "etapas": [{ "id": "e1", "titulo": "string", "inicio": "yyyy-mm-dd", "fim": "yyyy-mm-dd", "texto": "string",
               "figura": "figura-id | null", "marcos": [{ "data": "yyyy-mm-dd", "nome": "string" }] }],
  "precisamosDeVoce": [{ "titulo": "string", "texto": "string", "ate": "yyyy-mm-dd | null" }],
  "decisoes": [{ "titulo": "string", "texto": "string", "estado": "aberta | decidida" }],
  "riscos": [{ "titulo": "string", "texto": "string" }],
  "interno": { "acoes": [{ "numero": "string", "nome": "string", "planos": ["string"] }] }
}
```

`interno` exists only when `publico` is `interno`. `idioma` follows the app language at generation time.

`figuras` (list, at most 12, each `codigo` at most 200 KB):

```json
{ "id": "fichas", "titulo": "string", "dica": "Hover to pick a card", "leitura": "card 07 · on time",
  "chaves": [{ "marca": "lit", "rotulo": "string" }], "legenda": "string", "rotulo": "accessible description",
  "impressao": "sha256 hex of the content this figure explains",
  "codigo": "function (L) { /* build */ return { svg: '<svg…>', vivo: function (palco, dizer) { … } }; }" }
```

`codigo` is a JS function expression. It receives `L` (the library, below), returns `{ svg, vivo? }`; `svg` is the full `<svg>` string from `L.figureSvg`; `vivo(palco, dizer)` attaches motion to the stage element (`palco`) and writes the readout with `dizer(text)`. The page wraps each figure in the kit card (`plateHtml`: `Fig n`, title, hint, readout, keys, caption). One figure that throws shows an error card with the message; the rest of the page still works.

## Library `L` (browser, `apresentacao-ia/biblioteca/`)

A global script set (classic scripts, no modules), each file at most 350 lines:

- the anatomy kit as is (`kit/iso-kit` and `kit/lathe`, all exports on `L`: `fitProjection`, `iso`, `slabOf`, `cylinder`, `solidSvg`, `lineSvg`, `dotsSvg`, `wireSvg`, `figureSvg`, `plateHtml`, `translateAlong`, `lineOnTop`, `topTicks`, `radialTicks`, `knurl`, `coil`, `haloOf`, `depthOf`, …);
- `L.pecas`: reusable parts (surface plate with feet, screws and ruler strip; rails; cards; dial gauge with needle; knobs; label plates), each returning an SVG string;
- `L.laco(palco, passo)`: one frame loop per figure, stopped when off screen (IntersectionObserver, last entry wins) and when settled, a single jump under `prefers-reduced-motion`; `passo(dt, calmo)` returns true to keep running;
- `L.segue(valor, alvo, dt, tau)` and `L.mola(estado, alvo, dt, rigidez, amortecimento, calmo)`;
- `L.auditarCliente(conteudo)`: banned-term audit; `L.impressao(objeto)`: stable hash of a JSON value (sha256 hex, sync).

## Page assembly

`apresentacao-ia/modelo.html` is a template with these placeholders, replaced by plain string substitution (so Rust can do it): `{{titulo}}`, `{{idioma}}`, `{{css}}` (app tokens plus the presentation CSS plus `ISO_CSS`), `{{kit}}` (the kit scripts, inlined), `{{biblioteca}}` (the library scripts), `{{conteudo}}` (the content JSON, with `</` escaped as `<\/`), `{{figuras}}` (a JSON array of the figures, same escaping). The page script builds the interface from `conteudo`, runs each figure, and mounts them in their stage. Screen layout: top bar with breadcrumb and title in a box, section navigation (overview, stages, timeline, what we need from you, decisions and risks), cover with the main figure, panels with header and ruler, plan progress, A4 print stylesheet. The `interno` audience adds an Actions section. No network, no storage, no `localStorage` writes.

## Storage, MCP and HTTP (Rust)

- MCP tool `anexar_apresentacao(workspace_id, blueprint_id, conteudo, figuras, apresentacao_id?)`. `conteudo` and `figuras` are objects/arrays (not strings). Replaces when `apresentacao_id` is given. Limits above; total at most 2 MB. Replies with the id, a view URL path and the warnings (banned terms in `cliente`, an unknown `figura` reference, a figure without `codigo`).
- Storage: `%APPDATA%\Blueprint\apresentacoes\<id>.json` (the content plus the figures); the blueprint keeps only metadata `apresentacoes: [{id, titulo, publico, nivel, em, figuras: [{id, impressao}]}]`. Never in the Git repo.
- `GET /apresentacao/<id>`: the assembled page, same sandbox headers as `/prototipos/<id>.html` (`Content-Security-Policy: sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:`), never `allow-same-origin`. `GET /apresentacao/<id>?baixar=1` adds `content-disposition: attachment`.
- `POST /api/mcp/apresentacao` (from the MCP process) and `POST /api/mcp/apresentacao/remover`, loopback only, JSON only.
- `POST /api/mcp/apresentar` `{workspaceId, blueprintId, publico, nivel, sessao?}`: the screen request. Same path and same rules as `POST /api/mcp/iniciar` (only by user click, only loopback, channel session or a new one). The prompt carries audience, level and the estimate, so the agent does not ask again.
- `/blueprint:apresentar`: a plugin skill (user-invoked only) that starts the conversation, like `/blueprint:status`.
- Tool list, MCP instructions (the first 512 characters keep the rule and the tool names; add `anexar_apresentacao` without breaking that test) and the guide topic `apresentar` document all of it.

## Web UI (`web/`)

In the existing Apresentar dialog (`web/js/apresentacao.js`): a second group in the side rail, "Generate with AI": audience (Client, Internal), level (Quick, Standard, Complete) each with its time estimate, `Epico` never listed, and a Generate button that calls `/api/mcp/apresentar`. When the blueprint has generated presentations, the preview area lists them (title, audience, level, date), shows the selected one in a sandboxed `<iframe sandbox="allow-scripts" src="/apresentacao/<id>">`, and offers Regenerate, Export HTML (`?baixar=1`) and Remove. Every new string goes through `tr()` with an English entry (`node scripts/i18n-completude.mjs`). Files stay under 350 lines, no work at rest.

## Rules that hold everywhere

No emoji. Figures use the app's own tones. The generated page and the stored data are never committed. The `cliente` audience never carries technical terms, hours per plan, internal instructions or invented promises. Open decisions show as "to be decided". Sample data in the repo tests; the real client content lives outside the repository.

## Implementation notes (adopted, 2026-10-26)

The code diverges from the text above in these places; the code is the truth.

- **Metadata and list.** There is no `blueprint.apresentacoes` in the blueprint data and no migration: the metadata live in each `<id>.json` and the list is read from the folder (`%APPDATA%\Blueprint\apresentacoes\`, or inside the prototypes folder when that is moved). The web reads `GET /api/mcp/apresentacoes?workspace=&blueprint=` (newest first) and the SSE event `apresentacao` `{workspaceId, blueprintId, apresentacaoId}`; `ler_blueprint` also returns `apresentacoes`. In team mode they do not sync between machines.
- **Request.** `POST /api/mcp/apresentar` takes `{workspaceId, blueprintId, publico, nivel, sessaoId?, semCanal?}` (like `/iniciar`; not `sessao`) and answers `{ok, pasta, terminal: wt|cmd|canal|copiar, prompt?}`. Local app only: the route does not exist in hosted mode or for guests, and `MCP_REDE=1` does not open it. `remover` takes `{apresentacaoId}` and is idempotent.
- **Figure call.** `codigo(L, c)`: the library also receives `conteudo` (`L.idiomaDe(c.idioma)`). `impressao` only has to be a non-empty string; the tool warns when it is missing.
- **Audit.** Banned terms match whole words (not prefixes: `cli` would flag "cliente"); identifier fields (`publico`, `nivel`, `id`, `figura`, dates, `estado`) are skipped; hours such as "3 h" or "1,5 horas" also warn. `interno` inside a `cliente` presentation is an error (`interno-no-cliente`), not a warning.
- **Page.** A no-network CSP `<meta>` follows `<head>` for the exported file, where the HTTP header does not apply. The tool stores nothing when the app is closed (unlike `anexar_prototipo`).
- **Guide topics.** The guide is split to keep every topic under 4,500 characters: `apresentar` (the app's own three types), `apresentacao_ia` (conversation, audience, level), `apresentacao_conteudo` (the `conteudo` object and sending) and `apresentacao_figuras` (the figure programs). `/blueprint:apresentar` points to `apresentacao_ia`.
