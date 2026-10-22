// Captura a figura do agente em zoom, em pedaços, para conferir a olho (e para o relatório de 4×): desenha a pose com agentes-figura.mjs,
// põe numa página com as regras de marca.css (o papel azul do blueprint) e fotografa a grade de pedaços no Edge sem janela.
// Uso: node scripts/captura-figura.mjs [--papel construtor] [--estado escrevendo] [--posicao 0.5] [--progresso 0.6] [--grade 3x3] [--escala 2] [--tema escuro|claro] [--saida target/figura]
// Grava <saida>-inteira.png e <saida>-<linha>-<coluna>.png. Cada pedaço é 1/grade da figura desenhada em 880 px de largura (4× o cartão do app).
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { desenhaEmblema } from "../emblemas.mjs";
import { pecasDoAgente } from "../agentes-figura.mjs";

const raiz = resolve(import.meta.dirname, "..");
const arg = (nome, padrao) => { const i = process.argv.indexOf(`--${nome}`); return i > 0 ? process.argv[i + 1] : padrao; };
const pose = { papel: arg("papel", "construtor"), estado: arg("estado", "escrevendo"), posicao: Number(arg("posicao", 0.5)), progresso: Number(arg("progresso", 0.6)) };
const [linhas, colunas] = arg("grade", "3x3").split("x").map(Number);
const escala = Number(arg("escala", 2));
const tema = arg("tema", "escuro");
const saida = resolve(raiz, arg("saida", "target/figura"));
mkdirSync(dirname(saida), { recursive: true });

const LARG = 880, ALT = 680; // 4× o cartão do app (220×170)
const lista = pecasDoAgente(pose);
const e = desenhaEmblema(lista, { largura: LARG, altura: ALT, pad: 4, progresso: 1, sombra: false, limite: 400 });
const css = readFileSync(join(raiz, "web/css/marca.css"), "utf8");
const claro = tema === "claro"
  ? ".emblema{--anatomy-paper:#f3f7fd;--anatomy-face:#e3ecf8;--anatomy-top:#fff;--anatomy-shade-0:#c3d3e8;--anatomy-shade-1:#cddbee;--anatomy-shade-2:#d7e3f2;--anatomy-shade-3:#e1eaf6;--anatomy-hi:#0b2a57;--anatomy-mid:#3d6aa3;--anatomy-lo:#7a9fcb;--anatomy-faint:#bcd0e8;--anatomy-lit:#000;--anatomy-lit-top:#cfdff3;--anatomy-lit-shade:#b0c6e3;--anatomy-glow:#6d9bd6;--anatomy-dot:#0b2a57}"
  : "";
const fundo = tema === "claro" ? "#f3f7fd" : "radial-gradient(ellipse at 50% 40%,#1250a0 0,#0c3a78 62%,#082b5e 100%)";
const html = `<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:${fundo}}${css}${claro}svg{display:block}</style>
<svg id="fig" class="emblema" width="${e.largura}" height="${e.altura}" viewBox="0 0 ${e.largura} ${e.altura}">${e.svg}</svg>`;
const pagina = join(tmpdir(), `figura-${process.pid}.html`);
writeFileSync(pagina, html);

const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const depuracao = await livre();
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmpdir(), "figura-perfil-" + process.pid)}`, "about:blank"], { stdio: "ignore" });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
try {
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map();
  ws.onmessage = (m) => { const d = JSON.parse(m.data); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  await cdp("Emulation.setDeviceMetricsOverride", { width: LARG, height: ALT, deviceScaleFactor: escala, mobile: false });
  await cdp("Page.navigate", { url: pathToFileURL(pagina).href });
  await dormir(1200);
  const foto = async (arquivo, clip) => { const r = await cdp("Page.captureScreenshot", { format: "png", clip: { ...clip, scale: 1 } }); writeFileSync(arquivo, Buffer.from(r.result.data, "base64")); };
  await foto(`${saida}-inteira.png`, { x: 0, y: 0, width: LARG, height: ALT });
  for (let l = 0; l < linhas; l++) for (let c = 0; c < colunas; c++) await foto(`${saida}-${l + 1}-${c + 1}.png`, { x: (LARG / colunas) * c, y: (ALT / linhas) * l, width: LARG / colunas, height: ALT / linhas });
  console.log(`${lista.length} peças, SVG de ${(e.svg.length / 1024).toFixed(0)} KB. Pedaços em ${saida}-<linha>-<coluna>.png (${linhas}×${colunas}, escala ${escala}).`);
} finally { edge.kill(); }
