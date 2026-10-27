# Apresentações com IA: contrato (interno)

[English](../../interno/apresentacao-ia.md)

Estado: contrato de desenho, 2026-10-26. Blueprint App, projeto "Apresentacao", ações "Niveis e publicos das apresentacoes", "Gerar as apresentacoes do Atendimento" e "Gerar com IA na tela e no terminal". Este arquivo é a fonte única que as três frentes de trabalho (biblioteca, Rust e web) seguem. Mude-o primeiro, depois o código.

## O que é

O usuário escolhe um **público** (`cliente` | `interno`) e um **nível** (`rapido` | `padrao` | `completo`; `epico` só quando pedido pelo nome), no diálogo Apresentar ou numa conversa no terminal (`/blueprint:apresentar`). O agente escreve um **JSON de conteúdo** e um **programa de figura** por figura; o app monta uma página HTML autônoma e isolada (sandbox) com uma biblioteca embutida. A página é uma interface de apresentação de verdade, no visual do próprio app (papel de plotagem azul, tipografia mono, cabeçalhos em caixa, navegação por seções), com figuras vivas do anatomy como parte da interface.

Por que dados mais código, e não um HTML inteiro vindo do agente: uma figura densa tem de 0,3 a 1 MB de SVG, demais para uma chamada de ferramenta; o programa da figura tem de 5 a 30 KB porque o navegador monta o SVG com o kit. Não é preciso Node na máquina do usuário, e um projeto cujo conteúdo não mudou mantém a figura (veja `impressao`).

## Público

- `cliente`: um PLANO DE PROJETO. Mostra o que o cliente recebe, as etapas com datas, o que se espera do cliente, as decisões em aberto e os riscos, em linguagem simples. Nunca mostra tecnologia, rotas, arquivos, horas por plano nem as instruções internas das ações. Nada que o blueprint não diga (sem preços, funcionalidades ou datas inventados).
- `interno`: a visão técnica completa para o dono.

`apresentacao-ia/termos-tecnicos.json` lista os termos que não podem aparecer numa apresentação `cliente` (minúsculas, casados como palavras inteiras ou prefixos de palavra). A biblioteca (`auditarCliente`) e a ferramenta MCP usam a mesma lista; um achado é um aviso na resposta da ferramenta, nunca silencioso.

## Níveis

| Nível | Figuras | Tamanho no anatomy | Estimativa (anunciada antes de começar) |
| --- | --- | --- | --- |
| `rapido` | 1 (a capa) | Figure, 40 a 60 peças | até cerca de 1 h |
| `padrao` | 1 por grupo de etapas, todas objetos diferentes | várias Figures | cerca de 1 a 3 h |
| `completo` | o conjunto `padrao` mais uma máquina densa do sistema inteiro | Hero, 80 a 150 peças | cerca de 2 a 4 h |
| `epico` | só a pedido pelo nome | Epic | 6 h ou mais |

## Modelo de dados

`conteudo` (JSON, no máximo 128 KB):

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
  "resumo": ["frase curta", "..."],
  "entregas": [{ "titulo": "string", "texto": "string" }],
  "etapas": [{ "id": "e1", "titulo": "string", "inicio": "yyyy-mm-dd", "fim": "yyyy-mm-dd", "texto": "string",
               "figura": "figura-id | null", "marcos": [{ "data": "yyyy-mm-dd", "nome": "string" }] }],
  "precisamosDeVoce": [{ "titulo": "string", "texto": "string", "ate": "yyyy-mm-dd | null" }],
  "decisoes": [{ "titulo": "string", "texto": "string", "estado": "aberta | decidida" }],
  "riscos": [{ "titulo": "string", "texto": "string" }],
  "interno": { "acoes": [{ "numero": "string", "nome": "string", "planos": ["string"] }] }
}
```

`interno` existe só quando `publico` é `interno`. `idioma` segue o idioma do app no momento da geração.

`figuras` (lista, no máximo 12, cada `codigo` com no máximo 200 KB):

```json
{ "id": "fichas", "titulo": "string", "dica": "Passe o mouse para escolher uma ficha", "leitura": "ficha 07 · em dia",
  "chaves": [{ "marca": "lit", "rotulo": "string" }], "legenda": "string", "rotulo": "descrição acessível",
  "impressao": "sha256 em hexadecimal do conteúdo que esta figura explica",
  "codigo": "function (L) { /* monta */ return { svg: '<svg…>', vivo: function (palco, dizer) { … } }; }" }
