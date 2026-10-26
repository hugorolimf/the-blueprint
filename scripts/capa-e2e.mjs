// Confere no navegador de verdade (Edge sem janela) o cartão do blueprint na lista quando tem muita coisa: com ~100 ações a barra vira um bloco por situação
// (concluídas | em andamento | a fazer) e os checkpoints que se tocam viram um losango só; um blueprint pequeno continua com uma célula por ação.
// Uso: node scripts/capa-e2e.mjs [exe] [pasta-com-web]   (precisa do Edge; serve web/ da pasta, sem recompilar). CAPA_CAPTURA=<arquivo.png> grava a captura.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const [porta, depuracao] = [await portaLivre(), await portaLivre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-capa-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  let pagina;
  for (let i = 0; i < 100 && !pagina; i++) { try { pagina = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(pagina.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 1500)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Page.enable");
  await cdp("Page.navigate", { url: `${url}/?idioma=pt-BR#/c/sm` });
  await dormir(3000);
  // por cartão: quantas células de ação (rect.ln dentro da barra) e quantos losangos de checkpoint
  const medida = () => js(`[...document.querySelectorAll("[data-goto].alvo")].map((g) => ({ id: g.dataset.goto, celulas: g.querySelectorAll("rect.ln").length, losangos: g.querySelectorAll("path.dm").length }))`);
  const antes = await medida();
  assert(antes.length >= 2 && antes.every((c) => c.celulas > 0), "a lista desenha os cartões: " + JSON.stringify(antes));
  const alvo = antes[0].id;
  // 95 ações misturadas: 45 concluídas, 4 em andamento, o resto a fazer; 2 checkpoints por ação, em datas que se sobrepõem
  await js(`(() => {
    const p = D.blueprints.find((x) => x.id === ${JSON.stringify(alvo)});
    for (let i = 0; i < 95; i++) { const st = i < 45 ? "concluido" : i < 49 ? "andamento" : "planejado", d = 1 + (i % 28); p.acoes.push({ id: "g" + i, nome: "Ação gerada " + i, inicio: "2026-11-" + String(d).padStart(2, "0"), fim: "2026-12-" + String(d).padStart(2, "0"), status: st,
      planos: [0, 1, 2].map((k) => ({ id: "gp" + i + "_" + k, texto: "Plano " + k, descricao: "d", feito: st === "concluido" || (st === "andamento" && k === 0) })), checkpoints: [0, 1].map((k) => ({ id: "gm" + i + "_" + k, nome: "Marco " + k, data: "2026-11-" + String(1 + ((i + k * 3) % 28)).padStart(2, "0"), feito: st === "concluido" })) }); }
    render();
  })()`);
  await dormir(800);
  const depois = await medida();
  const grande = depois.find((c) => c.id === alvo), outros = depois.filter((c) => c.id !== alvo);
  assert(grande.celulas <= 3, "95 ações viram no máximo um bloco por situação: " + JSON.stringify(grande));
  assert(grande.losangos <= 40, "os losangos que se tocam foram juntados: " + JSON.stringify(grande));
  assert.deepEqual(outros, antes.filter((c) => c.id !== alvo), "os cartões pequenos ficam como eram (uma célula por ação)");
  const dicas = await js(`[...document.querySelectorAll('[data-goto="${alvo}"] title')].map((t) => t.textContent).filter((t) => /ações? (concluída|em andamento|a fazer)/.test(t))`);
  assert(dicas.some((t) => /concluídas/.test(t)) && dicas.some((t) => /em andamento/.test(t)) && dicas.some((t) => /a fazer/.test(t)), "cada bloco diz quantas ações juntou: " + JSON.stringify(dicas));
  if (process.env.CAPA_CAPTURA) writeFileSync(process.env.CAPA_CAPTURA, Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log("Capa: " + JSON.stringify(grande) + " (95 ações agrupadas por situação, losangos juntados, cartões pequenos iguais), sem erro na página");
} finally { limpar(); }
process.exit(0);
