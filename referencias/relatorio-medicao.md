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
