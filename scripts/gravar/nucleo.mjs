// Gravador de tela do app: sobe um servidor temporário com os dados de exemplo, abre o Edge sem janela, grava o screencast do DevTools
// (Page.startScreencast) e codifica MP4 e GIF com o ffmpeg. Um cursor desenhado no tema do app (anel azul-papel) mostra por onde "o mouse" passa.
// Sempre os dados de exemplo (data.exemplo.json), nunca o banco vivo. Uso pelos roteiros: scripts/gravar/index.mjs.
import { spawn, spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

export const raiz = resolve(import.meta.dirname, "..", "..");
export const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const EDGE = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);

// o cursor usa as cores do tema (papel, tinta): anel claro com miolo escuro, e um pulso ao clicar
const CURSOR = `(() => { if (document.getElementById("cur-gravacao")) return;
  const s = document.createElement("style"); s.textContent = "#cur-gravacao{position:fixed;left:0;top:0;width:22px;height:22px;margin:-11px 0 0 -11px;border:2px solid #eaf4ff;border-radius:50%;background:#0c3a7899;box-shadow:0 0 0 1px #02132e,0 4px 14px #02132ecc;pointer-events:none;z-index:2147483647;transition:transform .12s}#cur-gravacao.p{transform:scale(.7);background:#eaf4ffcc}";
  document.head.appendChild(s); const c = document.createElement("div"); c.id = "cur-gravacao"; document.body.appendChild(c);
  addEventListener("mousemove", (e) => { c.style.left = e.clientX + "px"; c.style.top = e.clientY + "px"; }, true);
  addEventListener("mousedown", () => c.classList.add("p"), true); addEventListener("mouseup", () => c.classList.remove("p"), true); })()`;

export async function abrirAmbiente({ exe, dados, largura = 1280, altura = 800, extraEnv = {}, idioma = "" }) {
  dados ||= join(raiz, /^pt/i.test(idioma) ? "data.exemplo.json" : "data.exemplo.en.json"); // os dados de exemplo no mesmo idioma da interface
  if (!EDGE) throw new Error("precisa do Microsoft Edge");
  const tmp = mkdtempSync(join(tmpdir(), "gravar-"));
  copyFileSync(dados, join(tmp, "dados.json"));
  const [porta, depuracao] = [await livre(), await livre()];
  const url = `http://127.0.0.1:${porta}`;
  const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), DADOS_ARQ: join(tmp, "dados.json"), BLUEPRINT_DB: join(tmp, "blueprint.db"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1", ...extraEnv };
  const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
  const edge = spawn(EDGE, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, `--window-size=${largura},${altura}`, ...(idioma ? [`--lang=${idioma}`, `--accept-lang=${idioma}`] : []), "about:blank"], { stdio: "ignore" });
  const fechar = () => { for (const p of [servidor, edge]) try { p.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); };
  for (let i = 0; i < 150; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(100); } }
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const sock = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (sock.onopen = r));
  let n = 0; const pend = new Map(); const ouvintes = new Map(); const erros = [];
  sock.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); if (d.method) ouvintes.get(d.method)?.(d.params); else { pend.get(d.id)?.(d); pend.delete(d.id); } };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); sock.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Emulation.setDeviceMetricsOverride", { width: largura, height: altura, deviceScaleFactor: 1, mobile: false });
  await cdp("Runtime.enable"); await cdp("Page.enable");
  const A = { url, js, cdp, erros, fechar, largura, altura, idioma: idioma || "en-US", mouse: { x: largura / 2, y: altura / 2 }, porta,
    async ir(hash, espera = 2500) { await cdp("Page.navigate", { url: `${url}/${hash}` }); await dormir(espera); await js(`location.hash = ${JSON.stringify(hash.replace(/^#?/, ""))}`).catch(() => {}); await dormir(600); await js(CURSOR); },
    async mover(x, y, passos = 14) { // desliza o cursor, no ritmo de uma mão
      const { x: x0, y: y0 } = A.mouse;
      for (let i = 1; i <= passos; i++) { const t = i / passos, e = t * t * (3 - 2 * t); await cdp("Input.dispatchMouseEvent", { type: "mouseMoved", x: x0 + (x - x0) * e, y: y0 + (y - y0) * e }); await dormir(16); }
      A.mouse = { x, y };
    },
    async clicar(x, y, botao = "left") { await A.mover(x, y); await cdp("Input.dispatchMouseEvent", { type: "mousePressed", x, y, button: botao, buttons: botao === "left" ? 1 : 2, clickCount: 1 }); await dormir(90); await cdp("Input.dispatchMouseEvent", { type: "mouseReleased", x, y, button: botao, clickCount: 1 }); await dormir(250); },
    async clicarEm(seletor, botao = "left") { const r = await js(`(() => { const e = document.querySelector(${JSON.stringify(seletor)}); if (!e) return null; const b = e.getBoundingClientRect(); return { x: b.left + b.width / 2, y: b.top + b.height / 2 }; })()`); if (!r) throw new Error("não achei " + seletor); await A.clicar(r.x, r.y, botao); },
    async evento(tipo, corpo) { return fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, ...corpo }) }); },
    async post(rota, corpo) { return fetch(url + rota, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) }); },
    // grava o que `fazer` desenha na tela; devolve os quadros com a hora de cada um
    async gravar(fazer) {
      const quadros = []; let aguarda = Promise.resolve();
      ouvintes.set("Page.screencastFrame", (p) => { quadros.push({ t: p.metadata.timestamp, b64: p.data }); aguarda = cdp("Page.screencastFrameAck", { sessionId: p.sessionId }); });
      await cdp("Page.startScreencast", { format: "jpeg", quality: 92, maxWidth: largura, maxHeight: altura, everyNthFrame: 1 });
      await dormir(400);
      try { await fazer(); await dormir(500); } finally { await cdp("Page.stopScreencast"); ouvintes.delete("Page.screencastFrame"); await aguarda; }
      return quadros;
    } };
  return A;
}

