// A impressão da apresentação em A4 (web/css/impressao.css) e a exportação num arquivo só (web/js/apresentacao-exportar.js).
// Abre o documento de exemplo no Edge sem janela (um servidor estático temporário serve a pasta do projeto; o app não é usado), imprime em PDF pelo DevTools e confere:
// 7 páginas A4 em pé, o rodapé "n / 7" (se o pdftotext do Git estiver à mão), o arquivo exportado autocontido (sem rede, sem script, imagem embutida) que ele imprime as mesmas 7 páginas e que páginas 16:9 saem uma por folha A4 deitada. ~10 s, sem tokens.
//   node scripts/apresentacao-pdf-e2e.mjs
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createServer } from "node:http";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { extname, join, normalize, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const EDGE = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(EDGE, "precisa do Microsoft Edge");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const TIPOS = { ".html": "text/html; charset=utf-8", ".css": "text/css", ".js": "text/javascript", ".png": "image/png", ".svg": "image/svg+xml" };

let exportado = ""; // o arquivo exportado, servido em /__exportado.html para ser impresso como se fosse aberto no navegador
const servidor = createServer((req, res) => {
  const caminho = decodeURIComponent(new URL(req.url, "http://x").pathname);
  if (caminho === "/__exportado.html") { res.setHeader("content-type", TIPOS[".html"]); return res.end(exportado); }
  const arq = normalize(join(raiz, caminho));
  if (!arq.startsWith(raiz) || !existsSync(arq) || caminho.endsWith("/")) { res.statusCode = 404; return res.end(); }
  res.setHeader("content-type", TIPOS[extname(arq)] || "application/octet-stream"); res.end(readFileSync(arq));
});
await new Promise((r) => servidor.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${servidor.address().port}`;

const tmp = mkdtempSync(join(tmpdir(), "ap-pdf-"));
const depuracao = 9300 + Math.floor(Math.random() * 500);
const edge = spawn(EDGE, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1000,900", "about:blank"], { stdio: "ignore" });
const encerrar = () => { try { edge.kill(); } catch {} servidor.close(); setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); };

try {
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  assert(alvo, "o Edge não abriu a porta de depuração");
  const sock = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (sock.onopen = r));
  let n = 0; const pend = new Map();
  sock.onmessage = (m) => { const d = JSON.parse(m.data); if (!d.method) { pend.get(d.id)?.(d); pend.delete(d.id); } };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); sock.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 500)); return r.result.result.value; };
  const ir = async (url) => { await cdp("Page.navigate", { url }); await dormir(1200); };
  const imprimir = async (nome) => { // A4 vem do @page do CSS
    const r = await cdp("Page.printToPDF", { preferCSSPageSize: true, printBackground: true });
    assert(r.result?.data, "o Edge não devolveu o PDF: " + JSON.stringify(r.error));
    const arq = join(tmp, nome); writeFileSync(arq, Buffer.from(r.result.data, "base64")); return arq;
  };
  const paginasDoPdf = (arq) => {
    const t = readFileSync(arq).toString("latin1");
    const caixas = [...t.matchAll(/\/MediaBox\s*\[\s*0\s+0\s+([\d.]+)\s+([\d.]+)\s*\]/g)].map((m) => [Number(m[1]), Number(m[2])]);
    return { n: [...t.matchAll(/\/Type\s*\/Page(?![s\w])/g)].length, caixas };
  };
  const texto = (arq) => {
    for (const exe of ["pdftotext", "C:/Program Files/Git/mingw64/bin/pdftotext.exe"]) {
      const r = spawnSync(exe, ["-layout", arq, "-"], { encoding: "utf8" });
      if (r.status === 0) return r.stdout;
    }
    return null;
  };
  await cdp("Runtime.enable"); await cdp("Page.enable");

  // 1) o documento de exemplo, impresso direto: 5 tipos de página + uma ação longa que escorre para a página seguinte = 7 páginas A4
  await ir(`${base}/scripts/fixtures/apresentacao-exemplo.html`);
  assert.equal(await js(`document.querySelectorAll(".ap-pagina").length`), 6, "o exemplo tem 6 seções");
  const direto = paginasDoPdf(await imprimir("direto.pdf"));
  assert.equal(direto.n, 7, `6 seções, a ação longa ocupa 2 páginas: 7 no PDF (veio ${direto.n})`);
  for (const [l, a] of direto.caixas) assert(Math.abs(l - 595.28) < 1.5 && Math.abs(a - 841.89) < 1.5, `página A4 (veio ${l} x ${a} pt)`);
  const lido = texto(join(tmp, "direto.pdf"));
  if (lido) {
    for (let i = 1; i <= 7; i++) assert(new RegExp(String.raw`\b${i}\s*/\s*7\b`).test(lido), `rodapé "${i} / 7"`);
    assert.equal((lido.match(/Sample plan \d\d/g) || []).length, 40, "os 40 planos aparecem uma vez cada (nenhum cortado ou repetido)");
  } else console.log("  (pdftotext não encontrado: rodapé e planos não conferidos)");

  // 2) exportar: um arquivo só, sem rede, sem script
  const r = await js(`(async () => { const x = await exportarApresentacao(document.getElementById("doc"), { formato: "html", baixar: false, estilos: document.querySelector("style").textContent, folhas: ["tokens", "marca", "claude", "impressao"] }); return { html: x.html, nome: x.nome }; })()`);
  assert.equal(r.nome, "sample-blueprint.html", "o nome do arquivo vem do título da capa");
  assert.equal((r.html.match(/<section class="ap-pagina"/g) || []).length, 6, "as 6 páginas estão no arquivo");
  assert(!/<link\b|<script\b/i.test(r.html), "nada de <link> nem <script>");
  assert(!/(?:src|href)=["']?https?:|url\(\s*["']?https?:/i.test(r.html), "nenhum endereço de rede");
  assert(r.html.includes("Content-Security-Policy") && r.html.includes("@page") && r.html.includes("--paper"), "traz a política, o CSS de impressão e os tokens");

  // imagem embutida (o arquivo não pode depender de nada fora dele) e erro claro sem páginas
  const img = await js(`(async () => (await exportarApresentacao('<section class="ap-pagina"><h2 class="ap-titulo">T</h2><img src="/claude-plugin/plugins/blueprint-terminal/assets/marca.png"></section>', { baixar: false })).html)()`);
  assert(/<img src="data:image\/png;base64,/.test(img), "a imagem virou data:");
  const vazio = await js(`exportarApresentacao("<p>nada</p>", { baixar: false }).then(() => "passou", (e) => e.message)`);
  assert.equal(vazio, "A apresentação não tem páginas.", "sem páginas, erro claro");

  // 3) o arquivo exportado, aberto no navegador, imprime as mesmas 7 páginas
  exportado = r.html;
  await ir(`${base}/__exportado.html`);
  const avulso = paginasDoPdf(await imprimir("exportado.pdf"));
  assert.equal(avulso.n, direto.n, `o arquivo exportado imprime ${direto.n} páginas (veio ${avulso.n})`);

  // 4) páginas 16:9 (as de apresentacao.css): o exportador marca como ap-slide e elas imprimem deitadas, uma por folha A4
  await ir(`${base}/scripts/fixtures/apresentacao-slides.html`);
  const slides = await js(`(async () => (await exportarApresentacao(document.getElementById("doc"), { baixar: false, estilos: document.querySelector("style").textContent, folhas: ["tokens", "impressao"] })).html)()`);
  assert.equal((slides.match(/class="ap-pagina ap-slide"/g) || []).length, 2, "as duas páginas 16:9 ganharam ap-slide");
  exportado = slides;
  await ir(`${base}/__exportado.html`);
  const deitado = paginasDoPdf(await imprimir("slides.pdf"));
  assert.equal(deitado.n, 2, `2 slides, 2 folhas (veio ${deitado.n})`);
  for (const [l, a] of deitado.caixas) assert(Math.abs(l - 841.89) < 1.5 && Math.abs(a - 595.28) < 1.5, `A4 deitado (veio ${l} x ${a} pt)`);
  console.log(`ok: ${direto.n} páginas A4 no exemplo e no arquivo exportado (${(r.html.length / 1024).toFixed(0)} KB); 2 slides em A4 deitado`);
} finally { encerrar(); }
