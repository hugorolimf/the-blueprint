# Issues iniciais (`good first issue`)

1. **Ícone embutido no `blueprint.exe`** — hoje o executável não tem ícone no Explorer (a janela e a bandeja têm). Embutir o recurso de ícone no build (Windows). *Validação:* o Explorer mostra o ícone.
2. **Exportar um blueprint para Markdown** — `blueprint exportar-md <banco> <blueprint_id>` gerando ações, planos e checkpoints em texto. *Validação:* teste em `src/` com o `data.exemplo.json`.
3. **Mostrar `tempoReal` na folha** — o MCP já devolve a média real de horas por plano (com 3 planos medidos); mostrar no cabeçalho do blueprint. *Validação:* captura nova em `scripts/capturas.mjs`.
4. **Tema claro** — o app tem um tema só (papel azul de plotagem); propor um conjunto de variáveis claro em `web/css/tokens.css` e a troca no cabeçalho. *Validação:* as 7 capturas nos dois temas.
5. **Atalhos de teclado na folha** — setas para percorrer ações e Espaço para marcar o plano em foco. *Validação:* teste em `scripts/ui-e2e.mjs`.
