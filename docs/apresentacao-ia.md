# AI presentations

[Back to the README](../README.md) · [Português](pt-BR/apresentacao-ia.md)

Status: in development. The "Generate with AI" controls are in the Presentation screen; the server side that stores the result and the `/blueprint:apresentar` command are still being built, so a request may not complete yet.

A presentation made by your agent (Claude Code) for a client or for yourself: a real interface in the app's own look (blue plotting paper, boxed headers, section navigation) with live anatomy figures as part of it, built from your blueprint. It is different from the three page types the Presentation screen already offers (Executive summary, Progress by project, Full plan), which the app draws from your data without an agent.

## Pick an audience and a level

| Audience | What it shows | What it never shows |
| --- | --- | --- |
| **Client** | A project plan: what the client receives, the stages and dates, what is expected from the client, open decisions, risks in plain language | Technology, routes, files, hours per plan, the internal instructions of the actions |
| **Internal** | The full technical view, for the person who builds | Nothing is hidden |

| Level | What you get | Estimated time |
| --- | --- | --- |
| **Quick** | One live figure on the cover and the pages of the plan | up to about 1 h |
| **Standard** | One figure per group of stages, each a different object | about 1 to 3 h |
| **Complete** | The Standard set plus one dense machine of the whole system | about 2 to 4 h |

The times are estimates the agent announces before it starts. A bigger level (Epic, 6 hours or more) exists only if you ask the agent for it by name; the screen does not offer it.

## From the screen

1. Open the blueprint, then **Present**.
2. In the side rail, under **Generate with AI**, choose the audience and the level, then click **Generate**.
3. Pick the session that receives the request, or let the app open a new terminal. The request already carries your choices, so the agent does not ask again. It costs tokens, and the dialog says so.
4. When the agent finishes, the presentation shows up under **Generated presentations**. Open it in the viewer, export it as a single HTML file, ask for it again, or remove it.

## From the terminal

Type `/blueprint:apresentar` in a Claude Code session in the blueprint's folder (in development). The agent asks for the audience and the level, asks what the client must provide or decide, announces the time, builds the presentation and tells you where to open it.

## Good to know

- The agent writes only the content and the figure programs; the app assembles the page with its own library, so a big figure does not cost a big message.
- Regenerating redraws only the figures whose project changed.
- The page runs isolated: no network, no access to the app, no storage.
- Generated presentations stay on your machine, never in the Git repository. Read the presentation once before you show it: the exported file leaves the app and nothing recalls it.
- The agent only says what the blueprint says: no invented prices, features or dates; an undecided decision shows as open.
