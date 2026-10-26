// Confere o agente na folha no navegador de verdade (Edge sem janela): progresso com plano leva a figura do Claude (camada #claude-camada) e a linha realçada ao plano, a faixa mostra a última fala,
// concluído e bloqueio mudam a cor da faixa, o cartão do blueprint na lista ganha a figura e ela repousa (parada, no cabeçalho) depois de 10 minutos sem notícia, em vez de sumir. Captura em target/robo-folha.png.
// Uso: node scripts/robo-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-robo-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.some((t) => !t.feito)));
  const acao = blueprint.acoes.find((f) => f.planos?.some((t) => !t.feito)), plano = acao.planos.find((t) => !t.feito);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  const aviso = (tipo, corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, workspaceId: "sm", blueprintId: blueprint.id, ...corpo }) });

  // sem notícia, nada de robô
  await cdp("Page.navigate", { url: `${url}/#/c/sm` });
  await dormir(2500);
  assert.equal(await js(`document.getElementById("robo-agente").hidden`), true, "sem evento o aviso fica escondido");

  // progresso com plano: robô + linha realçada ao lado do plano, faixa com a última fala
  await aviso("progresso", { texto: "Escrevendo o teste do robô", acaoId: acao.id, planoId: plano.id });
  await dormir(1200); // chega por SSE
  assert.equal(await js(`document.getElementById("robo-agente").hidden`), false, "a faixa aparece");
  assert((await js(`document.getElementById("robo-agente").textContent`)).includes("Escrevendo o teste do robô"), "a faixa traz a última fala");
  assert(await js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])").length`) >= 1, "o cartão do blueprint na lista ganha a figura do Claude");
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(1500);
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render()`);
  await dormir(500);
  assert.equal(await js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])").length`), 1, "uma figura só na folha");
  assert.equal(await js(`document.querySelectorAll(".robo-linha").length`), 1, "a linha do plano fica realçada");
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "robo-folha.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));

  // concluído: verde; bloqueio: vermelho
  await aviso("concluir", { texto: "Concluí com evidência", acaoId: acao.id, planoId: plano.id, evidencia: "teste" });
  await dormir(1200);
  assert(await js(`!!document.querySelector("#robo-agente.ok")`), "concluído deixa a faixa na cor de ok");
  await aviso("bloqueio", { texto: "Travei: falta acesso" });
  await dormir(1200);
  assert(await js(`!!document.querySelector("#robo-agente.erro")`), "bloqueio deixa a faixa vermelha");

  // 10 minutos sem notícia: a faixa some, mas a figura NÃO sai da folha: repousa parada (o servidor segue com o agente)
  await js(`agenteAgora.em = Date.now() - 11 * 60 * 1000; render(); montaRobo()`);
  assert.equal(await js(`document.getElementById("robo-agente").hidden`), true, "depois de 10 min a faixa some");
  assert.equal(await js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])").length`), 1, "a figura repousa na folha (sem evento da fila, só a lista do servidor a mantém)");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("agente na folha: ok (plano certo, faixa, ok/erro, cartão da lista, repouso após 10 min). Captura em target/robo-folha.png");
} finally { limpar(); }
