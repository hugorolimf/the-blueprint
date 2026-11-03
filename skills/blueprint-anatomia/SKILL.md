---
name: blueprint-anatomia
description: Draws and changes The Blueprint's plan floor with our Anatomy engine in Rust (ui/src/anatomia) and shapes the plans that it draws. Use whenever the work touches the action's stage (ui/src/plano/acao_obra_*), a building, a belt, a piece, the Screen component and its variants, the zoom levels, a figure or an icon drawn in line, or when writing a blueprint plan that the user will review on that floor.
---

# Anatomy in The Blueprint

One skill for both halves of the same thing: the engine that draws the plan, and the plan that is drawn. A plan written without thinking of the floor draws a generic factory; a floor changed without thinking of the plan stops saying what the plan means.

## The world

One world, four levels, each one inside the other:

| Level | On screen | Code |
| --- | --- | --- |
| Blueprint | the sheet: a ladder of actions, in the order written (no dates, no durations) | `ui/src/folha/`, `plano/linha_bloco.rs` |
| Action | the stage: one plate, the whole width, its plans standing on it | `plano/acao_obra.rs`, `acao_obra_figura.rs` |
| Plan | a building, shaped by `forma`, numbered `1.1`, `1.2` | `acao_obra_formas.rs`, `acao_obra_cena.rs` |
| Piece | what the person decides, answers, sees or checks inside it: `1.1.1`, `1.1.2` | `acao_obra_pecas.rs`, `acao_obra_lista.rs` |

- **Numbers follow the parent.** Pieces of plan `1.1` are `1.1.1`, `1.1.2`; pieces of the whole action `2` are `2 · 1`, so they never look like plan `2.1`.
- **Dependencies are belts** (the idea comes from Factorio): `acao_obra_dutos.rs`. A belt leaves the plan that must come first and enters the one that waits, and the plan that waits shows its padlock only in the flag icon (`acao_obra_icones.rs`), never on the belt. One from another action ends on a numbered plate (`acao_obra_fora.rs`).
- **Time is the fourth dimension:** `acao_obra_tempo.rs` slides the stage through the building order.
- **Everything is on the stage.** There are no tabs and no cards: the plan's sheet (`acao_obra_ficha.rs`), the action's table (`acao_obra_mesa.rs`) and the board (`acao_obra_bordo.rs`) are written straight on the blueprint, with no box behind them. What is behind a column dissolves (`acao_obra_sumico.rs`: the column records where it is, the stage reads it on the next frame and fades each building, piece, belt, flag and plate by how much of it sits under the text). The plate and the paper stay: they are the same drawing.
- **The agent is on the building's label** (`acao_obra_rotulos.rs`, `acao_obra_sinal.rs`): a live figure next to the number, never standing on the building.
- **The current point is highlighted** with `tema::DESTAQUE`: the plan in work, or else the first one that can start (`acao_obra_regra::ponto_da_folha`). Its patch of floor, its label chip, the «Next» chip, the thumbnail and the action's name on the sheet all use it.

## Pieces

| Kind | Who chose | What the person does |
| --- | --- | --- |
| Question | nobody yet | picks an option or writes another |
| Decision | the agent | confirms, asks for an adjustment or disputes |
| Screen | the agent made it | looks at it, approves, asks for changes or rejects |
| Checkpoint | nobody | yes or no |

The stored review is the same for decision and screen (`revisao.rs`); what changes is what the person has in front of them. `acao_obra_lista.rs` draws them all as one list that opens in place, with the verbs (`revisao::verbos`) inside the open line, and goes to the next pending piece after an answer.

## The Screen component

`acao_obra_tela.rs` is the only way a screen appears on the floor: the real figure in a frame, standing in its plan's building, colored by its review.

- **Variants** live in `CATALOGO` (`janela`, `celular`, `dialogo`, `faixa`, `quadro`). A plan asks for one with `variante` on the detail; without it, the figure's proportion chooses. A new variant is one line in `CATALOGO`, the same name in `VARIANTES` (`src/plano/validar.rs`) and in the guide (`skills/blueprint-planejamento/topicos/plano.md`, `src/mcp/ferramentas.json`).
- **Zoom levels of a plan**, one more click or more wheel each, and back the same way: (1) the building comes close and its sheet opens (`acao_obra_mergulho.rs`); (2) the expanded view, opened on the stage itself (`acao_obra_explosao.rs`; the layers are `acao_partes_explodida.rs`, the callouts `acao_partes_chamada.rs`): the chosen building separates into layers where it stands, with the stage camera, and the other buildings fade (`apagar`); the column on the right is the same sheet, only fuller, with the plan's tables; the page height does not change between levels, and there is no button to open it; (3) inside a screen (`acao_obra_na_tela.rs`): the camera faces it (azimuth 90, elevation 0) until it fills the stage, with its own bar (back, previous and next screen, number, review). Wheel back below each level's threshold, Esc or «Back» returns to the level before; leaving a screen returns to the expanded view with the layers still open. Layer click keys start at `1 << 27` (`CHAVE_TOPO`), above buildings, pieces and belts.
- **The image never leaves its component.** No panel or sheet shows the figure itself: they offer «View on screen».
- A screen without a figure is an empty state that says so, not a hole.

