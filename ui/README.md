# Interface nativa (egui)

A interface do The Blueprint em egui: o mesmo código abre uma janela no desktop (Windows, Linux) e roda no navegador (WebAssembly).
É cliente do servidor do Blueprint (`blueprint serve` ou o app aberto): lê e grava tudo em `/api/dados`.
Fora do pacote da raiz de propósito: o `cargo test` e o CI de lá não compilam esta pasta (ainda).

```
cd ui
BLUEPRINT_URL=http://127.0.0.1:3000 cargo run      # a janela (BLUEPRINT_URL é opcional)
trunk serve --release                              # o navegador, em http://127.0.0.1:8088 (o /api passa para a porta 3000)
cargo test                                         # regras dos dados, larguras do texto e o inglês de cada texto
```

## De onde vem o desenho

O desenho segue a página (`web/css/entrada.css`, `primeiro-uso.css`): as medidas são as da tela web, em pixels do CSS.

- **Fonte:** JetBrains Mono (OFL) embutida em `assets/fontes/`; a página usa a monoespaçada do sistema, que aqui é essa.
- **Emblemas:** SVG exportados do kit com `node scripts/exportar-emblemas.mjs` (as letras viram contornos), em `assets/emblemas/`.
- **Inglês:** o catálogo da página, exportado com `node scripts/exportar-i18n.mjs` para `assets/i18n-en.json`. O teste `todo_texto_do_app_tem_ingles` falha se um `tr("…")` ficar sem tradução.
- **Texto:** o egui arredonda o avanço de cada letra; `planta::Texto` devolve a diferença no espaçamento, e o teste `largura_do_texto_bate_com_a_pagina` confere as larguras contra a página.

## Testar sem olhar a tela (só desktop)

Variáveis de ambiente, para rodar o app sem mouse e comparar com a página:

| Variável | O que faz |
| --- | --- |
| `BLUEPRINT_UI_CAPTURA=<arquivo.png>` | salva uma imagem da janela e fecha (`BLUEPRINT_UI_ESPERA=<s>` muda os 2 s de espera) |
| `BLUEPRINT_UI_ROTEIRO=nome:Ana,criar:Alfa,exemplo,mover:Alfa,excluir:Alfa,voltar,novo` | roda as ações em ordem (também `abrir:ws`, `novobp:ws` abre «Novo blueprint», `criarbp:ws/Nome` cria e abre) |
| `BLUEPRINT_UI_ABRIR=novo` | abre direto a tela «Novo workspace» |
| `BLUEPRINT_UI_PPP=1` | força a escala (o monitor pode ser 1,25) para comparar pixel a pixel com a página |
| `BLUEPRINT_UI_APPID=<id>` | muda o `app_id` da janela (para uma regra do compositor mandá-la para outra área) |

## O que já existe

Nome, workspaces (cartões, arrastar para reordenar, Alt+setas, excluir pelo menu de contexto, novo com marca), primeiros passos e o exemplo.
A folha do blueprint e o resto ainda são da página: o workspace aberto mostra a lista e um botão «Abrir na versão web».
Ficam para depois: login hospedado e convite, a arte por texto ou JSON no novo workspace, e a versão estreita (menos de 650 px).
