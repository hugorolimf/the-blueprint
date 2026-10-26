// Duas janelas na mesma base não se sobrescrevem (gravação por operações em /api/mudancas + SSE /api/eventos, web/js/ao-vivo.js). Edge sem janela, banco temporário.
//   node scripts/sincronia-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; ~15 s)
// 1) uma janela grava e a outra vê na hora, sem recarregar; 2) uma janela que ficou velha (sem SSE) grava só o que mudou e não apaga o que a
// outra fez; ao reconectar, recebe pela versão o que perdeu.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
import { dormir, exePadrao, http, ligarServidor, pastaDeDados, portaLivre, raiz } from "./lib-e2e-rust.mjs";
import { join } from "node:path";

const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const pasta = await pastaDeDados("sincronia", {});
pasta.env.BLUEPRINT_ASSETS_DIR = raiz; // serve web/ da pasta: o teste vale para o que está no disco, sem recompilar a interface
const servidor = await ligarServidor(exePadrao(), pasta);
const filhos = [servidor];

async function janela(nome) {
  const porta = await portaLivre();
  const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${porta}`, `--user-data-dir=${join(pasta.dir, "perfil-" + nome)}`, "--window-size=1200,800", pasta.url + "/"], { stdio: "ignore" });
  filhos.push(edge);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((t) => t.type === "page" && t.url.startsWith(pasta.url)); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  await cdp("Runtime.enable");
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(`${nome}: ${JSON.stringify(r.result.exceptionDetails).slice(0, 300)}`); return r.result.result.value; };
  return { js, erros };
}
const espera = async (f, motivo, ms = 5000) => { for (let t = 0; t < ms; t += 100) { if (await f()) return; await dormir(100); } assert.fail(motivo); };
const dados = async () => (await http(pasta.url + "/api/dados")).json;
try {
  const [A, B] = [await janela("a"), await janela("b")];
  for (const j of [A, B]) await espera(async () => (await j.js(`typeof D !== "undefined" && D?.blueprints?.length > 0 && typeof versaoDados === "string" && versaoDados !== ""`)) === true, "a página carregou os dados e a versão");

  // 1) tempo real: B grava e A mostra sem recarregar
  await B.js(`D.blueprints[0].nome = "Mudado em B"; mudou();`);
  await espera(async () => (await A.js(`D.blueprints[0].nome`)) === "Mudado em B", "A vê a gravação de B em tempo real");

  // 2) A fica velha de propósito (não recebe mais atualizações); B cria uma ação; A edita outra coisa e grava
  await A.js(`carregar = async () => {}; aoVivoES.close(); aoVivoES = { readyState: 2, close() {} }`);
  await B.js(`D.blueprints[0].acoes.push({ id: "fB", nome: "Nova de B", inicio: "2026-11-06", fim: "2026-11-07", status: "planejado", progresso: 0, obs: "", planos: [], checkpoints: [] }); mudou();`);
  await dormir(1500);
  assert.equal(await A.js(`D.blueprints[0].acoes.some((f) => f.id === "fB")`), false, "A está velha: não conhece a ação de B");
  await A.js(`D.blueprints[0].acoes[0].nome = "Editada em A"; mudou();`);
  await espera(async () => { const d = await dados(); return d.blueprints[0].acoes.some((f) => f.nome === "Editada em A"); }, "a edição de A foi gravada");
  const final = await dados();
  const nomes = final.blueprints[0].acoes.map((f) => f.nome);
  assert(nomes.includes("Nova de B"), "a ação de B NÃO foi apagada pela janela velha: " + nomes);
  assert(nomes.includes("Editada em A"), "e a edição de A ficou");
  assert.equal(final.blueprints[0].nome, "Mudado em B", "a mudança anterior de B também ficou");
  await A.js(`aoVivoConectar(true)`);
  await espera(async () => (await A.js(`D.blueprints[0].acoes.some((f) => f.id === "fB")`)) === true, "ao reconectar, A recebe pela versão a ação de B");
  assert.deepEqual([...A.erros, ...B.erros], [], "sem erro no console");
  console.log("sincronia e2e: ok (tempo real entre janelas; janela velha grava sem apagar o trabalho da outra e recupera ao reconectar)");
} finally {
  for (const f of filhos) { try { f.kill(); } catch {} }
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800);
}
