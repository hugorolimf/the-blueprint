# Registro da pesquisa horária

[English](../../interno/pesquisa-horaria.md)

Uma entrada por rodada. As fontes são resultados de busca na web, algumas secundárias; confira a página oficial antes de citar.

## 26 de outubro de 2026

Fontes:
- Backlog.md (MrLesk/Backlog.md): tarefas como arquivos Markdown no Git, quadro no terminal e na web, completar comandos em Zsh, Bash, Fish e PowerShell, MCP para agentes. Pelo [README do projeto](https://cdn.jsdelivr.net/gh/MrLesk/Backlog.md@main/README.md) e pela [página de uma palestra](https://ai.engineer/talks/backlog-md-terminal-kanban-board-for-managing-tasks-with-ai-agents-alex-gavrilescu-funstage). O número da versão (1.48.0) veio de um índice e não foi verificado.
- Linear e Claude Code: a busca não achou entrada oficial ligando os dois. Fontes secundárias (uma comparação da Firecrawl de junho de 2026 e um changelog de comunidade) só confirmam que o Claude Code lê o AGENTS.md quando não há CLAUDE.md. Nada a aproveitar.

Ideias acrescentadas ao blueprint App (planejadas, não iniciadas):
1. Exportar o blueprint como arquivos Markdown (um por ação mais um índice), só leitura.
2. Importar issues do GitHub como proposta numerada pela fila que já existe, sem gravar antes do clique.
3. Completar comandos no PowerShell e no Bash.

Também entraram no `roadmap.json`: `exportar-markdown`, `importar-issues`, `completar-comandos`.

### Segunda rodada, 26 de outubro de 2026

Fontes:
- Plane Agents, lançado em 18 de setembro de 2026: os agentes têm instruções, habilidades, gatilhos (atribuído, mencionado, item alterado, agenda), memória e ferramentas MCP, e um painel de execução mostra o plano e as ferramentas chamadas. Pelo [anúncio do Plane](https://plane.so/blog/agents-are-now-live-in-plane) e pelo [changelog de 30 de setembro de 2026](https://plane.so/changelog/2026-09-30-plane-agents-propel-new-ai-models). Os créditos e limites de plano vêm de um agregador e não foram verificados.
- Um servidor MCP da comunidade para o Plane limita as ferramentas com `PLANE_MCP_PROFILE=worker|planner|full` ([pkg.go.dev](https://pkg.go.dev/github.com/ItsJennyFiggy/plane-mcp@v1.2.0), junho de 2026). Mantido pela comunidade, não é oficial.
- Task Master: a busca não achou notas de versão, então nada foi aproveitado.

Ideias acrescentadas ao blueprint App (planejadas, não iniciadas):
1. Perfis de ferramentas do MCP (`BLUEPRINT_MCP_PERFIL`), para o agente carregar menos descrições.
2. Linha do tempo do agente em cada ação, só leitura, com limite de retenção.

Também entraram no `roadmap.json`: `perfis-mcp`, `linha-do-tempo-agente`.

### Terceira rodada, 26 de outubro de 2026

Fontes:
- Linear: [Linear Agent](https://linear.app/changelog/2026-03-24-introducing-linear-agent) (março de 2026) e uma entrada "Agent assisted project updates" de 18 de junho de 2026; sessões de código e loops (fluxos recorrentes de agente) aparecem só em um rastreador de terceiros, então as datas não foram verificadas. Confira em [linear.app/changelog](https://linear.app/changelog/timeline).
- Vibe Kanban: cada workspace tem o seu worktree e a sua branch, e os comentários no diff voltam direto ao agente ([Virtus Lab](https://virtuslab.com/blog/ai/vibe-kanban/), [README do fork](https://github.com/MrEggs/vibe-kanban-fork)). A empresa encerrou em abril de 2026 e o projeto é mantido pela comunidade; a data e a atividade do fork não foram confirmadas.

Ideias acrescentadas ao blueprint App (planejadas, não iniciadas):
1. Resumo de progresso por período, em Markdown, calculado dos dados que já existem (sem IA, sem rede).
2. Enviar o comentário do plano ao agente, reaproveitando o canal por clique e por sessão (sem ampliar a exceção).

Também entraram no `roadmap.json`: `resumo-progresso`, `comentario-ao-agente`.

### Quarta rodada, 26 de outubro de 2026

Fontes (todas secundárias; as releases oficiais do `openai/codex` não foram lidas):
- Codex CLI: hooks estáveis desde a v0.124.0, de 23 de abril de 2026, lendo um `hooks.json` e observando as ferramentas MCP e o `apply_patch` ([Daniel Vaughan](https://codex.danielvaughan.com/2026/04/23/codex-cli-hooks-graduate-stable-v0124-mcp-observation-inline-config/)); opções `codex plugin ... --json` na v0.137 e na v0.138 alpha e `codex agents` na v0.148 alpha ([Tech Dev Notes](https://techdevnotes.com/releases/codex-cli/0.137.0), [changelog do Gradually](https://www.gradually.ai/en/changelogs/codex-cli/)).
- ORCA v1.4.235 (20 de outubro de 2026, via [Releasebot](https://releasebot.io/updates/stablyai/orca)): rebobinar o chat, configurações por chat, saída completa do agente guardada, chats no celular retomam após reconectar. Nada a aproveitar: o The Blueprint não executa agentes, e a página já relê a fila e os dados quando o fluxo de eventos reconecta (`web/js/mcp.js`).

Ideia acrescentada ao blueprint App (planejada, não iniciada): hooks do Codex para o progresso automático. Depende de a ação de pesquisa do Codex confirmar o formato antes.

Também entrou no `roadmap.json`: `hooks-codex`.
