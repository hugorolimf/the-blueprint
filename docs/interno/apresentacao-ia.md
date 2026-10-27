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

## Manual editing and the screen (2026-10-26)

The presentation is fully editable by hand. The viewer is isolated (opaque origin, no app access), so the editor lives in the app, beside the viewer, and edits the stored data.

- **Stored data stays the same** (`conteudo` plus `figuras`); an edit rewrites them. Metadata gain `editadoEm` (ISO time of the last manual edit; absent when the AI wrote the current version).
- `GET /api/mcp/apresentacao?id=<id>` returns `{id, titulo, publico, nivel, em, editadoEm?, conteudo, figuras}` (figures with `codigo`). Local app and team members who can edit; 404 `apresentacao-nao-encontrada`.
- `POST /api/mcp/apresentacao/editar` with `{apresentacaoId, conteudo?, figuras?}` (send only what changed; each sent part replaces that part whole). Same validation and the same error codes as `anexar`; `publico` and `nivel` come from `conteudo`; the metadata `titulo` follows `conteudo.titulo`; sets `editadoEm`; answers `{ok, apresentacaoId, editadoEm, avisos[]}` (the `avisos` are the same warnings as `anexar`: banned terms in `cliente`, unknown `figura`, empty `codigo`); emits the SSE event `apresentacao`. Loopback and roles like the other routes.
- The list route (`GET /api/mcp/apresentacoes`) and `ler_blueprint` carry `editadoEm`.
- **What can be edited**: title, subtitle, client, date, audience, level, language; every item of `resumo`, `entregas`, `etapas` (with `marcos`, `figura`, `situacao`), `precisamosDeVoce`, `decisoes`, `riscos` and, for `interno`, `interno.acoes`; each figure's `titulo`, `dica`, `leitura`, `chaves`, `legenda`, `rotulo` and `codigo` (advanced). Every list: add, remove, move up and down. Figures: duplicate and remove.
- **The editor** (web): sections that mirror the page, autosave after a pause with a visible state (saving, saved, error), the server warnings shown beside the field, a syntax check of `codigo` in the app page only (never run), Undo of the last change, and the viewer reloaded after each save keeping the section (`#` fragment, which the page restores).
- **The AI must not overwrite a manual edit by surprise**: regenerating (button or agent) when `editadoEm` exists asks first ("this replaces your edits"); the guide tells the agent the same.

Screen: when the blueprint has generated presentations the screen opens on the newest one (viewer plus toolbar: Edit, Present, Print, Export HTML, Regenerate, Remove). The rail lists the generated presentations and the "Generate with AI" group; the app's own types (Progress by project, Full plan) move to a collapsed group "Internal view". With none generated the screen opens as before, with "Generate with AI" first.

## Generating in the app terminal and talking to the agent (2026-10-26)

- **Generate runs in the app's own terminal** (the embedded ConPTY panel, `src/terminal/`): "Generate with AI" opens a tab running `blueprint claude "<prompt>" --canal` in the blueprint's folder, where the user watches and answers the agent. The dialog offers "App terminal" (default), a new external terminal, or an open session, like "Start with Claude". `POST /api/terminal/iniciar` accepts `{workspaceId, blueprintId, apresentar: {publico, nivel}, titulo?}` (the prompt is the same one `POST /api/mcp/apresentar` builds) and, to adjust, `{workspaceId, blueprintId, apresentacaoId, ajuste: "<text>", titulo?}`.
- **The agent edits the file itself, in small steps, never rewrites it.** Two MCP tools: `ler_apresentacao(workspace_id, blueprint_id, apresentacao_id, figuras_codigo?: [ids])` returns `conteudo` and the figures (without `codigo` unless that figure id is listed in `figuras_codigo`, to spare context), and `editar_apresentacao(workspace_id, blueprint_id, apresentacao_id, operacoes[])`.
- **Operations** (applied in order, all or none; the result is validated like `anexar`; each reply carries the warnings and the new `editadoEm`):
  - `{op: "definir", caminho: "etapas.2.titulo", valor: <json>}`: set a field (the path is dotted, list items by 0-based index; a missing last key is created);
  - `{op: "inserir", caminho: "etapas", indice: 3, valor: <json>}`: insert into a list (`indice` absent = append);
  - `{op: "remover", caminho: "etapas.2"}`;
  - `{op: "mover", caminho: "etapas.2", para: 0}`: move within its list;
  - `{op: "figura_trecho", figura: "capa", de: "<exact text>", para: "<text>"}`: replace one exact piece of a figure's `codigo`; the piece must occur exactly once, otherwise the whole call fails with the number of matches (no silent guess);
  - `{op: "figura_definir", figura: "capa", campo: "legenda", valor: "..."}`: set a figure field other than `codigo`; `{op: "figura_remover", figura: "capa"}`.
  Whole-part replacement (`conteudo`/`figuras` in `POST /api/mcp/apresentacao/editar`) remains for the manual editor's "restore" and for big pastes; the manual editor sends operations too, so a user edit and an agent edit of different fields never overwrite each other.
