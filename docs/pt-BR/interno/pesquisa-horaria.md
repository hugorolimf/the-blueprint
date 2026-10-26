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

### Quinta rodada, 26 de outubro de 2026

O Task Master deixa escolher quais ferramentas MCP carregar com a variável de ambiente `TASK_MASTER_TOOLS` (`all`, `standard`, `core` ou uma lista própria; o padrão é um conjunto enxuto de sete), conforme a [documentação de MCP](https://docs.task-master.dev/capabilities/mcp.md). A data em que isso entrou não foi confirmada. Isso reforça a ação já planejada "Perfis de ferramentas do MCP", então nenhuma ação nova foi criada. Focalboard: só apareceu um servidor MCP da comunidade e nada de 2026.

Decisão da rodada: o blueprint já tem 77 ações, então esta rodada só registra a fonte e passa a executar.

### Sexta rodada, 26 de outubro de 2026

Nenhuma entrada do changelog do Claude Code de outubro de 2026 apareceu. Guias de terceiros ([claudelog](https://www.claudelog.com/faqs/what-is-plugins-system-in-claude-code/), [fast.io](https://fast.io/resources/claude-code-plugins-development-guide.md)) dizem que o Claude Code carrega as ferramentas MCP sob demanda ("MCP Tool Search", até 95% menos contexto) e lista `http` e `mcp_tool` entre os tipos de hook; as duas afirmações não foram conferidas na documentação oficial. Se o carregamento sob demanda for real, os "Perfis de ferramentas do MCP" valem sobretudo para agentes que não o têm (o Codex entre eles), então a ação deve começar medindo o custo de contexto em cada agente. Nenhuma ação nova.

### Sétima rodada, 26 de outubro de 2026

Backlog.md: a busca não devolveu nada sobre o projeto, então não há fonte nova. O [changelog do Plane](https://plane.so/changelog/commercial) lista Releases e Collections no Wiki na versão self-hosted 2.6.0 (18/05/2026), um widget "Catch Up" na tela inicial do celular com o modo Autopilot e Skills do Plane AI na versão mobile 2.4.0 (31/07/2026) e o progresso de subitens contando os cancelados (3.3.2, 07/10/2026); isso vem de um resumo da busca, não da leitura de cada entrada. A ideia do "Catch Up" já é coberta pelo "resumo de progresso por período" planejado, e contar cancelados não se aplica (o Blueprint não tem estado cancelado). Nenhuma ação nova; a rodada volta a executar.

### Oitava rodada, 26 de outubro de 2026

Claude Code, outubro de 2026 (só fontes de terceiros: [Havoptic](https://www.havoptic.com/releases/claude-code/2026/10) e um [resumo de outubro](https://angelo-lima.fr/en/claude-code-cheatsheet-october-2026-state-of-play-en/); não conferido em code.claude.com): versões 2.1.287 a 2.1.296 entre 1 e 9/10, com hooks que podem bloquear a ação quando falham, instalação de plugin direto do marketplace, ajuste de quanto os subagentes se esforçam e 33 eventos de hook. Os hooks do Blueprint são só indicadores de propósito (nunca bloqueiam o agente) e o plugin já é um marketplace local, então nada se aplica. Backlog.md (1.48.0) e Task Master (0.43.1): versões de páginas de índice sem data, sem notas de release de outubro de 2026. Nenhuma ação nova; a rodada volta a executar o projeto "Integração nativa com o Claude".

### Nona rodada, 26 de outubro de 2026

Colaboração, porque o projeto "Colaboração em tempo real" começou hoje. Plane ([changelog](https://plane.so/changelog/cloud)): edição sincronizada com cursores dos membros nas Pages desde set/2024 ([entrada](https://plane.so/changelog/cloud-sep-20-2024)), histórico de versões com diferenças por pessoa nas Pages (fev/2026, [entrada](https://plane.so/changelog/2026-02-01-product-tour-try-section-and-plane-ai-updates)) e visões compartilhadas com pessoas específicas (ago/2026, de um resumo da busca). Um [pedido no fórum](https://forum.plane.so/t/formal-request-real-time-collaboration-websockets-should-be-prioritized-over-new-features/276) (sem data) diz que as mudanças dos colegas nos itens só aparecem depois de recarregar, o que confirma que a atualização ao vivo por item (ação planejada "Presença e mudanças ao vivo") importa mais que editar texto junto. Focalboard: só páginas de hospedagem; o primeiro a se cadastrar vira admin ([Railway](https://railway.com/deploy/focalboard)), onde o Blueprint imprime uma vez o link do dono. Linear e Vibe Kanban: nada encontrado. Ideia guardada para o desenho da colaboração, não como ação nova: histórico por item com quem mudou o quê, sobre o registro de atividade planejado. Nenhuma ação nova.

### Décima rodada, 26 de outubro de 2026

Linear (só fontes secundárias; nenhuma entrada de outubro de 2026 em linear.app/changelog): o Linear Agent saiu do beta em junho de 2026 ([PricingSaaS, semana 26](https://pricingsaas.com/companies/linear/diffs/2026W26)), e a Agent API faz agentes de fora, como o Claude Code, aparecerem como membros do workspace que recebem issues e postam atualizações ([guia da BuildBetter](https://blog.buildbetter.ai/linear-ai-agents-2026-guide-5-alternatives-for-engineering-teams/), de fornecedor). Vibe Kanban: a v0.0.164 (29/01/2026) mostra a conversa dos subagentes na interface ([newreleases.io](https://newreleases.io/project/github/BloopAI/vibe-kanban/release/v0.0.164-20260129121853)); o Blueprint já mostra os subagentes na folha e guarda a linha do tempo por ação. Task Master: nenhuma nota de release de 2026.

Ideia acrescentada ao blueprint App (planejada, não iniciada): responsável por plano, uma pessoa ou o agente dela, com filtro "Meus planos" e o MCP dizendo a cada agente quais planos são seus. Fica no projeto de colaboração em tempo real e depende dos membros.

Também acrescentado ao `roadmap.json`: `responsavel-plano`.

### Décima primeira rodada, 26 de outubro de 2026

Túneis, para o botão Compartilhar que está sendo construído hoje (fontes secundárias; a documentação da Cloudflare não foi lida). O túnel rápido da Cloudflare não pede conta e dá um endereço aleatório em trycloudflare.com até o processo parar, é chamado de ferramenta de demonstração sem garantia, e um guia relata limite de cerca de 200 pedidos em andamento e falta de Server-Sent Events ([Flavio Copes](https://flaviocopes.com/cloudflare-quick-tunnels/)); o túnel nomeado é grátis com conta e domínio na Cloudflare e mantém o endereço ([mesmo autor](https://flaviocopes.com/cloudflare-tunnel/)). O Tailscale Funnel expõe um serviço local à internet por HTTPS, por um relay que não decifra o tráfego; quem visita não precisa de Tailscale ([docs do Tailscale](https://tailscale.com/kb/1223/tailscale-funnel)). A falta de SSE importa: as mudanças ao vivo usam SSE, então o construtor vai medir com o cloudflared real e pôr um plano B se precisar.

Ideia acrescentada ao blueprint App (planejada, não iniciada): endereço fixo para o compartilhamento (túnel nomeado ou Tailscale Funnel). Também no `roadmap.json`: `compartilhar-workspace` (agora).
