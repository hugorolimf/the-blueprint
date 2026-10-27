// A interface da atualização no navegador de verdade (Edge sem janela) contra o app Rust e um "GitHub" falso: o aviso "pronta" aparece, "Depois" some, o clique em Atualizar com item
// na fila do MCP roda o instalador (anotado, não executado): a fila fica no banco e não impede mais. Opcional: `--captura <arquivo.png>` salva uma imagem do aviso.
// Uso: node scripts/atualizacao-ui-e2e.mjs [blueprint.exe] [--captura arquivo.png]   (precisa do Microsoft Edge; ~10 s, sem gastar tokens)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { dormir, exePadrao, http, ligarServidor, pastaDeDados, portaLivre, postJson } from "./lib-e2e-rust.mjs";

const argv = process.argv.slice(2).filter((a) => !a.endsWith(".exe"));
const exe = exePadrao();
const captura = argv.includes("--captura") ? argv[argv.indexOf("--captura") + 1] : null;
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const instalador = Buffer.from("MZ instalador de mentira ".repeat(400)), nome = "Blueprint-0.9.0-instalador.exe";
const portaGh = await portaLivre(), base = `http://127.0.0.1:${portaGh}`, hash = createHash("sha256").update(instalador).digest("hex");
const gh = createServer((req, res) => {
  if (req.url.startsWith("/releases")) return res.end(JSON.stringify([{ tag_name: "v0.9.0", draft: false, prerelease: true, body: "Baixa em segundo plano e só instala no clique.\nOutra linha das notas.", assets: [{ name: nome, size: instalador.length, browser_download_url: `${base}/b/${nome}` }, { name: "SHA256SUMS", size: 99, browser_download_url: `${base}/b/SHA256SUMS` }] }]));
  if (req.url === `/b/${nome}`) return res.end(instalador);
  if (req.url === "/b/SHA256SUMS") return res.end(`${hash}  ${nome}\n`);
  res.statusCode = 404; res.end();
});
await new Promise((ok) => gh.listen(portaGh, "127.0.0.1", ok));
const lancou = join(tmpdir(), `blueprint-ui-lancou-${process.pid}.txt`);
const d = await pastaDeDados("blueprint-ui-atualizacao", { BLUEPRINT_ATUALIZACAO_URL: base, BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR: lancou });
const depuracao = await portaLivre(), perfil = mkdtempSync(join(tmpdir(), "blueprint-ui-perfil-"));
const servidor = await ligarServidor(exe, d);
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${perfil}`, "--window-size=1280,800", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} try { gh.close(); } catch {} setTimeout(() => { try { rmSync(perfil, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); rmSync(lancou, { force: true }); } catch {} }, 500); };
process.on("exit", limpar);
try {
  await postJson(d.url + "/api/atualizacao/verificar");
  for (let i = 0; i < 100 && (await http(d.url + "/api/atualizacao")).json.fase !== "pronta"; i++) await dormir(100);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(), erros = [];
  ws.onmessage = (m) => { const x = JSON.parse(m.data); if (x.method === "Runtime.exceptionThrown") erros.push(x.params.exceptionDetails.exception?.description || x.params.exceptionDetails.text); pend.get(x.id)?.(x); pend.delete(x.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 600)); return r.result.result.value; };
  const caixa = () => js(`(() => { const e = document.getElementById("atualizacao"); return e && { visivel: !e.hidden, titulo: e.querySelector(".at-t").textContent, notas: e.querySelector(".at-notas").textContent, acoes: !e.querySelector(".at-acoes").hidden, motivo: e.querySelector(".at-motivo").hidden ? "" : e.querySelector(".at-motivo").textContent }; })()`);
  await cdp("Runtime.enable");
  await cdp("Page.navigate", { url: d.url + "/" });
  for (let i = 0; i < 60 && !(await caixa())?.visivel; i++) await dormir(150);
  let c = await caixa();
  assert(c?.visivel, "o aviso da versão pronta aparece");
  assert.match(c.titulo, /A versão 0\.9\.0 está pronta/);
  assert.match(c.notas, /Baixa em segundo plano/);
  assert.equal(c.acoes, true);
  if (captura) { const img = await cdp("Page.captureScreenshot", { format: "png" }); writeFileSync(captura, Buffer.from(img.result.data, "base64")); console.log(`captura: ${captura}`); }
  assert.equal(existsSync(lancou), false, "só aparecer não instala");

  // "Depois" some; recarregar a página continua adiado (24 h para essa versão)
  await js(`document.querySelector("#atualizacao .at-depois").click()`);
  assert.equal((await caixa()).visivel, false);
  await cdp("Page.navigate", { url: d.url + "/" });
  await dormir(1500);
  assert.equal((await caixa())?.visivel ?? false, false, "adiado continua adiado depois de recarregar");
  await js(`localStorage.clear()`);
  await cdp("Page.navigate", { url: d.url + "/" });
  for (let i = 0; i < 60 && !(await caixa())?.visivel; i++) await dormir(150);
  assert((await caixa()).visivel);

  // item na fila do MCP não impede (a fila fica no banco e volta ao reabrir): o clique roda o instalador (anotado) e o aviso vira "Atualizando".
  // O motivo de uma recusa na tela (terminal com sessão viva, compartilhamento) é conferido em atualizacao.test.mjs.
  assert.equal((await postJson(d.url + "/api/mcp/evento", { tipo: "progresso", texto: "trabalhando", workspaceId: "sm", blueprintId: "p1" })).status, 200);
  await js(`document.querySelector("#atualizacao .at-ok").click()`);
  for (let i = 0; i < 40 && !existsSync(lancou); i++) await dormir(100);
  assert(existsSync(lancou), "o clique rodou o instalador");
  assert.match(readFileSync(lancou, "utf8"), /Blueprint-0\.9\.0-instalador\.exe .*\/ATUALIZACAO=1/);
  for (let i = 0; i < 40 && !/Atualizando/.test((await caixa()).titulo); i++) await dormir(100);
  assert.match((await caixa()).titulo, /Atualizando: o app fecha e abre de novo sozinho/);
  assert.deepEqual(erros, [], "sem erros de JavaScript na página");
  console.log("ok: aviso pronta, Depois (e continua adiado), clique que instala com item na fila");
} finally {
  limpar();
}
