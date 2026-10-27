# AI presentation: the `conteudo` object and sending it

A JSON object, at most 128 KB. Everything is plain text, no HTML, no emoji. Write it for the audience in `apresentacao_ia`.

```json
{ "versao": 1, "publico": "cliente", "nivel": "padrao", "idioma": "pt-BR",
  "titulo": "…", "subtitulo": "…", "cliente": "…", "data": "yyyy-mm-dd",
  "resumo": ["two to four short sentences: what the client receives"],
  "entregas": [{ "titulo": "…", "texto": "…" }],
  "etapas": [{ "id": "e1", "titulo": "…", "inicio": "yyyy-mm-dd", "fim": "yyyy-mm-dd", "texto": "…",
               "figura": "figura-id or null", "marcos": [{ "data": "yyyy-mm-dd", "nome": "…" }] }],
  "precisamosDeVoce": [{ "titulo": "…", "texto": "…", "ate": "yyyy-mm-dd or null" }],
  "decisoes": [{ "titulo": "…", "texto": "…", "estado": "aberta" }],
  "riscos": [{ "titulo": "…", "texto": "…" }],
  "interno": { "acoes": [{ "numero": "1.2", "nome": "…", "planos": ["…"] }] } }
```

- `interno` only when `publico` is `interno`; with `cliente` the tool refuses it (`interno-no-cliente`), because it would leak.
- `idioma` is the language the user speaks to you; every text follows it.
- Group the blueprint's actions into 3 to 7 stages (a stage can span several actions). Every date comes from the blueprint; nothing is invented.
- `figura` points to the `id` of one of your figures (see `apresentacao_figuras`); a stage without a figure is fine. The figure of a section is not referenced here: it carries its own `secao`.
- `decisoes`: open ones are `"estado": "aberta"` and show as "to be decided".

## Sending it

Call `anexar_apresentacao(workspace_id, blueprint_id, conteudo, figuras, apresentacao_id?)` with **objects and arrays, never JSON strings**. Limits: content 128 KB, at most 12 figures, each `codigo` 200 KB, total 2 MB. The reply has `avisos`: a banned term, an unknown `figura` reference, a figure without `codigo` or `impressao`. Fix them and send again with the id the reply returned (that replaces, it does not duplicate). Then tell the user where to open it and what you left out. Errors come with a stable `codigo` (for example `publico-invalido`, `figura-id-repetido`, `conteudo-grande`).

With the app closed the tool does not store anything: tell the user to open it and send again.
