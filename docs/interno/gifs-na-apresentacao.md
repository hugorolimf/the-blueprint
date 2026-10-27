# GIFs of the app in the presentation

[Português](../pt-BR/interno/gifs-na-apresentacao.md)

The presentation can show a short GIF of the app on a page (for example the proposal flow). The GIFs come from the recorder (`node scripts/gravar/index.mjs`), which writes `videos/*.gif` and `*.mp4` in English and Brazilian Portuguese, recorded with the sample data. `videos/` is ignored by Git, so a GIF is never part of the repository or of the installer.

## How a GIF gets onto a page

- The app does not read `videos/` by itself: that folder is not served. The user picks the file (a file input gives the page a `blob:` address), or the page already holds a `data:image/gif;base64,...` string.
- `window.gifParaApresentacao(origem, { maxKb })` (in `web/js/apresentacao-figuras.js`) returns, in a Promise, a `data:image/gif;base64,...` string ready for an `<img>`, or `null` when it refuses.
- Embedding as a data URI keeps the exported presentation a **single file**: no outside request, nothing to carry along.

## What it refuses

| Case | Why |
| --- | --- |
| Larger than `maxKb` (default 1600 KB, counted on the raw bytes) | base64 adds about a third, and the exported file carries every GIF |
| Not a GIF | the signature (`GIF87a` or `GIF89a`) is checked, not the file name or the declared type |
| Any address that is not `blob:` or the app's own origin | the app runs isolated; the presentation never pulls anything from the network |
| `data:` types other than `image/gif`, `javascript:` and the like | only GIF is accepted |
| A network error or a non-OK answer | the page simply shows no GIF |

## Sizes to expect

The recorded GIFs go from about 0.5 to 2.5 MB (the first 12 s of each video). Only some fit the default limit; for the rest, record a shorter clip or raise `maxKb` knowingly. A presentation with three GIFs of 1.5 MB becomes a file of about 6 MB.

## Privacy

Record with the sample data only (the recorder already does). A GIF of a real blueprint carries whatever was on screen, and the exported file leaves the app.
