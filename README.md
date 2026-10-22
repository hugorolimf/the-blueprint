# The Blueprint (beta) — roadmap com folha de projeto, feito para agentes de IA

Um roadmap local-first que o **seu agente** (Claude Code, Codex…) lê e atualiza pelo MCP, e que **você** revisa numa folha de blueprint. O agente propõe, você aplica; o agente marca o que concluiu com evidência, você vê o robozinho andando pela folha.

![Folha de um blueprint](docs/img/blueprint.png)

> **English, in short:** a local-first roadmap app (single Rust binary, SQLite, web UI in a native WebView2 window) that your own coding agent drives over MCP. Workspace → Blueprint → Action → Plan/Checkpoint. Agents *propose* changes (you review and apply them) and *mark* work done only with evidence. The app never launches or talks to agents itself. Windows is the primary target (window + tray + installer); `blueprint serve` runs the same app as a plain local web server.

## O modelo

```
Workspace
└─ Blueprint            o escopo (era "projeto")
   └─ Ação              início, fim, status, instruções (era "fase")
      ├─ Plano          título, descrição, horas estimadas, feito (era "tarefa")
      └─ Checkpoint     nome, data, feito, evidência (era "marco")
```

- A porcentagem de cada ação e blueprint vem só dos planos e checkpoints marcados.
- **Prazos no ritmo da IA:** plano de 0,5 a 4 h, ação de 1 a 3 dias; o validador avisa de prazo frouxo, apertado ou longo demais, e o app mede o tempo real de cada plano.
- Tudo o que o agente envia é **proposta**: nada muda nos seus dados até você aplicar na tela **✦ Planejar**. Concluir um plano exige **evidência** (comando ou teste rodado e o resultado), que vira comentário do plano.
- O app **não abre, não conversa e não executa agentes**. Ele só mostra o que o seu agente informa (painel de agentes e robô na folha).

## Instalar

**Windows 10/11 (janela, bandeja e instalador)**

