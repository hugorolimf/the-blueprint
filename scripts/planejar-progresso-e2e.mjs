// Edge: progresso não remonta a proposta nem perde foco; uma proposta nova atualiza a fila.
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
  const acao = blueprint.acoes.find((a) => a.planos?.some((t) => !t.feito));
  const plano = acao.planos.find((t) => !t.feito);
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  for (let i = 0; i < 100 && !(await js(`!!document.querySelector("svg.desenho")`)); i++) await dormir(100);
  assert.equal((await proposta({ acoes: [{ id: acao.id, observacao: "Escopo: teste de foco." }] })).status, 200);
  for (let i = 0; i < 100 && !(await js(`conv(${JSON.stringify(blueprint.id)}).proposta?.length > 0`)); i++) await dormir(100);
  await js(`agenteTela(true, { proj: ${JSON.stringify(blueprint.id)} })`);
  await js(`document.querySelector('#cl-prop [data-pact="editar-acao"]').click()`);
  await js(`(() => { const e = document.querySelector('#cl-prop input[data-pc="nome"]'); e.focus(); e.value = "Nome em edição"; e.dispatchEvent(new Event("input", {bubbles:true})); e.setSelectionRange(2, 6); window.__campo = e; window.__redesenhos = 0; new MutationObserver(() => window.__redesenhos++).observe(document.getElementById("cl-prop"), {childList:true, subtree:true}); })()`);
  const r = await fetch(url + "/api/mcp/evento", { method: "POST", headers: {"content-type":"application/json"}, body: JSON.stringify({ tipo:"progresso", texto:"Lendo o projeto", workspaceId:"sm", blueprintId:blueprint.id, acaoId:acao.id, planoId:plano.id }) });
  assert.equal(r.status, 200);
  for (let i = 0; i < 60 && !(await js(`mcpFeed.some(x => x.texto?.includes("Lendo o projeto"))`)); i++) await dormir(100);
  const estado = await js(`(() => { const e = document.querySelector('#cl-prop input[data-pc="nome"]'); return { mesmo: e === window.__campo, foco: document.activeElement === e, texto: e.value, selecao: [e.selectionStart,e.selectionEnd] }; })()`);
  assert.deepEqual(estado, { mesmo:true, foco:true, texto:"Nome em edição", selecao:[2,6] }, "progresso mantém o campo e a seleção");
  assert.equal(await js(`window.__redesenhos`), 0, "progresso não reconstrói a proposta");
  assert.equal((await proposta({ acoes: [{ id: acao.id, observacao: "Outra proposta." }] })).status, 200);
  for (let i = 0; i < 60 && !(await js(`document.querySelectorAll("#cl-prop [data-fila]").length === 2`)); i++) await dormir(100);
  assert.equal(await js(`document.querySelectorAll("#cl-prop [data-fila]").length`), 2, "nova proposta ainda atualiza a fila visível");
  assert.deepEqual(erros, []);
  console.log("Planejar durante progresso: ok");
} finally { limpar(); }
