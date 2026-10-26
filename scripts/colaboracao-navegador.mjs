// Apoio dos e2e da colaboração (convite-tela-e2e.mjs, visitante-supervisor-e2e.mjs): o servidor (stub ou binário real atrás do proxy do stub)
// e um Edge sem janela por pessoa, cada um com o próprio perfil (cookies separados), falando com a página pelo DevTools.
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { subirStub } from "./stub-colaboracao.mjs";

export const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const EDGE = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);

// `--rust <exe>` na linha de comando: o mesmo teste contra o binário real (serve --hospedar), com o stub só de proxy
export async function servidorDoTeste() {
  const i = process.argv.indexOf("--rust");
  return subirStub({ rust: i > 0 ? process.argv[i + 1] : "" });
}

export async function abrirNavegador(nome, { idioma = "pt-BR", largura = 1280, altura = 800 } = {}) {
  if (!EDGE) throw new Error("precisa do Microsoft Edge");
  const tmp = mkdtempSync(join(tmpdir(), "colab-" + nome + "-")), depuracao = await livre();
  const edge = spawn(EDGE, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${tmp}`, `--window-size=${largura},${altura}`, `--lang=${idioma}`, `--accept-lang=${idioma}`, "about:blank"], { stdio: "ignore" });
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const sock = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (sock.onopen = r));
  let n = 0; const pend = new Map(), erros = [];
  sock.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); else if (!d.method) { pend.get(d.id)?.(d); pend.delete(d.id); } };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); sock.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(`[${nome}] ` + JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable"); await cdp("Page.enable");
  const P = {
    nome, js, cdp, erros,
    async ir(url, espera = 1500) { await cdp("Page.navigate", { url }); await dormir(espera); },
    // espera uma condição da página (expressão JS) por até `ms`
    async ate(expr, ms = 8000, rotulo = expr) {
      const fim = Date.now() + ms;
      for (;;) { try { if (await js(expr)) return; } catch {} if (Date.now() > fim) throw new Error(`[${nome}] não aconteceu: ${rotulo}`); await dormir(120); }
    },
    clicar: (seletor) => js(`(() => { const e = document.querySelector(${JSON.stringify(seletor)}); if (!e) throw new Error("não achei ${seletor.replace(/"/g, "'")}"); e.click(); return true; })()`),
    // escolhe a opção de um <select> e avisa a página, como o usuário faria
    escolher: (seletor, valor) => js(`(() => { const e = document.querySelector(${JSON.stringify(seletor)}); e.value = ${JSON.stringify(valor)}; e.dispatchEvent(new Event("change", { bubbles: true })); return e.value; })()`),
    // aceita a confirmação (confirmar.js) que estiver aberta
    confirmar: () => js(`(() => { const b = document.querySelector("dialog.confirmar [data-r='1']"); b.click(); return true; })()`),
    api: (rota, corpo, metodo = corpo === undefined ? "GET" : "POST") => js(`fetch(${JSON.stringify(rota)}, ${JSON.stringify(metodo === "GET" ? { cache: "no-store" } : { method: metodo, headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) })}).then(async (r) => ({ status: r.status, corpo: await r.json().catch(() => null) }))`),
    fechar() { try { sock.close(); } catch {} try { edge.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); },
  };
  return P;
}
