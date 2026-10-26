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