```

`codigo` é uma expressão de função JS. Recebe `L` (a biblioteca, abaixo) e devolve `{ svg, vivo? }`; `svg` é o `<svg>` completo de `L.figureSvg`; `vivo(palco, dizer)` liga o movimento ao elemento do palco (`palco`) e escreve a leitura com `dizer(texto)`. A página envolve cada figura no cartão do kit (`plateHtml`: `Fig n`, título, dica, leitura, chaves, legenda). Uma figura que lança erro mostra um cartão de erro com a mensagem; o resto da página continua funcionando.

## Biblioteca `L` (navegador, `apresentacao-ia/biblioteca/`)

Um conjunto de scripts globais (scripts clássicos, sem módulos), cada arquivo com no máximo 350 linhas:

- o kit do anatomy como está (`kit/iso-kit` e `kit/lathe`, todos os exports em `L`: `fitProjection`, `iso`, `slabOf`, `cylinder`, `solidSvg`, `lineSvg`, `dotsSvg`, `wireSvg`, `figureSvg`, `plateHtml`, `translateAlong`, `lineOnTop`, `topTicks`, `radialTicks`, `knurl`, `coil`, `haloOf`, `depthOf`, …);
- `L.pecas`: peças reutilizáveis (placa de base com pés, parafusos e régua; trilhos; fichas; mostrador com agulha; botões; plaquinhas), cada uma devolvendo uma string de SVG;
- `L.laco(palco, passo)`: um laço de quadros por figura, parado fora da tela (IntersectionObserver, vale a última entrada) e quando assentado, com um único salto sob `prefers-reduced-motion`; `passo(dt, calmo)` devolve true para continuar;
- `L.segue(valor, alvo, dt, tau)` e `L.mola(estado, alvo, dt, rigidez, amortecimento, calmo)`;
- `L.auditarCliente(conteudo)`: auditoria dos termos proibidos; `L.impressao(objeto)`: hash estável de um valor JSON (sha256 em hexadecimal, síncrono).

## Montagem da página

`apresentacao-ia/modelo.html` é um modelo com estes marcadores, trocados por substituição simples de texto (para o Rust poder fazer): `{{titulo}}`, `{{idioma}}`, `{{css}}` (os tokens do app mais o CSS da apresentação mais `ISO_CSS`), `{{kit}}` (os scripts do kit, embutidos), `{{biblioteca}}` (os scripts da biblioteca), `{{conteudo}}` (o JSON de conteúdo, com `</` escrito como `<\/`), `{{figuras}}` (um array JSON das figuras, com o mesmo escape). O script da página monta a interface a partir de `conteudo`, executa cada figura e a monta no seu palco. Layout da tela: barra superior com migalhas e título em caixa, navegação por seções (visão geral, etapas, linha do tempo, o que precisamos de você, decisões e riscos), capa com a figura principal, painéis com cabeçalho e régua, progresso do plano, folha de estilo de impressão A4. O público `interno` acrescenta uma seção de Ações. Sem rede, sem armazenamento, sem escrita em `localStorage`.

## Armazenamento, MCP e HTTP (Rust)

- Ferramenta MCP `anexar_apresentacao(workspace_id, blueprint_id, conteudo, figuras, apresentacao_id?)`. `conteudo` e `figuras` são objetos/arrays (não strings). Substitui quando `apresentacao_id` é dado. Limites acima; no total, no máximo 2 MB. Responde com o id, um caminho de visualização e os avisos (termos proibidos em `cliente`, uma referência `figura` desconhecida, uma figura sem `codigo`).
- Armazenamento: `%APPDATA%\Blueprint\apresentacoes\<id>.json` (o conteúdo mais as figuras); o blueprint guarda só os metadados `apresentacoes: [{id, titulo, publico, nivel, em, figuras: [{id, impressao}]}]`. Nunca no repositório Git.
- `GET /apresentacao/<id>`: a página montada, com os mesmos cabeçalhos de sandbox de `/prototipos/<id>.html` (`Content-Security-Policy: sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:`), nunca `allow-same-origin`. `GET /apresentacao/<id>?baixar=1` acrescenta `content-disposition: attachment`.
- `POST /api/mcp/apresentacao` (vindo do processo MCP) e `POST /api/mcp/apresentacao/remover`, só loopback, só JSON.
- `POST /api/mcp/apresentar` `{workspaceId, blueprintId, publico, nivel, sessao?}`: o pedido da tela. Mesmo caminho e mesmas regras de `POST /api/mcp/iniciar` (só por clique do usuário, só loopback, sessão com canal ou uma nova). O prompt leva público, nível e a estimativa, para o agente não perguntar de novo.
- `/blueprint:apresentar`: uma skill do plugin (só o usuário chama) que inicia a conversa, como `/blueprint:status`.
- A lista de ferramentas, as instruções do MCP (os primeiros 512 caracteres guardam a regra e os nomes das ferramentas; acrescente `anexar_apresentacao` sem quebrar esse teste) e o tópico `apresentar` do guia documentam tudo isso.

## Interface web (`web/`)

No diálogo Apresentar que já existe (`web/js/apresentacao.js`): um segundo grupo no trilho lateral, "Gerar com IA": público (Cliente, Interno), nível (Rápido, Padrão, Completo), cada um com a sua estimativa de tempo, `Epico` nunca listado, e um botão Gerar que chama `/api/mcp/apresentar`. Quando o blueprint tem apresentações geradas, a área de prévia as lista (título, público, nível, data), mostra a escolhida num `<iframe sandbox="allow-scripts" src="/apresentacao/<id>">` e oferece Gerar de novo, Exportar HTML (`?baixar=1`) e Remover. Todo texto novo passa por `tr()` com entrada em inglês (`node scripts/i18n-completude.mjs`). Arquivos com menos de 350 linhas, nenhum trabalho em repouso.

## Regras que valem em todo lugar

Sem emoji. As figuras usam os tons do próprio app. A página gerada e os dados guardados nunca são commitados. O público `cliente` nunca leva termos técnicos, horas por plano, instruções internas nem promessas inventadas. Decisões em aberto aparecem como "a definir". Dados de exemplo nos testes do repositório; o conteúdo real do cliente fica fora do repositório.
