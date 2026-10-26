# Action prototypes

When the user asks for a **prototype** of an action (a screen, a dashboard, a figure that shows the idea), generate it and attach it with `anexar_prototipo(workspace_id, blueprint_id, acao_id, titulo, html)`. The app shows it in a viewer next to the action: a screen icon on the blueprint sheet, or "View prototype" in the action's right-click menu.

- The `html` is **a single, self-contained HTML document**: CSS, SVG and JavaScript inline. No outside images or libraries and no network calls: the viewer runs isolated (sandbox, no network, no access to the app) and blocks whatever comes from outside. 2 MB maximum.
- For figures and visual explanations, **use the anatomy skill** (isometric SVG figures, with a title, a hint and calm motion) and deliver the result already embedded in an HTML file. For screens, use plain HTML and CSS.
- Use sample data, never the user's real data; make it clear in the prototype itself that it is a simulation.
- To **revise**, call again with the `prototipo_id` (it comes in `ler_blueprint` with `acao_id`, under `prototipos`): it is replaced, not duplicated. Without `prototipo_id` it becomes a new prototype of the action.
- The prototype does not count toward progress and does not change plans or checkpoints; it illustrates the action's instructions.
