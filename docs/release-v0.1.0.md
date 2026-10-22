# The Blueprint v0.1.0 (beta)

Primeira versão pública, em beta (pode mudar entre versões): um roadmap local-first, em Rust, que **o seu agente de IA** (Claude Code, Codex…) lê e atualiza pelo MCP e que **você** revisa numa folha de blueprint.

## O que faz

- Modelo **Workspace → Blueprint → Ação → Plano / Checkpoint**, com datas, porcentagem calculada e folha de plotagem.
- O agente **propõe** (`propor_blueprint`) e você **aplica** na tela Planejar; concluir um plano ou checkpoint exige **evidência**.
- Prazos no ritmo da IA: horas por plano, avisos de prazo longo ou frouxo e medição do tempo real.
- Painel de agentes e robô na folha mostram o que o seu agente informa; protótipos HTML de ação rodam num visor isolado (sandbox).
- App para Windows (janela WebView2, bandeja, instalador de 3 MB, dados em `%APPDATA%\Blueprint`) e `blueprint serve` para qualquer sistema.

## Como instalar

Baixe `Blueprint-0.1.0-instalador.exe` (instala só para o seu usuário) ou compile com `cargo build --release`. Conecte o agente: `claude mcp add blueprint -- <caminho>\blueprint.exe --mcp`. Detalhes no [README](../README.md).

## Limitações

- A janela, a bandeja e o instalador são só para Windows; nos outros sistemas use `blueprint serve` no navegador.
- Sem login nem papéis: é um app local (as rotas do MCP só respondem em loopback). O modo servidor colaborativo ainda não existe.
- O app não abre nem conversa com agentes: só mostra o que o seu agente informa.
- O instalador não é assinado (o Windows pode avisar na primeira execução).
