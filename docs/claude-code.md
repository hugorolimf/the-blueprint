# Claude Code: the Blueprint plugin

[Back to the README](../README.md) · [Português](pt-BR/claude-code.md)

**The installer already does this for you**, and `blueprint registrar` repeats the step by hand. It installs the plugin `blueprint@blueprint-local` in Claude Code, for your user and in any folder:

- the `blueprint` **MCP** (the tools in the table in the [MCP guide](mcp.md); in Claude they appear as `plugin:blueprint:blueprint`);
- the **skill** `blueprint-planejamento` (the guide to planning and executing, which Claude loads by itself) and the `blueprint` **agent** (ask for it with `claude --agent blueprint:blueprint`; it does not become the default of your sessions);
- **hooks**: when a session opens, Claude already knows the folder's blueprint and the current action; on every file edit, and when a subagent starts or ends, the Claude on the sheet follows along in the app (an indicator only: nothing is marked done by itself);
- the **`/blueprint:status`** command: the state of the folder's blueprint (current action, plans and next checkpoint).

**Project folder:** a new blueprint proposed by the agent links the session's folder when you apply it. In a folder without a link, the app offers the choice of blueprint; just select one. In **Edit → Blueprint data → Project folder** you see, change, add and remove the links, without writing files to the folder.

**Status line:** `blueprint claude` turns it on for that session only. For all sessions, use `blueprint registrar --status-line` (optional and unchecked in the installer); `blueprint registrar --status-line --remover` restores the previous one. The installation keeps `settings.json.bak`, preserves the other options and repeating it does not duplicate the integration. If there is already a command, the wrapper runs both with the same JSON, keeping its output; a simple pipe would lose the status line's input. It configures `~/.claude` and also `CLAUDE_CONFIG_DIR` when set. Outside linked folders The Blueprint prints nothing.

To test without touching your configuration: `node scripts/testar-plugin-claude.mjs <path to blueprint.exe>` installs and removes the plugin using temporary folders.

**Terminal and notices:** `blueprint claude` also loads the app's mark (a panel in wide terminals, a strip above the prompt in narrow ones; `/blueprint-marca` reopens it) and the `blueprint watch` monitor, which receives applied proposals and ticked checkpoints after the app saves. Notices use fixed text, a folder filter and a limit of 20 per minute. See [the terminal mod](mod-terminal.md) and [the monitor's tests and limits](monitor-e-channels.md) (both in Portuguese for now).

| | Monitor (default) | Channels (optional) |
| --- | --- | --- |
| Activation | Plugin, no channel flag | `BLUEPRINT_CHANNEL=1` and a development flag |
| Confirmation | The plugin's normal trust | Confirmation in every session |
| Stability | Experimental monitors API | Research preview |
| Cost | Context when it receives a notice | Context when it receives a notice; replaces the monitor |

To try it with the plugin installed, in PowerShell: `$env:BLUEPRINT_CHANNEL='1'` and `blueprint claude --dangerously-load-development-channels plugin:blueprint@blueprint-local`. Use `Remove-Item Env:BLUEPRINT_CHANNEL` before the next normal session. For an MCP registered directly, the identifier is `server:blueprint`. The channel is informational, with no reply tools and no permission approval. [Comparison and real validation](monitor-e-channels.md).
