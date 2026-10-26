# Terminal do app: o Claude Code oficial dentro do The Blueprint

Decisão de 2026-10-26 (ação "Claude Code dentro do app", projeto "Integração nativa com o Claude"). A pedido do usuário, ela reabre a decisão de
2026-10-21 que tirou o terminal embutido (xterm e node-pty), mas segue as regras de hoje: poucas crates, carregamento sob demanda, zero trabalho
em repouso, binário abaixo de 5 MB, só loopback.

## O que foi decidido

**Opção C (híbrida): um pseudoconsole do Windows (ConPTY) roda o `claude` oficial, o xterm.js desenha na página, e os hooks e o MCP que já
existem organizam em volta.** O processo é exatamente o que o terminal externo abre (`blueprint claude "<prompt>" --canal`, `src/cli/claude.rs`),
então a sessão tem tudo o que o Claude Code instalado tem: comandos de barra, plugins, skills, hooks, MCP, permissões, subagentes, o canal do app.

| Opção | O que é | Peso | Atraso de tecla | O que não consegue |
| --- | --- | --- | --- | --- |
| A. ConPTY + emulador | o `claude` de verdade num pseudoconsole, bytes até a página | 296 KB embutidos (68 KB com gzip), carregados só ao abrir | eco p50 2 a 19 ms, p95 ~20 ms (medido) | nada do Claude Code fica de fora; só depende do ConPTY (Windows 10 1809+) |
| B. `claude -p --output-format stream-json` + interface própria | modo de impressão, eventos JSON | sem emulador, mas tudo o que aparecer na tela precisa ser reescrito | não medido (cada turno é um processo) | sem a interface interativa: comandos de barra, prompts de permissão (exige `--permission-prompt-tool`), seletores, modo de plano, telas de plugin; contradiz "todas as capacidades" |
| C. A + hooks/MCP em volta | A para a sessão; o Claude na folha, as sessões e o estado vêm do que já existe | o de A | o de A | o de A |

A opção B não foi prototipada com um Claude real porque gastaria tokens; a recusa vem da documentação do modo `-p` (não interativo) e do pedido
explícito de ter todas as capacidades.

## Peças

- **PTY (`src/terminal/conpty.rs`):** chamadas diretas a `CreatePseudoConsole`, `CreateProcessW` com `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE`,
  `ResizePseudoConsole` e `ClosePseudoConsole` pelo `windows-sys` que o app já usa. **Nenhuma crate nova**: só as features `Win32_System_Pipes`
  e `Win32_System_JobObjects`. O `portable-pty` traria várias crates (filedescriptor, serial, winreg, shared_library, log...) para o mesmo
  punhado de chamadas. O processo nasce suspenso e entra num Job Object com `KILL_ON_JOB_CLOSE` antes de rodar: fechar a sessão, ou o app cair,
  derruba a árvore inteira (claude, node, git), sem processo órfão.
- **Sessões (`src/terminal/mod.rs`):** na memória do servidor, no máximo 8 vivas e 16 guardadas; cada uma guarda os últimos 256 KB da saída
  para quem reconecta. Duas threads por sessão, ambas paradas em chamadas bloqueantes (ReadFile e WaitForSingleObject): zero CPU em repouso.
  A janela pode ir para a bandeja (o WebView some) e o Claude continua; ao voltar, a página reconecta e recebe o histórico.
- **Transporte (`src/terminal/rotas.rs`, `fluxo.rs`):** **SSE para a saída e POST para as teclas**, pelo servidor HTTP que já existe. O SSE
  reaproveita o `Corpo::Fluxo` do servidor síncrono (tiny_http, uma thread por conexão); o WebSocket exigiria o handshake com SHA-1 e o
  enquadramento (uma crate ou ~150 linhas) para ganhar pouco: no loopback o POST por tecla custa ~1 ms, e a página junta o que for digitado
  enquanto um POST anda (`termEnviador`). O texto vai como string JSON (UTF-8 partido entre dois pedaços espera o resto). Contrapressão: cada
  página tem uma fila de 256 pedaços; se encher, ela é desligada e reconecta com o histórico, e o leitor do pseudoconsole nunca espera por ela.
- **Emulador (`web/vendor/xterm/`):** xterm.js 5.5.0 + addon-fit (MIT, aviso no `NOTICE.md`), embutidos no binário e carregados por `import()`
  só quando o terminal abre. Escrever um emulador mínimo não compensa: a interface do Claude Code (Ink) usa movimento de cursor, cores de 24 bits,
  bracketed paste, OSC 8 e redesenho parcial, e qualquer falha quebra a tela. Alternativas como o hterm são maiores.
- **Página (`web/js/terminal*.js`, `web/css/terminal.css`):** painel preso embaixo, uma aba por sessão, Dividir em duas telas, fonte, leitor de
  tela, maximizar. Só as telas visíveis existem: aba escondida não tem xterm nem conexão, e o painel fechado não tem nenhuma. Tema a partir de
  `web/css/tokens.css` (papel azul, tinta, mono do app), `minimumContrastRatio` 4,5, cursor fixo (piscar é trabalho em repouso).
- **Iniciar com o Claude:** o diálogo "Iniciar a ação" oferece "Terminal do app" (padrão), "Nova sessão" (o terminal externo, como antes) e as
  sessões já abertas. `POST /api/terminal/iniciar` reaproveita `mcp::iniciar` (mesma pasta ligada, mesmo prompt) com outro lançador.
- **Estado da aba:** vem dos avisos do agente (`registrar_progresso` e hooks, já na página em `agentesLista`): o último aviso na ação da sessão.

## Segurança

- Rotas `/api/terminal/*` só respondem com Host e Origin em loopback, e nem `MCP_REDE=1` libera (elas abrem processo e digitam nele). Todo POST
  exige `application/json`. Sem token na URL.
- Uma sessão só abre pelo clique confirmado no diálogo "Iniciar a ação". O app não digita nada por conta própria e não manda trabalho a ninguém.
- O processo herda o ambiente do usuário; nada da saída é gravado em disco nem em log (o histórico fica só na memória do servidor).
- `BLUEPRINT_TERMINAL_COMANDO` troca a linha de comando do `claude` por outra: existe para os testes de ponta a ponta (sem tokens).

## Testes

- `cargo test`: `src/terminal/tests.rs` (UTF-8 partido, citação da linha de comando, SSE, `cmd /c echo` com código de saída, sessão interativa
  que lê teclas, redimensiona e fecha sem deixar o neto vivo, conferido no `tasklist`) e `src/http/tests/terminal_rotas.rs` (loopback, JSON,
  iniciar sem pasta, iniciar com o programa falso).
- `node terminal-logica.test.mjs`: tema e contraste, fonte, rótulo e estado da aba, links, atalhos, fila de teclas.
- `node scripts/terminal-e2e.mjs [exe]` (~20 s, Edge sem janela, cmd falso): carga sob demanda, tema, eco, duas sessões, teclado, leitor de tela,
  esconder e reabrir com histórico; imprime as medidas.
- `node scripts/iniciar-botao-e2e.mjs [exe]`: o caso "Terminal do app" ao lado do terminal externo.

## O que ficou de fora (por ora)

- Busca no histórico (o addon de busca do xterm), arrastar abas e escolher cores próprias.
- Conferir com um Claude real (comandos de barra, um plugin, a sessão ligada à ação na folha): é o checkpoint do usuário.
- Fora do Windows o terminal do app responde que só existe no Windows (o app é só Windows).
