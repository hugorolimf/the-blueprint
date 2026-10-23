// Confere as opções da folhinha do agente (Edge sem janela, banco de exemplo): o agente concluído e o parado (aguardando) têm "Concluído" e "Fechar"; Fechar só fecha e a figura
// fica; Concluído dispensa a figura até o agente avisar de novo; o agente trabalhando só tem Fechar. Uso: node scripts/concluido-botao-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge)
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
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
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
  const acao = blueprint.acoes.find((f) => f.planos?.some((t) => !t.feito)), plano = acao.planos.find((t) => !t.feito);
  const aviso = (corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo: "progresso", workspaceId: "sm", blueprintId: blueprint.id, acaoId: acao.id, planoId: plano.id, agente: "claude", ...corpo }) });
  const figuras = () => js(`[...document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])")].map((e) => e.dataset.roboNome)`);
  const botoes = () => js(`[...document.querySelectorAll("#nota-folha .acoes button")].map((b) => b.textContent)`);
  const clicaFigura = async () => { await js(`document.querySelector('#claude-camada [data-robo-nome="claude"]').click()`); await dormir(700); };
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);

  // trabalhando: só Fechar
  await aviso({ texto: "Escrevendo o teste", estado: "escrevendo" });
  await dormir(1500);
  assert.deepEqual(await figuras(), ["claude"], "o agente aparece");
  await clicaFigura();
  assert.deepEqual(await botoes(), ["Fechar"], "trabalhando: só a opção Fechar");
  await js(`document.querySelector("#nota-folha [data-nfecha]").click()`);
  await dormir(800);

  // aguardando: Concluído e Fechar; Fechar mantém a figura; Concluído a dispensa
  await aviso({ texto: "Aguardando o usuário", estado: "aguardando" });
  await dormir(1500);
  await clicaFigura();
  assert.deepEqual(await botoes(), ["Concluído", "Fechar"], "aguardando: as opções Concluído e Fechar");
  await js(`document.querySelector("#nota-folha [data-nfecha]").click()`);
  await dormir(1200);
  assert.deepEqual(await figuras(), ["claude"], "Fechar não tira o agente da folha");
  await clicaFigura();
  await js(`document.querySelector("#nota-folha [data-nacao]").click()`);
  await dormir(1500);
  assert.deepEqual(await figuras(), [], "Concluído: o agente sai da folha");

  // avisou de novo: volta (o visto vale só para aquele aviso)
  await aviso({ texto: "Voltei a trabalhar", estado: "escrevendo" });
  await dormir(1500);
  assert.deepEqual(await figuras(), ["claude"], "um aviso novo traz o agente de volta");

  // concluído: Concluído e Fechar, com a mensagem de conclusão
  await aviso({ texto: "Terminei: tudo certo", estado: "concluido" });
  await dormir(1500);
  await clicaFigura();
  assert.deepEqual(await botoes(), ["Concluído", "Fechar"], "concluído: as opções Concluído e Fechar");
  assert.equal((await js(`document.getElementById("nota-folha").innerText`)).includes("Terminei: tudo certo"), true, "com a mensagem de conclusão");
  await js(`document.querySelector("#nota-folha [data-nacao]").click()`);
  await dormir(1500);
  assert.deepEqual(await figuras(), [], "Concluído: o concluído sai da folha");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("opções do agente: ok (trabalhando só Fechar; aguardando e concluído com Concluído e Fechar; Concluído dispensa até novo aviso)");
} finally { limpar(); }
