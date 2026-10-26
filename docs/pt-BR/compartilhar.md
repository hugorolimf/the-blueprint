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

## Abrir o convite no app instalado

O instalador registra o protocolo `blueprint://`. Quando o convidado abre o link de convite, a página tenta primeiro abrir **o app
instalado**: o navegador pergunta uma vez ("Abrir o The Blueprint?"), o app pergunta qual endereço está abrindo e uma janela própria mostra o
workspace compartilhado (mesma entrada pelo nome, mesmas telas, separada da sua janela local). Se o app não estiver instalado, ou não
responder em 2,5 s, a versão web continua na mesma aba (o botão "Continuar no navegador" está sempre lá; `?web` depois de `/convite` pula a
tentativa). Só vale endereço `https://` e código de convite no formato que o servidor gera; qualquer outra coisa só abre o app normalmente.
Nessa janela, **Minha chave → Gerar chave** mostra também **Conectar meu Claude neste computador**: depois de uma pergunta nativa com o
endereço, faz o que o `blueprint conectar` faz (próxima seção), sem comando para copiar.

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

## Endereço fixo

No diálogo Compartilhar, **Endereço** tem duas opções (a escolha fica guardada para a próxima vez e só muda com o compartilhamento desligado):

- **Temporário** (padrão): quick tunnel da Cloudflare, sem conta. O endereço muda a cada vez que você liga.
- **Fixo**: Tailscale Funnel. O endereço é `https://<seu-dispositivo>.<sua-tailnet>.ts.net` e nunca muda, então os links de convite, as sessões e o Claude dos convidados (`blueprint conectar`) continuam valendo depois de desligar e ligar. Precisa do Tailscale instalado e conectado, MagicDNS e certificados HTTPS ligados e o atributo `funnel` liberado para o seu dispositivo na política da tailnet ([requisitos do Funnel](https://tailscale.com/kb/1223/funnel)). O app não guarda senha nem chave: roda o seu próprio `tailscale`. Ele confere `tailscale status --json` (conectado e o nome `.ts.net`), escolhe a primeira porta livre do Funnel entre 443, 8443 e 10000 em `tailscale serve status --json` (nunca toma uma porta que você já usa) e mantém `tailscale funnel --https=<porta> http://127.0.0.1:<porta dos convidados>` rodando em primeiro plano, dentro de um Job Object: fechar o app fecha o Funnel (um Funnel com `--bg` ficaria apontando para a porta depois de uma queda). O endereço que o comando imprime tem de ser o deste dispositivo, senão o compartilhamento para com um erro.

Trocar de um tipo para o outro muda o endereço: quem já entrou precisa de um link de convite novo uma vez (e de rodar `blueprint conectar` de novo). O navegador não leva o cookie de sessão para outro endereço, então isso não tem como evitar.

Conferido em 2026-10-26:

| | Temporário (quick tunnel da Cloudflare) | Fixo (Tailscale Funnel) | Túnel nomeado da Cloudflare (fora do app) |
| --- | --- | --- | --- |
| Conta | nenhuma | conta Tailscale | conta Cloudflare e um domínio na Cloudflare |
| Endereço | muda a cada vez | `*.ts.net` fixo | o seu próprio nome |
| Portas | qualquer | só 443, 8443 e 10000 | qualquer |
| Limites | "para testes e desenvolvimento", sem garantia de disponibilidade, 200 pedidos em andamento (os demais levam 429), sem Server-Sent Events (o app contorna, veja abaixo) | "limites de banda não configuráveis", só TLS | os termos do plano Free restringem servir arquivos grandes e vídeo |
| Fonte | [quick tunnels da Cloudflare](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/do-more-with-tunnels/trycloudflare/) | [Tailscale Funnel](https://tailscale.com/kb/1223/funnel), [comando funnel](https://tailscale.com/kb/1311/tailscale-funnel) | [criar um túnel remoto](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/get-started/create-remote-tunnel/), [FAQ do Tunnel](https://developers.cloudflare.com/cloudflare-one/faq/cloudflare-tunnels-faq/) |

O quick tunnel não entrega Server-Sent Events como fluxo ao vivo, então na porta dos convidados cada conexão de atualização ao vivo fecha depois do primeiro evento e o navegador reconecta com `Last-Event-ID` (medido: 41 eventos chegaram juntos depois de 42 s). O Tailscale Funnel não tem esse limite. O túnel nomeado da Cloudflare fica fora do app de propósito: pede um domínio, a configuração no painel e um token, o que é mais que "ligar". Se você usa um, rode-o por conta própria apontando para `blueprint serve --hospedar` ([guia de hospedagem](hospedar.md)). Código: `src/compartilhar/tailscale.rs`, `provedor.rs`; testes: `cargo test compartilhar` e `node scripts/endereco-fixo-e2e.mjs`.

- **Um servidor da equipe:** `blueprint serve --hospedar` numa máquina pequena que fica ligada ([guia de hospedagem](hospedar.md)).

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

## Voltar à equipe depois de fechar o app (convidado)

A sessão do convidado é o cookie da janela da equipe (30 dias, perfil persistente do WebView2), mas o link de convite é de uso único: sem um registro local, depois de fechar o app não havia como abrir a janela da equipe de novo. Agora a página da equipe informa ao app (IPC `equipe`) o nome da pessoa e os workspaces que ela vê, e o app guarda só isso em `%APPDATA%\Blueprint\equipes.json` (`src/equipes.rs`; nunca o convite nem o cookie). A tela de workspaces mostra um cartão "Equipe" por equipe lembrada; o clique chama `POST /api/equipes/abrir`, que só aceita uma origem já lembrada e abre a janela da equipe, onde a sessão continua. `Tirar da lista` esquece a equipe (`/api/equipes/esquecer`). Rotas só locais; teste: `node scripts/equipes-locais-e2e.mjs`.

Limite: o endereço do quick tunnel muda cada vez que o anfitrião religa o compartilhamento, e o cookie vale só para o endereço antigo. Nesse caso o cartão abre uma tela "Entre pelo link de convite" e o convidado precisa de um convite novo; endereço fixo é a ação seguinte.

## MCP remoto por HTTP (sem instalar o app)

Quem foi convidado liga o Claude Code ou o Codex **sem instalar o The Blueprint**: o servidor (modo hospedado e porta dos convidados do Compartilhar) atende `POST /mcp`, o transporte "Streamable HTTP" do MCP, com a chave do membro em `Authorization: Bearer bpk_...` e as mesmas ferramentas e instruções do `blueprint --mcp`. A tela Minha chave mostra os comandos.

Decisões (conferidas em 2026-10-26 nas fontes abaixo):

| Ponto | Decisão | Fonte |
| --- | --- | --- |
| Endereço | um caminho só, `/mcp`, só POST; GET e DELETE dão 405 (a especificação permite servidor sem fluxo SSE e sem encerrar sessão) | [Transportes do MCP, 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18/basic/transports) |
| Respostas | pedido recebe um objeto `application/json`; notificação recebe `202` sem corpo; sem lotes | idem |
| Sessão | nenhuma (`Mcp-Session-Id` é opcional); cada pedido leva a chave | idem |
| Versão | `MCP-Protocol-Version`, se vier, tem de ser 2025-06-18, 2025-03-26 ou 2024-11-05, senão 400 | idem |
| Origin | pedido com cabeçalho `Origin` (navegador) leva 403, pelo aviso de DNS rebinding | idem |
| Autenticação | chave `Bearer`; sem OAuth e sem `WWW-Authenticate`, para o cliente usar o cabeçalho. `initialize`, `ping` e `tools/list` respondem sem chave (a descrição pública do servidor: o pré-teste de conexão do Claude Code não manda o cabeçalho e trata 401 como fatal); `tools/call` sem chave é 401; chave inválida ou revogada é 401 `chave-invalida` | [MCP do Claude Code](https://code.claude.com/docs/en/mcp) |
| Claude Code | `claude mcp add --transport http --scope user blueprint-equipe <url>/mcp --header 'Authorization: Bearer ${BLUEPRINT_CHAVE}'` (aspas simples deixam `${...}` literal; o Claude Code o expande ao conectar, então a chave não aparece no comando que ele imprime) | idem |
| Codex | `[mcp_servers.blueprint-equipe]` com `url = "<url>/mcp"` e `bearer_token_env_var = "BLUEPRINT_CHAVE"` em `~/.codex/config.toml` | [MCP do Codex](https://developers.openai.com/codex/mcp) |
| HTTPS | o túnel e o modo hospedado são HTTPS; a chave nunca viaja em HTTP puro fora do loopback | modo hospedado (docs/hospedar.md) |

Diferenças do `--mcp`: `vincular_pasta` é recusado (o servidor não vê a sua pasta: escreva a linha `Blueprint:` em `CLAUDE.local.md`, ou use `blueprint conectar` com o app instalado), `propor_blueprint` nunca manda a pasta do próprio servidor, e não há canal nem hooks (o progresso mostra só o que o agente registra com `registrar_progresso`). Não substitui o plugin instalado. Código: `src/mcp/http.rs`; testes: `cargo test mcp_http` e `node scripts/mcp-http-e2e.mjs`.