- Baixe o instalador `Blueprint-<versão>-instalador.exe` da página de Releases (instala só para o seu usuário, sem administrador), ou
- compile: instale o [Rust](https://rustup.rs) e rode `cargo build --release`; o executável sai em `target/release/blueprint.exe` (cerca de 2,5 MB, a interface vai embutida). O instalador sai de `ISCC instalador\blueprint.iss` (Inno Setup 6).

Os dados ficam em `%APPDATA%\Blueprint\` (banco SQLite `blueprint.db` e `prototipos\`). Para começar com dados de exemplo: `blueprint importar data.exemplo.json %APPDATA%\Blueprint\blueprint.db`.

**Qualquer sistema (só o servidor)**

```
cargo build --release
./target/release/blueprint serve        # http://127.0.0.1:3000
```

Variáveis do `serve`: `HOST` (padrão 127.0.0.1), `PORT` (3000), `BLUEPRINT_DB` (banco), `DADOS_ARQ` (JSON importado na primeira execução com o banco vazio), `PROTOTIPOS_DIR`, `BLUEPRINT_ASSETS_DIR` (serve `web/` direto da pasta, sem recompilar) e `MCP_REDE=1` (veja [SECURITY.md](SECURITY.md) antes de usar).

Comandos: `blueprint` (janela), `blueprint serve`, `blueprint --mcp` (MCP por stdio), `blueprint importar <json> <banco>`, `blueprint exportar <banco> <json>` e `blueprint registrar [--remover]` (liga ou desliga o MCP no Claude Code e no Codex).

## Conectar o seu agente (MCP)

**Com o instalador isso já vem feito:** ele liga o MCP `blueprint` no Claude Code e no Codex que achar no PATH (para o seu usuário, em qualquer pasta) e a desinstalação desliga; `blueprint registrar` repete o passo à mão. Sem o instalador, com o app aberto (ou `blueprint serve` rodando):

```
claude mcp add blueprint -- <caminho>\blueprint.exe --mcp
codex mcp add blueprint -- <caminho>\blueprint.exe --mcp
```

Se o app não estiver em `http://127.0.0.1:3000`, passe `BLUEPRINT_URL` (`claude mcp add blueprint -e BLUEPRINT_URL=http://127.0.0.1:3917 -- …`). A tela **✦ Planejar → Como conectar** mostra o comando exato. Depois, grave na pasta do seu projeto, no `CLAUDE.md` ou `AGENTS.md`, a linha `Blueprint: workspace_id `…`, blueprint_id `…` (MCP `blueprint`).` (o agente descobre os ids com `listar_blueprints`).

Ferramentas:

| Ferramenta | Para quê |
| --- | --- |
| `listar_blueprints` | workspaces e blueprints, com ids |
| `guia_de_planejamento` | o guia por partes (`planejar`, `editar`, `executar`, `prototipos`) |
| `ler_blueprint` | só a estrutura, ou uma ação inteira (`acao_id`) |
| `propor_blueprint` | propõe ações, planos e checkpoints (parciais completam o que existe; remover é pedido que você aprova) |
| `registrar_progresso`, `reportar_bloqueio` | o que está fazendo agora; aparece no painel de agentes e no robô da folha |
| `marcar_concluido`, `marcar_nao_concluido` | plano concluído (com evidência) ou não |
| `marcar_checkpoint_feito`, `marcar_checkpoint_nao_feito` | idem para checkpoints |
| `anexar_prototipo` | protótipo HTML autocontido de uma ação, visto num visor isolado |

O guia que o agente lê está em [`skills/blueprint-planejamento/`](skills/blueprint-planejamento/SKILL.md).

## Desenvolver e testar

```
cargo dev                                    # o app em modo dev, com recarga automática (precisa do cargo-watch)
```

`cargo dev` abre a janela do app com os seus dados (`%APPDATA%\Blueprint`), então feche o app antes. Salvou `web/`, `kit/` ou `blueprint.html`: a janela recarrega (CSS troca sem perder a tela; F12 abre as ferramentas). Salvou `src/`, `skills/` ou `Cargo.toml`: o `cargo-watch` (`cargo install cargo-watch`, uma vez) recompila e reabre. Sem ele: `cargo run -- dev`. Para trabalhar no navegador com dados de exemplo, sem Node na janela: `node dev.mjs` (http://127.0.0.1:3917; opções `--porta N`, `--limpar`, `--abrir`).

```
cargo test                                   # 95 testes (92 fora do Windows): dados, validação, MCP, HTTP
node emblemas.test.mjs                       # marcas e figuras (kit do anatomy)
node agentes-figura.test.mjs
node scripts/migrar-vocabulario.test.mjs
```

Verificações de ponta a ponta (precisam do `target/release/blueprint.exe` e, as de tela, do Microsoft Edge): `scripts/contrato-http.mjs`, `fumaca-rust.mjs`, `ui-e2e.mjs`, `robo-e2e.mjs`, `painel-agentes-e2e.mjs`, `capturas.mjs --comparar`, `mutacoes.mjs` (quebra as regras de segurança de propósito e confere que algum teste falha) e `testar-instalador.mjs`. Veja [CONTRIBUTING.md](CONTRIBUTING.md).

## Desempenho (medido)

Binário de 2,5 MB, servidor com ~11 MB de memória e 0% de CPU em repouso, `GET /api/dados` abaixo de 5 ms, janela visível em ~0,1 s. Detalhes em [`referencias/relatorio-medicao.md`](referencias/relatorio-medicao.md).

## Segurança

As rotas `/api/mcp/*` só respondem em loopback e conferem `Host` e `Origin`; os protótipos rodam num `iframe` com sandbox de origem opaca e sem rede. Leia [SECURITY.md](SECURITY.md).

## Licença

[MIT](LICENSE). O kit isométrico descende do [Anatomy](https://github.com/wheresryan22/anatomy) (MIT); veja [NOTICE.md](NOTICE.md).
