# Medição final: Node (linha de base) × app Rust

Medido em 2026-10-22, Windows 11, `target/release/blueprint.exe` (perfil release com `lto`, `codegen-units = 1`, `panic = "abort"`, `strip`, `+crt-static`).
Reproduzir: `node scripts/medir-rust.mjs` e `node scripts/medir-janela.mjs`. Dados brutos em `referencias/medicao-rust.json` e `referencias/medicao-janela.json`.

| Métrica | Meta | Node (antes) | Rust (agora) | Atende |
| --- | --- | --- | --- | --- |
| Binário | < 5 MB | n/a (precisa do Node instalado) | 2,5 MB | sim |
| Partida do servidor até o 1º 200 | — | 164 ms | 177 ms | — |
| Memória do servidor após 20 requisições | < 20 MB | 60,4 MB | 10,9 MB | sim |
| CPU em repouso, 10 s com SSE aberto | 0% | 0% | 0% | sim |
| p95 `GET /api/dados` (http persistente) | < 5 ms | — | 4,43 ms | sim |
| p95 `GET /api/mcp/fila` (http persistente) | < 5 ms | — | 0,42 ms | sim |
| p95 `GET /api/dados` (fetch, com abertura de conexão) | — | 18,7 ms | 16,6 ms | — |
| Janela visível (fria) | < 500 ms | — | 111 ms | sim |
| Janela visível (quente, média de 2) | < 500 ms | — | 131,5 ms | sim |
| App na bandeja (sem WebView2) | — | — | 26,3 MB | — |
| Janela aberta (blueprint.exe + 6 processos WebView2) | — | — | 381,9 MB (soma conta páginas compartilhadas mais de uma vez) | — |

## Limites no CI

O job `desempenho` de `.github/workflows/test.yml` compila em release no `windows-latest` e roda `node scripts/medir-rust.mjs target/release/blueprint.exe --limites`, que sai com erro se uma medida passar do limite. O runner oscila, então os limites do CI são folgados e só um desvio claro reprova; as metas finas continuam valendo nesta tabela de medição.

| Meta (CLAUDE.md) | Limite no CI | Opção para trocar |
| --- | --- | --- |
| Binário < 5 MB | 5 MB (sem folga: o tamanho não oscila) | `--binario-mb=N` |
| `/api/*` < 5 ms | p95 de `GET /api/dados`, `GET /`, `GET /api/mcp/fila` e `POST /api/mcp/fila/ack` < 15 ms (http persistente, 200 pedidos, a melhor de 3 rodadas por rota) | `--p95-ms=N` |
| Backend em repouso < 20 MB | memória depois de 10 s de repouso com SSE aberto < 40 MB | `--rss-mb=N` |
| 0% de CPU em repouso | < 1% de um núcleo nos mesmos 10 s | `--cpu-pct=N` |

Mudar um limite é mudar o padrão em `scripts/medir-rust.mjs` e esta tabela no mesmo PR, com a medição que justifica; quem decide é quem mantém o projeto (veja `CONTRIBUTING.md`). Sem histórico de tendência: só limite fixo.

### Prova local de que cada guarda reprova (2026-10-27)

Exe release (`cargo build --release`, Windows 11, máquina com outras compilações rodando). A regressão proposital foi um patch temporário num build de teste, desfeito antes do commit: nada dela existe no binário publicado.

| Caso | Medido | Saída | Tempo |
| --- | --- | --- | --- |
| Exe limpo (2 execuções) | binário 4,6 MB; p95 0,6 a 0,9 ms nas 4 rotas; 10,9 MB e 0% de CPU em repouso | 0 | ~25 s |
| Exe de teste sem regressão ligada | dentro dos limites | 0 | 32 s |
| Sleep de 50 ms em `/api/mcp/fila*` | p95 `GET /api/mcp/fila` 86,4 ms, `POST /api/mcp/fila/ack` 88,3 ms | 1 | 121 s |
| Fio em repouso: 2 ms de trabalho a cada 20 ms | CPU 7,9% | 1 | 18 s |
| 64 MB presos ao subir | memória 74,9 MB | 1 | 17 s |

- A latência usa a melhor de 3 rodadas porque, com a máquina ocupada, uma rodada isolada do exe limpo variou de 0,9 a 23,3 ms no p95 de `GET /api/dados` (vermelho falso); uma rota lenta de verdade estoura as 3.
- Limite conhecido: polling leve passa. O mesmo fio a cada 100 ms deu 1,7% (reprova), a cada 1 s deu 0,3% (passa). A guarda pega laço apertado, não um timer lento; esse continua sendo pego só na revisão.
- O binário está em 4,6 MB, perto do limite de 5 MB: uma crate média já reprova.

## Leitura

