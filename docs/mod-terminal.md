# Marca do Blueprint no Claude Code

> Portuguese only for now (a technical reference for maintainers). Documentação só em português por enquanto.

O lançador `blueprint claude` extrai e carrega `blueprint-terminal` com um segundo
`--plugin-dir`. O plugin principal continua fornecendo skill, agente, MCP e hooks.
A marca não consulta dados do app, não inicia processos e não usa polling.

A marca aparece só numa faixa acima do prompt, em qualquer largura (decisão de
2026-10-23: sem painel lateral). O botão “fechar ×” da faixa a esconde;
`/blueprint-marca` a reabre. Fechar preserva a escolha na sessão, inclusive durante
recarga do mod. A faixa mostra a imagem, “The Blueprint” e a linha curta do
blueprint da pasta.

Raster usa células com blocos de meia altura e cores reais em qualquer terminal.
Image usa o mesmo PNG em kitty, Ghostty e iTerm2 3.5 ou posterior. O PNG tem
16.569 bytes; as grades têm 32×16 e 16×8 células, cada uma com 12 bytes
(`[glifo, frente, fundo]`, u32 little endian). Transparência usa espaço ou bloco
inferior, para não transformar o fundo transparente em traço branco.

`turn.complete` gera um toast sem chamadas ao app. `classic.Notification` não gera toast (decisão de 2026-10-22: o aviso abria em dois cantos);
o hook `Notification` do plugin principal chama
`blueprint hook notificacao`: retorna `terminalSequence` com OSC 9, sem controles
vindos da mensagem. Esse aviso funciona também sem carregar o mod; sua exibição
depende das notificações habilitadas no terminal.

## Evidências em 22/10/2026

Claude Code local: **2.1.309**. Foram consultados os contratos oficiais e os tipos
que essa versão escreveu ao carregar o mod. A API é experimental e pode mudar.

| Verificação | Resultado |
| --- | --- |
| `node scripts/marca-terminal.mjs` | PNG e células gerados pelo Edge sem janela |
| `node scripts/marca-terminal.test.mjs` | Assinatura PNG, limite, equivalência dos assets e tamanho de células corretos |
| `claude plugin validate claude-plugin/plugins/blueprint-terminal` | Passou; aviso editorial sobre ícone para publicação no catálogo |
| `claude plugin test claude-plugin/plugins/blueprint-terminal` | 5 passaram: Raster 100/160 colunas, eventos/toasts, fechamento/recarga/reabertura e Image em kitty |
| `cargo test hook::tests` | 5 passaram, incluindo Notification com pedido de resposta e bloqueio de injeção de escapes |
| `cargo test cli::claude` | 4 passaram, incluindo dois `--plugin-dir` e caminhos com espaços |
| `cargo test registrar::tests::grava_o_plugin` | Extração integral dos assets e Notification passou |

Foi aberta uma sessão interativa real no PTY do agente com `--plugin-dir`, sem
enviar prompt ao modelo. A marca apareceu, `/blueprint-marca` abriu o painel e o
debug confirmou carregamento e renderização sem hooks ignorados do mod. O log
temporário fica em `target/mod-terminal-debug.log` (não versionado).

O teste de fechamento chama o handler com `origin.kind = person` e exercita o
estado durante recarga e a reabertura. Os demais testes do mod usam o host real
de `claude plugin test`. A API do harness não expõe `$.ui.close` diretamente.

O checkpoint **“Usuário viu a marca no próprio terminal”** continua pendente:
renderização no PTY e testes não substituem a observação do usuário. Image em
kitty/Ghostty/iTerm2 foi validado pelo host com ambiente simulado, sem esses
terminais instalados nesta máquina.

## Referências

- [Visão geral de mods](https://code.claude.com/docs/en/plugins/mods/overview)
- [Referência da API](https://code.claude.com/docs/en/plugins/mods/reference)
- [Hooks e terminalSequence](https://code.claude.com/docs/en/hooks)

Para regenerar os assets após mudar a marca: `node scripts/marca-terminal.mjs`.
