[English](../../AGENTS.md)

# AGENTS.md — guia para agentes que trabalham neste repositório

The Blueprint (beta), roadmap local-first em Rust (`blueprint.exe`) com interface web. Vocabulário fixo: **Workspace → Blueprint → Projeto (opcional) → Ação → Plano / Checkpoint** (nada de "fase", "tarefa", "marco"; **única exceção, pedida pelo usuário em 2026-10-23:** o aviso na tela de que o agente concluiu um plano se chama "Tarefa concluída" ("Tarefa não concluída"), porque "plano" também é a proposta que o agente envia e o aviso confundia; "projeto" é só o grupo de ações dentro do blueprint, nunca o nome do blueprint).

## Mapa

| Onde | O quê |
| --- | --- |
| `src/dados/` | única camada que fala com o SQLite (migrações em `src/dados/migracoes/NNN_*.sql`; `formato.rs` mapeia o JSON para tabelas) |
| `src/blueprint/` | validação das propostas do agente (`validar_proposta`), prazos, resumos para `ler_blueprint` |
| `src/mcp/` | fila em memória, ações do MCP, atividade dos agentes, SSE, `ferramentas.json` e `instrucoes.txt` (a fonte da verdade do MCP) |
| `src/http/` | servidor HTTP síncrono (`tiny_http`), rotas puras, estáticos com ETag e gzip |
| `src/janela/` | janela WebView2, bandeja e instância única (só Windows) |
| `web/`, `blueprint.html` | interface: CSS em `web/css`, scripts clássicos em `web/js` na ordem listada em `blueprint.html` |
| `kit/`, `emblemas.mjs`, `agentes-figura.mjs`, `agentes-claude.mjs` | kit isométrico (anatomy) e figuras; rodam no navegador |
| `skills/blueprint-planejamento/` | o guia que o agente lê pelo MCP (`guia_de_planejamento`) |
| `scripts/` | verificações de ponta a ponta (contrato HTTP, e2e com Edge, mutações, medições, instalador) |

## Comandos

```
cargo test                        # testes Rust (um tests.rs por módulo)
cargo build --release             # target/release/blueprint.exe
node emblemas.test.mjs && node agentes-figura.test.mjs && node agentes-claude.test.mjs && node scripts/migrar-vocabulario.test.mjs
node scripts/contrato-http.mjs    # API contra referencias/api-snapshots.json
node scripts/ui-e2e.mjs           # interface no Edge sem janela
node scripts/capturas.mjs --comparar
```

## Regras

- Execute testes e compilações somente quando puderem terminar em menos de um minuto; interrompa os que atingirem o limite e informe o que não foi verificado.

- Arquivos de até 350 linhas (teto 500 com motivo no topo). Um arquivo, uma responsabilidade.
- Poucas dependências; sem framework nem bundler na interface.
- Segurança: `/api/mcp/*` só em loopback, `Host` e `Origin` conferidos, todo POST `application/json`; protótipos em `iframe sandbox="allow-scripts"` de origem opaca (nunca `allow-same-origin`). O que o agente envia é proposta; concluir exige evidência.
- Mudou o MCP? Atualize `src/mcp/ferramentas.json`, `instrucoes.txt` (a regra e os nomes das ferramentas ficam nos primeiros 512 caracteres: o Codex só lê isso), o guia em `skills/` e os testes em `src/mcp/tests.rs`.
- Não coloque dados reais no repositório: use `data.exemplo.json`.

## Usar o The Blueprint enquanto trabalha

Se o app estiver rodando e o MCP `blueprint` registrado (`claude mcp add blueprint -- <caminho>\blueprint.exe --mcp`), mantenha o blueprint atualizado: planeje com `propor_blueprint`, avise o que está fazendo com `registrar_progresso` e conclua com `marcar_concluido` + evidência. A linha que liga uma pasta ao blueprint é:

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
```
