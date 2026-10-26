// Uma página do app num Edge sem janela, controlada pelo DevTools (os e2e da sincronia: duas-pessoas, reconectar).
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { dormir, portaLivre } from "./lib-e2e-rust.mjs";

const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);

/** Abre `pasta.url + caminho` num perfil próprio; devolve `js(expr)` (avalia na página), `cdp` e os erros do console. O processo vai para `filhos`.
 *  `cookie`: o valor de `bp_sessao` (modo hospedado), posto antes de recarregar a página. */
export async function janela(pasta, nome, caminho, filhos, { cookie } = {}) {
  assert(edgeExe, "o teste precisa do Microsoft Edge");
  const porta = await portaLivre();
  const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${porta}`, `--user-data-dir=${join(pasta.dir, "perfil-" + nome)}`, "--window-size=1200,800", pasta.url + caminho], { stdio: "ignore" });
  filhos.push(edge);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((t) => t.type === "page" && t.url.startsWith(pasta.url)); } catch {} await dormir(100); }
  assert(alvo, `${nome}: a página não abriu`);
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  await cdp("Runtime.enable");
  if (cookie) { // modo hospedado: a sessão de quem entrou pelo convite (sem Secure: o proxy de teste é http) e a página de novo, já com ela
    await cdp("Network.setCookie", { name: "bp_sessao", value: cookie, url: pasta.url, httpOnly: true, sameSite: "Lax" });
    await cdp("Page.reload");
  }
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(`${nome}: ${JSON.stringify(r.result.exceptionDetails).slice(0, 300)}`); return r.result.result.value; };
  return { js, cdp, erros };
}

/** Espera `f()` ficar verdadeiro (a cada 50 ms) e devolve quanto levou; falha com `motivo` depois de `ms`. */
export async function espera(f, motivo, ms = 5000) {
  const t0 = Date.now();
  while (Date.now() - t0 < ms) { if (await f()) return Date.now() - t0; await dormir(50); }
  assert.fail(motivo);
}