- `POST /api/mcp/apresentacao/editar` therefore accepts `{apresentacaoId, operacoes: [...], autor: "usuario" | "agente"}` as well; the metadata keep `editadoEm` and `editadoPor`.
- **The chat.** In the Apresentar screen, beside the viewer: a box "Talk to the agent" (text, Send). `POST /api/mcp/apresentacao/pedido` `{workspaceId, blueprintId, apresentacaoId, texto, sessaoId?}` delivers the message to the chosen session by its channel with the prefix «The Blueprint · pedido do usuário» (it is an order from the user's click, like Start with Claude); with no session it opens an app-terminal tab with `ajuste`. The prompt tells the agent: read with `ler_apresentacao`, change ONLY what was asked with `editar_apresentacao` (small operations, never `anexar_apresentacao` over an existing presentation unless the user asks to regenerate), confirm in one sentence what changed. The user can also type the request directly in the terminal where the agent is running.
- **Guide topic** `apresentacao_ajustar` (the protocol above, kept under 4,500 characters) joins `apresentacao_ia`, `apresentacao_conteudo` and `apresentacao_figuras`.
- The `editadoEm` guard stays: regenerating over a manually edited presentation asks first.

## One figure per section (2026-10-26)

- **The overview figure does not change.** The figure the AI already made for the cover (`capa`, now `secao: "visao"`) stays exactly as generated. The app's own tower figure is NOT brought back, and no structure is injected by the server (an earlier draft of this section proposed `estrutura` and `L.figuraBlueprint`; the user rejected it: only more figures are added).
- **Figures by section.** A figure may carry `secao`, one of `visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`, `acoes` (the section ids of the page). The page puts that figure in its section's panel, beside the text (below it on a narrow screen). A stage's own figure (`etapas[].figura`) keeps working and sits inside that stage. A figure with neither is shown at the end of Stages, as today.
- **Levels, corrected:** `rapido` = one AI figure (`visao`); `padrao` = one different AI figure per section (`visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`; `acoes` only for `interno`); `completo` = `padrao` plus one dense machine of the whole system in `visao` and one figure per stage (`etapas[].figura`). Each object is different from the others and helps to understand its section (index cards on a rail for what the client receives, a conveyor of stages, a calendar drum for the timeline, a request tray for what we need from the client, a pair of scales for decisions, a gauge cluster for risks).
- `validar` accepts and checks `secao` (a known id, otherwise the warning `secao-desconhecida`); it adds a warning when two figures claim the same section.

## Text and figure linked (2026-10-26)

The figure of a section is part of its text: the text refers to it and the two drive each other.

- **Items.** In each section the page lists items in `conteudo` order and gives each element `data-item="<i>"` (0-based) and `data-sessao="<secao>"`, focusable (`tabindex="0"`): `recebe` -> `entregas`, `etapas` -> `etapas`, `linha` -> `etapas` (the timeline rows), `precisamos` -> `precisamosDeVoce`, `decisoes` -> `decisoes`, `riscos` -> `riscos`. `visao` has no items.
- **The bridge.** `vivo(palco, dizer, ponte)` gets a third argument (it may be absent when a figure runs elsewhere: always guard `if (ponte)`). `ponte.total` is the number of items of its section. `ponte.liga({ selecionar(i) })` registers how the figure selects item `i`; the page calls `selecionar(i)` when the pointer enters, or the keyboard focuses, a text item, and `selecionar(null)` when it leaves (the figure goes back to its own tour). The figure calls `ponte.mudou(i)` (or `null`) whenever its current item changes by its own mouse, keys or tour; the page then marks the matching text item (`data-ativo`, `aria-current="true"`) and clears the previous one. A figure whose own position is not an item index maps it (the calendar drum maps an item to the start day of that stage and reports the stage of the current day).
- **The reference.** Under each section title that has a figure the page writes one line, `Fig n · <figure's dica>`, where `n` is the card number, and the `Fig n` is a link that scrolls to the figure and focuses it. The line says, in the page language, that an item of the text can be pointed at to see it in the figure.
- **Look.** The marked item uses the app's selected state (accent border and a left bar), no animation beyond the figure's own; reduced motion changes nothing else. Touch: a tap on an item selects it (no hover on touch); the figure's tour pauses while an item is selected by text.
- Figures written before this keep working (no `ponte` use); the guide teaches the bridge for new ones.