## The engine (`ui/src/anatomia/`)

It came from the JavaScript kit (`kit/iso-kit`, `kit/lathe`, see `NOTICE.md`): same ideas, now live, in the app's theme.

| File | What it gives |
| --- | --- |
| `vetor.rs` | `V3 = [x, y, z]`, `z` up; `soma`, `sub`, `vezes`, `dot`, `cruz`, `unit`, `mistura` |
| `camera.rs` | `Camera::nova(azimute, elevacao, escala, foco, origem, perspectiva)`, `enquadrar(pontos, rect, pad, az, el)`, `girada`, `rumo(&outra, t)` to animate, `viva(ctx, id, alvo, seg)`; `ponto`, `profundidade`, `aumento`, `alcanca`. `AZIMUTE` 45, `ELEVACAO` 30. Perspective 0 is the kit's axonometric view |
| `malha.rs` | `Malha::de_faces(vertices, aneis)`; each frame it finds the outline (one face seen, one turned away) and the creases (both seen, sharp angle). `desenhar`, `contem`, `na_face`; `imagem_na_face` glues a texture on a quad, in pieces so perspective does not bend it |
| `solidos.rs` | `extrusao`, `caixa`, `laje` (rounded corners), `chapa` (a plate with holes, for the floor), `alca` (a handle arch), `cilindro`, `torno(centro, z, perfil, Ponta)`, `esfera`, `viga` |
| `tinta.rs` | `Tinta::de(escura, clara, traco)`, `parede`, `chao`, `estado(cor)`, `fantasma(cor)`, `com_alfa`, `com_traco`. Light comes in bands, never a gradient: that is what keeps it a drawing |
| `cena.rs` | `Cena`: `solido(camada, malha, tinta, chave)`, `solido_folgado`, `fio`, `extra`, `grupo`; `desenhar` paints back to front by `Camada` (`Base`, `Chao`, `Dutos`, `DutosTopo`, `Pecas`, `Acima`), then depth; `tocar(cam, pos)` returns the key under the mouse (0: nothing) |

A building, in short:

```rust
let mut cena = Cena::default();
cena.solido(Camada::Chao, Malha::laje([x, y, 0.0], [w, d, 0.4], 0.6), Tinta::chao(), 0);
cena.solido(Camada::Pecas, Malha::caixa([x + 1.0, y + 1.0, 0.4], [w - 2.0, d - 2.0, h]), Tinta::estado(cor), chave);
let cam = Camera::enquadrar(&pontos, area, 24.0, AZIMUTE, ELEVACAO);
cena.desenhar(&cam, ui.painter(), |n| { /* the caller's extras: labels, the agent figure */ });
let tocada = cena.tocar(&cam, ponteiro); // see acao_obra_toque.rs
```

## Rules that keep the identity

- Theme from `ui/src/tema.rs` only: `PAPEL`, `TINTA`, `MEDIO`, `BAIXO`, `TENUE`, `CAMPO`, `QUADRA`; `DESTAQUE` (amber) only as a fill, a mark or text, never as a UI border.
- Cascadia Mono, the buttons of `planta::Botao`, no cards, no orange borders. Improve without changing font, buttons or frame; compare against `ui/src/folha/`.
- Figures and icons are drawn with the engine (`acao_obra_icones.rs`), never emojis or bitmaps.
- Every visible text through `tr()`, with its English entry (`plano/acao_textos.rs` and the catalog).
- Motion with `acao_movimento::mistura(ctx, id, alvo, seg)`: eased, short, only where it shows something real.
- Text size is not zoom: zooming changes the camera, never the font.
- Files up to 350 lines (ceiling 500 with the reason at the top), one responsibility each. The stage is split by job: when one grows, split it the same way.
- Do not run `cargo test`, `cargo build` or open the app unless the user asks in the task.

## Planning so the floor says something

When you write the plan (topics `anatomia` and `plano` of `guia_de_planejamento`):

- One plan, one thing that exists when it is done (`entrega`), with the `forma` of what it builds.
- Only real dependencies: plans that do not wait for each other stand side by side and run at the same time.
- Put `plano` on every decision, question, screen, table and checkpoint, so it sits inside its building.
- A screen is this project's real screen, as SVG in the app's theme, with the `variante` that fits it.
- Ask for approval only where the answer changes what gets built.
