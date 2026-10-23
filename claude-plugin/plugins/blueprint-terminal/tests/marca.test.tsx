import { describe, expect, test } from "claude-code/testing";
import { register } from "../hooks/register";

describe("marca Blueprint", () => {
  for (const columns of [100, 160]) {
    test(`Raster e superfície terminal em ${columns} colunas`, async ($, on) => {
      let abriu = 0;
      on("session.start", ($, e) => ({ cwd: e.cwd }));
      on("ui.open", () => { abriu++; return { value: undefined }; });
      on("env.get", () => ({ value: undefined }));
      on("command.register", () => ({ value: undefined }));
      on("ui.panes", () => ({ value: [] }));
      await $.session.start({ surface: "terminal", isInteractive: true, cwd: "/work" });
      expect(abriu).toBe(1);
      const component = columns < 144 ? "AbovePrompt" : "Pane";
      const ui = await $.ui.mount({
        plugin: "blueprint-terminal", surface: "terminal", component,
        requestId: component === "Pane" ? "blueprint-marca" : undefined,
        viewport: { columns, rows: 50 },
        props: { hasSurvey: false, bodyColumns: columns < 144 ? columns : 38,
          title: "The Blueprint", isFocused: false, placement: "dock" },
      } as any);
      expect(await ui.find({ type: "Text", text: /The Blueprint/ })).toBeDefined();
      expect(await ui.find({ type: "Raster" })).toBeDefined();
      await ui.unmount();
    });
  }
  test("sem o app aberto, os eventos notificam de forma genérica", async ($, on) => {
    const avisos: string[] = [];
    on("turn.complete", () => ({ text: "" }));
    on("ui.toast", ($, e) => { avisos.push(e.text); return { value: undefined }; });
    await $.turn.complete({ reason: "answer", answer: "feito", durationMs: 1 } as any);
    expect(avisos).toEqual(["The Blueprint · resposta concluída"]);
    await $.turn.complete({ reason: "answer", answer: "feito", durationMs: 1, agentId: "auxiliar" } as any);
    expect(avisos.length).toBe(1);
    await $.classic.Notification({ hook_event_name: "Notification", cwd: "/work", session_id: "test", transcript_path: "", message: "Preciso da sua resposta", notification_type: "permission_prompt" });
    expect(avisos.length).toBe(1); // o aviso padrão (hook OSC 9) basta: o mod não o repete em outro canto
  });
  test("fechar preserva a escolha durante recarga e comando reabre", async () => {
    const handlers = new Map<string, Function>();
    register((event, matcher, hook) => {
      handlers.set(event, hook ?? matcher);
      return { catch() {} };
    });
    let abriu = 0;
    const state = new Map<string, unknown>();
    const $ = {
      state: { get: async ({ key }) => ({ value: state.get(key) }), set: async ({ key }, value) => { state.set(key, value); } },
      env: { get: async () => undefined }, command: { register: async () => {} },
      ui: { open: async () => { abriu++; } },
    };
    const start = () => handlers.get("session.start")!($, { surface: "terminal", isInteractive: true }, async () => ({}));
    await start();
    await handlers.get("ui.close")!($, { id: "blueprint-marca", origin: { kind: "person" } }, async () => ({}));
    await start();
    expect(abriu).toBe(1);
    await handlers.get("command.run")!($);
    expect(abriu).toBe(2);
  });
  test("kitty usa PNG em vez de Raster", async ($, on) => {
    on("session.start", ($, e) => ({ cwd: e.cwd }));
    on("ui.open", () => ({ value: undefined }));
    on("env.get", ($, e) => ({ value: e.name === "KITTY_WINDOW_ID" ? "1" : undefined }));
    on("command.register", () => ({ value: undefined }));
    await $.session.start({ surface: "terminal", isInteractive: true, cwd: "/work" });
    const ui = await $.ui.mount({ plugin: "blueprint-terminal", surface: "terminal", component: "Pane", requestId: "blueprint-marca", props: { bodyColumns: 38, title: "The Blueprint", isFocused: false, placement: "dock" }, viewport: { columns: 160, rows: 50 } } as any);
    expect(await ui.find({ type: "Image" })).toBeDefined();
    await ui.unmount();
  });

  const estado = (feitos: number) => ({ vinculado: true, texto: `The Blueprint · Integração
Ação 2.5: Notificações (28/10 a 29/10) · ${feitos}/5 planos
  [x] um
Próximo checkpoint: 29/10 · Revisão`, linha: `Blueprint · ação 2.5 · Notificações · ${feitos}/5` });
  test("o painel mostra o estado do blueprint e o toast avisa tarefa concluída", async ($, on) => {
    let feitos = 2;
    const avisos: string[] = [];
    on("session.start", ($, e) => ({ cwd: e.cwd }));
    on("ui.open", () => ({ value: undefined }));
    on("env.get", () => ({ value: undefined }));
    on("command.register", () => ({ value: undefined }));
    on("http.fetch", () => ({ value: { status: 200, ok: true, headers: {}, text: JSON.stringify(estado(feitos)) } }));
    on("turn.complete", () => ({ text: "" }));
    on("ui.toast", ($, e) => { avisos.push(e.text); return { value: undefined }; });
    await $.session.start({ surface: "terminal", isInteractive: true, cwd: "/work" });
    const ui = await $.ui.mount({ plugin: "blueprint-terminal", surface: "terminal", component: "Pane", requestId: "blueprint-marca", props: { bodyColumns: 38, title: "The Blueprint", isFocused: false, placement: "dock" }, viewport: { columns: 160, rows: 50 } } as any);
    expect(await ui.find({ type: "Text", text: /Ação 2.5/ })).toBeDefined();
    await ui.unmount();
    feitos = 3;
    await $.turn.complete({ reason: "answer", answer: "feito", durationMs: 1 } as any);
    expect(avisos).toEqual(["The Blueprint · tarefa concluída · ação 2.5 · Notificações · 3/5"]);
  });
});
