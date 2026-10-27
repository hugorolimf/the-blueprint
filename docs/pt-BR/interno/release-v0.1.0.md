# The Blueprint v0.1.0 (beta)

[English](../../interno/release-v0.1.0.md)

Primeira versão pública, em beta (pode mudar entre versões): um roadmap local-first, em Rust, que **o seu agente de IA** (Claude Code, Codex…) lê e atualiza pelo MCP e que **você** revisa numa folha de blueprint.

## O que faz

- Modelo **Workspace → Blueprint → Projeto → Ação → Plano / Checkpoint**, com datas, porcentagem calculada e folha de plotagem; cada projeto é uma seção numerada da folha.
- O agente **propõe** (`propor_blueprint`) e você **aplica** na tela Planejar (uma proposta de cada vez, numerada); concluir um plano ou checkpoint exige **evidência**.
- Prazos no ritmo da IA: horas por plano, avisos de prazo longo ou frouxo e medição do tempo real.
- **Plugin do Claude Code** (skill, agente, MCP e hooks), mod do terminal e status line: a sessão já começa no blueprint e na ação certa, e as edições viram progresso.
- O **Claude** (e os subagentes) aparece na folha, anda até o plano em que trabalha e repousa quando para; o botão **Iniciar com o Claude** abre uma sessão nova numa ação.
- Com o app **fechado**, o agente continua registrando: o que ele fez fica guardado e entra no app quando ele abre.
- **Atualização automática com o seu clique**: o app baixa a versão nova em segundo plano, confere o SHA-256 e só instala quando você clica em Atualizar (janela, bandeja ou `blueprint atualizar --instalar`).
- Protótipos HTML de ação rodam num visor isolado (sandbox).
- App para Windows (janela WebView2, bandeja, instalador de 3 MB, dados em `%APPDATA%\Blueprint`) e `blueprint serve` para qualquer sistema.

## Como instalar

Baixe `Blueprint-0.1.0-instalador.exe` (instala só para o seu usuário) ou compile com `cargo build --release`. O instalador conecta o plugin e o MCP ao Claude Code e ao Codex que achar. À mão: `claude mcp add blueprint -- <caminho>\blueprint.exe --mcp`. Detalhes no [README](../../../README.pt-BR.md). O `SHA256SUMS` desta release traz o hash do instalador.

## Limitações

- A janela, a bandeja e o instalador são só para Windows; nos outros sistemas use `blueprint serve` no navegador.
- Sem login nem papéis: é um app local (as rotas do MCP só respondem em loopback). O modo servidor colaborativo ainda não existe.
- O app não conversa com agentes: só mostra o que o seu agente informa (e abre a sessão do Claude quando você clica).
- O instalador não é assinado (o Windows pode avisar na primeira execução) e o `SHA256SUMS` vem da mesma release: confere o download, não o autor. Assinatura é o próximo passo.
