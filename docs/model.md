# The model: Workspace, Blueprint, Project, Action, Plan and Checkpoint

[Back to the README](../README.md) · [Português](pt-BR/model.md)

```
Workspace
└─ Blueprint            the scope
   └─ Project           groups actions inside the blueprint (optional; without a project, the action stays under "Geral"/"General")
      └─ Action         status, instructions (it used to be called "phase")
         ├─ Plan        title, description, done (it used to be called "task")
         └─ Checkpoint  name, done, evidence (it used to be called "milestone")
```

On the sheet, each project is a numbered section (01, 02…) with a blueprint-style header and its actions below it (it starts collapsed); actions are numbered per project (02.3 = project 2, action 3).

- The percentage of each action and blueprint comes only from the ticked plans and checkpoints.
- **No dates or durations:** blueprints carry no start, end, deadline, estimate or measured time. Actions run in the order they are written, and the sheet draws them as a staircase, each one starting where the previous one ends, with its progress inside and its checkpoints at the end of the bar.
- Everything the agent sends is a **proposal**: nothing changes in your data until you apply it on the **Plan** screen. Completing a plan requires **evidence** (the command or test that was run and its result), which becomes a comment on the plan.
- The app never sends work to anyone on its own: **you** start an action, either by typing `blueprint claude`, or with the **Start with Claude** button on the action's sheet. It asks for confirmation (it spends tokens), opens a new terminal in the blueprint's folder with Claude Code and the action's prompt as the argument of the new session (never through the monitor or the channel, and nothing goes to a session that is already open). Apart from that, the app only shows what the agents report: Claude on the sheet (one per agent and subagent, on the plan it works on; the finished one stays until you click it and read the message).
