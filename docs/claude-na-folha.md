# O Claude na folha (contrato, 2026-10-22)

> Portuguese only for now (a technical reference for maintainers). Documentação só em português por enquanto.

O agente que trabalha no app aparece na folha como o próprio **Claude**, desenhado com o anatomy (`agentes-claude.mjs`), e os subagentes aparecem como versões dele do **mesmo tamanho** (decisão do usuário, 2026-10-22). Este documento é o contrato entre o desenho (esta ação) e o movimento pela folha (ação seguinte). Prévia para aprovar o desenho: `referencias/claude-previa.html`.

## O que ele é

- **Só indicador, com uma exceção.** Mostra o que o agente do usuário disse pelo MCP; não executa nada e **não tem clique para mandar trabalho** (nada de "peça isto ao Claude"), exceto o botão "Iniciar com o Claude" da folha da ação (seção "Claude esperando e iniciar a ação", abaixo), que abre uma sessão nova por clique confirmado. Passar o mouse pode mostrar a última fala; um clique só pode levar a tela até o plano onde ele está (navegar, como o robozinho antigo), nunca enviar tarefa.
- **Substitui o robozinho antigo da folha** (`web/js/robo.js`, `web/css/robo.css` e a faixa `#robo-agente`). A troca foi feita na ação "Claude anda pela folha trabalhando": camada `#claude-camada` (`web/js/claude-camada.js`) e lógica pura `web/js/claude-logica.js` (`claude-logica.test.mjs`); o painel separado de agentes saiu.
- **Zero trabalho em repouso.** Cada pose é desenhada uma vez e o SVG é guardado; o app troca de SVG quando o estado muda. Nada de animar centenas de elementos; sem loop de animação parado na folha.

## De onde vêm os dados

Tudo vem do contrato que já existe em `registrar_progresso` (`src/mcp/atividade.rs`, `ferramentas.json`):

| Campo | Valores | Para que serve na figura |
| --- | --- | --- |
| `agente` | nome estável (até 40 caracteres) | identifica quem é quem |
| `papel` | `explorador`, `construtor`, `revisor` | variante do subagente (ferramenta e cor de apoio) |
| `estado` | `lendo`, `escrevendo`, `testando`, `aguardando`, `bloqueado`, `concluido` | a pose |
| `onde` | até 5 caminhos | texto ao lado; mudou de `onde` ou de plano = "andando" |
| `acao_id`, `plano_id` | ids | onde na folha o personagem fica |
| `percentual` | 0 a 100 | não muda a figura (fica no texto) |

## Poses

`poseDoEstado(estado, andando)` (em `agentes-claude.mjs`) faz o mapa; `pecasDoClaude({ pose, papel, quadro })` devolve a lista de peças.

| Pose | Quando | O que o desenho mostra | Peça `lit` (a que o olhar segue) |
| --- | --- | --- | --- |
| `parado` | sem estado (nada reportado, expirou ou `ocioso`) | braços ao lado, olhos abertos | nenhuma |
| `andando` (2 quadros) | `lendo`/`escrevendo`/`testando` logo depois de mudar de `onde` ou de plano; a folha alterna `quadro` 0 e 1 enquanto o personagem se desloca (ação seguinte) | um par de pernas sobe e avança, o corpo sobe 1 e os braços balançam | nenhuma |
| `trabalhando` | `lendo`, `escrevendo`, `testando` | braços para a frente; o Claude principal mexe num teclado, o subagente segura a ferramenta | a faísca sobre a cabeça |
| `esperando` | `aguardando` | braços cruzados à frente e "..." sobre a cabeça | o último ponto do "..." |
| `bloqueado` | `bloqueado` | olhos franzidos, sobrancelhas e uma cancela listrada à frente | a lâmpada da cancela |
| `concluido` | `concluido` | braços para o alto, olhos "fechados" de contentamento, bandeira na placa | o pano da bandeira |

Regras: no máximo uma peça `lit` por figura (exatamente uma nas poses acima que têm `lit`); o enquadramento é o mesmo em todas as poses e papéis (opção `enquadra` de `desenhaEmblema`, constante `ENQUADRA`); `lendo`, `escrevendo` e `testando` têm o mesmo desenho (a diferença fica no texto), por escolha, para o desenho continuar legível em 24 a 48 px.

