# AI presentation: the figure programs

`figuras` is a list (at most 12). Each item: `id`, `titulo`, `secao` (the section of the page that holds it, see below), `dica` (the hover or key hint), `leitura` (the first readout text), `chaves` (`[{ "marca": "lit|raised|flat", "rotulo": "…" }]`), `legenda` (a true caption), `rotulo` (an accessible description of the object), `impressao` and `codigo` (at most 200 KB).

- **`impressao`**: a stable string for the data the figure explains (same data, same string; a compact join of the stage name, dates and plan titles is enough). The app only compares it to know whether to redraw.
- **`codigo`**: a JavaScript function expression as a string, called as `codigo(L, c)`: `L` is the library, `c` is your `conteudo` (`L.idiomaDe(c.idioma)` gives `"pt-BR"` or `"en"`; every text in the figure follows it). It returns `{ svg, vivo }`: `svg` is the full `<svg>` from `L.figureSvg`, `vivo(palco, dizer)` attaches motion to the stage element and writes the readout with `dizer("text")`. The page wraps it in the card. A figure that throws shows an error card; the rest of the page works. No network, storage or `eval`.
- **What `L` has**: the anatomy kit (`fitProjection`, `boxCorners`, `iso`, `slabOf`, `cylinder`, `solidSvg`, `lineSvg`, `dotsSvg`, `wireSvg`, `figureSvg`, `translateAlong`, `lineOnTop`, `topTicks`, `radialTicks`, `knurl`, `coil`, `haloOf`, the `lathe` family), the parts `L.pecas.base` (`{ regua: false }` drops its ruler strip, whose ticks end loose unless the figure uses it), `trilhos`, `ficha`, `botao`, `mostrador` (`{ svg, agulha(turn) }`), `placa`, the loop `L.laco(palco, passo)` (stops off screen and when settled; `passo(dt, calmo)` returns `true` to keep going), `L.segue`, `L.mola(state, target, dt, stiffness, damping, calmo)`, `L.calmo()` and `L.idiomaDe`. Keep it usable by keyboard (`keydown` on the stage: arrows, Home, End).

**The craft, short** (the anatomy skill has the long form):

- **One figure per section.** `secao` is one of `visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`, `acoes` (the last only for `interno`); the page puts the figure in that section's panel, beside the text. A stage's own figure (`etapas[].figura`) sits inside that stage; a figure with neither goes at the end of Stages. By level: `rapido` one figure (`visao`); `padrao` one per section; `completo` the same plus a dense machine of the whole system in `visao` and one per stage.
- Invent a real physical object whose working is the idea, a different one per figure, never boxes and arrows. Suggested: index cards on a rail for `recebe`, a conveyor of stages for `etapas`, a calendar drum for `linha`, a tray of requests for `precisamos`, a pair of scales for `decisoes`, a gauge cluster for `riscos`.
- 40 to 80 small precise parts; a bare face is unfinished. Painter's order (back to front, bottom to top), the lit tone on exactly one part, every line ends on a face, a line or a dot.
- App tones only (`hi`, `mid`, `lo`, `faint`, `lit`). No emoji, no words in the drawing beyond one tiny label. Calm time-based motion, still under reduced motion. Sample data only: a simulation, never real customer data.

A minimal working figure (extend it, do not ship it as is):

```js
function (L, c) {
  const pt = L.idiomaDe(c && c.idioma) === "pt-BR";
  const W = 600, H = 380, plano = { x: -100, y: -50, w: 200, d: 100, r: 8 };
  const P = L.fitProjection([...L.boxCorners(plano, -17, 0), [0, 0, 40]], W, H, { pad: 30, azimuth: 42 });
  const corpo = L.pecas.base(P, plano, { regua: false }) + L.pecas.trilhos(P, { de: -90, ate: 90, ys: [-20, 15] })
    + `<g data-peca="carro">${L.solidSvg(L.slabOf({ x: -20, y: -18, w: 40, d: 36, r: 3 }, 3, 8, P, 5, 0.6), { tone: "hi", lit: true })}</g>`;
  const svg = L.figureSvg({ width: W, height: H, label: "A sled on a rail", body: corpo });
  return { svg, vivo(palco, dizer) {
    const g = palco.querySelector('[data-peca="carro"]'), s = { x: 0, v: 0 };
    let t = 0;
    L.laco(palco, (dt, calmo) => {
      t += dt;
      L.mola(s, Math.sin(t * 0.8) * 60, dt, 120, 14, calmo);
      g.setAttribute("transform", L.translateAlong(P, [s.x, 0, 0]));
      dizer((pt ? "posição " : "position ") + s.x.toFixed(0));
      return true;
    });
  } };
}
```
