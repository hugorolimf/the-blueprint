# Shape the plan to this project

The app draws your plan as a blueprint. Each plan is a building on the sheet, each dependency a conveyor belt between two buildings, and each decision, question, screen or checkpoint a numbered piece inside its building. A generic plan draws a generic factory, so write the blueprint of this project and no other.

## Before you write

- Read the project: its folders and modules, the names in the code, what exists and what does not.
- Use its real names: files, screens, commands, tables. Never "Backend changes", "Implement the feature" or "Add tests".
- Each plan is one thing that will exist when it is done. If you cannot say what, split it or drop it.

## Numbers follow their parent

Action `1` has plans `1.1`, `1.2`; action `2.3` has plans `2.3.1`, `2.3.2`. The server refuses a number outside its action. The app numbers the pieces of a plan itself (`1.1.1`, `1.1.2`). A new plan takes the next free number; keep the number of what already exists.

## What each plan says

| Field | What it is |
| --- | --- |
| `texto` | what to do, in one line |
| `entrega` | what exists when it is done, one line up to 80 characters, for the person: "Model and effort selectors on the orchestrator card" |
| `forma` | what its building looks like |
| `area` | the part of the project it touches, as a path |
| `depende_de` | what it waits for: only real dependencies |
| `instrucoes` | everything the agent needs |

`forma` is one of:

| Value | For |
| --- | --- |
| `tela` | a screen or part of the interface |
| `dados` | tables, migrations, storage |
| `servico` | server logic |
| `integracao` | what links two systems: API, MCP, webhook |
| `ia` | prompts, agents, the model |
| `teste` | a test suite or a check |
| `documento` | documentation |
| `infra` | build, deploy, machines |

## Dependencies are belts

The person sees a belt going into each plan that waits for another, with a padlock until the first is done. So declare only what truly must come first. Plans that do not depend on each other stand side by side and run at the same time, which is the point: a long line of plans where each waits for the last is a plan that was not thought through.

## What belongs to a plan stays in the plan

Put `plano` on every decision, question, screen, table and checkpoint, so it sits inside its building. Use the kind that says what you need from the person:

- **Decision:** you already chose. The person confirms, asks for an adjustment or disputes it. Say what you chose and why.
- **Question:** you do not know. Give the options and the one you recommend.
- **Screen:** a result to look at. The person approves it or asks for changes. Show this project's real screen.
- **Checkpoint:** something anyone can answer yes or no.

Ask for approval only where the answer changes what gets built.
