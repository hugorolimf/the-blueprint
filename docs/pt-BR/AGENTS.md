[English](../../AGENTS.md)

# AGENTS.md — guia para agentes que trabalham neste repositório

The Blueprint (beta), roadmap local-first em Rust (`blueprint.exe`) com interface web. Vocabulário fixo: **Workspace → Blueprint → Projeto (opcional) → Ação → Plano / Checkpoint** (nada de "fase", "tarefa", "marco"; **única exceção, pedida pelo usuário em 2026-10-23:** o aviso na tela de que o agente concluiu um plano se chama "Tarefa concluída" ("Tarefa não concluída"), porque "plano" também é a proposta que o agente envia e o aviso confundia; "projeto" é só o grupo de ações dentro do blueprint, nunca o nome do blueprint).

## Mapa

| Onde | O quê |
| --- | --- |
| `src/dados/` | única camada que fala com o SQLite (migrações em `src/dados/migracoes/NNN_*.sql`; `formato.rs` mapeia o JSON para tabelas) |
| `src/blueprint/` | validação das propostas do agente (`validar_proposta`), resumos para `ler_blueprint` |
| `src/mcp/` | fila (lida da memória, guardada no banco), ações do MCP, atividade dos agentes, SSE, `ferramentas.json` e `instrucoes.txt` (a fonte da verdade do MCP) |
| `src/http/` | servidor HTTP síncrono (`tiny_http`), rotas puras, estáticos com ETag e gzip |
| `src/janela/` | janela WebView2, bandeja e instância única (só Windows) |
| `ui/` | a interface nativa (egui): um crate que roda como janela no Windows e no Linux e no navegador como WebAssembly; veja [ui/README.md](../../ui/README.md). Substitui o `web/` tela por tela |
| `ui/src/anatomia/` | o nosso motor Anatomy em Rust: câmera com perspectiva, malha de faces e arestas, sólidos, tinta em faixas de luz, cena com teste de clique; veja [skills/blueprint-anatomia/SKILL.md](../../skills/blueprint-anatomia/SKILL.md) |
| `web/`, `blueprint.html` | a interface web antiga, até cada tela migrar para `ui/`: CSS em `web/css`, scripts clássicos em `web/js` na ordem listada em `blueprint.html` |
| `kit/`, `emblemas.mjs`, `agentes-figura.mjs`, `agentes-claude.mjs` | kit isométrico (anatomy) e figuras; rodam no navegador |
| `skills/blueprint-planejamento/` | o guia que o agente lê pelo MCP (`guia_de_planejamento`) |
| `skills/blueprint-anatomia/` | a skill da planta: o motor Anatomy, o palco da ação, o componente Tela e as variantes, e como planejar para a planta dizer algo (com link em `.claude/skills/`) |
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

- **Não execute testes nem compilações do programa, a não ser que o usuário peça na tarefa atual.** Isso vale para `cargo test`, `cargo build`, `cargo run`, `trunk`, as verificações em Node, os scripts e2e e abrir o app para conferir algo. O trabalho dos agentes aqui é escrever código: corrigir bugs, mudar e alterar o que for preciso, colocar em produção. Escrever testes pode e é bem-vindo; só rodá-los depende de o usuário pedir. O usuário roda depois. Diga claramente que a mudança não foi compilada nem testada. Quando o usuário pedir, interrompa o que passar de um minuto e informe o que não foi verificado.

- Arquivos de até 350 linhas (teto 500 com motivo no topo). Um arquivo, uma responsabilidade.
- Poucas dependências. A interface é nativa (`ui/`, egui, o mesmo código em todo sistema); `web/` é legado: nenhuma tela nova ali e sem framework nem bundler nele.
- Segurança: `/api/mcp/*` só em loopback, `Host` e `Origin` conferidos, todo POST `application/json`; protótipos em `iframe sandbox="allow-scripts"` de origem opaca (nunca `allow-same-origin`). O que o agente envia é proposta; concluir exige evidência.
- Mudou o MCP? Atualize `src/mcp/ferramentas.json`, `instrucoes.txt` (a regra e os nomes das ferramentas ficam nos primeiros 512 caracteres: o Codex só lê isso), o guia em `skills/` e os testes em `src/mcp/tests.rs`.
- Não coloque dados reais no repositório: use `data.exemplo.json`.

## Usar o The Blueprint enquanto trabalha

Se o app estiver rodando e o MCP `blueprint` registrado (`claude mcp add blueprint -- <caminho>\blueprint.exe --mcp`), mantenha o blueprint atualizado: planeje com `propor_blueprint`, avise o que está fazendo com `registrar_progresso` e conclua com `marcar_concluido` + evidência. A linha que liga uma pasta ao blueprint é:

```
Blueprint: workspace_id `<workspace_id>`, blueprint_id `<blueprint_id>` (MCP `blueprint`).
```
