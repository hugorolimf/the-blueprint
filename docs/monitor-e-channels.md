# Avisos do app ao Claude Code

`blueprint watch` acompanha `/api/mcp/eventos` por SSE, em `http://127.0.0.1:3000` ou `BLUEPRINT_URL`. O monitor do plugin o inicia com `when: always`; não precisa de `userConfig` nem de flag de canal. O instalador grava o caminho absoluto do executável no comando. O manifesto usa `experimental.monitors`.

O alvo vem da ligação da pasta no banco, consultada por `hook/sessao` a cada reconexão. Assim, a linha `Blueprint:` no arquivo de instruções não precisa existir para o filtro funcionar. Pastas sem ligação ficam silenciosas. O processo dorme entre tentativas de conexão (1, 2, 4, 8, 16 e 30 segundos); uma conexão aberta bloqueia esperando dados. Não há sondagem em repouso.

A página envia `/api/mcp/aviso` depois de um `PUT /api/dados` bem-sucedido: proposta aplicada ou checkpoint marcado pelo usuário. O ack da fila apenas confirma recebimento e não gera aviso de aplicação. O servidor confere que o blueprint existe e, para checkpoint, que ele está marcado no banco. Os eventos levam somente ids; o monitor imprime mensagens fixas, nunca nomes, comentários, HTML, comandos ou conteúdo da proposta. Cada mensagem tem menos de 200 bytes; no máximo 20 por minuto. Linhas/eventos SSE maiores que 2048 bytes encerram a conexão e acionam a espera. Cada assinante possui fila limitada a 256 eventos; excedentes são descartados. Os avisos são informativos, sem histórico nem garantia de entrega durante desconexão.

Os monitors rodam só em sessões interativas e provedores compatíveis. Desabilitar o plugin no meio da sessão não encerra um monitor já iniciado; encerre/reabra a sessão. Estes limites vêm da [referência oficial de componentes](https://code.claude.com/docs/en/plugins/components) e do [manifesto de monitors](https://code.claude.com/docs/en/plugins/manifest-reference#monitors).

## Validação reproduzível

```text
cargo test watch --lib
cargo test aviso_so --lib
cargo test o_sse --lib
cargo build
node scripts/testar-monitor.mjs
claude plugin validate claude-plugin/plugins/blueprint
```

Em 22/10/2026: os testes do leitor, teto de 100 eventos, filtro, mensagens fixas e servidor falso com três eventos e reconexão passaram. O teste com servidor real, banco temporário e processo `watch` confirmou entrega, filtro e reconexão após fechar/reabrir o app; mediu 15,625 ms de CPU em quatro segundos com app fechado. `plugin validate` passou, com aviso de ícone para catálogo. Uma sessão interativa real aberta por `blueprint claude` mostrou «1 monitor»; após aviso de proposta aplicada no alvo temporário `sm/integracao`, exibiu «Monitor event: The Blueprint: o usuário aplicou uma proposta…» e o modelo chamou `ler_blueprint`.

## Comparação com Channels

Decisão do usuário em 22/10/2026: **manter Channels como opção experimental**. O monitor continua sendo o padrão.

O canal do próprio Blueprint também foi validado em sessão interativa isolada: MCP `server:blueprint`, `BLUEPRINT_CHANNEL=1` e a flag de desenvolvimento. Após um aviso `proposta-aplicada` no servidor temporário, o terminal exibiu `← blueprint: The Blueprint: o usuário aplicou uma proposta…` e o Claude consultou as ferramentas do Blueprint. O canal compartilha o filtro, o limite de taxa e o texto fixo do monitor. A ativação é opcional; com ela, o monitor não imprime o mesmo aviso novamente.

| Aspecto | Monitor | Channels próprio |
| --- | --- | --- |
| Ativação | Plugin e sessão interativa | MCP stdio e opção explícita da sessão |
| Flag | Nenhuma de canal | `--dangerously-load-development-channels server:blueprint` |
| Confirmação | Fluxo normal de confiança do plugin | Confirmação de desenvolvimento a cada sessão |
| Contrato | `experimental.monitors`; pode mudar | Research preview; capacidade e notificação específicas |
| Conteúdo | Linhas de stdout, texto fixo limitado | `notifications/claude/channel`, conteúdo filtrado |
| Custo | Contexto apenas quando há aviso; processo ocioso | Contexto quando há aviso; processo MCP persistente |
| Histórico | Apenas sessão aberta | Apenas sessão aberta |

Pré-avaliação local em 22/10/2026: Claude Code 2.1.309 autenticado por claude.ai, provedor firstParty, assinatura Pro. O shim global de Bun não tinha versão configurada (`proto::detect::failed`), mas um executável temporário Bun 1.4.4 permitiu testar o fakechat oficial. Uma sessão interativa isolada recebeu a mensagem por WebSocket local e respondeu `BLUEPRINT_CHANNEL_OK`: pré-requisito de entrega cumprido, sem modificar a configuração global.

O MCP declara `capabilities.experimental['claude/channel']`, sem capacidade de permission relay nem ferramenta de resposta. A emissão é opt-in com `BLUEPRINT_CHANNEL=1`: depois de `notifications/initialized`, acompanha os mesmos eventos filtrados do monitor e envia `notifications/claude/channel` em linhas JSON-RPC indivisíveis. Sem opt-in não abre conexão extra. Quando a variável está ativa, o monitor encerra silenciosamente para evitar duplicação. A decisão permanente de manter/descartar continua sendo do usuário.

Para experimentar no PowerShell com o plugin instalado:

```powershell
$env:BLUEPRINT_CHANNEL = '1'
blueprint claude --dangerously-load-development-channels plugin:blueprint@blueprint-local
# Na próxima sessão normal, volte ao monitor:
Remove-Item Env:BLUEPRINT_CHANNEL
```

Para um MCP registrado diretamente, use `server:blueprint` no lugar de `plugin:blueprint@blueprint-local`. Essa escolha depende da forma de instalação, não do nome exibido da ferramenta.

A [documentação oficial de Channels](https://code.claude.com/docs/en/channels) informa os provedores e as contas compatíveis. A [referência de desenvolvimento](https://code.claude.com/docs/en/channels-reference#test-during-the-research-preview) exige sessão interativa para a flag de desenvolvimento: em `-p`/SDK ela é ignorada; políticas de organização continuam valendo. Esta integração não oferece resposta por canal, permission relay nem envio de trabalho ao agente.