## Animação (2026-10-22)

Movimento por **quadros**: cada quadro é um desenho pronto (lista de peças, guardado uma vez como SVG) e a folha só alterna qual está à vista. O kit do anatomy não muda. `agentes-claude.mjs` exporta `QUADROS` (quantos), `SEQUENCIA` (a ordem) e `RITMO` (ms por quadro); `pecasDoClaude({ pose, papel, quadro })` desenha qualquer quadro. O **quadro 0 de toda pose é o desenho aprovado** (o teste confere os hashes antigos) e os outros mexem em poucas peças. Fluidez: os quadros intermediários (meio caminho entre dois extremos) e as sequências que vão e voltam por eles dão ~8 quadros por segundo, até 5 desenhos por pose.

| Pose | Quadros | Sequência · ritmo | Movimento |
| --- | --- | --- | --- |
| `parado` | 5 | 0 1 2 1 0 1 2 1 0 1 2 1 0 3 4 3 · 150 ms | respira (o corpo sobe até 1, em 3 passos) três vezes e pisca (meio fechado, fechado, meio fechado) |
| `andando` | 3 | 0 2 1 2 · 110 ms | um par de pernas sobe, passa pelo meio do passo (quadro 2) e o outro par sobe; só durante o deslocamento |
| `trabalhando` | 5 | 0 1 2 1 0 3 4 3 · 110 ms | um braço avança e volta passando pelo meio, depois o outro; a faísca cresce e gira junto |
| `esperando` | 3 | 0 1 2 · 300 ms | o ponto aceso do "..." anda de um lado ao outro |
| `bloqueado` | 4 | 0 1 2 3 · 200 ms | a lâmpada fica acesa nos quadros 0 e 1 e apagada nos 2 e 3, enquanto a barra da cancela sobe e desce 1,5 |
| `concluido` | 3 | 0 1 2 1 0 0 0 0 · 120 ms | pulinho (corpo sobe até 2) e pausa; o pano da bandeira ondula junto |

A ferramenta dos subagentes acompanha a mão, então mexe junto com o braço. Regras (as do projeto): o relógio de uma figura só roda enquanto ela está visível na tela e a aba está aberta; para ao andar, ao trocar de pose e ao remover a figura; com `prefers-reduced-motion` fica no quadro 0; sem agente ativo não há timer nenhum. Prévia animada sem script: `referencias/claude-previa.html`. **Custo medido (2026-10-23, `node scripts/medir-animacoes.mjs`):** parada 0,03% de um núcleo; uma figura animada 0,34%; seis figuras 3,4%; aba oculta, figura fora da tela e movimento reduzido 0 relógios. Os quadros ficam empilhados e o relógio só troca a classe `on` (`opacity` em camada composta, `claude.css`): `display:none` custava ~11% com seis figuras. Detalhes em `referencias/relatorio-medicao.md`.

## Claude esperando e iniciar a ação (contrato, 2026-10-22; reabre o "só indicador")

A decisão de 2026-10-21 ("o agente é só indicador, sem clique para mandar trabalho") foi **reaberta pelo usuário** em 2026-10-22 num ponto só: o usuário pode, quando quiser, **iniciar uma ação com o Claude** por um botão. O Claude continua sem receber ordens pelo MCP, monitor ou canal; o que muda é que o app pode abrir uma sessão nova do Claude Code por clique confirmado.

