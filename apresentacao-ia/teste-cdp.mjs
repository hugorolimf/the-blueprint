// Ajuda dos testes no navegador: abre o Edge/Chrome sem janela e o controla pelo DevTools (tempo REAL, mouse, tela estreita, movimento reduzido).
// const nav = await abrirNavegador(caminho); await nav.ir(url, { w, h, reduzido }); await nav.ev("document.title"); await nav.mouse(x, y); nav.fechar();
import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";

export const CANDIDATOS = [process.env.CHROME_PATH, "C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe",
  "C:/Program Files/Google/Chrome/Application/chrome.exe", "/usr/bin/google-chrome", "/usr/bin/chromium", "/usr/bin/chromium-browser"];
export const achaNavegador = () => CANDIDATOS.find((c) => c && existsSync(c));
export const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const portaLivre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });

export async function abrirNavegador(caminho) {
  const porta = await portaLivre(), perfil = mkdtempSync(join(tmpdir(), "ap-cdp-"));
  const filho = spawn(caminho, ["--headless=new", "--disable-gpu", "--no-first-run", `--remote-debugging-port=${porta}`, `--user-data-dir=${perfil}`, "about:blank"], { stdio: "ignore" });
  let alvo = null;
  for (let i = 0; i < 80 && !alvo; i++) { await dormir(250); try { alvo = (await (await fetch(`http://127.0.0.1:${porta}/json/list`)).json()).find((t) => t.type === "page" && t.url === "about:blank"); } catch { /* subindo */ } }
  if (!alvo) { filho.kill(); throw new Error("o navegador não abriu"); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((ok, falha) => { ws.onopen = ok; ws.onerror = () => falha(new Error("DevTools não conectou")); });
  let n = 0;
  const cmd = (method, params = {}) => new Promise((ok, falha) => { const id = ++n; const h = (e) => { const m = JSON.parse(e.data); if (m.id !== id) return; ws.removeEventListener("message", h); m.error ? falha(new Error(method + ": " + m.error.message)) : ok(m.result); }; ws.addEventListener("message", h); ws.send(JSON.stringify({ id, method, params })); });
  await cmd("Emulation.setFocusEmulationEnabled", { enabled: true }); // sem janela de verdade a página precisa se achar focada para o foco do teclado valer
  let rodada = 0;
  return {
    cmd,
    async ir(url, { w = 1440, h = 900, reduzido = false } = {}) {
      await cmd("Emulation.setDeviceMetricsOverride", { width: w, height: h, deviceScaleFactor: 1, mobile: w < 600 });
      await cmd("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: reduzido ? "reduce" : "no-preference" }] });
      await cmd("Page.navigate", { url: url.includes("#") ? url.replace("#", `?r=${++rodada}#`) : `${url}?r=${++rodada}` });
      await dormir(1800);
    },
    async ev(expressao) { const r = await cmd("Runtime.evaluate", { expression: expressao, returnByValue: true, awaitPromise: true }); if (r.exceptionDetails) throw new Error(r.exceptionDetails.text + " " + (r.exceptionDetails.exception?.description || "")); return r.result.value; },
    mouse: (x, y) => cmd("Input.dispatchMouseEvent", { type: "mouseMoved", x, y, buttons: 0 }),
    fechar() { try { ws.close(); } catch { /* já fechado */ } filho.kill(); setTimeout(() => { try { rmSync(perfil, { recursive: true, force: true }); } catch { /* o Edge ainda segura o perfil */ } }, 2500); },
  };
}
