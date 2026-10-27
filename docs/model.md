# The model: Workspace, Blueprint, Project, Action, Plan and Checkpoint

[Back to the README](../README.md) · [Português](pt-BR/model.md)

```
Workspace
└─ Blueprint            the scope
   └─ Project           groups actions inside the blueprint (optional; without a project, the action stays under "Geral"/"General")
      └─ Action         start, end, status, instructions (it used to be called "phase")
         ├─ Plan        title, description, estimated hours, done (it used to be called "task")
         └─ Checkpoint  name, date, done, evidence (it used to be called "milestone")
```

On the sheet, each project is a numbered section (01, 02…) with a blueprint-style header and its actions below it (it starts collapsed); actions are numbered per project (02.3 = project 2, action 3).

- The percentage of each action and blueprint comes only from the ticked plans and checkpoints.
- **Estimated and elapsed time:** plans can have any positive estimate. The first progress notice and the completion notice record start and finish durably, even with no page open; completion still needs review. Their difference is elapsed time, including pauses. The planning guide uses measured history as context, not as a promise.
- **Days and Hours:** on a blueprint, choose Hours (or Alt+H) and pick a day to compare the estimated sequence with elapsed time on the same sheet. Estimates begin at midnight on the action's start date; a plan without an estimate uses 1.5 hours. Hover or focus a bar for its plan and duration. The running time refreshes when new agent notices arrive. The preference stays in this browser.
- Everything the agent sends is a **proposal**: nothing changes in your data until you apply it on the **Plan** screen. Completing a plan requires **evidence** (the command or test that was run and its result), which becomes a comment on the plan.
- The app never sends work to anyone on its own: **you** start an action, either by typing `blueprint claude`, or with the **Start with Claude** button on the action's sheet. It asks for confirmation (it spends tokens), opens a new terminal in the blueprint's folder with Claude Code and the action's prompt as the argument of the new session (never through the monitor or the channel, and nothing goes to a session that is already open). Apart from that, the app only shows what the agents report: Claude on the sheet (one per agent and subagent, on the plan it works on; the finished one stays until you click it and read the message).
