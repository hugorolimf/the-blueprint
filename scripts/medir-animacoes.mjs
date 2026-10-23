// Mede o custo das animações do Claude na folha (Edge sem janela, por CDP) contra a meta "zero trabalho em repouso": CPU da página parada, com 5 agentes animados (+ o Claude
// esperando), com a aba oculta, com a figura fora da tela e com prefers-reduced-motion; e confere que cada figura tem no máximo UM relógio (setInterval) e que nenhum sobra ao
// trocar de pose ou remover a figura. Os números saem no console (e vão para referencias/relatorio-medicao.md).
// Uso: node scripts/medir-animacoes.mjs [blueprint.exe]   (precisa do Microsoft Edge; o servidor sobe com a cópia dos dados de exemplo, nunca o banco vivo)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const [porta, depuracao] = [await livre(), await livre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-medir-anim-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos"), BLUEPRINT_ASSETS_DIR: raiz };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", "--disable-features=CalculateNativeWinOcclusion", "--disable-backgrounding-occluded-windows", "--disable-renderer-backgrounding", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);

// limites (da meta de desempenho do CLAUDE.md: zero trabalho em repouso)
// % de um núcleo na thread principal da página (mediana de fatias de 1 s). Regime medido: ~0,04% parada, ~0,4% com uma figura, ~3,5% com seis; os tetos deixam folga para a máquina carregada.
const TETO_REPOUSO = Number(process.env.TETO_REPOUSO) || 0.5, TETO_UMA = Number(process.env.TETO_UMA) || 2, TETO_ANIMADO = Number(process.env.TETO_ANIMADO) || 10;
const TETO_ANIMACOES = Number(process.env.TETO_ANIMACOES) || 700, TETO_TAREFA = Number(process.env.TETO_TAREFA) || 250, TETO_ABRIR = Number(process.env.TETO_ABRIR) || 15; // % médio de CPU nos 12 cliques (inclui a espera do Claude e a pintura por software do Edge headless) // (7) abrir/fechar: animações simultâneas e tarefa mais longa tolerada (ms)
const JANELA = 5000;
const relatorio = {};
let mantemVisivel;
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.length));
  const acao = blueprint.acoes.find((f) => f.planos?.length), plano = acao.planos[0];
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
  // o Edge headless passa a página para "oculta" depois de uns 45 s sem foco (aí nenhum relógio roda, e com razão): foco emulado e bringToFront mantêm a aba visível durante a medição
  await cdp("Emulation.setFocusEmulationEnabled", { enabled: true });
  mantemVisivel = setInterval(() => cdp("Page.bringToFront"), 3000);
  // conta os setInterval vivos da página (id -> período), para achar relógio órfão
  await cdp("Page.addScriptToEvaluateOnNewDocument", { source: `(() => { const ativos = new Map(), si = window.setInterval, ci = window.clearInterval; window.__iv = ativos; window.setInterval = (f, ms, ...a) => { const id = si(f, ms, ...a); ativos.set(id, ms); return id; }; window.clearInterval = (id) => { ativos.delete(id); return ci(id); }; })();` });
  await cdp("Page.navigate", { url: `${url}/#/c/sm` }); // a lista de blueprints: nenhuma figura (nem a de espera), o repouso de verdade
  await dormir(3000);

  // o Windows pode marcar a janela headless como oculta (oclusão): aí nenhum relógio roda, e com razão; o teste não vale
  const visivel = () => js(`document.visibilityState === "visible" && !document.hidden`);
  assert(await visivel(), "a aba do Edge precisa estar visível para medir");
  const metricas = async () => Object.fromEntries((await cdp("Performance.getMetrics")).result.metrics.map((m) => [m.name, m.value]));
  // fatias de 1 s: o custo contínuo é a MEDIANA das fatias (um evento isolado, como um render() da folha, aparece só no `pico`)
  const mediana = (v) => [...v].sort((x, y) => x - y)[Math.floor(v.length / 2)];
  const medir = async (ms = JANELA) => {
    await eventos();
    const fatias = [], ini = await metricas(); let a = ini;
    for (let t = 0; t < ms; t += 1000) {
      await dormir(Math.min(1000, ms - t)); const b = await metricas(), dt = b.Timestamp - a.Timestamp;
      fatias.push({ cpu: ((b.TaskDuration - a.TaskDuration) / dt) * 100, script: ((b.ScriptDuration - a.ScriptDuration) / dt) * 100, layouts: (b.LayoutCount - a.LayoutCount) / dt, recalc: (b.RecalcStyleCount - a.RecalcStyleCount) / dt }); a = b;
    }
    const dt = a.Timestamp - ini.Timestamp, f = (k) => +mediana(fatias.map((x) => x[k])).toFixed(2);
    const ev = await eventos();
    return { eventos: `render ${ev.render}, camada ${ev.camada}, fetch ${ev.fetch}, tarefas longas [${ev.longas}]${ev.pilhas?.length ? ", pilhas " + JSON.stringify(ev.pilhas) : ""}`, cpu: f("cpu"), pico: +Math.max(...fatias.map((x) => x.cpu)).toFixed(2), media: +(((a.TaskDuration - ini.TaskDuration) / dt) * 100).toFixed(2), script: f("script"), layoutsPorS: +f("layouts").toFixed(1), recalcPorS: +f("recalc").toFixed(1), heapKB: Math.round(a.JSHeapUsedSize / 1024), nos: Math.round(a.Nodes) };
  };
  const intervalos = () => js(`window.__iv.size`);
  const relogios = () => js(`[...claudeFiguras.values()].filter((r) => r.vida).length`);
  const relogiosAndar = () => js(`[...claudeFiguras.values()].filter((r) => r.relogio).length`);
  const animadas = () => js(`[...document.querySelectorAll("#claude-camada .claude-fig:not([data-saindo])")].filter((e) => e.querySelectorAll("span[data-q]").length > 1).length`);
  const figuras = () => js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-saindo])").length`);
  const animacoes = () => js(`document.getAnimations().length`);
  // quantas vezes a camada reconstruiu SVG (childList) durante a janela: o ciclo só deve trocar o atributo `hidden`
  await js(`window.__recons = 0; new MutationObserver((ms) => { window.__recons += ms.filter((m) => m.type === "childList").length; }).observe(document.getElementById("claude-camada"), { childList: true, subtree: true })`);
  const recons = () => js(`window.__recons`);
  // o que acontece dentro da janela: quantos render() da folha, atualizações da camada, fetch e tarefas longas (>50 ms) houve (explica os picos)
  await js(`window.__c = { render: 0, camada: 0, fetch: 0, longas: [] }; for (const [k, f] of [["render", "render"], ["camada", "claudeCamadaAtualiza"]]) { const o = window[f]; window[f] = (...a) => { __c[k]++; if (k === "render") (__c.pilhas ||= []).push(new Error().stack.split(String.fromCharCode(10)).slice(2, 5).map((l) => l.trim()).join(" < ")); return o(...a); }; } const fe = window.fetch; window.fetch = (...a) => { __c.fetch++; return fe(...a); }; new PerformanceObserver((l) => l.getEntries().forEach((e) => __c.longas.push(Math.round(e.duration)))).observe({ type: "longtask", buffered: true });`);
  const eventos = async () => { const e = await js(`(() => { const x = { ...__c, longas: __c.longas.slice(), pilhas: (__c.pilhas || []).slice() }; __c.render = __c.camada = __c.fetch = 0; __c.longas = []; __c.pilhas = []; return x; })()`); return e; };
  const wsId = await js(`workspaceAtual.id`);
  const agente = (nome, papel, estado) => ({ agente: nome, ...(papel ? { papel } : {}), estado, blueprintId: blueprint.id, acaoId: acao.id, planoId: plano.id, workspaceId: wsId, em: new Date().toISOString(), mensagem: `${nome}: ${estado}` });
  const poe = async (lista, espera = 1500) => { await js(`agentesLista = ${JSON.stringify(lista)}; agenteAgora = null; agentesAtualiza(); render()`); await dormir(espera); };
  const CINCO = [agente("Claude", null, "escrevendo"), agente("Explore a1b2", "explorador", "aguardando"), agente("fork c3d4", "construtor", "bloqueado"), agente("code-reviewer e5f6", "revisor", "concluido"), agente("general g7h8", "construtor", "ocioso")];

  // (1) sem figura nenhuma (lista de blueprints): a folha parada não faz trabalho
  const base = await intervalos();
  await dormir(500);
  const repouso = await medir();
  relatorio.repouso = { ...repouso, intervalosDaPagina: base, animacoes: await animacoes(), relogios: await relogios(), figuras: await figuras() };
  console.log("(1) sem figura:", JSON.stringify(relatorio.repouso));
  assert(repouso.cpu < TETO_REPOUSO, `parada, a página gasta ${repouso.cpu}% de um núcleo (teto ${TETO_REPOUSO}%)`);
  assert.equal(await animacoes(), 0, "sem figura nada anima");
  assert.equal(await relogios(), 0, "sem figura nenhum relógio");
  assert.equal(await figuras(), 0, "a lista de blueprints não tem figura de espera");

  // (1b) dentro do blueprint, sem agente: só o Claude esperando na próxima ação (uma figura animada)
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(3000);
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render()`);
  await dormir(1500);
  const soEspera = await medir();
  relatorio.soEspera = { ...soEspera, figuras: await figuras(), relogios: await relogios() };
  console.log("(1b) só o Claude esperando:", JSON.stringify(relatorio.soEspera));
  assert.equal(await relogios(), 1, "só a figura de espera tem relógio");
  assert(soEspera.cpu < TETO_UMA, `uma figura animada: ${soEspera.cpu}% (teto ${TETO_UMA}%)`);

  // (2) cinco agentes animados à vista (+ o Claude esperando na próxima ação)
  await poe(CINCO);
  const figs = await figuras(), anim = await animadas(), rel = await relogios();
  assert(figs >= 5, `as figuras aparecem (${figs})`);
  assert.equal(rel, anim, "um relógio por figura animada, nem mais nem menos");
  assert.equal((await intervalos()) - base, rel, "nenhum setInterval além dos relógios das figuras");
  assert.equal(await relogiosAndar(), 0, "ninguém andando: sem relógio de passos");
  const r0 = await recons();
  const animado = await medir();
  const reconstruiu = (await recons()) - r0;
  relatorio.animado = { ...animado, figuras: figs, figurasAnimadas: anim, relogios: rel, reconstrucoesDoSvgNaJanela: reconstruiu, memoriaExtraKB: animado.heapKB - repouso.heapKB, nosExtras: animado.nos - repouso.nos, htmlDaCamadaKB: Math.round((await js(`document.getElementById("claude-camada").innerHTML.length`)) / 1024) };
  console.log("(2) com agentes:", JSON.stringify(relatorio.animado));
  assert(animado.cpu < TETO_ANIMADO, `animadas, a página gasta ${animado.cpu}% de um núcleo (teto ${TETO_ANIMADO}%)`);
  assert.equal(reconstruiu, 0, "o ciclo só troca `hidden`: o SVG não é refeito a cada quadro");

  // (3) aba oculta: os relógios param; ao voltar, voltam
  await js(`Object.defineProperty(document, "hidden", { get: () => true, configurable: true }); document.dispatchEvent(new Event("visibilitychange"))`);
  await dormir(300);
  const ocultaRel = await relogios();
  const oculta = await medir(3000);
  relatorio.abaOculta = { ...oculta, relogios: ocultaRel, intervalosAlemDaPagina: (await intervalos()) - base };
  console.log("(3) aba oculta:", JSON.stringify(relatorio.abaOculta));
  assert.equal(ocultaRel, 0, "aba oculta: nenhum relógio");
  assert.equal((await intervalos()) - base, 0, "aba oculta: nenhum setInterval das figuras");
  assert(oculta.cpu < TETO_REPOUSO, `aba oculta: ${oculta.cpu}% (teto ${TETO_REPOUSO}%)`);
  await js(`delete document.hidden; document.dispatchEvent(new Event("visibilitychange"))`);
  await dormir(600);
  assert.equal(await relogios(), await animadas(), "aba de volta: os relógios voltam, um por figura");

  // (4) uma figura fora da tela (clipada pela camada): o relógio dela para; ao voltar, volta
  await js(`(() => { const r = [...claudeFiguras.values()].find((x) => x.dados && x.vida); window.__fora = r; window.__ant = r.el.style.transform; r.el.style.transform = "translate(-900px,-900px)"; })()`);
  await dormir(500);
  const aposFora = await relogios();
  assert.equal(aposFora, (await animadas()) - 1, "uma figura fora da tela: o relógio dela para e os outros seguem");
  assert.equal(await js(`!!window.__fora.vida`), false, "o relógio da figura fora da tela é zero");
  await js(`window.__fora.el.style.transform = window.__ant`);
  await dormir(500);
  assert.equal(await relogios(), await animadas(), "ao voltar para a tela o relógio volta");
  relatorio.foraDaTela = { relogiosComUmaFora: aposFora, relogiosDepois: await relogios() };

  // (5) prefers-reduced-motion: sem relógio e sem animação
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await dormir(600);
  const reduzido = await medir(3000);
  relatorio.movimentoReduzido = { ...reduzido, relogios: await relogios(), animacoes: await animacoes(), figurasComVariosQuadros: await animadas() };
  console.log("(5) movimento reduzido:", JSON.stringify(relatorio.movimentoReduzido));
  assert.equal(await relogios(), 0, "movimento reduzido: nenhum relógio");
  assert.equal(await animadas(), 0, "movimento reduzido: um quadro só por figura");
  assert.equal(await animacoes(), 0, "movimento reduzido: nada animando");
  assert((await intervalos()) - base === 0, "movimento reduzido: nenhum setInterval das figuras");
  assert(reduzido.cpu < TETO_REPOUSO, `movimento reduzido: ${reduzido.cpu}% (teto ${TETO_REPOUSO}%)`);
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "no-preference" }] });
  await dormir(600);
  assert.equal(await relogios(), await animadas(), "sem a preferência, os relógios voltam");

  // (6) trocar de pose e remover figuras não deixa relógio órfão
  const estados = ["lendo", "testando", "aguardando", "bloqueado", "concluido", "escrevendo", "ocioso"];
  let maximoOrfao = 0;
  for (let rodada = 0; rodada < 8; rodada++) {
    await poe(CINCO.map((a, i) => ({ ...a, estado: estados[(rodada + i) % estados.length], em: new Date().toISOString() })), 2100); // dá tempo de as figuras que mudaram de lugar chegarem (o relógio de passos acaba)
    maximoOrfao = Math.max(maximoOrfao, (await intervalos()) - base - (await relogios()) - (await relogiosAndar())); // o que sobra além dos relógios de quadros e de passos
    const detalhe = await js(`JSON.stringify({ iv: [...window.__iv.values()], figuras: [...claudeFiguras].map(([n, r]) => [n, !!r.vida, !!r.relogio, !!r.anim, r.dados?.f.pose ?? null, r.visivel]) })`);
    assert.equal((await intervalos()) - base, (await relogios()) + (await relogiosAndar()), `rodada ${rodada}: todo setInterval pertence a uma figura viva (${detalhe})`);
    assert(await relogios() <= (await figuras()), `rodada ${rodada}: no máximo um relógio por figura`);
  }
  await poe([], 1200);
  relatorio.limpeza = { figurasNaFolha: await figuras(), registrosNoMapa: await js(`claudeFiguras.size`), intervalosAlemDaPagina: (await intervalos()) - base, maiorOrfaoNasTrocas: maximoOrfao };
  console.log("(6) trocas e remoção:", JSON.stringify(relatorio.limpeza));
  assert(await visivel(), "a aba continuou visível até o fim");
  // sem agente sobra só o Claude esperando (figura da interface): uma figura, um relógio, e mais nada
  assert.equal(await js(`claudeFiguras.size`), 1, "sem agentes o mapa fica só com a figura de espera");
  assert.equal(await js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])").length`), 0, "e nenhuma figura de agente");
  assert.equal((await intervalos()) - base, 1, "nenhum relógio sobra além do da espera");
  const depois = await medir(3000);
  relatorio.depoisDeLimpar = depois;
  assert(depois.cpu < TETO_UMA, `de volta só com a espera: ${depois.cpu}% (teto ${TETO_UMA}%)`);
  // (7) abrir e fechar um blueprint grande (ação 1.13): 45 ações com planos e checkpoints, seis ciclos de recolher e expandir; mede o pico de cada clique
  await js(`(() => { const b = D.blueprints.find((x) => x.id === ${JSON.stringify(blueprint.id)}); const molde = b.acoes.find((f) => f.planos?.length); b.acoes = Array.from({ length: 45 }, (_, i) => { const f = JSON.parse(JSON.stringify(molde)); f.id = "g" + i; f.planos.forEach((t, j) => { t.id = "g" + i + "p" + j; }); (f.checkpoints || []).forEach((m, j) => { m.id = "g" + i + "m" + j; }); return f; }); abertos[b.id] = true; render(); })()`);
  await dormir(2500);
  const grande = await js(`document.querySelectorAll("svg.desenho .chk").length`);
  assert(grande > 100, `o blueprint grande tem planos e checkpoints à vista (${grande})`);
  await eventos();
  const m0 = await metricas();
  const ciclos = await js(`(async () => { const id = ${JSON.stringify(blueprint.id)}, dorme = (ms) => new Promise((r) => setTimeout(r, ms)), saida = []; for (let i = 0; i < 12; i++) { const t0 = performance.now(); alternar(id); const sync = performance.now() - t0; await dorme(120); saida.push({ acao: i % 2 ? "expandir" : "recolher", sincronoMs: +sync.toFixed(1), animacoes: document.getAnimations().length }); await dorme(2300); } return saida; })()`);
  const m1 = await metricas(), ev7 = await eventos();
  const picoAnim = Math.max(...ciclos.map((c) => c.animacoes)), picoSync = Math.max(...ciclos.map((c) => c.sincronoMs));
  relatorio.abrirEFechar = { planosENaFolha: grande, cliques: ciclos.length, picoDeAnimacoes: picoAnim, maiorTrechoSincronoMs: picoSync, tarefasLongasMs: ev7.longas, cpuMedia: +(((m1.TaskDuration - m0.TaskDuration) / (m1.Timestamp - m0.Timestamp)) * 100).toFixed(2), animacoesNoFim: await animacoes() };
  console.log("(7) abrir e fechar, 45 ações:", JSON.stringify(relatorio.abrirEFechar));
  assert(picoAnim <= TETO_ANIMACOES, `no máximo ${TETO_ANIMACOES} animações de uma vez (houve ${picoAnim})`);
  assert(relatorio.abrirEFechar.cpuMedia < TETO_ABRIR, `abrir e fechar: ${relatorio.abrirEFechar.cpuMedia}% de um núcleo em média (teto ${TETO_ABRIR}%)`);
  assert(ev7.longas.every((ms) => ms < TETO_TAREFA), `nenhuma tarefa longa passa de ${TETO_TAREFA} ms: [${ev7.longas}]`);
  assert.equal(await animacoes(), 0, "depois de abrir e fechar nenhuma animação sobra");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("RELATORIO " + JSON.stringify(relatorio));
  console.log(`animações do Claude: ok (repouso ${repouso.cpu}%, só a espera ${soEspera.cpu}%, ${figs} figuras animadas ${animado.cpu}%, aba oculta ${oculta.cpu}%, movimento reduzido ${reduzido.cpu}%; um relógio por figura, nenhum órfão)`);
} finally { clearInterval(mantemVisivel); limpar(); }