- **Próxima ação (`claudeProximaAcao(p, agentes)`, pura, em `web/js/claude-logica.js`):** por blueprint, a primeira ação não concluída, por data de início (empate: a ordem do blueprint), que tenha plano pendente e **nenhum agente trabalhando nela** (agente com `acaoId` igual e estado diferente de `ocioso`). Sem ação assim, não há figura de espera.
- **Figura de espera:** do Claude principal, na pose `esperando`, ao lado do nome/cabeçalho da ação (mesmo lugar do cabeçalho do blueprint quando a ação não está à vista). É **só da interface**: não é agente, não vai ao servidor, não conta no filtro "Em andamento" nem no limite de figuras. Some quando um agente assume a ação. Clicar nela abre a **folha da ação** (a nota "Instruções da ação", `abreNotaObs`).
- **Botão na folha da ação:** a nota ganha o botão **"Iniciar com o Claude"** (`acoes` do `abreNota`). Ele pede confirmação ("abre um terminal novo e gasta tokens"), avisa se já há agente trabalhando nela e, sem pasta ligada ao blueprint, explica como ligar. Função global `abreFolhaDaAcao(pid, fid, ancora)` (em `web/js/iniciar-acao.js`) abre a folha da ação; é o que o clique na figura de espera chama.
- **Rota:** `POST /api/mcp/iniciar` com `{ "workspaceId", "blueprintId", "acaoId" }`, só em loopback, com `Host`/`Origin` conferidos, `content-type: application/json` e **nunca liberada por `MCP_REDE`**. Valida que workspace, blueprint e ação existem (404), acha a pasta do blueprint na tabela `pastas` (409 com `{"codigo": "sem-pasta"}` se não houver) e abre um **terminal novo** na pasta: Windows Terminal (`wt.exe`) se existir, senão o prompt do Windows (`cmd /c start`); 501 com `{"codigo": "sem-terminal"}` se nenhum abrir. Roda `blueprint claude` com o prompt inicial como **argumento separado** (nada de linha de comando montada por texto): "Execute a ação `<número>` «`<nome>`» do blueprint «`<nome>`» (workspace_id …, blueprint_id …, acao_id …): leia com ler_blueprint, registre o progresso com registrar_progresso e conclua cada plano com marcar_concluido e a evidência." Resposta 200: `{ "ok": true, "pasta": "…", "terminal": "wt" | "cmd" }`; erros: `{ "erro": "…", "codigo"?: "…" }`. As permissões do Claude Code são as normais (nada de pular confirmações).
- **O prompt não passa pelo monitor nem pelo canal** (eles são informativos; o agente não executa instruções de eventos). Vai como argumento da sessão nova, por um clique do usuário.
- **Concluído e Fechar (decisão do usuário, 2026-10-23):** o agente que concluiu fica no plano, com a bandeira, até o usuário **clicar**; o clique abre a folhinha com a mensagem de conclusão e **duas opções**: **Concluído** (o usuário já viu: o agente sai da folha, não repousa, e o Claude de espera aparece na próxima ação) e **Fechar** (só fecha a folhinha; o agente continua onde está). O mesmo botão **Concluído** aparece no agente parado (aguardando ou em repouso) e o dispensa até ele avisar de novo (o "visto" vale só para aquele aviso); o agente que está trabalhando (lendo, escrevendo, testando, bloqueado) só tem Fechar. O repouso por 10 min sem aviso continua parado no cabeçalho. O aviso do agente na tela chama o que foi concluído de **Tarefa concluída** (e "Tarefa não concluída"), porque "plano" também é a proposta que o agente envia.
- **Fora do escopo:** iniciar várias ações de uma vez, escolher o modelo. (Mandar o pedido para uma sessão já aberta saiu daqui em 2026-10-23: veja a seção seguinte.)

## Escolher a sessão ao iniciar a ação (contrato aprovado em 2026-10-23; implementado, ainda sem teste com um Claude real)

Pedido do usuário em 2026-10-23: no diálogo "Iniciar a ação N?" aparecer **Nova sessão** (padrão) ou as **sessões do Claude já abertas** naquela pasta; escolhida uma, o prompt da ação vai para ela. Isso **reabre de novo** o "o Claude não recebe ordens": a exceção vale só para o clique confirmado nesse diálogo.

