# Codex: paridade com o plugin do Claude Code

[Voltar ao README](../../README.pt-BR.md) · [English](../codex.md)

O que o Codex CLI da OpenAI oferece hoje para o The Blueprint, comparado com o que o plugin do Claude Code já faz, e o desenho de uma
integração nativa com o Codex. Conferido em 2026-10-27 contra o `codex-cli 0.164.0` local, o repositório
[openai/codex](https://github.com/openai/codex) na tag `rust-v0.164.0` (publicada em 2026-10-23) e
[developers.openai.com/codex](https://developers.openai.com/codex). Todo comando `codex` daqui rodou com `CODEX_HOME` numa pasta
temporária nova; nada chamou um modelo.

## O que funciona hoje (medido)

O `blueprint registrar` registra o servidor MCP no Codex (`src/registrar/mod.rs`, `codex mcp add blueprint -- <exe> --mcp`). Rodado com
`CLAUDE_CONFIG_DIR`, `CODEX_HOME` e `APPDATA` em pastas temporárias, usando o `blueprint.exe` instalado (0.3.0):

```
$ blueprint registrar                      (12 s, junto com o plugin do Claude Code)
Claude Code: plugin `blueprint` instalado (skill, agente e MCP)
Codex: MCP `blueprint` registrado

$ type %CODEX_HOME%\config.toml
[mcp_servers.blueprint]
command = '<tmp>\appdata\Blueprint\bin\blueprint.exe'
args = ["--mcp"]

$ codex mcp list
Name       Command                                     Args   Env  Cwd  Status   Auth
blueprint  <tmp>\appdata\Blueprint\bin\blueprint.exe  --mcp  -    -    enabled  Unsupported

$ blueprint registrar --remover
Claude Code: plugin e MCP removido
Codex: MCP removido

$ codex mcp list
No MCP servers configured yet. Try `codex mcp add my-tool -- my-command`.
```

O `CODEX_HOME` só tinha o `config.toml`, que ficou vazio depois do `--remover`; o `~/.codex/config.toml` real manteve tamanho e data. O
comando aponta para a cópia em `%APPDATA%\Blueprint\bin` porque o `.exe` instalado estava em uso pelo app aberto (a regra do `registrar`
para arquivo travado). Além do MCP, o Codex só recebe a linha `Blueprint: workspace_id …` no `AGENTS.md`.

## O que o Codex oferece (fontes)

| Recurso | No Codex 0.164.0 | Fonte |
| --- | --- | --- |
| Servidores MCP | `codex mcp add <nome> -- <cmd>` (stdio) ou `--url` (HTTP streamable); `[mcp_servers.<nome>]` em `$CODEX_HOME/config.toml` ou no `.codex/config.toml` de um projeto confiável; aprovação por servidor e por ferramenta (`default_tools_approval_mode`, `[mcp_servers.X.tools.<ferramenta>] approval_mode`) | `codex mcp --help` local; [MCP](https://developers.openai.com/codex/mcp) |
| `AGENTS.md` | global `$CODEX_HOME/AGENTS.md` (ou `AGENTS.override.md`), depois um arquivo por pasta da raiz do git até a pasta atual; o mais próximo vence; limite `project_doc_max_bytes`, 32 KiB por padrão | [guia do AGENTS.md](https://developers.openai.com/codex/guides/agents-md) |
| `notify` | `notify = ["prog", "arg"]`; só `agent-turn-complete`, JSON como último argumento (`type`, `thread-id`, `turn-id`, `cwd`, `input-messages`, `last-assistant-message`); o código o chama de "legacy" | [config avançada](https://developers.openai.com/codex/config-advanced), `codex-rs/hooks/src/legacy_notify.rs` |
| Hooks | **estáveis desde a 0.124.0** (notas da `rust-v0.124.0`, 2026-04-23: "Hooks are now stable … can observe MCP tools as well as apply_patch"); `codex features list` mostra `hooks stable true` | [release rust-v0.124.0](https://github.com/openai/codex/releases/tag/rust-v0.124.0), [hooks](https://developers.openai.com/codex/hooks) |
| Arquivos de hook | `hooks.json` ou `[hooks]` no próprio config, ao lado de cada camada de configuração (`~/.codex/hooks.json`, `<repo>/.codex/hooks.json`); plugins trazem `hooks/hooks.json`; todos os hooks que casam rodam | `codex-rs/hooks/src/engine/discovery.rs` (`load_hooks_json`) |
| Eventos | PreModelRequest, PostModelRequest, PreToolUse, PermissionRequest, PostToolUse, PreCompact, PostCompact, SessionStart, SessionEnd, UserPromptSubmit, SubagentStart, SubagentStop, Stop, Interrupt | `codex-rs/config/src/hook_config.rs` (`HookEventsToml`) |
| Tipos de hook | só `command` e `mcp_tool`; `prompt` e `agent` são lidos e ignorados; **não há `http`** | `hook_config.rs` (`HookHandlerConfig`), `discovery.rs` |
| Entrada do hook | JSON na entrada padrão com `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`, `permission_mode`; o PostToolUse acrescenta `turn_id`, `tool_name`, `tool_use_id`, `tool_input`, `tool_response` e, opcionais, `agent_id`/`agent_type`; o SessionStart acrescenta `source` | `codex-rs/hooks/schema/generated/*.input.schema.json` |
| Edições vistas pelos hooks | o `apply_patch` chega como `tool_name: "apply_patch"`, `tool_input: {"command": "<patch>"}` (sem `file_path`); os matchers `Edit` e `Write` o selecionam; ferramentas MCP chegam como `mcp__<servidor>__<ferramenta>` | `codex-rs/core/src/tools/hook_names.rs`, `codex-rs/core/src/tools/handlers/apply_patch.rs` |
| Saída do hook | SessionStart `hookSpecificOutput: {hookEventName, additionalContext}` (o mesmo formato do Claude Code); o Stop aceita `continue`, `decision`, `reason`, `stopReason`, `systemMessage` (sem `hookSpecificOutput`) | `session-start.command.output.schema.json`, `stop.command.output.schema.json` |
| Confiança nos hooks | hook não gerenciado só roda depois que o usuário confia nele em `/hooks`; a confiança é um hash da definição, então hook alterado fica de fora até confiar de novo; vale também para hooks de plugin | `discovery.rs` (`hook_trust_status`), [hooks](https://developers.openai.com/codex/hooks) |
| Skills | pastas com `SKILL.md` (`name`, `description`) em `.agents/skills` (da pasta atual até a raiz do repositório), `$HOME/.agents/skills` ou dentro de um plugin; os prompts personalizados (`~/.codex/prompts`) estão obsoletos em favor das skills | [skills](https://developers.openai.com/codex/skills), [prompts personalizados](https://developers.openai.com/codex/custom-prompts) |
| Plugins | `codex plugin add/list/remove` e `codex plugin marketplace add <pasta local ou git>`; `plugins` estável; o plugin leva `plugin.json`, `skills/`, `.mcp.json` e hooks; hooks de plugin recebem `PLUGIN_ROOT` e também `CLAUDE_PLUGIN_ROOT` | `codex plugin --help` e `codex features list` locais; [criar plugins](https://developers.openai.com/plugins/build/plugins) |
| Subagentes | `multi_agent` estável; agentes personalizados são arquivos TOML em `~/.codex/agents/` ou `.codex/agents/`; hooks SubagentStart e SubagentStop | [subagentes](https://developers.openai.com/codex/subagents) |
| Status line | `/statusline` só liga itens prontos (modelo, contexto, branch…); sem comando próprio | [comandos de barra](https://developers.openai.com/codex/cli/slash-commands) |
| Mandar mensagem a uma sessão aberta | só pelo `codex app-server` (JSON-RPC: `turn/steer`, `thread/inject_items`), ao qual uma TUI se junta com `--remote`; nada parecido com o canal do Claude Code vindo de um servidor MCP | [app-server](https://developers.openai.com/codex/app-server), `codex-rs/app-server-protocol/schema/json/v2/` |
| Abrir com um pedido | `codex [OPTIONS] [PROMPT]` | `codex --help` local |
| Sandbox | o padrão `workspace-write` desliga a rede para os comandos que o modelo roda; os comandos de hook parecem rodar fora do sandbox (não documentado; inferido de `codex-rs/hooks/src/engine/command_runner.rs`) | [aprovações e segurança](https://developers.openai.com/codex/agent-approvals-security) |

## Tabela de paridade

Uma linha por coisa que o plugin do Claude Code faz (`src/registrar/plugin.rs`, `claude-plugin/`, `skills/`). **Tem**, **parcial** ou
**não tem** no Codex 0.164.0; "não verificado" onde nenhuma fonte resolve.

| Plugin do Claude Code | Codex | Como, no Codex | Fonte |
| --- | --- | --- | --- |
| MCP `blueprint` | tem (em uso hoje) | `codex mcp add`, feito pelo `blueprint registrar` | medido acima |
| Skill `blueprint-planejamento` | tem | o mesmo formato de `SKILL.md`, num plugin do Codex ou em `.agents/skills` | [skills](https://developers.openai.com/codex/skills) |
| Agentes `blueprint`, `-explorer`, `-builder`, `-reviewer` | parcial | agentes personalizados em TOML, formato diferente dos agentes em Markdown | [subagentes](https://developers.openai.com/codex/subagents) |
| Comando `/blueprint:status` | parcial | nenhum comando de barra de plugin roda um programa; uma skill pode mandar o agente chamar `ler_blueprint` | [skills](https://developers.openai.com/codex/skills), [prompts personalizados](https://developers.openai.com/codex/custom-prompts) |
| SessionStart: blueprint e ação atual no contexto | tem | hook `command` rodando `blueprint hook sessao`; a saída dele já bate com o esquema do Codex | `session-start.command.output.schema.json` |
| PostToolUse de Edit/Write vira `registrar_progresso` | tem (depois desta mudança) | hook `command` `blueprint hook edicao` (não há hook `http`); a rota agora lê o arquivo no patch do `apply_patch` | `apply_patch.rs`, `src/mcp/hooks.rs` |
| Stop: um lembrete de `registrar_trabalho` | parcial | o hook Stop existe, mas a nossa rota responde `hookSpecificOutput`, que o Stop do Codex não aceita; precisaria de `systemMessage` ou `decision` | `stop.command.output.schema.json` |
| SubagentStart / SubagentStop na folha | tem | os mesmos eventos; carga com `agent_id`/`agent_type` (não conferido numa execução real) | `hook_config.rs` |
| SessionEnd tira a sessão da lista | tem | hook SessionEnd (tempo-limite de 1 s por padrão, 3 s no máximo) | [hooks](https://developers.openai.com/codex/hooks) |
| Notification, PermissionDenied, StopFailure para o estado do agente | parcial | PermissionRequest, PreCompact e PostCompact existem; Notification, PermissionDenied e StopFailure não | `hook_config.rs` |
| Status line (`blueprint hook statusline`) | não tem | só itens prontos | [comandos de barra](https://developers.openai.com/codex/cli/slash-commands) |
| Status line de subagentes | não tem | nenhum equivalente achado | não verificado |
| Monitor (`blueprint watch`) | não tem | o Codex não tem monitores de plugin | não verificado |
| Canal: "Iniciar" manda o pedido a uma sessão aberta | parcial | `codex app-server` + `turn/steer`, só para sessões que entraram com `--remote` | [app-server](https://developers.openai.com/codex/app-server) |
| Lista de sessões do diálogo Iniciar | tem | alimentada pelos hooks SessionStart/SessionEnd, como no Claude | `hook_config.rs` |
| O botão Iniciar abre uma sessão nova | tem | `codex "<pedido>"` num terminal | `codex --help` local |
| Instalado pelo `blueprint registrar` como marketplace local | tem | `codex plugin marketplace add <pasta>` + `codex plugin add` | `codex plugin --help` local |
| Hooks rodam sem perguntar | não tem | todo hook precisa da confiança do usuário em `/hooks` uma vez (e de novo quando muda) | `discovery.rs` (`hook_trust_status`) |
| Modo equipe (`blueprint conectar`) | tem, pelo mesmo caminho | os hooks são comandos com `--equipe`, como no plugin do convidado | `src/hook.rs` |

## Desenho da integração nativa

1. **Um plugin do Codex, não um `hooks.json` global.** O `blueprint registrar` já grava um marketplace local para o Claude Code; grava um
   segundo para o Codex (`plugin.json`, o `SKILL.md`, `.mcp.json` com `blueprint --mcp`, `hooks/hooks.json` com hooks `command` que rodam
   `blueprint hook <evento>`) e depois chama `codex plugin marketplace add` e `codex plugin add`. Gravar `$CODEX_HOME/hooks.json`
   misturaria os nossos hooks com o arquivo do próprio usuário; um plugin sai inteiro no `--remover`.
2. **Só hooks de comando:** SessionStart `hook sessao`, PostToolUse (matcher `Edit|Write`) `hook edicao`, SubagentStart/Stop
   `hook subagente`, Stop `hook parada`, SessionEnd `hook fim` (tempo-limite curto). Um processo por edição (p95 de cerca de 17 ms
   medido para o Claude, `scripts/medir-hooks.mjs`).
3. **Rotas:** reaproveitar `/api/mcp/hook/*`. Feito nesta mudança: o `apply_patch` dá o arquivo pelo patch. Falta: a resposta do Stop no
   formato do Codex e o agente com o nome "Codex" no lugar de "Claude" (a carga tem `model` e `turn_id`).
4. **Confiança:** a primeira sessão depois de instalar mostra os hooks como não confiáveis. A tela Planejar e o `docs/claude-code.md`
   precisam dizer "abra `/hooks` e confie nos hooks do Blueprint uma vez"; sem isso o Codex continua com o MCP e funciona como hoje.
5. **Agora não:** status line, monitor e canal não têm equivalente no Codex; o caminho do app-server exige sessões com `--remote` e fica para depois.

Não verificado: as cargas de SubagentStart/Stop e a resposta do Stop num Codex real; se os hooks de comando alcançam o 127.0.0.1 em todo
modo de sandbox. Os três precisam de uma sessão com um Codex real (checkpoint "Hooks do Codex testados com um Codex real").
