# Codex e outros agentes MCP

[Voltar ao README](../../README.pt-BR.md) · [English](../codex.md)

O Claude Code ganha um plugin ([guia](claude-code.md)). O Codex, e qualquer outro agente que fale MCP, ganha só o servidor MCP: as mesmas ferramentas, sem hooks, skill nem status line.

Pesquisa e desenho da integração nativa com o Codex (hooks, plugin, tabela de paridade): [codex-parity.md](codex-parity.md).

## Codex

**Conectar.** O instalador (tarefa *conectar*) e o `blueprint registrar` já rodam isto para o `codex` que acharem no PATH. À mão, com o app aberto:

```
codex mcp add blueprint -- <pasta>\blueprint.exe --mcp
codex mcp list
```

O `codex mcp list` mostra `blueprint`. `blueprint registrar --remover` (ou `codex mcp remove blueprint`) desfaz. Se o app não estiver em `http://127.0.0.1:3000`, acrescente `--env BLUEPRINT_URL=http://127.0.0.1:<porta>` antes do `--`.

**Vincular a pasta.** O Codex não tem hook de sessão, então não descobre sozinho o blueprint da pasta. Escreva esta linha no `AGENTS.md` da pasta (o agente acha os ids com `listar_blueprints`):

```
Blueprint: workspace_id <id>, blueprint_id <id> (MCP blueprint).
```

**O que o Codex enxerga e o que não enxerga:**

| | Claude Code com o plugin | Codex |
| --- | --- | --- |
| Ferramentas MCP (propor, progresso, concluir com evidência) | sim | sim |
| Blueprint e ação atual no contexto ao abrir a sessão | sim (hook) | não: ele lê a linha do `AGENTS.md` |
| Figura na folha acompanha cada edição de arquivo | sim (hook) | só quando ele chama `registrar_progresso` |
| Guia de planejamento | skill, carregada sozinha | ferramenta `guia_de_planejamento` |
| Status line, `/blueprint:status` | sim | não |

A regra de manter o blueprint atualizado e os nomes das ferramentas ficam nos primeiros 512 caracteres das instruções do MCP, que é o que o Codex lê.

## Outros agentes MCP (não testados)

Qualquer agente que rode um servidor MCP por stdio pode usar o mesmo comando: `<pasta>\blueprint.exe --mcp`, mais a linha `Blueprint:` no arquivo de instruções que esse agente lê. Só o Claude Code e o Codex foram testados; conte nas [Issues](https://github.com/hugorolimf/the-blueprint/issues) se outro funcionar.

Ferramentas, fila com o app fechado e idioma: [MCP e outros agentes](mcp.md).