- **Quem entrega: o canal do MCP, não o hook nem o mod.** O hook só *registra* as sessões (SessionStart/SessionEnd); o mod do terminal é indicador (a API de mods não tem chamada para escrever um prompt numa sessão aberta). O canal (`notifications/claude/channel`, `docs/monitor-e-channels.md`) já foi validado em sessão interativa real em 2026-10-22.
- **Limite que a validação anterior mostra:** só recebe a sessão iniciada com `BLUEPRINT_CHANNEL=1` e `--dangerously-load-development-channels` (research preview; o Claude Code pede uma confirmação de desenvolvimento a cada sessão). Sessão aberta à mão sem isso aparece na lista, mas o clique só **copia o prompt**. As sessões que o botão abre (`blueprint claude`) passam a nascer com o canal ligado.
- **Cada processo `blueprint --mcp` com o canal ligado é um destinatário:** gera um id (16 hex), se registra no app ao abrir o SSE (id e pasta, sem pid) e sai da lista quando a conexão cai (sem polling). O app endereça o evento `iniciar-acao` a um id; os outros processos o ignoram.
- **O canal hoje só leva texto fixo e filtrado**, e o MCP manda o agente **não executar instruções de eventos**. Para isto funcionar: um tipo de evento próprio (`iniciar-acao`), com o texto montado pelo app (mesma limpeza do `iniciar`: sem controle, sem `;`, curto) e uma frase nas instruções do MCP dizendo que **só esse tipo**, vindo do clique do usuário, é uma ordem. Qualquer outro conteúdo de evento continua informativo.
- **Segurança:** só loopback, `Host`/`Origin` conferidos, `content-type: application/json`, nunca por `MCP_REDE`; recusa sessão que não é da pasta do blueprint (409), sessão que caiu (410) e sessão sem canal (409, com o prompt para copiar). Nada grava nos dados.
- **Rotas (feitas):** `GET /api/mcp/sessoes?workspace=&blueprint=` devolve `{sessoes: [{id, pasta, canal}]}` (cada canal da pasta com `canal: true` e, no máximo, as sessões que sobram sem canal) e `POST /api/mcp/iniciar` aceita `sessaoId` (e `semCanal: true` para só devolver o `prompt`). Erros: 410 `sessao-caiu`, 409 `sessao-de-outra-pasta`; a resposta sempre traz o `prompt`. O canal se registra abrindo o SSE com `?canal=<id>&cwd=<pasta>` e sai quando a conexão fecha (até 25 s depois, o batimento). O `SessionEnd` do plugin (`hook/fim`) tira a sessão da lista. **Expiração (2026-10-26):** uma sessão que fechou sem `SessionEnd` (travou, terminal fechado à força) some da lista depois de 24 h sem sinal (o SessionStart, qualquer hook com o `session_id` dela ou a queda do canal da pasta renovam); numa pasta com canal conectado nenhuma sessão expira. A conferência é preguiçosa, ao listar (`PRAZO` e `expirar` em `src/mcp/sessoes.rs`), sem thread nem polling.
- **Sessões abertas pelo botão** (`blueprint claude <prompt> --canal`) ligam `BLUEPRINT_CHANNEL=1` e a flag de desenvolvimento (a flag vai depois do prompt, porque pode ser variádica); o Claude Code pede a confirmação de desenvolvimento nelas.
- **Ainda não verificado com um Claude real:** (1) que a mensagem chega a uma sessão *ociosa* e vira um turno em que o modelo aceita o pedido como ordem (a frase nas instruções do MCP depende disso); (2) que `plugin:blueprint@blueprint-local` vale para o plugin carregado por `--plugin-dir`; (3) a junção entre a sessão do hook e o processo do canal não existe (sem pid): a lista não diz qual sessão da pasta é qual, só quantas têm canal. Cobertura sem Claude: `cargo test` e `node scripts/sessao-iniciar-e2e.mjs` (canal e terminal falsos).

## Enviar comentário do plano e conversar com o plano (contrato de 2026-10-26, a aprovar pelo usuário; sem teste com um Claude real)

Duas ações do projeto "Integração nativa com o Claude" mandam texto escrito pelo usuário à sessão do Claude que ele escolhe. As duas usam **o mesmo caminho** do "Escolher a sessão" (acima) e **não ampliam a exceção**: só por clique, só loopback, só para o canal endereçado, e nada grava nos dados.