- A memória do servidor caiu de 60 para 11 MB. A partida ficou parecida (o SQLite abre e migra no começo).
- A meta de p95 < 5 ms só vale com cliente HTTP persistente. Com `fetch` do Node, que abre conexão a cada pedido, o p95 é dominado pelo cliente (16,6 ms).
- A janela pesa por causa do WebView2 (processos do Windows, fora do nosso controle). Ao ir para a bandeja eles são destruídos e sobram 26 MB.
- O `.exe` só importa DLLs do Windows (`node scripts/dlls-do-exe.mjs`), sem depender do VC++ Redistributable.

## Animações do Claude (2026-10-23)

Medido com `node scripts/medir-animacoes.mjs` (Edge headless por CDP, `Performance.getMetrics`; servidor com os dados de exemplo; exe de debug). Os números são a **mediana de fatias de 1 s** da CPU da thread principal da página, em % de um núcleo (um evento isolado, como o `render()` que o `rolagem.js` faz quando a janela headless muda de altura, aparece só no "pico"). Máquina com outros processos rodando em paralelo: os tetos do script deixam folga (repouso 0,5%, uma figura 2%, seis figuras 10%).

| Situação | Antes (quadros com `display:none`) | Depois (quadros empilhados, `opacity` em camada composta) | Teto |
| --- | --- | --- | --- |
| Lista de blueprints, nenhuma figura | 0,03% | 0,03 a 0,04% | 0,5% |
| Blueprint aberto, só o Claude esperando (1 figura animada) | 1,14% (3,4 layouts/s) | **0,34%** (0 layouts/s) | 2% |
| 6 figuras animadas (5 agentes em poses diferentes + a espera) | 10,9% (23,9 layouts/s, 27 recálculos/s) | **3,4 a 3,5%** (0 layouts/s, 22 a 24 recálculos/s) | 10% |
| Aba oculta (6 figuras) | 0,01% | 0,03% (0 relógios) | 0,5% |
| `prefers-reduced-motion` (6 figuras) | 0,02% | 0,03 a 0,04% (0 relógios, 0 animações) | 0,5% |

- **O que custava:** o relógio trocava `hidden` (display:none) entre os quadros, e cada troca refazia estilo, layout e pintura de um SVG de ~1500 nós. Uma tentativa intermediária com `visibility` ficou pior (8,1% para uma só figura), porque `visibility` é herdada e reestiliza todos os nós. O que funcionou: `.claude-fig` com tamanho fixo e `contain: layout style`, quadros `position:absolute` com `opacity:0` e `will-change: opacity`, e o relógio só troca a classe `on`.
- **Relógios (`setInterval`):** um por figura animada (6 figuras = 6 relógios) e nenhum a mais; ninguém andando = nenhum relógio de passos. Aba oculta, figura fora da tela e movimento reduzido: **0** relógios (a figura fora da tela para e volta ao reaparecer). Oito rodadas trocando a pose dos 5 agentes e depois removendo todos: nenhum relógio órfão (os que sobram são só o da figura de espera, que é da interface).
- **O SVG não é refeito a cada quadro:** 0 mutações `childList` na camada durante a janela (o ciclo só troca a classe).
- **Memória (6 figuras):** +3,4 a 5,9 MB de heap JS, +8.200 nós de DOM e 854 KB de HTML na camada (cada figura guarda todos os quadros da pose, até 5 SVGs de ~30 KB); com movimento reduzido cai para um quadro por figura.
- **Teto de 5 figuras + espera:** o custo é por figura animada (~0,5% de um núcleo cada, no regime). Com o limite de 1 principal + 4 subagentes + a espera, o pior caso fica em ~3,5%.
- **Ressalva do ambiente:** o Edge headless marca a página como oculta depois de uns 45 s sem foco; o script mantém a aba visível (`Emulation.setFocusEmulationEnabled` e `Page.bringToFront` a cada 3 s) e confere `document.visibilityState` no começo e no fim.

## Abrir e fechar as ações (ação 1.13, 2026-10-23)

Medido na seção (7) de `node scripts/medir-animacoes.mjs` (Edge headless, exe de debug, `BLUEPRINT_ASSETS_DIR` na pasta do repositório): um blueprint com 45 ações (180 planos e checkpoints na folha) recolhe e expande 6 vezes (12 cliques de `alternar`, 2,4 s entre eles). O movimento (350 ms de deslize, 150 ms de esmaecer, entrada de cima para baixo) está em `docs/expandir-recolher.md`.

| Medida | Resultado | Teto do script |
| --- | --- | --- |
| Animações ao mesmo tempo (pico, 120 ms depois do clique) | 258 | 700 |
| Maior trecho síncrono de um clique | 33,8 ms | |
| Tarefas longas (> 50 ms) | nenhuma | 250 ms |
| CPU média da página nos 12 cliques (inclui a pintura por software do Edge headless) | 10,8% de um núcleo | 15% |
| Animações no fim de cada clique | 0 | 0 |

