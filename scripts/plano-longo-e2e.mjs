// Edge: título de plano longo, quebra de linha, sobreposição e pesquisa no trecho dividido.
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-figuras-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos"), BLUEPRINT_ASSETS_DIR: raiz };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} try { rmSync(tmp, { recursive: true, force: true }); } catch {} };
process.on("exit", limpar);
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
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  const titulo = "Mostrar um plano com um título suficientemente comprido para ocupar várias linhas sem perder conteúdo";
  await js(`(() => { const p = D.blueprints.find(x => x.id === ${JSON.stringify(blueprint.id)}), a = p.acoes.find(x => x.id === ${JSON.stringify(acao.id)}); a.planos[0].texto = ${JSON.stringify(titulo)}; abertos[p.id] = true; abrirTodosProjetos(p, true); render(); })()`);
  const resultado = await js(`(() => { const a = document.querySelector('.acao[data-f="${acao.id}"]'), g = a.querySelector('.chk[data-tipo="t"]'), t = g.querySelector('.tx'), r = g.querySelector('.foco2'); return { texto: t.textContent, linhas: t.querySelectorAll('tspan').length, altura: Number(r.getAttribute('height')), y: Number(r.getAttribute('y')), proximo: Number(a.querySelectorAll('.chk[data-tipo="t"]')[1]?.querySelector('.foco2')?.getAttribute('y')) || null }; })()`);
  assert.equal(resultado.texto, titulo);
  assert(resultado.linhas > 1 && resultado.altura > 22, JSON.stringify(resultado));
  if (resultado.proximo !== null) assert(resultado.proximo >= resultado.y + resultado.altura, "planos não se sobrepõem");
  const retangulos = await js(`(() => { const t = document.querySelector('.acao[data-f="${acao.id}"] .chk[data-tipo="t"] .tx'); return pesqMede([t], ["comprido para"]).flatMap(x => x[1]); })()`);
  assert(retangulos.length >= 2 && retangulos.every(r => r[2] > 0), "a pesquisa destaca o trecho nas duas linhas");
  assert.deepEqual(erros, []);
  console.log("plano longo na folha: ok");
} finally { limpar(); }
