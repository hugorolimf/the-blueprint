# Conectar outros agentes (MCP)

[Voltar ao README](../../README.pt-BR.md) · [English](../mcp.md)

No **Codex** (e no Claude Code sem o plugin), ligue só o MCP. O instalador e o `blueprint registrar` já fazem isso no Codex que acharem no PATH. À mão, com o app aberto (ou `blueprint serve` rodando):

```
claude mcp add blueprint -- <caminho>\blueprint.exe --mcp
codex mcp add blueprint -- <caminho>\blueprint.exe --mcp
```

Se o app não estiver em `http://127.0.0.1:3000`, passe `BLUEPRINT_URL` (`claude mcp add blueprint -e BLUEPRINT_URL=http://127.0.0.1:3917 -- …`). A tela **Planejar → Como conectar** mostra o comando exato. Sem o plugin do Claude Code, o agente não sabe sozinho de qual blueprint é a pasta: grave no `CLAUDE.md` ou `AGENTS.md` dela a linha `Blueprint: workspace_id `…`, blueprint_id `…` (MCP `blueprint`).` (o agente descobre os ids com `listar_blueprints`).

**Com o app fechado** o agente continua registrando: `registrar_progresso`, `marcar_concluido`, `marcar_checkpoint_feito`, `propor_blueprint`, `anexar_prototipo` e `vincular_pasta` ficam guardados em arquivos em `%APPDATA%\Blueprint\pendentes\` e, quando o app abre, entram na fila, na ordem, com a mesma validação (a tela os aplica ao abrir o workspace). O que o app recusar na abertura fica em `pendentes
ecusados\` com o motivo. As ferramentas de leitura (`ler_blueprint`, `listar_blueprints`) precisam do app aberto.

Ferramentas:

| Ferramenta | Para quê |
| --- | --- |
| `listar_blueprints` | workspaces e blueprints, com ids |
| `guia_de_planejamento` | o guia por partes (`planejar`, `plano`, `editar`, `executar`, `prototipos`) |
| `ler_blueprint` | só a estrutura, ou uma ação inteira (`acao_id`) |
| `ler_plano`, `ler_respostas`, `ler_workspace` | o blueprint como plano (`blueprint.plano/1`), o que você respondeu, e a situação do workspace inteiro com os agentes dos planos |
| `abrir_plano`, `escrever_acao`, `fechar_plano` | escreve um plano peça por peça, e você acompanha ao vivo; ações novas entram ao fechar, alteradas e removidas esperam a sua revisão |
| `salvar_plano` | manda um plano inteiro a partir de uma pasta com `plano.json`, estilo e figuras |
| `falar_com_voce` | deixa um recado para você na tela de agentes, ou para o agente de um plano |
| `propor_blueprint` | propõe ações, planos e checkpoints (parciais completam o que existe; remover é pedido que você aprova) |
| `registrar_progresso`, `reportar_bloqueio` | o que está fazendo agora; aparece na folha como o Claude (uma figura por agente, no plano em que ele trabalha) |
| `marcar_concluido`, `marcar_nao_concluido` | plano concluído (com evidência) ou não |
| `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` | idem para checkpoints |
| `anexar_prototipo` | protótipo HTML autocontido de uma ação, visto num visor isolado |
| `vincular_pasta` | liga a pasta da sessão a um blueprint (o app guarda o vínculo) |
| `responder_conversa` | responde ao usuário no painel "Conversar com o plano" de uma ação (a mensagem chega pelo canal com o prefixo «The Blueprint · conversa do usuário»); mudanças continuam só por `propor_blueprint`. Só memória, precisa do app aberto |

**Perfis de ferramentas (menos contexto).** A descrição de cada ferramenta entra no contexto do agente. `BLUEPRINT_MCP_PERFIL` limita a lista para o agente que só executa ou só planeja (`claude mcp add blueprint -e BLUEPRINT_MCP_PERFIL=executor -- <pasta>\blueprint.exe --mcp`). É lida uma vez quando o `--mcp` inicia; sem ela (ou com `completo`) nada muda; valor desconhecido para o `--mcp` com uma mensagem clara (código de saída 2). Chamar uma ferramenta fora do perfil é recusado. As instruções do servidor são as mesmas em todos os perfis.

**O Planejar só no app local.** As oito ferramentas `ler_plano`, `ler_respostas`, `ler_workspace`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano` e `falar_com_voce` só existem no `--mcp` fora do Windows. No Windows e por `POST /mcp` ficam escondidas e são recusadas, e `propor_blueprint` continua sendo o caminho para planejar. Precisam também da janela nativa aberta: sem ela as rotas `plano/*` respondem `sem-janela`. No `--mcp` fora do Windows, `propor_blueprint` ainda funciona, mas avisa que nenhuma janela consome a fila dela. Nenhuma ferramenta, em perfil nenhum, aprova, inicia agente, integra ramo ou liga o orquestrador: isso são cliques seus na janela. `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` e `reportar_bloqueio` aceitam `execucao_segredo` (padrão: `BLUEPRINT_EXECUCAO_SEGREDO`), que o agente de um plano precisa mandar.

