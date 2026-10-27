// A tela "Gerar com IA" (Apresentação) no Edge de verdade: o grupo no trilho (público e nível, sem Épico), o pedido ao agente, a lista das apresentações geradas,
// o visor isolado (iframe sandbox sem allow-same-origin), Gerar de novo, Exportar e Remover, em pt-BR e em inglês, e a 390 px sem rolagem horizontal.
// As rotas novas do servidor (/api/mcp/apresentar, /api/mcp/apresentacoes, /api/mcp/apresentacao/remover, /apresentacao/<id>) são testadas em Rust (cargo test): aqui um
// proxy em Node as FINGE na frente do `blueprint serve` real (que serve a interface desta pasta por BLUEPRINT_ASSETS_DIR), para provar só a tela. O evento SSE
// "apresentacao" é simulado chamando apIaRecebe na página.
// Uso: node scripts/apresentacao-ia-e2e.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe; precisa do Microsoft Edge). Capturas em target/apresentacao-ia-*.png.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createServer, request } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const BP = "bp1a2b3";
const APRESENTACOES = [
  { id: "ap-antiga", titulo: "Plano do cliente", publico: "cliente", nivel: "rapido", em: "2026-10-20T10:00:00Z" },
  { id: "ap-nova", titulo: "Visão técnica", publico: "interno", nivel: "completo", em: "2026-10-26T09:00:00Z" },
];
const [porta, portaProxy, depuracao] = [await portaLivre(), await portaLivre(), await portaLivre()];
const tmp = mkdtempSync(join(tmpdir(), "blueprint-ap-ia-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1" }, stdio: "ignore", windowsHide: true });
const pedidos = [], remocoes = [], removidas = new Set();
const lerCorpo = (req) => new Promise((ok) => { let t = ""; req.on("data", (c) => (t += c)); req.on("end", () => ok(t)); });
const proxy = createServer(async (req, res) => {
  const url = new URL(req.url, "http://x");
  const json = (obj, status = 200) => { res.writeHead(status, { "content-type": "application/json" }); res.end(JSON.stringify(obj)); };
  if (req.method === "POST" && url.pathname === "/api/mcp/apresentar") { const c = JSON.parse(await lerCorpo(req)); pedidos.push(c); return json({ ok: true }); }
  if (req.method === "POST" && url.pathname === "/api/mcp/apresentacao/remover") { const c = JSON.parse(await lerCorpo(req)); remocoes.push(c); removidas.add(c.apresentacaoId); return json({ ok: true }); }
  if (req.method === "GET" && url.pathname === "/api/mcp/apresentacoes") return json({ apresentacoes: APRESENTACOES.filter((a) => !removidas.has(a.id)).sort((a, b) => (a.em < b.em ? 1 : -1)) });
  const mine = /^\/apresentacao\/([\w-]+)$/.exec(url.pathname);
  if (mine) { // a página de mentira, com os mesmos cabeçalhos de isolamento que a real terá
    res.writeHead(200, { "content-type": "text/html; charset=utf-8", "content-security-policy": "sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:", ...(url.searchParams.get("baixar") ? { "content-disposition": "attachment" } : {}) });
    return res.end(`<!doctype html><meta charset="utf-8"><title>${mine[1]}</title><body style="background:#0c3a78;color:#eaf4ff;font:16px monospace"><h1>apresentação falsa ${mine[1]}</h1>`);
  }
  const alvo = request({ host: "127.0.0.1", port: porta, path: req.url, method: req.method, headers: { ...req.headers, host: `127.0.0.1:${porta}`, ...(req.headers.origin ? { origin: `http://127.0.0.1:${porta}` } : {}) } }, (r) => {
    res.writeHead(r.statusCode, r.headers);
    r.pipe(res);
  });
  alvo.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); }); // ao desligar, o SSE do servidor cai no meio
  req.pipe(alvo);
});
await new Promise((ok) => proxy.listen(portaProxy, "127.0.0.1", ok));
const url = `http://127.0.0.1:${portaProxy}`;
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} proxy.close(); setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(`http://127.0.0.1:${porta}/api/dados`); break; } catch { await dormir(50); } }
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  const espera = async (expr, quanto = 8000) => { for (let i = 0; i < quanto / 100; i++) { if (await js(expr)) return true; await dormir(100); } return false; };
  const foto = async (nome) => { mkdirSync(join(raiz, "target"), { recursive: true }); writeFileSync(join(raiz, "target", `apresentacao-ia-${nome}.png`), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); };
  const clica = (sel) => js(`document.querySelector(${JSON.stringify(sel)}).click()`);
  const texto = (sel) => js(`document.querySelector(${JSON.stringify(sel)})?.textContent || ""`);
  await cdp("Runtime.enable");
  await cdp("Page.enable");

  const abre = async (extra = "") => { // abre o blueprint e a tela Apresentação
    await cdp("Page.navigate", { url: `${url}/${extra}#/p/${BP}` });
    assert(await espera(`!!document.getElementById("b-apresentar") && typeof D !== "undefined" && D.blueprints?.length > 0`), "o blueprint abriu");
    await dormir(500);
    await clica("#b-apresentar");
    assert(await espera(`!!document.querySelector("#apresentacao-tela:not([hidden]) .ap-ia")`), "o grupo Gerar com IA aparece no trilho");
  };
  const radios = (grupo) => js(`[...document.querySelectorAll('input[name="ap-ia-${grupo}"]')].map((i) => i.value + (i.checked ? "*" : ""))`);

  // ---- português: o grupo, o padrão, sem Épico, com a estimativa de cada nível ----
  await abre();
  assert(await espera(`document.querySelectorAll(".ap-ia-item").length === 2`), "a lista vem do servidor");
  assert.deepEqual(await radios("publico"), ["cliente*", "interno"], "público: Cliente e Interno, Cliente por padrão");
  assert.deepEqual(await radios("nivel"), ["rapido", "padrao*", "completo"], "nível: Rápido, Padrão e Completo, sem Épico");
  const grupo = await texto(".ap-ia");
  for (const t of ["Gerar com IA", "Público", "Nível", "Cliente", "Interno", "até 1 h", "1 a 3 h", "2 a 4 h", "Apresentações geradas"]) assert(grupo.includes(t), `o grupo mostra «${t}»`);
  assert(!/pico/i.test(grupo), "o Épico nunca aparece na tela");
  assert.equal(await js(`document.querySelectorAll(".ap-tipo").length`), 3, "a tela antiga segue com os três tipos de apresentação");
  assert.deepEqual(await js(`[...document.querySelectorAll(".ap-ia-item b")].map((b) => b.textContent)`), ["Visão técnica", "Plano do cliente"], "a lista vem da mais nova para a mais antiga");
  await foto("desktop");

  // ---- o visor isolado, Exportar e Gerar de novo ----
  await clica('[data-ia-ver="ap-antiga"]');
  assert(await espera(`!!document.querySelector(".ap-ia-quadro")`), "abre o visor");
  assert.equal(await js(`document.querySelector(".ap-ia-quadro").getAttribute("sandbox")`), "allow-scripts", "iframe isolado: só allow-scripts, nunca allow-same-origin");
  assert.equal(await js(`document.querySelector(".ap-ia-quadro").getAttribute("src")`), "/apresentacao/ap-antiga");
  await dormir(600);
  const alvos = (await cdp("Target.getTargets")).result.targetInfos; // o quadro de origem opaca pode ir para outro processo: aparece como alvo próprio
  assert(alvos.some((t) => t.url.endsWith("/apresentacao/ap-antiga") && t.type === "iframe"), `o quadro carregou a página isolada: ${JSON.stringify(alvos.map((t) => [t.type, t.url.slice(-40)]))}`);
  assert.equal(await js(`document.querySelector('.ap-ia-barra a[download]').getAttribute("href")`), "/apresentacao/ap-antiga?baixar=1", "Exportar HTML baixa o arquivo");
  assert.equal(await js(`document.querySelector('.ap-ia-item[aria-pressed="true"] b').textContent`), "Plano do cliente");
  await foto("visor");
  await clica('[data-ia="refazer"]'); // Gerar de novo: pré-seleciona o público e o nível dela e abre o diálogo
  assert(await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`), "Gerar de novo pede a confirmação");
  assert.deepEqual(await radios("publico"), ["cliente*", "interno"]);
  assert.deepEqual(await radios("nivel"), ["rapido*", "padrao", "completo"], "o nível da apresentação aberta vem marcado");
  assert.equal(await js(`!!document.querySelector('dialog.escolher-sessao input[value="app"]')`), false, "sem a opção Terminal do app");
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "nova");
  await clica('dialog.escolher-sessao [data-r="0"]');
  await dormir(300);
  assert.equal(pedidos.length, 0, "cancelar não pede nada");

  // ---- o agente anexou outra apresentação: o evento SSE "apresentacao" refaz a lista, sem recarregar a tela ----
  APRESENTACOES.push({ id: "ap-extra", titulo: "Nova do agente", publico: "cliente", nivel: "padrao", em: "2026-10-27T08:00:00Z" });
  await js(`apIaRecebe({ blueprintId: "${BP}", apresentacaoId: "ap-extra" })`);
  assert(await espera(`document.querySelectorAll(".ap-ia-item").length === 3 && document.querySelector(".ap-ia-item b").textContent === "Nova do agente"`), "a lista ganha a nova no topo");
  await js(`apIaRecebe({ blueprintId: "outro", apresentacaoId: "x" })`);
  APRESENTACOES.pop();

  // ---- o diálogo de sempre (Iniciar com o Claude) não mudou: sem `semApp` o Terminal do app segue como padrão ----
  await js(`void escolherSessao({ titulo: "t", texto: "x", sessoes: [] })`);
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "app", "sem semApp o padrão continua sendo o Terminal do app");
  await clica('dialog.escolher-sessao [data-r="0"]');
  await dormir(200);

  // ---- Gerar: o pedido sai com público e nível, só depois do clique de confirmação ----
  await clica('[data-ia="voltar"]');
  assert(await espera(`!document.querySelector(".ap-ia-quadro")`), "Ver as páginas volta às páginas do app");
  await clica('input[name="ap-ia-publico"][value="interno"]');
  await clica('input[name="ap-ia-nivel"][value="completo"]');
  await clica('[data-ia="gerar"]');
  assert(await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`), "Gerar abre o diálogo");
  const dialogo = await texto("dialog.escolher-sessao");
  assert(dialogo.includes("Gasta tokens.") && dialogo.includes("Público: Interno. Nível: Completo (2 a 4 h)."), `o diálogo diz o que vai gastar e o que foi escolhido: ${dialogo.slice(0, 160)}`);
  assert.equal(pedidos.length, 0, "nada sai antes de confirmar");
  await clica('dialog.escolher-sessao [data-r="1"]');
  assert(await espera(`/Terminal aberto/.test(document.querySelector(".ap-ia-msg").textContent)`), "avisa o resultado do pedido");
  assert.deepEqual(pedidos, [{ workspaceId: "sm", blueprintId: BP, publico: "interno", nivel: "completo" }], "o corpo do pedido");
  assert.equal(await js(`document.querySelector(".ap-ia-msg").classList.contains("erro")`), false);

  // ---- a escolha é lembrada; remover pede confirmação e atualiza a lista ----
  await abre();
  assert.deepEqual(await radios("publico"), ["cliente", "interno*"], "a escolha foi lembrada");
  assert.deepEqual(await radios("nivel"), ["rapido", "padrao", "completo*"]);
  await clica('[data-ia-ver="ap-nova"]');
  assert(await espera(`!!document.querySelector(".ap-ia-quadro")`));
  await clica('[data-ia="remover"]');
  assert(await espera(`!!document.querySelector("dialog.confirmar[open]")`), "remover pede confirmação");
  await clica('dialog.confirmar [data-r="0"]');
  await dormir(300);
  assert.equal(remocoes.length, 0, "cancelar não remove");
  await clica('[data-ia="remover"]');
  await espera(`!!document.querySelector("dialog.confirmar[open]")`);
  await clica('dialog.confirmar [data-r="1"]');
  assert(await espera(`!document.querySelector(".ap-ia-quadro") && document.querySelectorAll(".ap-ia-item").length === 1`), "o visor fecha e a lista perde a removida");
  assert.deepEqual(remocoes, [{ workspaceId: "sm", blueprintId: BP, apresentacaoId: "ap-nova" }]);
  assert(/removida/.test(await texto(".ap-ia-msg")));

  // ---- 390 px: sem rolagem horizontal ----
  await cdp("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 1, mobile: false });
  await dormir(400);
  assert.equal(await js(`document.documentElement.scrollWidth <= 390 && document.getElementById("apresentacao-tela").scrollWidth <= 390`), true, "sem rolagem horizontal a 390 px");
  await clica('[data-ia-ver="ap-antiga"]');
  await espera(`!!document.querySelector(".ap-ia-quadro")`);
  assert.equal(await js(`document.documentElement.scrollWidth <= 390 && document.querySelector(".ap-ia-barra").scrollWidth <= 390`), true, "o visor também cabe a 390 px");
  await foto("mobile");
  await cdp("Emulation.clearDeviceMetricsOverride");

  // ---- inglês: os mesmos controles, os textos do catálogo ----
  await abre("?idioma=en");
  const en = await texto(".ap-ia");
  for (const t of ["Generate with AI", "Audience", "Level", "Client", "Internal", "Quick", "Standard", "Complete", "up to 1 h", "1 to 3 h", "2 to 4 h", "Generated presentations"]) assert(en.includes(t), `em inglês o grupo mostra «${t}»`);
  assert.equal(await texto('[data-ia="gerar"]'), "Generate");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("apresentacao-ia-e2e: ok (grupo sem Épico, estimativas, diálogo sem Terminal do app, pedido, visor com sandbox, Exportar, Gerar de novo, Remover, escolha lembrada, 390 px, en)");
} finally { limpar(); }
