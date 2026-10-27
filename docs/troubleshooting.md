# Troubleshooting

[Back to the README](../README.md) · [Português](pt-BR/troubleshooting.md)

Each problem below has a cause and a check you can run yourself. A `blueprint doctor` command that runs these checks for you is on the [roadmap](../ROADMAP.md); until then, use the commands here.

## The window shows old data, or the agent's proposal never appears

**Cause:** two processes are using the same database (for example the app and a `blueprint serve` started by hand). Each one keeps its own copy of the data and its own queue of agent notices, so the window does not see what the other one received.

**Check:** the app writes the port it listens on to `%APPDATA%\Blueprint\porta`. It should say `3000`:

```
type %APPDATA%\Blueprint\porta
```

If it shows another number, port 3000 was taken by another process. **Fix:** quit the app (tray icon → *Sair*, the Quit item), stop the other `blueprint` process (Task Manager), and open the app again. Run `blueprint serve` only when the app is closed.

## The agent says the app is closed

**Cause:** the agent reaches the app at `http://127.0.0.1:3000` (or `BLUEPRINT_URL`). With the app closed, the write tools (`registrar_progresso`, `marcar_concluido`, `propor_blueprint` and the others) are kept as files in `%APPDATA%\Blueprint\pendentes\`, and the agent's reply says so. They enter the queue, in order, when the app opens. The read tools (`ler_blueprint`, `listar_blueprints`) need the app open.

**Check:** `dir %APPDATA%\Blueprint\pendentes` lists what is waiting. Whatever the app refuses when it opens goes to `pendentes\recusados\` with the reason. **Fix:** open The Blueprint from the Start menu.

## Claude Code does not know the folder's blueprint, or the tools are missing

**Cause:** the plugin `blueprint@blueprint-local` is not installed (the installer's *connect* task was unchecked, or the plugin was removed).

**Check:** `claude plugin list` should show `blueprint@blueprint-local`. Inside a session, `/blueprint:status` answers only when the plugin is there. **Fix:** `blueprint registrar` installs the plugin again (and the MCP in Codex). In a folder that was never linked, ask the agent to link it, or pick the blueprint in **Edit → Blueprint data → Project folder**. Details in [the plugin guide](claude-code.md).

**Codex** has no plugin: `codex mcp list` should show `blueprint`, and the folder's `AGENTS.md` needs the `Blueprint: workspace_id …, blueprint_id …` line ([the MCP guide](mcp.md)).

## Windows shows a SmartScreen or Defender warning

**Cause:** the installer is not code-signed yet, so Windows does not know the publisher on the first run.

**Check:** compare the installer's hash with the `SHA256SUMS` file attached to the same release:

```
certutil -hashfile Blueprint-<version>-instalador.exe SHA256
```

If the two match, the file is the one the release workflow built. Signed releases are on the [roadmap](../ROADMAP.md).

## Update refuses to install

The **Update and restart** button (or `blueprint atualizar --instalar`) closes the app, so it refuses while closing would lose something. The notice says which case it is:

| Notice | Fix |
| --- | --- |
| agent notices the screen has not applied yet | open the blueprint, apply or discard them on **Plan**, click it again |
| a workspace is being shared | turn sharing off (**Share** button), click it again |
| sessions running in the app's terminal | close them in the **Terminal** panel, click it again |
| no update ready to install | `blueprint atualizar` looks for one and downloads it |

How updates work: [Updates](updates.md).

## `blueprint` is not recognized in the terminal

**Cause:** the installer does not put `blueprint` on the PATH (the Claude Code plugin does not need it). **Fix:** call it by its full path, `%LOCALAPPDATA%\Programs\Blueprint\blueprint.exe`, or add that folder to the PATH ([Install](install.md)).

## Still stuck?

Open an issue with `blueprint --version`, what you did and what you saw: [Issues](https://github.com/hugorolimf/the-blueprint/issues). Security problems go through [SECURITY.md](../SECURITY.md), not a public issue.
