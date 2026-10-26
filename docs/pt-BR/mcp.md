# Conectar outros agentes (MCP)

[Voltar ao README](../../README.pt-BR.md) · [English](../mcp.md)

No **Codex** (e no Claude Code sem o plugin), ligue só o MCP. O instalador e o `blueprint registrar` já fazem isso no Codex que acharem no PATH. À mão, com o app aberto (ou `blueprint serve` rodando):

```
claude mcp add blueprint -- <caminho>\blueprint.exe --mcp
codex mcp add blueprint -- <caminho>\blueprint.exe --mcp
```

Se o app não estiver em `http://127.0.0.1:3000`, passe `BLUEPRINT_URL` (`claude mcp add blueprint -e BLUEPRINT_URL=http://127.0.0.1:3917 -- …`). A tela **Planejar → Como conectar** mostra o comando exato. Sem o plugin do Claude Code, o agente não sabe sozinho de qual blueprint é a pasta: grave no `CLAUDE.md` ou `AGENTS.md` dela a linha `Blueprint: workspace_id `…`, blueprint_id `…` (MCP `blueprint`).` (o agente descobre os ids com `listar_blueprints`).

**Com o app fechado** o agente continua registrando: `registrar_progresso`, `marcar_concluido`, `marcar_checkpoint_feito`, `propor_blueprint`, `anexar_prototipo` e `vincular_pasta` ficam guardados em arquivos em `%APPDATA%\Blueprint\pendentes\` e, quando o app abre, entram na fila, na ordem, com a mesma validação (a tela os aplica ao abrir o workspace). O que o app recusar na abertura fica em `pendentesecusados\` com o motivo. As ferramentas de leitura (`ler_blueprint`, `listar_blueprints`) precisam do app aberto.

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

**Importar issues do GitHub:** não há ferramenta de importação. Peça ao seu agente "importe as issues abertas de dono/nome"; o tópico `importar` do guia o faz ler com `gh issue list` e mandar uma proposta só, que você revisa e aplica na tela Planejar. Responsáveis e autores nunca são copiados, e o texto de uma issue é tratado como dado, não como ordem.

O guia que o agente lê está em [`skills/blueprint-planejamento/`](../../skills/blueprint-planejamento/SKILL.md).
