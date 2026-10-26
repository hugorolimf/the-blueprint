# Claude Code: o plugin do Blueprint

[Voltar ao README](../README.pt-BR.md)

**Com o instalador isso já vem feito**, e `blueprint registrar` repete o passo à mão. Ele instala no Claude Code, para o seu usuário e em qualquer pasta, o plugin `blueprint@blueprint-local`:

- o **MCP** `blueprint` (as ferramentas da tabela abaixo; no Claude elas aparecem como `plugin:blueprint:blueprint`);
- a **skill** `blueprint-planejamento` (o guia de planejar e executar, que o Claude carrega sozinho) e o **agente** `blueprint` (peça com `claude --agent blueprint:blueprint`; não vira o padrão das suas sessões);
- **hooks**: ao abrir uma sessão, o Claude já sabe o blueprint da pasta e a ação atual; a cada edição de arquivo, e quando um subagente começa ou termina, o Claude na folha acompanha no app (só indicador: nada é marcado como concluído sozinho);
- o comando **`/blueprint:status`**: o estado do blueprint da pasta (ação atual, planos e próximo checkpoint).

**Pasta do projeto:** um blueprint novo proposto pelo agente liga a pasta da sessão quando você aplica. Numa pasta sem vínculo, o app oferece a escolha do blueprint; basta selecionar. Em **Editar → Dados do blueprint → Pasta do projeto** você vê, troca, adiciona e remove os vínculos, sem gravar arquivos na pasta.

**Status line:** `blueprint claude` ativa só nessa sessão. Para todas as sessões, use `blueprint registrar --status-line` (opcional e desmarcado no instalador); `blueprint registrar --status-line --remover` restaura a anterior. A instalação guarda `settings.json.bak`, preserva as demais opções e repetições não duplicam a integração. Se já houver um comando, o wrapper executa os dois com o mesmo JSON, conservando sua saída; um pipe simples perderia a entrada da status line. Configura `~/.claude` e também `CLAUDE_CONFIG_DIR` quando definido. Fora das pastas ligadas o Blueprint não imprime nada.

Para testar sem mexer na sua configuração: `node scripts/testar-plugin-claude.mjs <caminho do blueprint.exe>` instala e remove o plugin com pastas temporárias.

**Terminal e avisos:** `blueprint claude` também carrega a marca do app (painel em terminais largos, faixa acima do prompt nos estreitos; `/blueprint-marca` reabre) e o monitor `blueprint watch`, que recebe propostas aplicadas e checkpoints marcados após o app salvar. Os avisos usam texto fixo, filtro pela pasta e limite de 20 por minuto. Veja [o mod](docs/mod-terminal.md) e [os testes e limites do monitor](docs/monitor-e-channels.md).

| | Monitor (padrão) | Channels (opcional) |
| --- | --- | --- |
| Ativação | Plugin, sem flag de canal | `BLUEPRINT_CHANNEL=1` e flag de desenvolvimento |
| Confirmação | Confiança normal do plugin | Confirmação em cada sessão |
| Estabilidade | API experimental de monitors | Research preview |
| Custo | Contexto quando recebe aviso | Contexto quando recebe aviso; substitui o monitor |

Para experimentar com o plugin instalado no PowerShell: `$env:BLUEPRINT_CHANNEL='1'` e `blueprint claude --dangerously-load-development-channels plugin:blueprint@blueprint-local`. Use `Remove-Item Env:BLUEPRINT_CHANNEL` antes da próxima sessão normal. Para MCP registrado diretamente, o identificador é `server:blueprint`. O canal é informativo, sem ferramentas de resposta nem aprovação de permissões. [Comparação e validação real](docs/monitor-e-channels.md).
