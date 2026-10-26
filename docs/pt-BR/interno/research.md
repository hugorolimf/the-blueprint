# Pesquisa de referências (25/10/2026)

[English](../../interno/research.md)

Registro do que foi consultado para o posicionamento, o README e o roadmap. Tudo veio da API pública do GitHub e de páginas oficiais em 25/10/2026; números mudam, refaça a consulta antes de citá-los.

## Projetos do gênero (GitHub API, 25/10/2026)

| Projeto | Estrelas | Licença | Último push | Última release | O que é (descrição do próprio repositório) |
| --- | --- | --- | --- | --- | --- |
| [stablyai/orca](https://github.com/stablyai/orca) | 105.958 | MIT | 26/10/2026 | v1.4.240 (25/10) | app para trabalhar com agentes em worktrees (a fonte de várias ideias do Blueprint) |
| [makeplane/plane](https://github.com/makeplane/plane) | 61.440 | AGPL-3.0 | 23/10/2026 | v1.4.3-rc2 (23/10) | "Open-source Jira, Linear, Monday, and ClickUp alternative" |
| [BloopAI/vibe-kanban](https://github.com/BloopAI/vibe-kanban) | 28.456 | Apache-2.0 | 19/09/2026 | v0.1.44 (24/04) | quadro kanban para coordenar agentes de código (a empresa encerrou em abril de 2026, o projeto segue comunitário) |
| [eyaltoledano/claude-task-master](https://github.com/eyaltoledano/claude-task-master) | 28.276 | "Other" (ler o arquivo LICENSE) | 28/04/2026 | 0.43.1 (31/03) | "An AI-powered task-management system you can drop into Cursor, Lovable, Windsurf, Roo, and others" |
| [mattermost-community/focalboard](https://github.com/mattermost-community/focalboard) | 26.526 | "Other" | 18/05/2026 | v8.0.0 (13/06/2024) | alternativa open source e auto-hospedada ao Trello, Notion e Asana |
| [MrLesk/Backlog.md](https://github.com/MrLesk/Backlog.md) | 7.149 | MIT | 25/10/2026 | v1.55.3 (23/10) | "A tool for managing project collaboration between humans and AI Agents in a git ecosystem" |
| [openai/codex](https://github.com/openai/codex) | 130.902 | Apache-2.0 | 23/10/2026 | rust-v0.164.0 (23/10) | o agente de código da OpenAI (alvo da integração nativa) |
| [anthropics/claude-code](https://github.com/anthropics/claude-code) | 153.369 | (sem licença declarada) | 25/10/2026 | v2.1.310 (23/10) | o agente de código da Anthropic |

O Blueprint, no mesmo dia: 3 estrelas, MIT, 4 releases (v0.1.0 a v0.1.3), perfil de comunidade do GitHub em 85%.

## O que se repete nos READMEs que funcionam

Medido nos READMEs de Task Master, Backlog.md, Plane, Vibe Kanban, Focalboard, Excalidraw e Ollama:

1. Logo, uma frase e selos (versão, licença, estrelas) na primeira tela.
2. O comando de instalação aparece sem rolar: `npm i -g backlog.md`, `npx vibe-kanban`, o `curl | sh` do Ollama.
3. Muita imagem: Excalidraw 23, Task Master 16, Plane 14, Backlog.md 13, Vibe Kanban 6, Focalboard 5, Ollama 1; o Blueprint tinha 1.
4. Uma seção de "por que" ou de comparação (Backlog.md "Why Backlog.md in the AI era"; Plane "alternativa ao Jira e ao Linear").
5. O detalhe fica em documentação separada: o README do Plane tem 0 blocos de código e leva à documentação.
6. Guia por agente ou cliente (Task Master "Option 1: MCP / Option 2: Command Line"; Backlog.md "Client guides").
7. Comunidade e segurança visíveis (Discord, contribuidores, SECURITY).
8. Todos em inglês.

## Dois sinais sobre sustentabilidade

- **Bloop (Vibe Kanban) encerrou a empresa em abril de 2026**: "a grande maioria dos usuários era gratuita" e não acharam um modelo de negócio; o projeto continua como código aberto mantido pela comunidade. Antes disso o plano de equipe custava US$ 30 por mês para 2 a 49 usuários. Fonte: [vibekanban.com/shutdown](https://vibekanban.com/shutdown) e [a página de preços capturada antes](https://vibekanban.com/pricing).
- **Plane** vende duas coisas sobre o mesmo núcleo AGPL: a nuvem (Free até 12 usuários; Pro a US$ 6 por assento por mês; Business a US$ 13; Enterprise sob consulta) e uma edição comercial auto-hospedada com fluxos de aprovação, auditoria, SSO e LDAP. Fontes: [plane.so/pricing](https://plane.so/pricing.md), [as edições do Plane](https://plane.so/blog/plane-and-its-editions) (fontes do próprio fornecedor; preços podem ter mudado).

## Codex (para a integração nativa)

Fontes secundárias de abril e maio de 2026 e uma página oficial: o Codex tem AGENTS.md, skills (pasta com SKILL.md), MCP, subagentes, plugins com marketplace e hooks (`.codex/hooks.json`, exige `codex_hooks = true`). Fonte oficial: [developers.openai.com/codex/concepts/customization](https://developers.openai.com/codex/concepts/customization). **Não verificado:** o formato exato dos eventos de hook, o `notify` e a publicação de plugins; a ação "Integração com o Codex: pesquisa e desenho" deve ler a documentação oficial e o código do `openai/codex` antes de afirmar qualquer coisa.