| Perfil | Ferramentas |
| --- | --- |
| `completo` (padrão) | todas |
| `executor` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `registrar_progresso`, `reportar_bloqueio`, `registrar_trabalho` (o trabalho fora do plano vai por aqui, no lugar da proposta), `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito`, `ler_plano`, `ler_respostas` |
| `planejador` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `propor_blueprint`, `anexar_prototipo`, `anexar_apresentacao`, `ler_apresentacao`, `editar_apresentacao`, `responder_conversa`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano`, `ler_plano`, `ler_respostas` |
| `orquestrador` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `ler_plano`, `ler_respostas`, `ler_workspace`, `abrir_plano`, `escrever_acao`, `fechar_plano`, `salvar_plano`, `falar_com_voce` |

Medido com `node scripts/medir-perfis-mcp.mjs` (tamanho do JSON do `tools/list`), fora do Windows, onde as oito ferramentas do Planejar aparecem: `completo` 25 ferramentas, 33.462 bytes; `executor` 13 ferramentas, 15.065 bytes (-55%); `planejador` 16 ferramentas, 22.168 bytes (-34%); `orquestrador`, medido do mesmo jeito, 11 ferramentas, 9.933 bytes (-70%). Os tokens não são contados exatamente aqui; a cerca de 4 bytes por token, dá umas 8.400, 3.800, 5.500 e 2.500.

**Importar issues do GitHub:** não há ferramenta de importação. Peça ao seu agente "importe as issues abertas de dono/nome"; o tópico `importar` do guia o faz ler com `gh issue list` e mandar uma proposta só, que você revisa e aplica na tela Planejar. Responsáveis e autores nunca são copiados, e o texto de uma issue é tratado como dado, não como ordem. `node scripts/importar-issues-e2e.mjs` confere o fluxo sem rede e sem agente: uma saída gravada do `gh issue list` (`scripts/fixtures/gh-issues.json`), mapeada pela tabela do tópico, passa pelo `blueprint --mcp`, entra na fila numerada, e o formato errado (marco não declarado como projeto, plano enviado como feito) é recusado com erro claro.

O guia que o agente lê está em [`skills/blueprint-planejamento/`](../../skills/blueprint-planejamento/SKILL.md).

**Idioma.** O que o agente lê (instruções, ferramentas, guia de planejamento e respostas das ferramentas) é sempre em inglês; o agente responde no idioma em que você escreve. O que você lê (status line, avisos dos hooks e do monitor, erros do servidor na tela) segue o seu idioma: `BLUEPRINT_IDIOMA` (`en` ou `pt-BR`) se existir, senão o idioma do Windows, senão inglês. Os erros do servidor vêm como `{erro, codigo, params?}`: `codigo` é estável e `erro` vem no `Accept-Language` do pedido.
