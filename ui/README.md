# Interface nativa (egui)

A interface do The Blueprint em egui: o mesmo código abre uma janela no desktop (Windows, Linux) e roda no navegador (WebAssembly).
É cliente do servidor do Blueprint (`blueprint serve` ou o app aberto): lê e grava os dados em `/api/dados`, e o Planejar conversa por `/api/mcp/planejar/*` e `/api/mcp/gesto/*`.
Fora do pacote da raiz de propósito: o `cargo test` e o CI de lá não compilam esta pasta (ainda).

```
cd ui
BLUEPRINT_URL=http://127.0.0.1:3000 cargo run      # a janela (BLUEPRINT_URL é opcional)
trunk serve --release                              # o navegador, em http://127.0.0.1:8088 (o /api passa para a porta 3000)
cargo test                                         # regras dos dados, larguras do texto e o inglês de cada texto
```

## De onde vem o desenho

O desenho segue a página (`web/css/entrada.css`, `primeiro-uso.css`): as medidas são as da tela web, em pixels do CSS.

- **Fonte:** Cascadia Mono (OFL) embutida em `assets/fontes/`, nos pesos 400, 500 e 700 tirados da fonte variável: é a que a página mostra no Windows (`ui-monospace, "Cascadia Mono", …`).
- **Emblemas:** SVG exportados do kit com `node scripts/exportar-emblemas.mjs` (as letras viram contornos), em `assets/emblemas/`.
- **Inglês:** o catálogo da página, exportado com `node scripts/exportar-i18n.mjs` para `assets/i18n-en.json`. O teste `todo_texto_do_app_tem_ingles` falha se um `tr("…")` ficar sem tradução.
- **Texto:** o egui arredonda o avanço de cada letra; `planta::Texto` devolve a diferença no espaçamento, e o teste `largura_do_texto_bate_com_a_pagina` confere as larguras contra a página.

## Testar sem olhar a tela (só desktop)

Variáveis de ambiente, para rodar o app sem mouse e comparar com a página:

| Variável | O que faz |
| --- | --- |
| `BLUEPRINT_UI_CAPTURA=<arquivo.png>` | salva uma imagem da janela e fecha (`BLUEPRINT_UI_ESPERA=<s>` muda os 2 s de espera) |
| `BLUEPRINT_UI_ROTEIRO=nome:Ana,criar:Alfa,exemplo,mover:Alfa,excluir:Alfa,voltar,novo` | roda as ações em ordem (também `direito:x/y` clica com o botão direito, `abrir:ws`, `novobp:ws` abre «Novo blueprint», `criarbp:ws/Nome` cria e abre, `pausa:ms` espera antes do próximo, para ver uma animação no meio; no Planejar, `blueprint:ws/bp` abre a folha, `abrir-acao:ref` abre uma ação, `marcar-plano:id-ou-ref` marca um plano para iniciar e `agentes` abre a tela de agentes; `mouse-voltar` e `mouse-avancar` são os botões laterais do mouse) |
| `BLUEPRINT_UI_ABRIR=novo` | abre direto a tela «Novo workspace» |
| `BLUEPRINT_UI_PPP=1` | força a escala (o monitor pode ser 1,25) para comparar pixel a pixel com a página |
| `BLUEPRINT_UI_APPID=<id>` | muda o `app_id` da janela (para uma regra do compositor mandá-la para outra área) |

## O que já existe

Nome, workspaces (cartões, arrastar para reordenar, Alt+setas, excluir pelo menu de contexto, novo com marca) e a faixa «Remover exemplo».
O menu do botão direito (`src/menu/`) existe em todas as telas: cada tela marca o que está sob o cursor com `menu::marcar`, e o que depende de tela ainda não migrada (editar, a tela de proposta, conversar, iniciar com o Claude) vira «Abrir na versão web».
O blueprint aberto é o Planejar (`src/plano/`, [docs/planejar.md](../docs/planejar.md)): a folha, a ação aberta, o que chegou, os agentes e o detalhe, com os dados e as regras no servidor. A folha antiga (`src/folha/blueprint.rs`) continua atrás do botão «Folha antiga» e do menu do botão direito, porque período, Pendentes e busca só existem lá, e é para ela que a tela cai se o servidor não tiver a rota `planejar/folha`.
Os botões laterais do mouse voltam e avançam pelos lugares da interface (`src/historico.rs`): telas, ação e detalhe abertos. Quem muda de tela não precisa avisar, porque o histórico observa `Vista` a cada quadro.
Ficam para depois: login hospedado e convite, e a versão estreita (menos de 650 px).
