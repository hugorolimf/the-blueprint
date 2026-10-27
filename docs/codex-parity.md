# Codex: parity with the Claude Code plugin

[Back to the README](../README.md) · [Português](pt-BR/codex-parity.md)

What the OpenAI Codex CLI offers today for The Blueprint, compared with what the Claude Code plugin already does, and the design of a
native Codex integration. Checked on 2026-10-27 against the local `codex-cli 0.164.0`, the repository
[openai/codex](https://github.com/openai/codex) at tag `rust-v0.164.0` (published 2026-10-23) and
[developers.openai.com/codex](https://developers.openai.com/codex). Every `codex` command here ran with `CODEX_HOME` in a new temporary
folder; nothing called a model.

## What works today (measured)

`blueprint registrar` registers the MCP server in Codex (`src/registrar/mod.rs`, `codex mcp add blueprint -- <exe> --mcp`). Run with
`CLAUDE_CONFIG_DIR`, `CODEX_HOME` and `APPDATA` in temporary folders, using the installed `blueprint.exe` (0.3.0):

```
$ blueprint registrar                      (12 s, together with the Claude Code plugin)
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

`CODEX_HOME` held only `config.toml`, which was empty after `--remover`; the real `~/.codex/config.toml` kept its size and date. The
command points at the copy in `%APPDATA%\Blueprint\bin` because the installed `.exe` was in use by the open app (the `registrar` rule
for a locked file). Besides the MCP, Codex only gets the `Blueprint: workspace_id …` line in `AGENTS.md`.

## What Codex offers (sources)

| Capability | In Codex 0.164.0 | Source |
| --- | --- | --- |
| MCP servers | `codex mcp add <name> -- <cmd>` (stdio) or `--url` (streamable HTTP); `[mcp_servers.<name>]` in `$CODEX_HOME/config.toml` or a trusted project's `.codex/config.toml`; per-server and per-tool approval (`default_tools_approval_mode`, `[mcp_servers.X.tools.<tool>] approval_mode`) | local `codex mcp --help`; [MCP](https://developers.openai.com/codex/mcp) |
| `AGENTS.md` | global `$CODEX_HOME/AGENTS.md` (or `AGENTS.override.md`), then one file per folder from the git root down to the cwd; nearest wins; cap `project_doc_max_bytes`, 32 KiB by default | [AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md) |
| `notify` | `notify = ["prog", "arg"]`; only `agent-turn-complete`, JSON as the last argument (`type`, `thread-id`, `turn-id`, `cwd`, `input-messages`, `last-assistant-message`); called "legacy" in the code | [advanced config](https://developers.openai.com/codex/config-advanced), `codex-rs/hooks/src/legacy_notify.rs` |
| Hooks | **stable since 0.124.0** (release notes of `rust-v0.124.0`, 2026-04-23: "Hooks are now stable … can observe MCP tools as well as apply_patch"); `codex features list` shows `hooks stable true` | [release rust-v0.124.0](https://github.com/openai/codex/releases/tag/rust-v0.124.0), [hooks](https://developers.openai.com/codex/hooks) |
| Hook files | `hooks.json` or inline `[hooks]` next to each config layer (`~/.codex/hooks.json`, `<repo>/.codex/hooks.json`); plugins ship `hooks/hooks.json`; all matching hooks run | `codex-rs/hooks/src/engine/discovery.rs` (`load_hooks_json`) |
| Hook events | PreModelRequest, PostModelRequest, PreToolUse, PermissionRequest, PostToolUse, PreCompact, PostCompact, SessionStart, SessionEnd, UserPromptSubmit, SubagentStart, SubagentStop, Stop, Interrupt | `codex-rs/config/src/hook_config.rs` (`HookEventsToml`) |
| Hook handler types | `command` and `mcp_tool` only; `prompt` and `agent` are parsed and skipped; **no `http`** | `hook_config.rs` (`HookHandlerConfig`), `discovery.rs` |
| Hook input | stdin JSON with `session_id`, `transcript_path`, `cwd`, `hook_event_name`, `model`, `permission_mode`; PostToolUse adds `turn_id`, `tool_name`, `tool_use_id`, `tool_input`, `tool_response`, optional `agent_id`/`agent_type`; SessionStart adds `source` | `codex-rs/hooks/schema/generated/*.input.schema.json` |
| Edits seen by hooks | `apply_patch` arrives as `tool_name: "apply_patch"`, `tool_input: {"command": "<patch>"}` (no `file_path`); matchers `Edit` and `Write` select it; MCP tools arrive as `mcp__<server>__<tool>` | `codex-rs/core/src/tools/hook_names.rs`, `codex-rs/core/src/tools/handlers/apply_patch.rs` |
| Hook output | SessionStart `hookSpecificOutput: {hookEventName, additionalContext}` (same shape as Claude Code); Stop accepts `continue`, `decision`, `reason`, `stopReason`, `systemMessage` (no `hookSpecificOutput`) | `session-start.command.output.schema.json`, `stop.command.output.schema.json` |
| Hook trust | a hook that is not managed runs only after the user trusts it in `/hooks`; trust is a hash of the definition, so a changed hook is skipped until trusted again; plugin hooks too | `discovery.rs` (`hook_trust_status`), [hooks](https://developers.openai.com/codex/hooks) |
| Skills | folders with `SKILL.md` (`name`, `description`) in `.agents/skills` (cwd up to the repo root), `$HOME/.agents/skills`, or inside a plugin; custom prompts (`~/.codex/prompts`) are deprecated in favour of skills | [skills](https://developers.openai.com/codex/skills), [custom prompts](https://developers.openai.com/codex/custom-prompts) |
| Plugins | `codex plugin add/list/remove` and `codex plugin marketplace add <local path or git>`; `plugins` stable; a plugin carries `plugin.json`, `skills/`, `.mcp.json` and hooks; plugin hooks get `PLUGIN_ROOT` and also `CLAUDE_PLUGIN_ROOT` | local `codex plugin --help`, `codex features list`; [build plugins](https://developers.openai.com/plugins/build/plugins) |
| Subagents | `multi_agent` stable; custom agents are TOML files in `~/.codex/agents/` or `.codex/agents/`; SubagentStart and SubagentStop hooks | [subagents](https://developers.openai.com/codex/subagents) |
| Status line | `/statusline` only toggles built-in items (model, context, branch…); no custom command | [slash commands](https://developers.openai.com/codex/cli/slash-commands) |
| Pushing a message into a running session | only through `codex app-server` (JSON-RPC: `turn/steer`, `thread/inject_items`), which a TUI joins with `--remote`; nothing like a Claude Code channel from an MCP server | [app-server](https://developers.openai.com/codex/app-server), `codex-rs/app-server-protocol/schema/json/v2/` |
| Start with a prompt | `codex [OPTIONS] [PROMPT]` | local `codex --help` |
| Sandbox | default `workspace-write` turns network off for commands the model runs; hook commands appear to run outside the sandbox (not documented, inferred from `codex-rs/hooks/src/engine/command_runner.rs`) | [approvals and security](https://developers.openai.com/codex/agent-approvals-security) |

## Parity table

One line per thing the Claude Code plugin does (`src/registrar/plugin.rs`, `claude-plugin/`, `skills/`). **Has**, **partial** or
**none** in Codex 0.164.0; "not verified" where no source settles it.

| Claude Code plugin | Codex | How, in Codex | Source |
| --- | --- | --- | --- |
| MCP `blueprint` | has (in use today) | `codex mcp add`, done by `blueprint registrar` | measured above |
| Skill `blueprint-planejamento` | has | the same `SKILL.md` format, shipped in a Codex plugin or `.agents/skills` | [skills](https://developers.openai.com/codex/skills) |
| Agents `blueprint`, `-explorer`, `-builder`, `-reviewer` | partial | TOML custom agents, a different format from the Markdown agents | [subagents](https://developers.openai.com/codex/subagents) |
| Command `/blueprint:status` | partial | no plugin slash command runs a program; a skill can tell the agent to call `ler_blueprint` | [skills](https://developers.openai.com/codex/skills), [custom prompts](https://developers.openai.com/codex/custom-prompts) |
| SessionStart: blueprint and current action in the context | has | `command` hook running `blueprint hook sessao`; its output already matches Codex's schema | `session-start.command.output.schema.json` |
| PostToolUse of Edit/Write becomes `registrar_progresso` | has (after this change) | `command` hook `blueprint hook edicao` (no `http` hooks); the route now reads the file from the `apply_patch` patch | `apply_patch.rs`, `src/mcp/hooks.rs` |
| Stop: one reminder to `registrar_trabalho` | partial | the Stop hook exists, but our route answers `hookSpecificOutput`, which Codex's Stop does not take; it would need `systemMessage` or `decision` | `stop.command.output.schema.json` |
| SubagentStart / SubagentStop on the sheet | has | the same events; payload with `agent_id`/`agent_type` (not verified against a real run) | `hook_config.rs` |
| SessionEnd removes the session | has | SessionEnd hook (1 s timeout by default, 3 s at most) | [hooks](https://developers.openai.com/codex/hooks) |
| Notification, PermissionDenied, StopFailure to the agent state | partial | PermissionRequest, PreCompact and PostCompact exist; Notification, PermissionDenied and StopFailure do not | `hook_config.rs` |
| Status line (`blueprint hook statusline`) | none | built-in items only | [slash commands](https://developers.openai.com/codex/cli/slash-commands) |
| Subagent status line | none | no equivalent found | not verified |
| Monitor (`blueprint watch`) | none | no plugin monitors in Codex | not verified |
| Channel: "Start" sends the request to an open session | partial | `codex app-server` + `turn/steer`, only for sessions joined with `--remote` | [app-server](https://developers.openai.com/codex/app-server) |
| Session list for the Start dialog | has | fed by the SessionStart/SessionEnd hooks, as for Claude | `hook_config.rs` |
| Start button opens a new session | has | `codex "<prompt>"` in a terminal | local `codex --help` |
| Installed by `blueprint registrar` as a local marketplace | has | `codex plugin marketplace add <folder>` + `codex plugin add` | local `codex plugin --help` |
| Hooks run without asking | none | every hook needs the user's trust in `/hooks` once (and again after it changes) | `discovery.rs` (`hook_trust_status`) |
| Team mode (`blueprint conectar`) | has, by the same path | the hooks are commands with `--equipe`, as in the guest plugin | `src/hook.rs` |

## Design of the native integration

1. **A Codex plugin, not a global `hooks.json`.** `blueprint registrar` already writes a local marketplace for Claude Code; it writes a
   second one for Codex (`plugin.json`, the `SKILL.md`, `.mcp.json` with `blueprint --mcp`, `hooks/hooks.json` with `command` hooks
   that run `blueprint hook <event>`), then `codex plugin marketplace add` and `codex plugin add`. Writing `$CODEX_HOME/hooks.json`
   would mix our hooks with the user's own file; a plugin is removed as a unit by `--remover`.
2. **Command hooks only:** SessionStart `hook sessao`, PostToolUse (matcher `Edit|Write`) `hook edicao`, SubagentStart/Stop
   `hook subagente`, Stop `hook parada`, SessionEnd `hook fim` (short timeout). One process per edit (about 17 ms p95 measured for
   Claude, `scripts/medir-hooks.mjs`).
3. **Routes:** reuse `/api/mcp/hook/*`. Done in this change: `apply_patch` gives the file through its patch. Still to do: the Stop reply
   in Codex's shape, and the agent named "Codex" instead of "Claude" (the payload has `model` and `turn_id`).
4. **Trust:** the first session after install shows the hooks as untrusted. The Plan screen and `docs/claude-code.md` must say "open
   `/hooks` and trust the Blueprint hooks once"; without that, Codex still has the MCP and works as today.
5. **Not now:** status line, monitor and the channel have no Codex equivalent; the app-server route needs `--remote` sessions and is left for later.

Not verified: SubagentStart/Stop payloads and the Stop reply against a real Codex run; whether command hooks reach 127.0.0.1 under every
sandbox mode. All three need one session with a real Codex (checkpoint "Codex hooks tested with a real Codex").
