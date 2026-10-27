// Fluxo completo das decisões: o agente propõe só decisões pelo MCP (propor_blueprint com acoes: [] e blueprint.decisoes, via `blueprint --mcp`), a tela Planejar as mostra
// com caixas de marcar, aplica só as marcadas, ler_blueprint as devolve, e uma segunda proposta atualiza uma decisão pelo id. Banco de exemplo, sem recompilar.
// Uso: node scripts/decisoes-e2e.mjs [exe] [pasta-com-web]   (precisa do Microsoft Edge; serve web/ da pasta dada, por padrão a do repositório)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const [porta, depuracao] = [await portaLivre(), await portaLivre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-registro-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.length);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");

  // o agente chama uma ferramenta pelo `blueprint --mcp` (stdio), como o Claude faria; devolve { isError, texto }
  const chamar = async (ferramenta, argumentos) => {
    const filho = spawn(exe, ["--mcp"], { env: { ...process.env, BLUEPRINT_URL: url }, stdio: ["pipe", "pipe", "ignore"], windowsHide: true });
    let buf = "", id = 0; const esp = new Map();
    filho.stdout.on("data", (d) => { buf += d; for (let i; (i = buf.indexOf("\n")) >= 0;) { const l = buf.slice(0, i).trim(); buf = buf.slice(i + 1); if (l) { const m = JSON.parse(l); esp.get(m.id)?.(m); } } });
    const rpc = (method, params) => new Promise((ok) => { esp.set(++id, ok); filho.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
    try {
      await rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "decisoes-e2e", version: "1" } });
      const r = await rpc("tools/call", { name: ferramenta, arguments: { workspace_id: "sm", blueprint_id: blueprint.id, ...argumentos } });
      return { isError: !!(r.error || r.result?.isError), texto: r.result?.content?.map((x) => x.text).join("\n") ?? JSON.stringify(r.error) };
    } finally { filho.kill(); }
  };
  const painel = () => js(`(() => { const b = document.getElementById("cl-prop"); const a = b.querySelector("[data-pact=aplicar]"); return { texto: b.textContent, botao: a?.textContent || "", desligado: !!a?.disabled }; })()`);
  const decisoes = async () => (await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).decisoes || [];
  const fila = async () => (await (await fetch(url + "/api/mcp/fila")).json()).itens.filter((i) => i.tipo === "proposta");

  await cdp("Page.enable");
  await cdp("Page.addScriptToEvaluateOnNewDocument", { source: `try{localStorage.setItem("blueprint.autoAplicar.sm:${blueprint.id}","0")}catch{}` }); // este teste revisa e aplica à mão: o aplicar sozinho (auto-aplicar.js) fica desligado
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  assert.deepEqual(await decisoes(), [], "o blueprint de exemplo começa sem decisões");
  const r1 = await chamar("propor_blueprint", { mensagem: "Decisões da stack.", acoes: [], blueprint: { decisoes: [
    { titulo: "Servidor", escolha: "Kotlin com Ktor", motivo: "as regras ficam num lugar só" },
    { titulo: "Dados", escolha: "PostgreSQL principal e outro para a urna" },
    { titulo: "Hospedagem" }] } });
  assert.equal(r1.isError, false, r1.texto);
  assert.equal((await fila()).length, 1, "a proposta só de decisões entrou na fila");
  await dormir(1200);
  await js(`document.getElementById("b-ia").click()`); // abre o Planejar
  await dormir(1500);

  let p = await painel();
  assert.match(p.texto, /3 decisões/, "o cartão conta as decisões");
  assert.match(p.texto, /Servidor/);
  assert.match(p.texto, /Kotlin com Ktor/);
  assert.match(p.texto, /Em aberto/, "a sem escolha chega em aberto");
  assert.equal(p.botao.trim(), "Aplicar 3 decisões");
  assert.equal(p.desligado, false);
  assert.deepEqual(await decisoes(), [], "chegar não grava: só Aplicar grava");

  await js(`document.querySelector('#cl-prop input[data-pc=dec-inc][data-j="2"]').click()`); // desmarca a Hospedagem
  await dormir(300);
  assert.equal((await painel()).botao.trim(), "Aplicar 2 decisões");
  await js(`document.querySelector('#cl-prop [data-pact=aplicar]').click()`);
  await dormir(1500);
  const gravadas = await decisoes();
  assert.deepEqual(gravadas.map((d) => [d.titulo, d.estado]), [["Servidor", "decidida"], ["Dados", "decidida"]], "só as marcadas entraram");
  assert.ok(gravadas.every((d) => d.id && d.em), "cada uma ganhou id e data");
  assert.equal((await fila()).length, 0, "a proposta saiu da fila do servidor (ack)");

  const lida = await chamar("ler_blueprint", {});
  assert.match(lida.texto, /"decisoes"/);
  assert.match(lida.texto, /Kotlin com Ktor/, "ler_blueprint devolve as decisões");

  // segunda proposta: o id atualiza só o que veio e mantém o resto
  const idServidor = gravadas[0].id;
  const r2 = await chamar("propor_blueprint", { acoes: [], blueprint: { decisoes: [{ id: idServidor, motivo: "menos código duplicado" }] } });
  assert.equal(r2.isError, false, r2.texto);
  await dormir(1200);
  await js(`document.getElementById("b-ia").click()`);
  await dormir(1200);
  assert.match((await painel()).texto, /Alterada/, "a decisão existente aparece como alterada");
  await js(`document.querySelector('#cl-prop [data-pact=aplicar]').click()`);
  await dormir(1500);
  const depois = await decisoes();
  assert.equal(depois.length, 2, "atualizar não duplica");
  assert.deepEqual([depois[0].escolha, depois[0].motivo], ["Kotlin com Ktor", "menos código duplicado"]);

  // o que o servidor recusa volta ao agente com o motivo
  const ruim = await chamar("propor_blueprint", { acoes: [], blueprint: { decisoes: [{ titulo: "Sem escolha", estado: "decidida" }] } });
  assert.equal(ruim.isError, true);
  assert.match(ruim.texto, /escolha/);
  assert.deepEqual(erros, [], "nenhum erro de script na página");
  console.log("decisoes-e2e: ok");
} finally {
  limpar();
}
