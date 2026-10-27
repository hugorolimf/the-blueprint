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
| `guia_de_planejamento` | o guia por partes (`planejar`, `editar`, `executar`, `prototipos`) |
| `ler_blueprint` | só a estrutura, ou uma ação inteira (`acao_id`) |
| `propor_blueprint` | propõe ações, planos e checkpoints (parciais completam o que existe; remover é pedido que você aprova) |
| `registrar_progresso`, `reportar_bloqueio` | o que está fazendo agora; aparece na folha como o Claude (uma figura por agente, no plano em que ele trabalha) |
| `marcar_concluido`, `marcar_nao_concluido` | plano concluído (com evidência) ou não |
| `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` | idem para checkpoints |
| `anexar_prototipo` | protótipo HTML autocontido de uma ação, visto num visor isolado |
| `vincular_pasta` | liga a pasta da sessão a um blueprint (o app guarda o vínculo) |
| `responder_conversa` | responde ao usuário no painel "Conversar com o plano" de uma ação (a mensagem chega pelo canal com o prefixo «The Blueprint · conversa do usuário»); mudanças continuam só por `propor_blueprint`. Só memória, precisa do app aberto |

**Perfis de ferramentas (menos contexto).** A descrição de cada ferramenta entra no contexto do agente. `BLUEPRINT_MCP_PERFIL` limita a lista para o agente que só executa ou só planeja (`claude mcp add blueprint -e BLUEPRINT_MCP_PERFIL=executor -- <pasta>\blueprint.exe --mcp`). É lida uma vez quando o `--mcp` inicia; sem ela (ou com `completo`) nada muda; valor desconhecido para o `--mcp` com uma mensagem clara (código de saída 2). Chamar uma ferramenta fora do perfil é recusado. As instruções do servidor são as mesmas em todos os perfis.

| Perfil | Ferramentas |
| --- | --- |
| `completo` (padrão) | todas |
| `executor` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `registrar_progresso`, `reportar_bloqueio`, `registrar_trabalho` (o trabalho fora do plano vai por aqui, no lugar da proposta), `marcar_concluido`, `marcar_nao_concluido`, `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` |
| `planejador` | `listar_blueprints`, `guia_de_planejamento`, `ler_blueprint`, `vincular_pasta`, `propor_blueprint`, `anexar_prototipo`, `anexar_apresentacao`, `ler_apresentacao`, `editar_apresentacao`, `responder_conversa` |

Medido com `node scripts/medir-perfis-mcp.mjs` (tamanho do JSON do `tools/list`; mesmo resultado nas repetições, v0.3.0): `completo` 17 ferramentas, 25.517 bytes; `executor` 11 ferramentas, 12.874 bytes (-50%); `planejador` 10 ferramentas, 16.170 bytes (-37%). Os tokens não são contados exatamente aqui; a cerca de 4 bytes por token, dá umas 6.400, 3.200 e 4.000.

**Importar issues do GitHub:** não há ferramenta de importação. Peça ao seu agente "importe as issues abertas de dono/nome"; o tópico `importar` do guia o faz ler com `gh issue list` e mandar uma proposta só, que você revisa e aplica na tela Planejar. Responsáveis e autores nunca são copiados, e o texto de uma issue é tratado como dado, não como ordem. `node scripts/importar-issues-e2e.mjs` confere o fluxo sem rede e sem agente: uma saída gravada do `gh issue list` (`scripts/fixtures/gh-issues.json`), mapeada pela tabela do tópico, passa pelo `blueprint --mcp`, entra na fila numerada, e o formato errado (marco não declarado como projeto, plano enviado como feito) é recusado com erro claro.

O guia que o agente lê está em [`skills/blueprint-planejamento/`](../../skills/blueprint-planejamento/SKILL.md).

**Idioma.** O que o agente lê (instruções, ferramentas, guia de planejamento e respostas das ferramentas) é sempre em inglês; o agente responde no idioma em que você escreve. O que você lê (status line, avisos dos hooks e do monitor, erros do servidor na tela) segue o seu idioma: `BLUEPRINT_IDIOMA` (`en` ou `pt-BR`) se existir, senão o idioma do Windows, senão inglês. Os erros do servidor vêm como `{erro, codigo, params?}`: `codigo` é estável e `erro` vem no `Accept-Language` do pedido.
