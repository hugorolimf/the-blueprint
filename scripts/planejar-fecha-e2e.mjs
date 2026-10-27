// Confere que aprovar uma proposta fecha a tela Planejar: com uma só proposta, "Aplicar" aplica e a tela fecha; com duas na fila, a tela continua na próxima e só fecha
// quando acaba a fila. Edge sem janela, banco de exemplo. Uso: node scripts/planejar-fecha-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge)
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-notif-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos"), BLUEPRINT_ASSETS_DIR: raiz };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); }; // o Edge ainda segura o perfil por um instante: não derruba o teste
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.some((t) => !t.feito)));
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 900)); return r.result.result.value; };

  await cdp("Runtime.enable");
  const proposta = (corpo) => fetch(url + "/api/mcp/proposta", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: "sm", blueprintId: blueprint.id, ...corpo }) });
  const hubAberto = () => js(`!document.getElementById("agente-tela").hidden`);
  const abrePlanejar = async () => { await js(`document.getElementById("b-ia").click()`); await dormir(1800); };
  const aplica = async () => { await js(`document.querySelector("#agente-tela [data-pact=aplicar]").click()`); await dormir(1500); };
  const a1 = blueprint.acoes[0], a2 = blueprint.acoes[1] || blueprint.acoes[0];
  await cdp("Page.enable");
  await cdp("Page.addScriptToEvaluateOnNewDocument", { source: `try{localStorage.setItem("blueprint.autoAplicar.sm:${blueprint.id}","0")}catch{}` }); // este teste revisa e aplica à mão: o aplicar sozinho (auto-aplicar.js) fica desligado
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);

  // uma proposta só: aplicar fecha o Planejar
  assert.equal((await proposta({ acoes: [{ id: a1.id, observacao: "Primeira instrução." }] })).status, 200);
  await dormir(1200);
  await abrePlanejar();
  assert.equal(await hubAberto(), true, "o Planejar abre na proposta");
  assert.equal(await js(`!!document.querySelector("#agente-tela [data-pact=aplicar]")`), true, "a proposta tem o botão de aplicar");
  await aplica();
  assert.equal(await hubAberto(), false, "aprovou e não há outra proposta: a tela Planejar fecha");
  const dado1 = await (await fetch(url + "/api/dados")).json();
  assert.equal(dado1.blueprints.find((p) => p.id === blueprint.id).acoes.find((f) => f.id === a1.id).obs, "Primeira instrução.", "e a proposta foi aplicada nos dados");

  // duas na fila: aplicar a primeira deixa a tela na segunda; aplicar a segunda a fecha
  assert.equal((await proposta({ acoes: [{ id: a1.id, observacao: "Segunda instrução." }] })).status, 200);
  assert.equal((await proposta({ acoes: [{ id: a2.id, observacao: "Terceira instrução." }] })).status, 200);
  await dormir(1200);
  await abrePlanejar();
  await aplica();
  assert.equal(await hubAberto(), true, "com outra proposta esperando, o Planejar continua aberto nela");
  assert.equal(await js(`!!document.querySelector("#agente-tela [data-pact=aplicar]")`), true, "e mostra a próxima para aprovar");
  await aplica();
  assert.equal(await hubAberto(), false, "acabou a fila: a tela fecha");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("Planejar fecha ao aprovar: ok (uma proposta fecha; com duas na fila segue na próxima e fecha no fim)");
} finally { limpar(); }
