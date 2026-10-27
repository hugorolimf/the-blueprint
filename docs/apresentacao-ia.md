# AI presentations

[Back to the README](../README.md) · [Português](pt-BR/apresentacao-ia.md)

Status: in development. Generating, viewing, editing by hand and talking to the agent are built; the test with a real Claude session is still pending, so a request may not complete in every setup.

A presentation made by your agent (Claude Code) for a client or for yourself: a real interface in the app's own look (blue plotting paper, boxed headers, section navigation) with live anatomy figures as part of it, built from your blueprint. It is different from the two page types the Presentation screen also offers (Progress by project, Full plan), which the app draws from your data without an agent and which show the technical side.

## Pick an audience and a level

| Audience | What it shows | What it never shows |
| --- | --- | --- |
| **Client** | A project plan: what the client receives, the stages and dates, what is expected from the client, open decisions, risks in plain language | Technology, routes, files, hours per plan, the internal instructions of the actions |
| **Internal** | The full technical view, for the person who builds | Nothing is hidden |

| Level | What you get | Estimated time |
| --- | --- | --- |
| **Quick** | One AI figure in the overview and the pages of the plan | up to about 1 h |
| **Standard** | A different figure for each section of the page | about 1 to 3 h |
| **Complete** | The Standard set, plus one dense machine of the whole system and one figure per stage | about 2 to 4 h |

The times are estimates the agent announces before it starts. A bigger level (Epic, 6 hours or more) exists only if you ask the agent for it by name; the screen does not offer it.

## From the screen

1. Open the blueprint, then **Present**. If the blueprint already has generated presentations, the screen opens on the newest one; the app's own pages (Progress by project, Full plan) wait in the collapsed **Internal view** group.
2. In the side rail, under **Generate with AI**, choose the audience and the level, then click **Generate**.
3. Pick where the conversation runs: the **app terminal** (the default, a tab of the embedded panel where you watch the agent and answer what it asks), a new external terminal, or a session that is already open. The request already carries your choices, so the agent does not ask again. It costs tokens, and the dialog says so.
4. When the agent finishes, the presentation shows up under **Generated presentations**. Open it in the viewer, export it as a single HTML file, generate it again (it asks first if you edited it by hand) or remove it. The viewer is isolated and cannot print by itself: **Print** exports the file, which you print from the browser.

## Edit it by hand

**Edit**, next to the viewer, opens a panel that mirrors the page: general data, summary, what you receive, stages (with dates, status, figure and milestones), what we need from you, decisions, risks and, in the internal audience, actions. Every list has add, remove, move up and move down. Each figure has its title, hint, readout, caption, description, keys and the **Section** of the page where it sits (or none); its code is in an advanced field that only checks the syntax and never runs. Changes are saved by themselves, the viewer reloads where you were, **Undo** reverses the last change, and the warnings of the server (for example a technical term in a client presentation) appear in the panel.

## Talk to the agent

**Talk**, next to the viewer, sends a request such as "shorten the text of stage 3 and change its end date" to a Claude session (the one you pick, or a new tab of the app terminal). The agent reads the presentation and changes **only what you asked**, straight in the file, with small edits: it does not redo the rest, and it keeps what you edited by hand. You can also type the request in the terminal where it is running. When it changes the presentation, the viewer reloads by itself.

## Text and figure work together

In each section the figure is part of the text. Under the section title a line `Fig n · <hint>` links to the figure; point at an item of the text (or reach it with the keyboard, or tap it) and the figure shows that item; move the figure with its own mouse, keys or tour and the matching item of the text is marked. Leave the item and the figure goes back to its own tour. The overview has no items, so its figure keeps its own tour.

## From the terminal

Type `/blueprint:apresentar` in a Claude Code session in the blueprint's folder. The agent asks for the audience and the level, asks what the client must provide or decide, announces the time, builds the presentation and tells you where to open it.

## Good to know

- The agent writes only the content and the figure programs; the app assembles the page with its own library, so a big figure does not cost a big message.
- Regenerating redraws only the figures whose project changed.
- The page runs isolated: no network, no access to the app, no storage.
- Generated presentations stay on your machine, never in the Git repository. Read the presentation once before you show it: the exported file leaves the app and nothing recalls it.
- The agent only says what the blueprint says: no invented prices, features or dates; an undecided decision shows as open.
