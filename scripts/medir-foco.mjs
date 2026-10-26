// Mede o custo do modo foco contra a meta "zero trabalho em repouso" (Edge sem janela, por CDP, modelo de medir-animacoes.mjs): com um agente na folha de exemplo, a CPU da página parada
// com a visão fechada e aberta (mediana de fatias de 1 s), quantos timers e quadros o foco agenda em repouso (nenhum), animações da visão (nenhuma)
// e o custo de abrir/fechar a visão. Os números saem no console (e vão para referencias/relatorio-medicao.md).
// Uso: node scripts/medir-foco.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe, serve web/ da pasta; precisa do Microsoft Edge; ~25 s; banco de exemplo, nunca o vivo)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { dormir, exePadrao, ligarServidor, pastaDeDados, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dados = await pastaDeDados("blueprint-medir-foco", { BLUEPRINT_ASSETS_DIR: raiz });
const exemplo = JSON.parse(readFileSync(join(raiz, "data.exemplo.json"), "utf8"));
writeFileSync(join(dados.dir, "data.json"), JSON.stringify({ ...exemplo, usuario: { nome: "Teste" } }));
const depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", "--disable-features=CalculateNativeWinOcclusion", "--disable-backgrounding-occluded-windows", "--disable-renderer-backgrounding", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dados.dir, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
let servidor, mantemVisivel;
const limpar = () => { clearInterval(mantemVisivel); try { edge.kill(); } catch {} try { servidor?.kill(); } catch {} setTimeout(() => { try { rmSync(dados.dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const DELTA_CPU = Number(process.env.DELTA_CPU) || 0.6; // % de um núcleo que o foco pode somar à página parada (a linha de base varia de 1,0 a 1,7% entre execuções por causa do Claude animado na folha)
const TETO_LIGAR = Number(process.env.TETO_LIGAR) || 60; // ms para ligar ou desligar o foco (montar os cartões e deixar a folha inerte, pintura por software do Edge headless)
const JANELA = 5000;
try {
  servidor = await ligarServidor(exePadrao(), dados);
  const lista = (await (await fetch(dados.url + "/api/dados")).json()).blueprints;
  const blueprint = lista.find((p) => p.acoes?.some((f) => f.planos?.length)), acao = blueprint.acoes.find((f) => f.planos?.length), plano = acao.planos[0];
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Performance.enable");
  await cdp("Page.enable");
  await cdp("Emulation.setFocusEmulationEnabled", { enabled: true }); // sem isso o Edge headless oculta a aba depois de uns 45 s e nenhum relógio roda
  mantemVisivel = setInterval(() => cdp("Page.bringToFront"), 3000);
  // quem agenda timer ou quadro: conta, por origem na pilha, o que foi agendado por arquivo do foco (foco.js, foco-vista.js, foco-cartao.js, foco-logica.js)
  await cdp("Page.addScriptToEvaluateOnNewDocument", { source: `(() => { window.__foco = { agendas: 0 }; const de = () => /foco(-[a-z]+)?\\.js/.test(new Error().stack || ""); for (const nome of ["setTimeout", "setInterval", "requestAnimationFrame"]) { const o = window[nome]; window[nome] = function (...a) { if (de()) { window.__foco.agendas++; (window.__foco.pilhas ||= []).push(nome + " < " + new Error().stack.split(String.fromCharCode(10)).slice(2, 5).map((l) => l.trim().replace(/^at /, "")).join(" < ")); } return o.apply(this, a); }; } })();` });
  await cdp("Page.navigate", { url: `${dados.url}/#/p/${blueprint.id}` });
  await dormir(3000);
  assert(await js(`document.visibilityState === "visible" && !document.hidden`), "a aba do Edge precisa estar visível para medir");
  const metricas = async () => Object.fromEntries((await cdp("Performance.getMetrics")).result.metrics.map((m) => [m.name, m.value]));
  const mediana = (v) => [...v].sort((x, y) => x - y)[Math.floor(v.length / 2)];
  const medir = async (ms = JANELA) => {
    const fatias = []; let a = await metricas();
    for (let t = 0; t < ms; t += 1000) { await dormir(1000); const b = await metricas(), dt = b.Timestamp - a.Timestamp; fatias.push({ cpu: ((b.TaskDuration - a.TaskDuration) / dt) * 100, layouts: (b.LayoutCount - a.LayoutCount) / dt, recalc: (b.RecalcStyleCount - a.RecalcStyleCount) / dt }); a = b; }
    const f = (k) => +mediana(fatias.map((x) => x[k])).toFixed(2);
    return { cpu: f("cpu"), pico: +Math.max(...fatias.map((x) => x.cpu)).toFixed(2), layoutsPorS: f("layouts"), recalcPorS: f("recalc"), nos: Math.round(a.Nodes) };
  };
  const wsId = await js(`workspaceAtual.id`);
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; agentesLista = [${JSON.stringify({ agente: "Claude", estado: "escrevendo", blueprintId: blueprint.id, acaoId: acao.id, planoId: plano.id, workspaceId: wsId, em: new Date().toISOString(), mensagem: "medindo" })}]; agenteAgora = null; agentesAtualiza(); render()`);
  await dormir(1500);
  assert(await js(`!!document.querySelector(".robo-linha")`), "o agente tem um plano destacado");
  const animacoesDoFoco = () => js(`document.getAnimations().filter((a) => a.effect?.target?.closest?.("#foco-vista")).length`);
  const intervalos = () => js(`window.__foco.agendas`);

  const sem = await medir();
  console.log("visão fechada, um agente:", JSON.stringify(sem));
  await js(`document.getElementById("b-foco").click()`);
  await dormir(1500); // a transição de opacidade (.25 s) e a rolagem até o plano acabam
  assert(await js(`!!document.getElementById("foco-vista")`), "a visão do foco abriu");
  assert.equal(await animacoesDoFoco(), 0, "a visão do foco não tem animação");
  const agendasAntes = await intervalos();
  const com = await medir();
  const agendasDepois = await intervalos();
  console.log("visão aberta, um agente: ", JSON.stringify(com));
  assert.equal(agendasDepois, agendasAntes, `em repouso o foco não agenda timer nem quadro (${agendasAntes} -> ${agendasDepois})`);
  await js(`render()`); // sanidade do contador: um render() com a visão aberta agenda quadros do foco (só em resposta a um aviso)
  await dormir(200);
  assert((await intervalos()) - agendasDepois >= 1, "o contador enxerga o quadro que o foco agenda depois de um render()");
  assert(com.cpu - sem.cpu < DELTA_CPU, `o foco soma ${(com.cpu - sem.cpu).toFixed(2)}% de CPU à página parada (teto ${DELTA_CPU}%)`);
  assert(com.recalcPorS <= sem.recalcPorS + 0.5, `o foco não provoca recálculo de estilo em repouso (${sem.recalcPorS}/s -> ${com.recalcPorS}/s)`);

  // o custo de abrir e fechar a visão: um painel novo sobre a folha, que fica inerte
  const toggle = await js(`(async () => { const t = []; for (let i = 0; i < 20; i++) { const a = performance.now(); focoAlterna(); void document.body.offsetHeight; t.push(performance.now() - a); await new Promise((r) => setTimeout(r, 40)); } return t; })()`);
  const media = +(toggle.reduce((s, x) => s + x, 0) / toggle.length).toFixed(1), pior = +Math.max(...toggle).toFixed(1);
  console.log(`abrir/fechar a visão (20x, ${await js(`document.querySelectorAll("#desenho *").length`)} elementos na folha): média ${media} ms, pior ${pior} ms`);
  assert(media < TETO_LIGAR, `abrir/fechar a visão leva ${media} ms (teto ${TETO_LIGAR} ms)`);

  // o detalhe do agente (cartão grande com o histórico) também não deixa nada rodando em repouso
  await js(`focoAbre("Claude")`);
  await dormir(1500);
  const fixoAntes = await intervalos();
  const fixo = await medir(3000);
  console.log("detalhe do agente:     ", JSON.stringify(fixo));
  assert.equal(await intervalos(), fixoAntes, `com o detalhe do agente também não há timer em repouso: ${JSON.stringify((await js("window.__foco.pilhas")).slice(fixoAntes))}`);
  assert(fixo.cpu - sem.cpu < DELTA_CPU, `o detalhe do agente soma ${(fixo.cpu - sem.cpu).toFixed(2)}% de CPU (teto ${DELTA_CPU}%)`);
  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log(`medição do foco: ok (CPU parada ${sem.cpu}% sem foco, ${com.cpu}% com foco, ${fixo.cpu}% no detalhe; nenhum timer/quadro do foco em repouso)`);
} finally { limpar(); }
process.exit(0);
