# AI presentation: the figure programs

`figuras` is a list (at most 12). Each item: `id`, `titulo`, `secao` (the section of the page that holds it, see below), `dica` (hint), `leitura` (first readout text), `chaves` (`[{ "marca": "lit|raised|flat", "rotulo": "…" }]`), `legenda` (a true caption), `rotulo` (accessible description), `impressao` and `codigo` (up to 200 KB).

- **`impressao`**: a stable string for the data the figure explains; the app compares it to know whether to redraw.
- **`codigo`**: a JavaScript function expression as a string, called as `codigo(L, c)`: `L` is the library, `c` is your `conteudo` (`L.idiomaDe(c.idioma)`: `"pt-BR"` or `"en"`; every text follows it). It returns `{ svg, vivo }`: `svg` is the full `<svg>` from `L.figureSvg`; `vivo(palco, dizer, ponte)` attaches motion to the stage element and writes the readout with `dizer(text)`. A figure that throws shows an error card. No network, storage or `eval`.
- **What `L` has**: the anatomy kit (`fitProjection`, `boxCorners`, `slabOf`, `cylinder`, `solidSvg`, `lineSvg`, `figureSvg`, `translateAlong`, `lathe`…), the parts `L.pecas.base` (`{ regua: false }` drops its ruler strip), `trilhos`, `ficha`, `mostrador`, the loop `L.laco(palco, passo)` (stops off screen; `passo(dt, calmo)` returns `true` to keep going), `L.mola(state, target, dt, stiffness, damping, calmo)`, `L.idiomaDe`.
- **The bridge** links the figure to its section's text. `ponte` is the 3rd argument and may be absent: **always guard `if (ponte)`**. `ponte.total` counts the section's items in `conteudo` order: `recebe` = `entregas`, `etapas` and `linha` = `etapas`, `precisamos` = `precisamosDeVoce`, `decisoes`, `riscos`. `ponte.liga({ selecionar(i) })` says how the figure points at item `i`: the page calls `selecionar(i)` when the pointer or focus reaches a text item and `selecionar(null)` when it leaves (back to your own tour). Call `ponte.mudou(i)` (or `null`) whenever the current item changes by mouse, keys or tour: the page marks the text item (and writes `Fig n ·` under the title). If your position is not an item index (a drum shows days), map it both ways.

**Craft:**

- **One figure per section.** `secao`: `visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`, `acoes` (only `interno`). A stage's own figure (`etapas[].figura`) sits inside that stage. `rapido`: one (`visao`); `padrao`: one per section; `completo`: those plus a dense machine of the whole system in `visao` and one per stage.
- A real physical object whose working is the idea, a different one per figure, never boxes and arrows: cards on a rail (`recebe`), a conveyor of stages (`etapas`), a calendar drum (`linha`), a tray of requests (`precisamos`), scales (`decisoes`), a gauge cluster (`riscos`).
- 40 to 80 small parts, painter's order, one lit part, every line ends on something. App tones only; no emoji; calm motion, still when reduced. Sample data only.

A minimal `etapas` figure (extend it, do not ship it as is):

```js
function (L, c) {
  const plano = { x: -100, y: -50, w: 200, d: 100, r: 8 };
  const P = L.fitProjection([...L.boxCorners(plano, -17, 0), [0, 0, 40]], 600, 380, { pad: 30, azimuth: 42 });
  const corpo = L.pecas.base(P, plano, { regua: false }) + L.pecas.trilhos(P, { de: -90, ate: 90, ys: [-20, 15] })
    + `<g data-peca="carro">${L.solidSvg(L.slabOf({ x: -20, y: -18, w: 40, d: 36, r: 3 }, 3, 8, P, 5, 0.6), { tone: "hi", lit: true })}</g>`;
  return { svg: L.figureSvg({ width: 600, height: 380, label: "A sled on a rail", body: corpo }), vivo(palco, dizer, ponte) {
    const g = palco.querySelector('[data-peca="carro"]'), s = { x: 0, v: 0 };
    const n = Math.max(2, (ponte && ponte.total) || 3), pos = (i) => -60 + (120 * i) / (n - 1);
    let t = 0, atual = -1, manual = null;
    if (ponte) ponte.liga({ selecionar(i) { manual = i; } });
    L.laco(palco, (dt, calmo) => {
      t += dt;
      L.mola(s, pos(manual != null ? manual : Math.floor(t / 2.5) % n), dt, 120, 14, calmo);
      g.setAttribute("transform", L.translateAlong(P, [s.x, 0, 0]));
      const i = Math.round(((s.x + 60) / 120) * (n - 1));
      if (i !== atual) { atual = i; if (ponte) ponte.mudou(i); }
      dizer((c.idioma === "en" ? "stage " : "etapa ") + (atual + 1));
      return true;
    });
  } };
}
```
