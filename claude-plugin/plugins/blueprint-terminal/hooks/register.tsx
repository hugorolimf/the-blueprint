import { pequena, png } from "../assets/marca.mjs";

const visible = { plugin: "blueprint-terminal", key: "visible" } as const;
const image = { plugin: "blueprint-terminal", key: "image" } as const;
const estado = { plugin: "blueprint-terminal", key: "estado" } as const;
const pasta = { plugin: "blueprint-terminal", key: "pasta" } as const;
const id = "blueprint-marca";
const app = "http://127.0.0.1:3000/api/mcp/hook/status"; // a porta fixa que os hooks do plugin já usam

/** Pergunta ao app o estado do blueprint da pasta. Sem app, sem blueprint ou com erro: estado vazio e o painel volta a ser só a marca. */
async function atualizar($, cwd?: string) {
  let novo = { texto: "", linha: "" };
  try {
    if (cwd) await $.state.set(pasta, cwd);
    const dir = cwd ?? (await $.state.get(pasta)).value;
    const r = await $.http.fetch(app, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ cwd: dir }) });
    const j = r.ok ? JSON.parse(r.text) : {};
    if (j.vinculado) novo = { texto: String(j.texto), linha: String(j.linha) };
  } catch {}
  await $.state.set(estado, novo);
  return novo;
}
const feitos = (linha: string) => Number(/ (\d+)\/\d+$/.exec(linha)?.[1] ?? -1);

export function register(on) {
  on("session.start", async ($, e, next) => {
    const result = await next(e);
    if (!e.isInteractive || e.surface !== "terminal") return result;
    await $.command.register({ name: id, description: "Reabre a marca do The Blueprint" });
    const term = await $.env.get("TERM_PROGRAM");
    const version = await $.env.get("TERM_PROGRAM_VERSION");
    const kitty = await $.env.get("KITTY_WINDOW_ID");
    const iterm = term === "iTerm.app" && /^(?:[4-9]|3\.(?:[5-9]|[1-9]\d))/.test(version ?? "");
    await atualizar($, e.cwd);
    await $.state.set(image, !!kitty || term === "ghostty" || iterm);
    // Uma recarga do mod preserva o fechamento explícito da pessoa.
    if ((await $.state.get(visible)).value === undefined) await $.state.set(visible, true);
    return result;
  });

  on("command.run", { command: id }, async ($) => {
    await $.state.set(visible, true);
    return {};
  }).catch(($, e, next) => next(e));

  on("ui.render", { component: "AbovePrompt" }, async ($, e, next) => {
    if (e.surface !== "terminal" || e.props.hasSurvey) return next(e);
    if (!(await $.state.get(visible)).value) return next(e);
    return marca($, e);
  });

  on("turn.complete", async ($, e, next) => {
    const result = await next(e);
    if (e.agentId) return result;
    const antes = (await $.state.get(estado)).value?.linha ?? "";
    const { linha } = await atualizar($);
    const plano = linha && feitos(linha) > feitos(antes) && feitos(antes) >= 0;
    await $.ui.toast(linha ? `The Blueprint · ${plano ? "tarefa concluída · " : ""}${linha.replace(/^Blueprint · /, "")}` : "The Blueprint · resposta concluída");
    return result;
  });
}

/** A faixa sobre o prompt: marca, linha curta do blueprint e o botão que a fecha (o comando `blueprint-marca` a reabre). */
async function marca($, e) {
  const { Box, Text, Raster, Image, Button } = $.ui.resolve(e);
  const title = Text({ color: "cyan", bold: true, children: "The Blueprint" });
  const { linha } = (await $.state.get(estado)).value ?? { texto: "", linha: "" };
  if (e.props.bodyColumns < pequena.columns) return title;
  const picture = (await $.state.get(image)).value
    ? Image({ source: { png }, columns: pequena.columns, rows: pequena.rows, alt: "Marca do The Blueprint" })
    : Raster({ key: "marca", ...pequena });
  const largura = e.props.bodyColumns - 1;
  const linhas = linha.split("\n").filter(Boolean).map((l: string, i: number) => Text({ key: `l${i}`, children: l.slice(0, largura) }));
  const fechar = Button({ key: "fechar", label: "fechar ×", role: "dismiss", plain: true, dimColor: true, onPress: () => $.state.set(visible, false) });
  return Box({ flexDirection: "column", alignItems: "center", children: [title, picture, ...linhas, fechar] });
}
