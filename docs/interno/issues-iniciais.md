# Issues iniciais (`good first issue`)

> Portuguese only for now (a technical reference for maintainers). Documentação só em português por enquanto.

1. **Exportar um blueprint para Markdown** — `blueprint exportar-md <banco> <blueprint_id>` gerando ações, planos e checkpoints em texto. *Validação:* teste em `src/` com o `data.exemplo.json`.
2. **Mostrar `tempoReal` na folha** — o MCP já devolve a média real de horas por plano (com 3 planos medidos); mostrar no cabeçalho do blueprint. *Validação:* captura nova em `scripts/capturas.mjs`.
3. **Tema claro** — o app tem um tema só (papel azul de plotagem); propor um conjunto de variáveis claro em `web/css/tokens.css` e a troca no cabeçalho. *Validação:* as 7 capturas nos dois temas.
4. **Atalhos de teclado na folha** — setas para percorrer ações e Espaço para marcar o plano em foco. *Validação:* teste em `scripts/ui-e2e.mjs`.
5. **Assinar as atualizações** (`help wanted`) — o updater confere o SHA-256 do instalador, mas ele vem da mesma release. Assinar o `SHA256SUMS` com ed25519 (chave pública embutida no app) ou o instalador com Authenticode. *Validação:* teste em `src/atualizador/` com assinatura válida, inválida e ausente.
