// Tira uma captura de cada tela da interface (Edge sem janela, 1440×900, movimento reduzido) servida pelo blueprint.exe com uma cópia dos dados.
// Telas: lista de workspaces, workspace, blueprint (folha), Planejar, "Como conectar", Editar e visor de protótipo.
// Uso: node scripts/capturas.mjs [dados.json] [pasta de saída]    tira as capturas (padrão: data.exemplo.json → target/capturas)
//      node scripts/capturas.mjs --gravar                         grava o hash dos pixels de cada tela em referencias/capturas.json
//      node scripts/capturas.mjs --comparar                       confere os hashes contra referencias/capturas.json (sai 1 se alguma tela mudou)
// Os dados de exemplo e a porta fixa deixam as telas iguais de uma execução para outra; as imagens ficam na pasta de saída para você olhar.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const args = process.argv.slice(2);
const [gravar, comparar] = [args.includes("--gravar"), args.includes("--comparar")];
const posicionais = args.filter((a) => !a.startsWith("--"));
const dadosArq = resolve(posicionais[0] || join(raiz, "data.exemplo.json"));
const saida = resolve(posicionais[1] || join(raiz, "target", "capturas"));
const referencia = join(raiz, "referencias", "capturas.json");
const exe = resolve(process.env.BLUEPRINT_EXE || join(raiz, "target/release/blueprint.exe"));
mkdirSync(saida, { recursive: true });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "precisa do Microsoft Edge");

const tmp = mkdtempSync(join(tmpdir(), "capturas-"));
copyFileSync(dadosArq, join(tmp, "dados.json"));
// a porta aparece no texto de "Como conectar": a tela só sai igual se a porta for a mesma
const [porta, depuracao] = [39117, await livre()];
const url = `http://127.0.0.1:${porta}`;
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), DADOS_ARQ: join(tmp, "dados.json"), BLUEPRINT_DB: join(tmp, "blueprint.db"), PROTOTIPOS_DIR: join(tmp, "p") }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,900", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [servidor, edge]) try { p.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 800); };
process.on("exit", limpar);

try {
  for (let i = 0; i < 150; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(100); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const comPlanos = (w) => w.blueprints?.some((p) => p.acoes?.some((f) => f.planos?.length));
  const ws = comPlanos(dados) ? dados : (dados.workspaces || []).find(comPlanos);
  const blueprint = ws.blueprints.find((p) => p.acoes?.some((f) => f.planos?.length));
  const acao = blueprint.acoes.find((f) => f.planos?.length);
  const wsId = ws.id || "sm";
  const html = "<!doctype html><title>Prototipo</title><body style='margin:0;background:#fff'><svg viewBox='0 0 200 100' width='100%'><rect x='10' y='10' width='180' height='80' fill='#0c3a78'/><text x='100' y='58' fill='#fff' text-anchor='middle' font-size='20'>Protótipo</text></svg>";
  await fetch(url + "/api/mcp/prototipo", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: wsId, blueprintId: blueprint.id, acaoId: acao.id, titulo: "Tela de teste", html }) });

  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const sock = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (sock.onopen = r));
  let n = 0; const pend = new Map();
  sock.onmessage = (m) => { const d = JSON.parse(m.data); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); sock.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300)); return r.result.result.value; };
  await cdp("Emulation.setDeviceMetricsOverride", { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await cdp("Runtime.enable");

  const TELAS = [
    ["01-lista-de-workspaces", "#/", async () => {}],
    ["02-workspace", `#/c/${wsId}`, async () => {}],
    ["03-blueprint", `#/p/${blueprint.id}`, async () => {}],
    ["04-planejar", `#/p/${blueprint.id}`, async () => { await js(`document.getElementById("b-ia").click()`); }],
    ["05-como-conectar", `#/p/${blueprint.id}`, async () => { await js(`document.getElementById("b-ia").click()`); await dormir(1500); await js(`document.getElementById("cu-abrir").click()`); }],
    ["06-editar", `#/p/${blueprint.id}`, async () => { await js(`editar(true)`); }],
    ["07-visor-de-prototipo", `#/p/${blueprint.id}`, async () => { await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render(); abreProtos(${JSON.stringify(blueprint.id)}, ${JSON.stringify(acao.id)})`); }],
  ];
  const fotos = {};
  for (const [nome, hash, fazer] of TELAS) {
    await cdp("Page.navigate", { url: `${url}/${hash}` });
    await dormir(3000);
    await js(`location.hash = ${JSON.stringify(hash.slice(1))}`); await dormir(1200);
    await js(`agenteAgora = null; render()`); // o aviso do protótipo enviado pelo script não entra na captura (o robô é só indicador e muda com o tempo)
    await fazer();
    await dormir(2000);
    const r = await cdp("Page.captureScreenshot", { format: "png" });
    fotos[nome] = r.result.data;
    writeFileSync(join(saida, `${nome}.png`), Buffer.from(r.result.data, "base64"));
  }
  // o hash é dos pixels decodificados (SHA-256 do RGBA), não do arquivo PNG: não depende do compressor
  await cdp("Page.navigate", { url: `${url}/api/mcp/info` }); // contexto seguro (127.0.0.1): crypto.subtle existe
  await dormir(300);
  const hashes = {};
  for (const [nome] of TELAS) {
    hashes[nome] = await js(`(async () => {
      const i = await new Promise((ok) => { const m = new Image(); m.onload = () => ok(m); m.src = "data:image/png;base64," + ${JSON.stringify(fotos[nome])}; });
      const c = document.createElement("canvas"); c.width = i.width; c.height = i.height; const g = c.getContext("2d"); g.drawImage(i, 0, 0);
      const d = g.getImageData(0, 0, i.width, i.height).data;
      return [...new Uint8Array(await crypto.subtle.digest("SHA-256", d))].map((b) => b.toString(16).padStart(2, "0")).join("").slice(0, 16);
    })()`);
  }
  console.log(`${TELAS.length} capturas em ${saida}`);
  if (gravar) { writeFileSync(referencia, JSON.stringify({ dados: "data.exemplo.json", tamanho: "1440x900", telas: hashes }, null, 2) + "\n"); console.log(`hashes gravados em ${referencia}`); }
  if (comparar) {
    const esperado = JSON.parse(readFileSync(referencia, "utf8")).telas;
    let diferentes = 0;
    for (const nome of Object.keys(hashes)) {
      const igual = esperado[nome] === hashes[nome];
      if (!igual) diferentes++;
      console.log(`${nome.padEnd(26)} ${igual ? "idêntica" : `DIFERENTE (veja ${join(saida, nome + ".png")})`}`);
    }
    process.exitCode = diferentes ? 1 : 0;
  }
} finally { limpar(); }
