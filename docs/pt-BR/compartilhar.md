# Compartilhar um workspace pelo app

[Voltar ao README](../../README.pt-BR.md) · [English](../compartilhar.md)

O app de mesa pode dar acesso a **um workspace** a outras pessoas sem servidor: o próprio app aberto as atende, por um endereço HTTPS
gratuito da Cloudflare (um "quick tunnel", sem conta). As pessoas abrem o link no navegador, digitam o nome e trabalham nos mesmos
blueprints com você, ao vivo. Nada é copiado: continua havendo um banco só, o seu. Para um servidor de equipe que fica de pé sem o seu
computador, veja [Hospedar o The Blueprint para uma equipe](hospedar.md). Desenho e contrato: [colaboracao.md](../colaboracao.md#compartilhar-pelo-app).

## Como ligar

1. Abra o workspace e clique em **Compartilhar**. Escolha o workspace e ligue o compartilhamento.
2. Na primeira vez, o app baixa o `cloudflared` (cerca de 53 MB) das releases oficiais da Cloudflare no GitHub para
   `%APPDATA%\Blueprint\bin\`. A versão e o SHA-256 dele são fixos no app, e o arquivo é conferido antes de cada execução.
3. Em alguns segundos o diálogo mostra o endereço, como `https://palavras-aleatorias.trycloudflare.com`.
4. Crie um convite (papel, blueprint opcional, validade) e mande o link. Por padrão cada link vale uma vez; o link público só de leitura é
   um convite de visitante para um blueprint. Os papéis são os do modo hospedado: dono, colaborador, supervisor, visitante.
5. A pessoa abre o link, digita o nome e cai no workspace. O agente dela também pode se conectar, com a chave de MCP pessoal da tela
   **Minha chave** (o comando já aponta para o endereço do compartilhamento).

Para parar, clique em **Desligar** no mesmo diálogo: o túnel cai e a porta dos convidados fecha. Sair do app também desliga (o processo do
túnel vive num Job Object e morre com o app). Esconder a janela na bandeja **não** desliga.

## O Claude do convidado

O convidado pode ligar o próprio Claude Code (e o Codex) ao workspace compartilhado com um comando. Ele precisa do The Blueprint instalado
no computador dele (o instalador da release; o app não precisa estar aberto lá).

1. No navegador, no workspace compartilhado, o convidado abre **Minha chave** e clica em **Gerar chave**.
2. A tela mostra o comando pronto para copiar: `blueprint conectar https://<endereço> bpk_...`. Ele roda num terminal.
3. Ele abre uma sessão nova do Claude Code numa pasta cujo `CLAUDE.md` (ou `AGENTS.md`) tenha a linha
   `Blueprint: workspace_id <id>, blueprint_id <id>` de um blueprint do workspace compartilhado.

O que o comando faz: instala um plugin próprio, `blueprint-equipe` (marketplace `blueprint-equipe-local`, em
`%APPDATA%\Blueprint\claude-plugin-equipe`), que nunca briga com o plugin local `blueprint`, e registra o MCP `blueprint-equipe` no Codex. O
Claude passa a ler e propor no workspace compartilhado com o papel do convidado, e os hooks dele (progresso ao editar, subagentes, o
lembrete no fim do turno) chegam ao app do dono: o dono vê o Claude do convidado na folha como **Claude · <nome>**.

- **O endereço mudou?** Rode `blueprint conectar` de novo com o endereço novo (e uma chave nova, se a antiga foi revogada). As sessões
  abertas passam a usar o endereço novo na próxima chamada.
- **Desfazer:** `blueprint conectar --remover` tira o plugin, os MCPs e a chave do computador.
- **Onde fica a chave:** em `%APPDATA%\Blueprint\conexao-equipe.json` (só na pasta do seu usuário), lida a cada pedido pelo
  `blueprint --mcp --equipe` e pelo `blueprint hook <evento> --equipe`. Ela nunca vai nos argumentos dos processos do MCP e dos hooks nem na
  cópia do plugin que o Claude Code guarda; os hooks a mandam em `Authorization: Bearer`, como o MCP. Ela aparece uma vez no próprio comando
  `blueprint conectar` (e no histórico do seu terminal), e o Codex guarda só o comando, não a chave.
- **O que os hooks do convidado alcançam:** só os blueprints em que o papel grava; os vínculos de pasta, a lista de sessões, a status line e
  o "Iniciar com o Claude" continuam só na máquina do dono. Com o app do dono fechado ou o endereço velho, os hooks falham em silêncio (no
  máximo de 3 a 5 s) e nunca travam o Claude.

## Limites

- **O app precisa ficar aberto**, com o computador ligado e na internet. Fechou, os convidados perdem o acesso até você compartilhar de novo.
- **O endereço grátis muda a cada vez** que você liga. Os links antigos param de valer e a entrada dos convidados pertence ao endereço
  antigo, então mande convites novos a cada vez. Endereço fixo está no roteiro (abaixo).
- **Sem garantia da Cloudflare.** O quick tunnel é para testes: sem garantia de disponibilidade, valem os termos da Cloudflare e há um
  limite de cerca de 200 pedidos em andamento. Serve para poucas pessoas, não para uma equipe grande.
- **O ao vivo usa pedidos curtos.** O quick tunnel segura uma resposta em fluxo até ela terminar (medido: 41 eventos, um por segundo,
  chegaram todos juntos aos 42 s). Por isso, só para os convidados, o app termina cada fluxo ao vivo logo depois de um evento, e o
  navegador reconecta sozinho e recebe o que perdeu. Uma mudança depois de um tempo parado chega rápido (medido: cerca de 50 ms depois de
  o dono gravar); uma mudança logo depois de outra pode esperar até 3 s pela reconexão. Uma aba de convidado parada faz cerca de um pedido
  a cada 25 s por fluxo.
- **O endereço novo pode demorar um pouco.** A Cloudflare avisa que pode levar um tempo até responder. Se o navegador perguntar cedo demais,
  o computador pode guardar "esse nome não existe" por até um minuto; espere e tente de novo.
- Ainda sem os avatares de presença para convidados (só existem no modo hospedado). Por enquanto só no Windows. O **Atualizar** recusa
  enquanto o compartilhamento estiver ligado.

## Segurança

- **Uma porta separada.** Os convidados chegam ao app por uma segunda porta, só em `127.0.0.1`, que só existe com o compartilhamento
  ligado; o túnel aponta para ela. A sua janela continua na porta 3000, como sempre. O app separa convidados de você pela porta, nunca por
  cabeçalho, porque o próprio túnel conecta a partir desta máquina.
- **Nada na porta dos convidados é você.** Todo pedido ali precisa de uma sessão de convidado (de um convite) ou de uma chave de MCP
  pessoal, com o Host e a Origin do endereço do túnel, e só alcança o workspace compartilhado, com o papel do convidado.
- **O que o convidado nunca alcança:** o terminal, o "Iniciar com o Claude", as sessões e a status line do Claude Code (os hooks só com a
  chave de MCP do convidado, veja acima), os vínculos de pasta, o
  atualizador, os controles do compartilhamento, a importação dos dados inteiros (`PUT /api/dados`), a janela, os outros workspaces e o
  texto que você manda às sessões do seu agente. Essas rotas respondem 404 pela porta dos convidados.
- **A Cloudflare fica no meio.** O HTTPS termina na Cloudflare, que leva o tráfego até o app pelo túnel. Compartilhe só o que você aceita
  mandar pela Cloudflare.
- **Binário de terceiro.** O `cloudflared` roda em versão e SHA-256 fixos; arquivo que não bate é apagado e recusado. Roda com
  `--no-autoupdate`, sem janela de console, e morre junto com o app.

## Endereço fixo no futuro

- **Cloudflare com domínio próprio:** um túnel nomeado (conta grátis da Cloudflare e um domínio nela) dá um endereço estável e entrega o ao
  vivo em fluxo normalmente.
- **Tailscale Funnel:** um endereço `*.ts.net` estável, para quem já usa o Tailscale.
- **Um servidor de equipe:** `blueprint serve --hospedar` numa máquina pequena que fica ligada ([guia de hospedagem](hospedar.md)).

## Para desenvolvedores

| Rota (só porta local, Host e Origin em loopback, JSON) | Resposta |
| --- | --- |
| `GET /api/compartilhar` | `{"estado": "desligado" \| "baixando" \| "ligando" \| "ligado" \| "erro", "url"?, "workspaces"?, "workspaceId"? (o primeiro), "erro"?, "porta"?}` |
| `POST /api/compartilhar/ligar` `{"workspaceId"}` | 202 e o estado; 400 `falta-workspace`, 404 `workspace-inexistente`. Com o túnel já ligado, inclui o workspace no mesmo endereço |
| `POST /api/compartilhar/desligar` `{"workspaceId"?}` | 200 e o estado; com `workspaceId` tira só esse workspace (o túnel desliga quando não sobra nenhum), sem ele desliga tudo |

Cada mudança de estado também chega à página como o evento `compartilhar` em `/api/mcp/eventos` (só a página local). Convites e pessoas
usam as rotas do modo hospedado (`/api/convites`, `/api/pessoas`), chamadas pela página local enquanto compartilha; os links saem com o
endereço do túnel. `BLUEPRINT_TUNEL_COMANDO=<programa>` troca o `cloudflared` por um falso que imprime uma linha
`https://<palavras-com-hífen>.trycloudflare.com` na saída de erro (testes e verificações de ponta a ponta, sem rede). Código: `src/compartilhar/`.