- **Uma rota só:** `POST /api/mcp/conversa` com `{workspaceId, blueprintId, acaoId, planoId?, tipo, texto, sessaoId, semCanal?}`. Como `/api/mcp/iniciar`, só responde em loopback (`Host`/`Origin` conferidos; nem `MCP_REDE=1` libera), exige `content-type: application/json` e confere que a ação (e o plano, no comentário) existe no banco (404 se não). `texto`: até **800 caracteres**, caracteres de controle viram espaço; vazio é 400, longo é 413. O nome da ação, do plano e do blueprint passam pela mesma limpeza do `iniciar` (sem controle, sem `;`, 80 caracteres).
- **Entrega:** a sessão `sessaoId` precisa ter o canal conectado e ser de uma pasta do blueprint (410 `sessao-caiu`, 409 `sessao-de-outra-pasta`, como no iniciar). O evento SSE é endereçado (`canalId`) e só o `blueprint --mcp` daquele canal o repassa; os outros processos o ignoram. O JSON do evento tem de caber em 1.900 bytes (a linha SSE do canal tem teto de 2.048): passou, 413. `semCanal: true` não entrega nada (dispensa o `sessaoId`) e só devolve o `prompt` para a página copiar.
- **Comentário do plano (`tipo: "comentario"`, exige `planoId`):** evento `comentario-plano`. O canal o entrega com o **mesmo prefixo de ordem** do iniciar: «The Blueprint · pedido do usuário (ele clicou em Enviar ao agente num comentário de plano): …», seguido de "Comentário no plano «P» (plano_id …) da ação N «A» do blueprint «B» (workspace_id …, blueprint_id …, acao_id …): <texto>". É uma ordem porque o usuário escreveu e confirmou no diálogo. Na página: botão direito no plano → **Enviar comentário ao agente…**, um diálogo com o texto e a escolha da sessão (cada sessão aberta ou "Só copiar o texto"); o botão Enviar é a confirmação. O comentário **não é salvo no plano** (hoje não há tela de comentário do usuário no plano; os comentários do plano são os do agente). Resultado na própria janela: "enviado à sessão" ou "copiado".
- **Conversa (`tipo: "conversa"`):** evento `conversa-usuario`, com **prefixo próprio**: «The Blueprint · conversa do usuário (responda com responder_conversa; ajustes só por propor_blueprint): Ação N «A» do blueprint «B» (ids…): <texto>». Não é ordem de executar: as instruções do MCP dizem para responder com `responder_conversa`, propor qualquer mudança só por `propor_blueprint` (ou `ajustar`) e não editar arquivos nem rodar comandos por causa dela.
- **Volta:** o canal não tem resposta, então a IA responde pela ferramenta MCP **`responder_conversa(workspace_id, blueprint_id, acao_id, texto, proposta?)`** → `POST /api/mcp/conversa/resposta`. `texto` até 4.000 caracteres; `proposta` é o número de uma proposta **pendente** na fila daquele blueprint (409 com os números pendentes se não for). Com o app fechado, falha (a conversa é só memória; não vai para os pendentes).
- **Memória:** cada ação guarda as últimas **40** mensagens (`{de: "usuario"|"agente", texto, em, proposta?}`), no máximo **32** conversas (passou, esquece todas, como as sessões). Some ao reiniciar o app, como a fila. `GET /api/mcp/conversa?workspace=&blueprint=&acao=` devolve `{mensagens}`; cada mensagem nova avisa a página pelo evento SSE `conversa` (`{workspaceId, blueprintId, acaoId}`, sem texto), e a página com o painel daquela ação aberto busca a lista.
- **Painel na folha:** **Conversar com o plano** na folha da ação (ao lado de "Iniciar com o Claude") e no botão direito da ação. Abre um painel lateral com o histórico, a escolha da sessão (as abertas na pasta, com ou sem canal; sem nenhuma, só "Copiar a mensagem" e a dica de abrir uma com "Iniciar com o Claude"), o campo da mensagem e **Enviar** (o clique é a confirmação; a sessão escolhida fica visível ao lado do botão). Resposta que cita uma proposta mostra **Abrir a proposta nº N**, que abre o Planejar naquela proposta; aplicar continua sendo o clique de sempre. Textos por `tr()` (catálogo `web/i18n/en-conversa.js`), tema do app, sem emojis.
- **Fora do escopo:** abrir uma sessão nova só para conversar; guardar a conversa no banco; salvar o comentário no plano; resposta pelo próprio canal. Teste sem Claude: `node scripts/conversa-e2e.mjs` (canal e agente falsos).

