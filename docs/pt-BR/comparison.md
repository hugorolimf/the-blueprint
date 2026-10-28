# Blueprint e os parecidos

[English](../comparison.md)

Uma comparação honesta, com o que foi possível verificar nas páginas dos projetos em 25/10/2026 (fontes e números em [docs/interno/pesquisa-referencias.md](interno/research.md)). Onde o Blueprint perde, está escrito.

| | **The Blueprint** | [Backlog.md](https://github.com/MrLesk/Backlog.md) | [Task Master](https://github.com/eyaltoledano/claude-task-master) | [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | [Plane](https://github.com/makeplane/plane) |
| --- | --- | --- | --- | --- | --- |
| Em uma frase | roadmap local-first que o seu agente lê e atualiza pelo MCP, numa folha de blueprint | gerenciador de tarefas em Markdown e quadro kanban para qualquer repositório Git, para humanos e agentes | sistema de tarefas para desenvolvimento com IA, para usar em Cursor, Windsurf e outros | quadro kanban para coordenar agentes de código | alternativa open source ao Jira e ao Linear |
| Licença | MIT | MIT | "Other" (ler o LICENSE) | Apache-2.0 | AGPL-3.0 |
| Estrelas | 3 | ~7 mil | ~28 mil | ~28 mil | ~61 mil |
| Última release | v0.4.0 | v1.55.3 (23/10/2026) | 0.43.1 (31/03/2026) | v0.1.44 (24/04/2026) | v1.4.3-rc2 (23/10/2026) |
| Onde ficam os dados | banco SQLite no seu computador | arquivos Markdown no repositório | veja o repositório | veja o repositório | servidor próprio ou nuvem |
| Sistemas | Windows (janela, bandeja, instalador); o servidor roda em qualquer sistema | npm, bun, brew, nix | npm | npx | Docker ou nuvem |

## O que é diferente no Blueprint

- **O agente propõe, você aplica.** Tudo o que o agente envia é uma proposta numa fila numerada; nada muda nos seus dados antes de você aplicar na tela **Planejar**.
- **Concluir exige evidência.** Um plano só é marcado como feito com o comando ou o teste rodado e o resultado, que vira comentário do plano.
- **Uma folha, não só uma lista.** Workspace → Blueprint → Projeto → Ação → Plano e Checkpoint, com prazos no ritmo da IA e tempo real medido.
- **Você vê o agente trabalhando.** O Claude aparece na folha, no plano em que está, com os subagentes.
- **Pequeno e local.** Um executável de cerca de 2,5 MB, sem telemetria e sem conta.

## Onde os outros são melhores

- **Maturidade e comunidade.** O Blueprint está na v0.1 e tem 3 estrelas; todos os outros têm anos de uso e comunidade.
- **Plataformas.** O Blueprint é Windows na janela; Backlog.md, Task Master e Vibe Kanban instalam por npm em qualquer sistema.
- **Equipes.** O Plane tem usuários, papéis, sprints e documentos prontos; o modo colaborativo do Blueprint ainda é só desenho (veja o [ROADMAP](../../ROADMAP.pt-BR.md)).
- **Dados no repositório.** O Backlog.md guarda as tarefas em Markdown no próprio Git, o que dá histórico e revisão por pull request; o Blueprint guarda num banco local (e exporta para JSON).

## Qual escolher

- Quer tarefas versionadas junto do código, em Markdown: **Backlog.md**.
- Quer decompor um PRD em tarefas dentro do seu editor: **Task Master**.
- Quer um quadro para vários agentes em paralelo: **Vibe Kanban** (comunitário desde abril de 2026) ou o **ORCA**.
- Quer gerenciar a equipe inteira: **Plane**.
- Quer controlar o que o seu agente planeja, aprovar cada mudança e exigir prova do que foi feito, numa folha que mostra o projeto todo: **The Blueprint**.
