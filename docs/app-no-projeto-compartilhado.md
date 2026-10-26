# O app instalado no projeto compartilhado

Contrato da ação «App instalado no projeto compartilhado» (projeto Colaboração em tempo real, decidido em 2026-10-26). O convidado de um workspace
compartilhado (docs/compartilhar.md) trabalha no The Blueprint **instalado na máquina dele**: vê e edita o projeto ao vivo, configura a pasta do
repositório (docs/repositorio-github.md) e inicia o Claude numa ação, tudo nesta máquina.

## O desenho

A janela da equipe (`src/janela/equipe.rs`) continua carregando a **página do servidor do dono**: os dados e a sincronia ao vivo são os dele, e o
app instalado não vira cliente de dois servidores. As rotas locais do app (terminal, pastas, iniciar, repositório) continuam **inexistentes** para essa
página (404 `so-local`). O que roda na máquina da pessoa chega ao app por **mensagens IPC** do WebView2 (como `conectar` e `equipe`), e cada uma é
validada pelo app e **confirmada numa pergunta nativa do Windows** que mostra o que vai rodar. O servidor do dono nunca executa nada aqui.

## Mensagens (`projeto <json>`, só de página `https://`)

| tipo | campos | o que o app faz |
| --- | --- | --- |
| `configurar` | `workspaceId`, `blueprintId`, `nome`, `repositorio` | pergunta, cria `%USERPROFILE%\Blueprint\Projetos` e abre o Claude do convidado nela com o prompt de "Configurar projeto" (clonar em `Projetos\<repositório>`, preparar, `vincular_pasta`) |
| `iniciar` | os de cima + `acaoId`, `numero`, `acao` | só se `Projetos\<repositório>` já existir **e** tiver a linha deste blueprint; pergunta e abre o Claude **nessa pasta** com o pedido de executar a ação |

Validação (`src/janela/projeto_local.rs`, `interpretar`): corpo até 4000 bytes; ids só com letras, números, `-` e `_` (até 64); `repositorio` precisa passar
em `repositorio_valido` (https://github.com/dono/repo); `numero` só dígitos e `.`; `nome` e `acao` passam por `limpo` (sem controle, sem `;`, 80 caracteres).
**Nenhum caminho, comando ou texto livre da página chega ao terminal**: o prompt é montado pelo app. Se o Claude do convidado não foi conectado a **este**
servidor (`blueprint conectar`, `conexao-equipe.json`), o app avisa e não faz nada.

## A ligação pasta e blueprint do convidado

O servidor do dono não conhece as pastas da máquina do convidado. `vincular_pasta` no plugin do convidado (`--mcp --equipe`) grava a linha
`Blueprint: workspace_id …, blueprint_id …` em **`CLAUDE.local.md`** da pasta e põe esse arquivo em **`.git/info/exclude`** (a lista local, não versionada):
o repositório do projeto não muda. `blueprint hook --equipe` e o app leem a linha (`CLAUDE.local.md`, `CLAUDE.md`, `AGENTS.md`, nesta ordem, na pasta e nas de cima;
o app confere só a pasta do repositório). Código: `src/mcp/pasta_local.rs`.

## `blueprint claude --equipe`

Abre o `claude` só com `--name "The Blueprint"` e o prompt. Sem o plugin local (`--plugin-dir`) e sem o canal: eles falam com o app **desta** máquina, que não é
dono dos dados do workspace compartilhado. O plugin `blueprint-equipe` já instalado pelo `blueprint conectar` traz o MCP e os hooks do convidado.

## Na página da equipe

No modo hospedado com `window.ipc` (e papel diferente de visitante): o card Repositório de Editar ganha **Configurar projeto**, o aviso do canto da tela aparece uma vez
por blueprint com repositório (o IPC é de ida só, então a página não sabe se a pasta já existe; pedir ou "Agora não" silencia) e **Iniciar com o Claude** manda `iniciar`.
Supervisor só lê: nenhum pedido sai. No navegador comum nada disso aparece.

## Limites

- Sem seletor de pastas: o destino é sempre `%USERPROFILE%\Blueprint\Projetos\<repositório>` (`BLUEPRINT_PROJETOS` troca, para teste), à vista na pergunta.
- Só GitHub (`https://github.com`), como no app local. Blueprint sem repositório não inicia pela janela da equipe.
- A pergunta nativa e o Windows Terminal reais só foram exercitados com as pontas falsas (cargo test) e o `claude` falso (scripts/convidado-app-e2e.mjs); falta um teste à mão numa janela de equipe de verdade.

## Verificações

`cargo test projeto_local pasta_local repositorio`, `node repositorio.test.mjs`, `node scripts/repositorio-e2e.mjs` (inclui a simulação da janela da equipe) e
`node scripts/convidado-app-e2e.mjs` (túnel falso, `claude` falso).