## Onde o agente fica e como aparece e some (2026-10-22)

- **Lugar, do mais preciso ao mais geral** (`claudeLugar` em `web/js/claude-logica.js`): na **linha do plano** dele (seção aberta); senão ao lado da **barra da ação** (seção fechada); senão no **cabeçalho da seção (o projeto) da ação**, e não no do primeiro projeto; só sem ação conhecida, no cabeçalho do blueprint (ou no cartão da lista). Os agentes do mesmo lugar ficam lado a lado, e o lugar do cabeçalho é por seção. O Claude esperando segue a mesma regra (barra da ação aberta; senão cabeçalho da seção). Para o agente ficar em cima do plano, ele precisa mandar `acao_id` e `plano_id` no `registrar_progresso`; o hook de edição herda os do último aviso.
- **Aparecer e sumir:** ao entrar na folha a figura **cresce a partir do centro** (`scale` de 0 a 1, 380 ms) e ao sair **encolhe até o centro** (de 1 a 0, 300 ms) antes de ser removida (`claudeEntra` e `claudeSai` em `web/js/claude-camada.js`). Sem animação com `prefers-reduced-motion` ou aba oculta.

## Quantos aparecem

- **Um Claude principal** e **até `MAX_SUBAGENTES` = 7 figuras de subagente**. Até 5 figuras ficam lado a lado; com mais, elas **se juntam** (o passo entre elas encolhe e elas se sobrepõem, a mais perto do lugar, o principal, por cima) e o grupo nunca passa da largura de 5 (decisão do usuário, 2026-10-23: nada de contador "+N"). Passou de 8, as demais não são desenhadas e a dica da última figura diz quantas estão juntas ali. Ordem das figuras: quem avisou há menos tempo primeiro; o agente **nunca sai da folha por falta de sinal**: depois de concluir (20 s comemorando no plano) ou de 10 min sem avisar (`VIDA_MS`) ele vai para o **repouso**: pose `parado`, sem plano, no cabeçalho do blueprint (a figura anda até lá), e fica assim até chegar aviso novo; só sai ao reiniciar o app (a presença é só memória) ou quando o 33º agente o substitui. O filtro "Em andamento" conta só quem está num plano (trabalhando), não os que repousam. O hook de edição herda a ação e o plano do último aviso do agente, para a edição não o tirar do plano. **Subagente novo (2026-10-26):** nasce ao lado do plano do Claude principal quando o último aviso dele é de menos de 10 min e não está concluído (`Atividade::plano_de`); sem isso fica no cabeçalho do blueprint, como antes. Antes disso o subagente nunca herdava, e quem lançava subagentes sem avisar o plano os via só no cabeçalho.
- O principal é o agente **sem `papel`** ou, se todos tiverem, o mais antigo; ele nunca carrega ferramenta. Os demais são subagentes do papel que reportaram. (Suposição: o MCP não marca quem é subagente.)
- Tamanho: **todos os agentes têm o mesmo tamanho** (decisão do usuário, 2026-10-22; era 70% para o subagente): `ESCALA.subagente` = 1, o mesmo enquadramento e a mesma câmera. Alvo de leitura: 48 px de largura para todos; abaixo disso (24 px) ainda se distinguem pela ferramenta e pela cor da base.

## Subagentes por papel

O corpo é o mesmo do Claude. Muda a ferramenta (na mão esquerda) e a cor de apoio (placa da base, cinta no corpo e ferramenta):

| Papel | Ferramenta | Cor de apoio (`tom`) |
| --- | --- | --- |
| explorador | lupa | ciano (`exp`) |
| construtor | martelo | âmbar (`con`) |
| revisor | prancheta | verde (`rev`) |

## Escolhas do desenho (anatomy)

