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