- A maior parte das animações do pico é a de `desenharBarras` (barras da ação, já existente antes da ação 1.13); o deslize (`MAX_MOVIDOS`, 220) e a entrada (60) têm teto próprio e só valem para o que está na janela.
- Com movimento reduzido ou aba oculta nenhuma animação é criada (`scripts/expandir-e2e.mjs` confere).

## Terminal do app (ação "Claude Code dentro do app", 2026-10-26)

Medido em 2026-10-26, Windows 11, `target/release/blueprint.exe`, com um programa falso no pseudoconsole (`cmd.exe`; o `claude` real não entra nas
medições para não gastar tokens). Reproduzir: `node scripts/terminal-e2e.mjs target/release/blueprint.exe` (atraso, CPU da página, memória do
servidor) e a contagem de processos filhos do servidor com duas sessões abertas. Decisão e peças em `docs/terminal-no-app.md`.

| Métrica | Meta | Antes (main, da08b90) | Com o terminal | Atende |
| --- | --- | --- | --- | --- |
| Binário | < 5 MB | 3.360.256 B (3,20 MiB) | 3.709.952 B (3,54 MiB): +341 KB | sim |
| Peso do xterm.js embutido (xterm.js + addon-fit + css) | — | — | 296 KB (71 KB com gzip), baixado só ao abrir o terminal | — |
| JS e CSS do painel carregados com a página (terminal*.js, terminal.css, en-terminal.js) | — | — | 26 KB (10 KB com gzip) | — |
| Memória privada do servidor por sessão | — | — | +0,1 MB (histórico de até 256 KB e duas threads paradas) | — |
| Processo de console por sessão (conhost do pseudoconsole) | — | — | 1,5 MB, fora o programa (o claude custa o mesmo que no terminal externo) | — |
| CPU do servidor em 10 s de repouso com 2 sessões abertas | 0% | — | 0,0 ms | sim |
| CPU da página em 5 s de repouso com o terminal aberto (Edge sem janela) | 0% de trabalho do terminal | 86 a 117 ms sem o terminal (a folha e as figuras) | 14 a 16 ms com o painel aberto | sim (nenhum relógio do terminal: cursor fixo, SSE parado) |
| Eco de tecla (do evento de entrada à letra na tela: POST, pseudoconsole, cmd, SSE, desenho do xterm) | — | — | p50 2,3 a 2,8 ms, p95 18,7 a 20,3 ms (n=20, duas execuções) | — |

Leitura: o p95 de ~20 ms é o quadro de desenho do xterm (ele desenha no próximo `requestAnimationFrame`); o caminho servidor + pseudoconsole
fica abaixo de 3 ms. Com o painel escondido não há tela nem conexão; a sessão segue no servidor sem gastar CPU.
## Sincronização entre pessoas (frente SINCRONIA, 2026-10-26)

`node scripts/medir-sincronia.mjs` (binário de release do worktree, Windows 11, servidor e clientes na mesma máquina): N conexões SSE em
`/api/eventos` e 4 gravadores mandando 10 lotes cada em `/api/mudancas` ao mesmo tempo. Atraso = do envio do lote até cada cliente ver a
mudança (servidor + SSE, sem o navegador).

| Conexões SSE | Memória do servidor | Por conexão | Threads | CPU em 2 s parado | Lotes/s | Atraso p50 | p95 | Máx. | Perdidos |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 11,3 MB | — | 10 | — | — | — | — | — | — |
| 10 | 11,7 MB | 50 KB | 27 | 0 ms | 771 | 3,8 ms | 6,8 ms | 10,6 ms | 0 |
| 100 | 16,8 MB | 57 KB | 223 | 0 ms | 711 | 4,3 ms | 6,8 ms | 9,1 ms | 0 |
| 250 | 27,2 MB | 65 KB | 613 | 0 ms | 357 | 7,5 ms | 13,3 ms | 17,2 ms | 0 |

Leitura e limite: cada conexão SSE custa duas threads paradas (a de leitura do tiny_http e a do fluxo) e ~60 KB; em repouso não há CPU (só o
batimento de 25 s). Uma conexão que caiu só libera as threads no próximo envio ou batimento (por isso a contagem de 250 inclui as das rodadas
anteriores). Até 250 pessoas olhando o mesmo servidor o atraso fica abaixo de 20 ms; o limite prático do servidor síncrono é o número de
threads (o Windows aguenta alguns milhares), então para mais de algumas centenas de pessoas ao mesmo tempo o caminho é um laço de eventos no
SSE, não antes.

Na página: com 10 000 planos (3,7 MB de JSON), calcular a diferença do salvar (`opsEntre`) leva ~77 ms quando só há campos alterados e ~111 ms
com criar ou mover (a cópia da base só nasce nesse caso), e o lote mandado tem ~100 B contra os 3,7 MB do `PUT` inteiro de antes. No
`duas-pessoas-e2e` a mudança de uma página aparece na outra ~420 a 470 ms depois do `mudou()`, quase todo o tempo sendo os 400 ms de espera
do salvar.
