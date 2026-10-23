# O Claude na folha (contrato, 2026-10-22)

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
- **Fora do escopo:** mandar o pedido para uma sessão já aberta (não há canal seguro), iniciar várias ações de uma vez, escolher o modelo.

## Onde o agente fica e como aparece e some (2026-10-22)

- **Lugar, do mais preciso ao mais geral** (`claudeLugar` em `web/js/claude-logica.js`): na **linha do plano** dele (seção aberta); senão ao lado da **barra da ação** (seção fechada); senão no **cabeçalho da seção (o projeto) da ação**, e não no do primeiro projeto; só sem ação conhecida, no cabeçalho do blueprint (ou no cartão da lista). Os agentes do mesmo lugar ficam lado a lado, e o lugar do cabeçalho é por seção. O Claude esperando segue a mesma regra (barra da ação aberta; senão cabeçalho da seção). Para o agente ficar em cima do plano, ele precisa mandar `acao_id` e `plano_id` no `registrar_progresso`; o hook de edição herda os do último aviso.
- **Aparecer e sumir:** ao entrar na folha a figura **cresce a partir do centro** (`scale` de 0 a 1, 380 ms) e ao sair **encolhe até o centro** (de 1 a 0, 300 ms) antes de ser removida (`claudeEntra` e `claudeSai` em `web/js/claude-camada.js`). Sem animação com `prefers-reduced-motion` ou aba oculta.

## Quantos aparecem

- **Um Claude principal** e **até `MAX_SUBAGENTES` = 7 figuras de subagente**. Até 5 figuras ficam lado a lado; com mais, elas **se juntam** (o passo entre elas encolhe e elas se sobrepõem, a mais perto do lugar, o principal, por cima) e o grupo nunca passa da largura de 5 (decisão do usuário, 2026-10-23: nada de contador "+N"). Passou de 8, as demais não são desenhadas e a dica da última figura diz quantas estão juntas ali. Ordem das figuras: quem avisou há menos tempo primeiro; o agente **nunca sai da folha por falta de sinal**: depois de concluir (20 s comemorando no plano) ou de 10 min sem avisar (`VIDA_MS`) ele vai para o **repouso**: pose `parado`, sem plano, no cabeçalho do blueprint (a figura anda até lá), e fica assim até chegar aviso novo; só sai ao reiniciar o app (a presença é só memória) ou quando o 33º agente o substitui. O filtro "Em andamento" conta só quem está num plano (trabalhando), não os que repousam. O hook de edição herda a ação e o plano do último aviso do agente, para a edição não o tirar do plano.
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
