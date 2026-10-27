# The Blueprint v0.3.0 (beta)

[Português](pt-BR/release-v0.3.0.md)

## What is new

- **AI presentations.** On the Present screen you pick an audience (Client: a project plan in plain language, with no technology, hours or internal instructions; Internal: the full technical view) and a level (Quick, Standard, Complete), and your agent builds a real interface in the app's own look, with live anatomy figures. Each section of the page gets its own figure, and pointing at an item in the text highlights it in the figure and the other way around. Details: [AI presentations](apresentacao-ia.md).
- **Generate in the app terminal.** The request runs in a tab of the embedded terminal, already carrying your audience and level choices.
- **Edit by hand, or ask the agent for small changes.** The whole presentation is editable on screen. When you ask the agent for a change it edits the file itself with targeted operations instead of rewriting everything.
- **The Present screen opens on the generated presentation.** The app's own pages (Progress by project, Full plan) wait in the collapsed Internal view group. The old Summary page is gone.
- **Automatic apply.** What you ask for or approve in the chat enters the blueprint without a second click, with an Undo notice for 60 s. A switch per blueprint turns it off (off by default in team mode); removals and moves still wait for your click. The agent is told which item was applied and whether you or the automatic apply did it.

## Known limits

- The presentation flow was tested end to end with a stand-in agent, but not yet with a real Claude session, so a request may not complete in every setup.
- **Print** inside the generated page is blocked by its sandbox: use **Export HTML** and print from the browser.

## How to install

Download `Blueprint-0.3.0-instalador.exe` below (it installs for your user only). Anyone on 0.1.2 or later gets the update by itself and installs it with one click on Update. The `SHA256SUMS` carries the installer's hash.

The installer is not signed: Windows or Defender may warn or block on the first run.
