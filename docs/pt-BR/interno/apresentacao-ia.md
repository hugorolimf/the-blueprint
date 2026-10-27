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

## Notas de implementação (adotadas, 2026-10-26)

O código diverge do texto acima nestes pontos; o código é a verdade.

- **Metadados e lista.** Não há `blueprint.apresentacoes` nos dados do blueprint nem migração: os metadados ficam em cada `<id>.json` e a lista é lida da pasta (`%APPDATA%\Blueprint\apresentacoes\`, ou dentro da pasta de protótipos quando ela é movida). A web lê `GET /api/mcp/apresentacoes?workspace=&blueprint=` (da mais nova) e o evento SSE `apresentacao` `{workspaceId, blueprintId, apresentacaoId}`; `ler_blueprint` também devolve `apresentacoes`. No modo equipe elas não sincronizam entre máquinas.
- **Pedido.** `POST /api/mcp/apresentar` recebe `{workspaceId, blueprintId, publico, nivel, sessaoId?, semCanal?}` (como `/iniciar`; não `sessao`) e responde `{ok, pasta, terminal: wt|cmd|canal|copiar, prompt?}`. Só no app local: a rota não existe no modo hospedado nem para convidados, e `MCP_REDE=1` não a abre. `remover` recebe `{apresentacaoId}` e é idempotente.
- **Chamada da figura.** `codigo(L, c)`: a biblioteca também recebe `conteudo` (`L.idiomaDe(c.idioma)`). `impressao` só precisa ser um texto não vazio; a ferramenta avisa quando falta. `ler_blueprint` dá a cada projeto uma `impressao` (`projetos[].impressao`, `src/blueprint/impressao.rs`: 16 caracteres hexadecimais do SHA-256 do nome e da descrição do projeto e, das ações dele, nomes, datas, situação, títulos dos planos e checkpoints; marcar um plano feito ou mudar as instruções internas não a altera); a figura de um projeto a guarda.
- **Auditoria.** Os termos proibidos casam por palavra inteira (não por prefixo: `cli` acusaria "cliente"); os campos de identificador (`publico`, `nivel`, `id`, `figura`, datas, `estado`) são pulados; horas como "3 h" ou "1,5 horas" também avisam. `interno` dentro de uma apresentação `cliente` é erro (`interno-no-cliente`), não aviso.
- **Página.** Um `<meta>` de CSP sem rede vem depois de `<head>` para o arquivo exportado, onde o cabeçalho HTTP não vale. A ferramenta não guarda nada com o app fechado (ao contrário de `anexar_prototipo`).
- **Tópicos do guia.** O guia é dividido para manter cada tópico abaixo de 4.500 caracteres: `apresentar` (os três tipos do próprio app), `apresentacao_ia` (conversa, público, nível), `apresentacao_conteudo` (o objeto `conteudo` e o envio) e `apresentacao_figuras` (os programas das figuras). `/blueprint:apresentar` aponta para `apresentacao_ia`.

## Edição manual e a tela (2026-10-26)

A apresentação é totalmente editável à mão. O visor é isolado (origem opaca, sem acesso ao app), então o editor fica no app, ao lado do visor, e edita os dados guardados.

- **Os dados guardados continuam os mesmos** (`conteudo` mais `figuras`); uma edição os reescreve. Os metadados ganham `editadoEm` (hora ISO da última edição manual; ausente quando a IA escreveu a versão atual).
- `GET /api/mcp/apresentacao?id=<id>` devolve `{id, titulo, publico, nivel, em, editadoEm?, conteudo, figuras}` (figuras com `codigo`). App local e membros da equipe que podem editar; 404 `apresentacao-nao-encontrada`.
- `POST /api/mcp/apresentacao/editar` com `{apresentacaoId, conteudo?, figuras?}` (mande só o que mudou; cada parte enviada substitui aquela parte inteira). Mesma validação e mesmos códigos de erro do `anexar`; `publico` e `nivel` vêm de `conteudo`; o `titulo` dos metadados segue `conteudo.titulo`; grava `editadoEm`; responde `{ok, apresentacaoId, editadoEm, avisos[]}` (os `avisos` são os mesmos do `anexar`: termos proibidos em `cliente`, `figura` desconhecida, `codigo` vazio); emite o evento SSE `apresentacao`. Loopback e papéis como as outras rotas.
- A rota da lista (`GET /api/mcp/apresentacoes`) e `ler_blueprint` levam `editadoEm`.
- **O que dá para editar**: título, subtítulo, cliente, data, público, nível, idioma; cada item de `resumo`, `entregas`, `etapas` (com `marcos`, `figura`, `situacao`), `precisamosDeVoce`, `decisoes`, `riscos` e, para `interno`, `interno.acoes`; de cada figura, `titulo`, `dica`, `leitura`, `chaves`, `legenda`, `rotulo` e `codigo` (avançado). Toda lista: adicionar, remover, subir e descer. Figuras: duplicar e remover.
- **O editor** (web): seções que espelham a página, salvamento automático depois de uma pausa com estado visível (salvando, salvo, erro), os avisos do servidor ao lado do campo, conferência de sintaxe do `codigo` só na página do app (nunca executa), Desfazer da última mudança e o visor recarregado depois de cada salvamento mantendo a seção (fragmento `#`, que a página restaura).
- **A IA não pode sobrescrever uma edição manual de surpresa**: gerar de novo (botão ou agente) quando há `editadoEm` pergunta antes ("isto substitui as suas edições"); o guia diz o mesmo ao agente.

Tela: quando o blueprint tem apresentações geradas, a tela abre na mais nova (visor mais barra: Editar, Apresentar, Imprimir, Exportar HTML, Gerar de novo, Remover). O trilho lista as apresentações geradas e o grupo "Gerar com IA"; os tipos do próprio app (Andamento por projeto, Plano completo) passam para um grupo recolhido "Visão interna". Sem nenhuma gerada, a tela abre como antes, com "Gerar com IA" primeiro.

## Gerar no terminal do app e conversar com o agente (2026-10-26)

- **Gerar roda no terminal do próprio app** (o painel ConPTY embutido, `src/terminal/`): "Gerar com IA" abre uma aba rodando `blueprint claude "<prompt>" --canal` na pasta do blueprint, onde o usuário acompanha e responde o agente. O diálogo oferece "Terminal do app" (padrão), um terminal externo novo ou uma sessão aberta, como "Iniciar com o Claude". `POST /api/terminal/iniciar` aceita `{workspaceId, blueprintId, apresentar: {publico, nivel}, titulo?}` (o prompt é o mesmo que `POST /api/mcp/apresentar` monta) e, para ajustar, `{workspaceId, blueprintId, apresentacaoId, ajuste: "<texto>", titulo?}`.
- **O agente edita o próprio arquivo, em passos pequenos, e nunca o reescreve.** Duas ferramentas MCP: `ler_apresentacao(workspace_id, blueprint_id, apresentacao_id, figuras_codigo?: [ids])` devolve `conteudo` e as figuras (sem `codigo`, a menos que o id da figura esteja em `figuras_codigo`, para poupar contexto), e `editar_apresentacao(workspace_id, blueprint_id, apresentacao_id, operacoes[])`.
- **Operações** (aplicadas em ordem, todas ou nenhuma; o resultado é validado como no `anexar`; cada resposta traz os avisos e o novo `editadoEm`):
  - `{op: "definir", caminho: "etapas.2.titulo", valor: <json>}`: define um campo (o caminho é pontuado, itens de lista por índice a partir de 0; uma última chave que falta é criada);
  - `{op: "inserir", caminho: "etapas", indice: 3, valor: <json>}`: insere numa lista (sem `indice` = no fim);
  - `{op: "remover", caminho: "etapas.2"}`;
  - `{op: "mover", caminho: "etapas.2", para: 0}`: move dentro da própria lista;
  - `{op: "figura_trecho", figura: "capa", de: "<texto exato>", para: "<texto>"}`: troca um trecho exato do `codigo` de uma figura; o trecho precisa ocorrer exatamente uma vez, senão a chamada inteira falha com o número de ocorrências (sem adivinhar em silêncio);
  - `{op: "figura_definir", figura: "capa", campo: "legenda", valor: "..."}`: define um campo da figura que não seja `codigo`; `{op: "figura_remover", figura: "capa"}`.
  A troca de parte inteira (`conteudo`/`figuras` em `POST /api/mcp/apresentacao/editar`) continua para o "restaurar" do editor manual e para colagens grandes; o editor manual também manda operações, então uma edição do usuário e uma do agente em campos diferentes nunca se sobrescrevem.
- `POST /api/mcp/apresentacao/editar` aceita portanto também `{apresentacaoId, operacoes: [...], autor: "usuario" | "agente"}`; os metadados guardam `editadoEm` e `editadoPor`.
- **A conversa.** Na tela Apresentar, ao lado do visor: uma caixa "Conversar com o agente" (texto, Enviar). `POST /api/mcp/apresentacao/pedido` `{workspaceId, blueprintId, apresentacaoId, texto, sessaoId?}` entrega a mensagem à sessão escolhida pelo canal dela com o prefixo «The Blueprint · pedido do usuário» (é uma ordem do clique do usuário, como Iniciar com o Claude); sem sessão, abre uma aba do terminal do app com `ajuste`. O prompt diz ao agente: leia com `ler_apresentacao`, mude SÓ o que foi pedido com `editar_apresentacao` (operações pequenas, nunca `anexar_apresentacao` sobre uma apresentação existente a menos que o usuário peça para gerar de novo), confirme numa frase o que mudou. O usuário também pode digitar o pedido direto no terminal onde o agente está rodando.
- **Tópico do guia** `apresentacao_ajustar` (o protocolo acima, abaixo de 4.500 caracteres) se junta a `apresentacao_ia`, `apresentacao_conteudo` e `apresentacao_figuras`.
- A guarda do `editadoEm` continua: gerar de novo sobre uma apresentação editada à mão pergunta antes.

## Uma figura por sessão (2026-10-26)

- **A figura da visão geral não muda.** A figura que a IA já fez para a capa (`capa`, agora `secao: "visao"`) fica exatamente como foi gerada. A figura das torres do próprio app NÃO volta, e o servidor não injeta nenhuma estrutura (um rascunho anterior desta seção propôs `estrutura` e `L.figuraBlueprint`; o usuário rejeitou: só se acrescentam mais figuras).
- **Figuras por sessão.** Uma figura pode levar `secao`, uma entre `visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`, `acoes` (os ids das seções da página). A página põe essa figura no painel da sessão, ao lado do texto (embaixo dele em tela estreita). A figura própria de uma etapa (`etapas[].figura`) continua valendo e fica dentro da etapa. Uma figura sem nenhuma das duas aparece no fim de Etapas, como hoje.
- **Níveis, corrigidos:** `rapido` = uma figura da IA (`visao`); `padrao` = uma figura diferente da IA por sessão (`visao`, `recebe`, `etapas`, `linha`, `precisamos`, `decisoes`, `riscos`; `acoes` só para `interno`); `completo` = `padrao` mais uma máquina densa do sistema inteiro em `visao` e uma figura por etapa (`etapas[].figura`). Cada objeto é diferente dos outros e ajuda a entender a sua sessão (cartões em trilho para o que o cliente recebe, uma esteira de etapas, um tambor de calendário para a linha do tempo, uma bandeja de pedidos para o que precisamos do cliente, uma balança para as decisões, um painel de mostradores para os riscos).
- `validar` aceita e confere `secao` (um id conhecido, senão o aviso `secao-desconhecida`); acrescenta um aviso quando duas figuras reivindicam a mesma sessão.

## Texto e figura ligados (2026-10-26)

A figura de uma sessão faz parte do texto dela: o texto faz referência à figura e as duas se comandam.

- **Itens.** Em cada sessão a página lista os itens na ordem de `conteudo` e dá a cada elemento `data-item="<i>"` (a partir de 0) e `data-sessao="<secao>"`, focáveis (`tabindex="0"`): `recebe` -> `entregas`, `etapas` -> `etapas`, `linha` -> `etapas` (as linhas da linha do tempo), `precisamos` -> `precisamosDeVoce`, `decisoes` -> `decisoes`, `riscos` -> `riscos`. `visao` não tem itens.
- **A ponte.** `vivo(palco, dizer, ponte)` recebe um terceiro argumento (pode faltar quando uma figura roda em outro lugar: proteja sempre com `if (ponte)`). `ponte.total` é o número de itens da sessão dela. `ponte.liga({ selecionar(i) })` registra como a figura seleciona o item `i`; a página chama `selecionar(i)` quando o mouse entra, ou o teclado foca, num item do texto, e `selecionar(null)` quando sai (a figura volta ao próprio passeio). A figura chama `ponte.mudou(i)` (ou `null`) sempre que o item atual dela muda por mouse, teclas ou passeio próprios; a página então marca o item de texto correspondente (`data-ativo`, `aria-current="true"`) e limpa o anterior. Uma figura cuja posição própria não é um índice de item faz o mapeamento (o tambor de calendário mapeia um item para o dia de início daquela etapa e informa a etapa do dia atual).
- **A referência.** Sob cada título de sessão que tem figura, a página escreve uma linha, `Fig n · <dica da figura>`, em que `n` é o número do cartão, e o `Fig n` é um link que rola até a figura e a foca. A linha diz, no idioma da página, que dá para apontar um item do texto para vê-lo na figura.
- **Aparência.** O item marcado usa o estado selecionado do app (borda de destaque e uma barra à esquerda), sem animação além da própria figura; o movimento reduzido não muda mais nada. Toque: um toque num item o seleciona (não há passar o mouse no toque); o passeio da figura fica em pausa enquanto um item está selecionado pelo texto.
- As figuras escritas antes disso continuam funcionando (sem usar `ponte`); o guia ensina a ponte para as novas.
