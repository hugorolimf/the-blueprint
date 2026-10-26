# GIFs do app na apresentação

[English](../../interno/gifs-na-apresentacao.md)

A apresentação pode mostrar um GIF curto do app numa página (por exemplo, o fluxo da proposta). Os GIFs vêm do gravador (`node scripts/gravar/index.mjs`), que grava `videos/*.gif` e `*.mp4` em inglês e em português do Brasil, com os dados de exemplo. A pasta `videos/` é ignorada pelo Git, então um GIF nunca faz parte do repositório nem do instalador.

## Como um GIF chega à página

- O app não lê `videos/` sozinho: essa pasta não é servida. O usuário escolhe o arquivo (um campo de arquivo dá à página um endereço `blob:`), ou a página já tem uma string `data:image/gif;base64,...`.
- `window.gifParaApresentacao(origem, { maxKb })` (em `web/js/apresentacao-figuras.js`) devolve, numa Promise, uma string `data:image/gif;base64,...` pronta para um `<img>`, ou `null` quando recusa.
- Embutir como data URI mantém a apresentação exportada em **um arquivo só**: sem pedido de fora, nada para levar junto.

## O que ela recusa

| Caso | Motivo |
| --- | --- |
| Maior que `maxKb` (padrão 1600 KB, contados nos bytes originais) | o base64 acrescenta cerca de um terço, e o arquivo exportado leva todos os GIFs |
| Não é GIF | confere-se a assinatura (`GIF87a` ou `GIF89a`), não o nome nem o tipo declarado |
| Endereço que não seja `blob:` nem a origem do próprio app | o app roda isolado; a apresentação nunca busca nada na rede |
| `data:` de outro tipo que não `image/gif`, `javascript:` e semelhantes | só GIF é aceito |
| Erro de rede ou resposta que não seja OK | a página simplesmente fica sem o GIF |

## Tamanhos esperados

Os GIFs gravados vão de cerca de 0,7 a 2,2 MB. Só alguns cabem no limite padrão; nos demais, grave um trecho mais curto ou suba o `maxKb` sabendo o que faz. Uma apresentação com três GIFs de 1,5 MB vira um arquivo de uns 6 MB.

## Privacidade

Grave só com os dados de exemplo (o gravador já faz isso). Um GIF de um blueprint real leva tudo o que estava na tela, e o arquivo exportado sai do app.