/** Codifica os quadros: MP4 (30 fps) e, se `gif`, um GIF de 15 fps com paleta própria. Devolve os tamanhos. */
export function codificar(quadros, base, { gif = null } = {}) {
  if (quadros.length < 2) throw new Error("o screencast não trouxe quadros");
  const dir = mkdtempSync(join(tmpdir(), "quadros-")), lista = [];
  quadros.forEach((q, i) => {
    const f = join(dir, `q${String(i).padStart(5, "0")}.jpg`);
    writeFileSync(f, Buffer.from(q.b64, "base64"));
    const dur = i + 1 < quadros.length ? Math.max(0.001, quadros[i + 1].t - q.t) : 0.4;
    lista.push(`file '${f.replace(/\\/g, "/")}'`, `duration ${dur.toFixed(4)}`);
  });
  lista.push(lista[lista.length - 2]); // o concat precisa repetir o último arquivo
  writeFileSync(join(dir, "lista.txt"), lista.join("\n"));
  mkdirSync(resolve(base, ".."), { recursive: true });
  const exec = (args) => { const r = spawnSync("ffmpeg", ["-y", "-loglevel", "error", ...args], { encoding: "utf8" }); if (r.status !== 0) throw new Error("ffmpeg: " + (r.stderr || "").slice(0, 400)); };
  exec(["-f", "concat", "-safe", "0", "-i", join(dir, "lista.txt"), "-vf", "fps=30,format=yuv420p", "-c:v", "libx264", "-crf", "20", "-preset", "medium", "-movflags", "+faststart", `${base}.mp4`]);
  if (gif) exec(["-t", String(gif.segundos || 14), "-i", `${base}.mp4`, "-vf", `fps=${gif.fps || 10},scale=${gif.largura || 720}:-1:flags=lanczos,split[a][b];[a]palettegen=max_colors=${gif.cores || 64}:stats_mode=diff[p];[b][p]paletteuse=dither=none`, "-loop", "0", `${base}.gif`]);
  rmSync(dir, { recursive: true, force: true });
  return { quadros: quadros.length, duracao: +(quadros.at(-1).t - quadros[0].t + 0.4).toFixed(1) };
}