- **Reconhecível como o Claude:** corpo largo e laranja da marca (`#d97757`), dois olhos escuros e retangulares, braços curtos nas laterais e quatro pernas curtas, como a mascote do Claude Code; na lateral, um distintivo com a faísca (✻) do Claude.
- **Só dados:** lista de peças para `desenhaEmblema`, entre 44 e 62 peças por figura (limite 400); desenhada em poucos ms e guardada como SVG.
- **Sobre o papel azul do blueprint**, como a figura do agente de antes (`.ag-fig`). As cores novas ficam em `web/css/claude.css` (tons `claude`, `olho`, `exp`, `con`, `rev`, `aco`, `papel`); o azul do blueprint continua sendo o das outras marcas.
- **Câmera:** `AZIMUTE` 30, para a frente do personagem (+x) ficar virada para o leitor.
- **Testes:** `node agentes-claude.test.mjs`.

## Linha do tempo da execução por ação (decisão de 2026-10-26, a revisar pelo usuário)

A presença (`src/mcp/atividade.rs`) só guarda o último estado de cada agente e some ao reiniciar. A folha da ação ganha, abaixo das instruções, uma lista só de leitura com os últimos eventos do agente naquela ação: começou, mudou de arquivo, bloqueou, concluiu ou não concluiu um plano, marcou um checkpoint, cada um com a hora, o agente e o plano.

- **Onde fica: no banco, numa tabela própria** (`linha_do_tempo`, migração `004_linha_do_tempo.sql`; código em `src/dados/linha_do_tempo.rs`). Motivo: o app reinicia a cada atualização e cada vez que o usuário o fecha pela bandeja, e uma linha do tempo que some nesses momentos não responde "o que o agente fez e quando". Fica **fora do data.json**, como as `pastas`: o `PUT /api/dados` não a apaga, `GET /api/dados` não cresce com ela e `blueprint exportar` não a leva. Não é dado do blueprint, é registro do que o agente avisou; nada nela altera o blueprint.
- **O que entra:** só eventos que já passam por `acoes::evento` (os de `registrar_progresso`, `reportar_bloqueio`, `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_*` e os hooks de edição e de subagente) **e que têm ação**. Campos: ids do workspace, blueprint e ação; `alvo` (o id do plano ou do checkpoint, conforme o tipo; o nome a página tira dos dados); hora (ISO); nome do agente (até 40); tipo; estado; `onde` (o primeiro caminho, até 120, relativo à pasta da sessão, como já vem); texto curto (até 200 caracteres).
- **O que nunca entra:** conteúdo de arquivo (o servidor nem o recebe: o hook de edição só manda o caminho), comandos e saídas, a evidência de `marcar_concluido` e dos checkpoints (pode trazer comando e saída de teste; ela já vira comentário do plano quando a página aplica, então a linha do tempo grava esses eventos **sem texto**), `session_id`, `cwd` e o resto do `tool_input`. Sem dado pessoal além do nome que o próprio agente escolheu.
- **Retenção:** os **50 eventos mais recentes por ação** e **nada com mais de 30 dias** (as duas regras rodam na mesma transação de cada gravação; nada roda em repouso). Eventos repetidos em sequência (mesmo agente, tipo, alvo, texto, estado e arquivo, como várias edições seguidas do mesmo arquivo) viram um só, com a hora da última. Ação apagada: os eventos dela saem pelos 30 dias.
- **Leitura:** `GET /api/mcp/linha-do-tempo?workspace=&blueprint=&acao=` (mesmas regras de loopback das rotas `/api/mcp/*`) devolve `{eventos, limite}`, do mais recente ao mais antigo. Cada gravação avisa por SSE o evento `linha-do-tempo` com os três ids (nenhum texto livre no SSE); a página só busca de novo se a folha daquela ação estiver aberta. Sem polling.
- **Interface:** `web/js/linha-do-tempo.js` (lógica pura testada em `linha-do-tempo.test.mjs`), estilo em `web/css/linha-do-tempo.css`, textos em `web/i18n/en-linha-do-tempo.js`. Ponta a ponta: `node scripts/linha-do-tempo-e2e.mjs [exe]` (~3 s, Edge sem janela, banco temporário).
- **Limites conhecidos:** um evento guardado com o app fechado (`pendentes.rs`) entra com a hora em que o app o reenviou; uma falha ao gravar a linha do tempo não recusa o evento (só some da lista), para o agente não reenviar e duplicar a fila.
