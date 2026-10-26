// A linha do tempo do agente na folha da ação, de ponta a ponta: o agente avisa pela rota de verdade, a folha da ação (Edge sem janela) mostra o evento com o
// nome do plano, e um evento novo chega por SSE com a folha aberta, sem recarregar; a evidência do plano concluído não aparece. Banco temporário, nada de tokens.
// Uso: node scripts/linha-do-tempo-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; precisa do Microsoft Edge). Captura em target/linha-do-tempo.png.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { DADOS, dormir, exePadrao, ligarServidor, pastaDeDados, portaLivre, postJson, raiz } from "./lib-e2e-rust.mjs";

const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dados = await pastaDeDados("blueprint-ldt", { BLUEPRINT_ASSETS_DIR: raiz });
writeFileSync(join(dados.dir, "data.json"), JSON.stringify({ ...DADOS, usuario: { nome: "Teste" } })); // com nome, a página abre direto no blueprint
const depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dados.dir, "perfil")}`, "--window-size=1280,900", "about:blank"], { stdio: "ignore" });
let servidor;
const limpar = () => { try { edge.kill(); } catch {} try { servidor?.kill(); } catch {} setTimeout(() => { try { rmSync(dados.dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const evento = (extra) => postJson(`${dados.url}/api/mcp/evento`, { workspaceId: "sm", blueprintId: "p1", acaoId: "f1", planoId: "t1", ...extra });
try {
  servidor = await ligarServidor(exePadrao(), dados);
  assert.equal((await evento({ tipo: "progresso", texto: "Começando o plano", agente: "Claude", estado: "escrevendo", onde: ["src/a.rs"] })).status, 200);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  const itens = () => js(`[...document.querySelectorAll("#nota-folha .ldt li")].map((li) => li.textContent)`);
  const espera = async (cond, msg) => { for (let i = 0; i < 60; i++) { if (await cond()) return; await dormir(100); } assert.fail(msg); };
  await cdp("Runtime.enable");
  await cdp("Page.navigate", { url: `${dados.url}/#/p/p1` });
  await espera(() => js(`typeof D !== "undefined" && !!D.blueprints?.find((p) => p.id === "p1") && !!mcpSse`), "a página carrega os dados e abre o SSE");

  await js(`abreFolhaDaAcao("p1", "f1", document.querySelector("#desenho") || document.body)`);
  await espera(async () => (await itens()).length === 1, "a folha da ação mostra o evento já guardado");
  const [primeiro] = await itens();
  assert(primeiro.includes("Claude") && primeiro.includes("Começando o plano") && primeiro.includes("Plano um"), `evento com agente, frase e plano: ${primeiro}`);

  // com a folha aberta, o evento novo chega por SSE (sem recarregar nem consultar de tempo em tempo)
  assert.equal((await evento({ tipo: "progresso", texto: "Editando web/tela.js", agente: "Claude", estado: "escrevendo", onde: ["web/tela.js"] })).status, 200);
  await espera(async () => (await itens()).length === 2, "o evento novo aparece com a folha aberta");
  assert((await itens())[0].includes("Editando web/tela.js"), "o mais recente fica em cima");
  await dormir(700); // a folha termina de abrir antes da captura (a conclusão abaixo abre o aviso do agente por cima)
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "linha-do-tempo.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png", clip: { ...(await js(`(({ x, y, width, height }) => ({ x: x + scrollX, y: y + scrollY, width, height }))(document.querySelector("#nota-folha").getBoundingClientRect())`)), scale: 1 } })).result.data, "base64"));

  assert.equal((await evento({ tipo: "concluir", texto: "cargo test: 255 passaram" })).status, 200);
  await espera(async () => (await itens()).length === 3, "a conclusão aparece com a folha aberta");
  const [recente] = await itens();
  assert(recente.includes("Plano concluído") && !recente.includes("cargo test"), `a conclusão aparece em cima e sem a evidência: ${recente}`);
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("linha do tempo na folha da ação: ok (evento guardado aparece com o plano, evento novo chega por SSE, evidência fora)");
} finally { limpar(); }
