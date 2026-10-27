// No navegador de verdade: as figuras do exemplo param de pedir quadros (requestAnimationFrame) fora da tela e com movimento reduzido (à vista elas passeiam de propósito),
// e o tempo de CPU da página em repouso é medido pelo DevTools (Performance.getMetrics, TaskDuration). Sem Edge/Chrome o teste avisa e sai sem falhar.
import assert from "node:assert/strict";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { montar } from "./montar.mjs";
import { exemplo } from "./teste-comum.mjs";
import { abrirNavegador, achaNavegador, dormir } from "./teste-cdp.mjs";

const navegador = achaNavegador();
if (!navegador) { console.log("repouso.test.mjs: sem Edge/Chrome, pulado"); process.exit(0); }
const arquivo = join(mkdtempSync(join(tmpdir(), "ap-repouso-")), "pagina.html");
writeFileSync(arquivo, montar(exemplo()));

const nav = await abrirNavegador(navegador);
const contar = "window.__quadros = 0; const __raf = window.requestAnimationFrame.bind(window); window.requestAnimationFrame = (f) => { window.__quadros++; return __raf(f); };";
const quadrosEm = async (ms) => { await nav.ev("window.__quadros = 0"); await dormir(ms); return nav.ev("window.__quadros"); };
const cpu = async () => (await nav.cmd("Performance.getMetrics")).metrics.find((m) => m.name === "TaskDuration").value;
try {
  await nav.cmd("Page.enable");
  await nav.cmd("Performance.enable");
  await nav.cmd("Page.addScriptToEvaluateOnNewDocument", { source: contar });
  await nav.ir(pathToFileURL(arquivo).href);
  await dormir(1500); // a entrada das figuras assenta

  const palco = await nav.ev(`(() => { const r = document.querySelector("[data-figura]").closest(".iso-plate").getBoundingClientRect(); return JSON.stringify({ x: r.left + r.width / 2, y: r.top + r.height / 2 }); })()`);
  const { x, y } = JSON.parse(palco);
  await nav.ev("window.__quadros = 0");
  for (let i = 0; i < 6; i++) { await nav.mouse(x - 60 + i * 24, y - 30 + i * 12); await dormir(40); }
  assert.ok(await nav.ev("window.__quadros") > 0, "o laço anda quando a figura à vista recebe o mouse (senão o teste não prova nada)");

  // fora da tela: rola para um espaço sem figura e mexe o mouse; nenhum quadro
  await nav.ev(`(() => { const d = document.createElement("div"); d.style.height = "6000px"; document.body.append(d); scrollTo(0, document.documentElement.scrollHeight); })()`);
  await dormir(500);
  await nav.ev("window.__quadros = 0");
  for (let i = 0; i < 6; i++) { await nav.mouse(200 + i * 30, 300); await dormir(40); }
  const fora = await quadrosEm(1000);
  assert.equal(fora, 0, `figuras fora da tela: ${fora} quadros em 1 s`);

  const antes = await cpu();
  await dormir(2000);
  const gasto = (await cpu()) - antes;
  assert.ok(gasto < 0.1, `CPU da página em repouso: ${(gasto * 1000).toFixed(1)} ms em 2 s`);

  // movimento reduzido: à vista, o passeio das figuras não roda e o laço termina sozinho
  await nav.ir(pathToFileURL(arquivo).href, { reduzido: true });
  await dormir(1000);
  const calma = await quadrosEm(1000);
  assert.equal(calma, 0, `movimento reduzido, figuras à vista: ${calma} quadros em 1 s`);
  console.log(`repouso.test.mjs: ok (0 quadros fora da tela e com movimento reduzido; CPU em repouso ${(gasto * 1000).toFixed(1)} ms em 2 s)`);
} finally { nav.fechar(); setTimeout(() => process.exit(), 3000); }
