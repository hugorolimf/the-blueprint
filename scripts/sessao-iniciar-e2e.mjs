// Escolher a sessão ao iniciar a ação, com o servidor REAL (docs/claude-na-folha.md, "Escolher a sessão"): no Edge (sem janela), duas sessões abertas na pasta (hook) e um canal
// FALSO (um SSE aberto como o `blueprint --mcp` faz) fazem o diálogo oferecer "Nova sessão", a sessão com canal e a sem canal. O pedido vai ao canal pelo evento
// `iniciar-acao`, a sem canal só copia, e "Nova sessão" abre o `wt` FALSO. Nenhum terminal nem Claude de verdade.
// Uso: node scripts/sessao-iniciar-e2e.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe; precisa do Microsoft Edge). ~15 s.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dir = mkdtempSync(join(tmpdir(), "blueprint-sessao-"));
const pasta = join(dir, "projeto");
mkdirSync(pasta);
const falso = terminalFalso(dir), depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dir, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
let servidor;
const abortar = new AbortController();
const limpar = () => { abortar.abort(); try { edge.kill(); } catch {} servidor?.parar(); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const post = (rota, json) => fetch(`${servidor.url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(json) });
try {
  servidor = await sobeServidor(exe, dir, falso);
  assert.equal((await post("pasta/vincular", { cwd: pasta, workspaceId: "sm", blueprintId: "bp1a2b3" })).status, 200);
  for (const s of ["s1", "s2"]) assert.equal((await post("hook/sessao", { session_id: s, cwd: pasta })).status, 200);
  // o canal falso: o mesmo SSE que o `blueprint --mcp` com BLUEPRINT_CHANNEL=1 abre
  let recebido = "";
  const sse = await fetch(`${servidor.url}/api/mcp/eventos?canal=abc123&cwd=${encodeURIComponent(pasta)}`, { signal: abortar.signal });
  (async () => { const dec = new TextDecoder(); try { for await (const parte of sse.body) recebido += dec.decode(parte, { stream: true }); } catch {} })();
  const lista = await (await fetch(`${servidor.url}/api/mcp/sessoes?workspace=sm&blueprint=bp1a2b3`)).json();
  assert.deepEqual(lista.sessoes.map((s) => s.canal), [true, false], `a lista: 1 com canal e 1 sem (${JSON.stringify(lista)})`);

  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Browser.grantPermissions", { origin: servidor.url, permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"] });
  await cdp("Page.navigate", { url: `${servidor.url}/#/p/bp1a2b3` });
  for (let i = 0; i < 60 && !(await js(`!!document.querySelector("#claude-camada [data-espera]")`)); i++) await dormir(150);
  const aviso = () => js(`document.querySelector("#nota-folha .iniciar-aviso")?.textContent || ""`);
  const abreDialogo = async () => {
    await js(`document.querySelector("#claude-camada [data-espera]").click()`);
    await dormir(600);
    await js(`[...document.querySelectorAll("#nota-folha [data-nacao]")].find((b) => b.textContent === "Iniciar com o Claude").click()`);
    for (let i = 0; i < 40 && !(await js(`!!document.querySelector("dialog.escolher-sessao[open]")`)); i++) await dormir(100);
  };
  const escolhe = async (valor) => { await js(`document.querySelector('dialog.escolher-sessao input[value="${valor}"]').click()`); await js(`document.querySelector('dialog.escolher-sessao [data-r="1"]').click()`); };

  // 1. o diálogo oferece Nova sessão (marcada) e as duas sessões; cancelar não faz nada
  await abreDialogo();
  assert.equal(await js(`!!document.querySelector("dialog.escolher-sessao")`), true, "com sessões abertas o diálogo é o da escolha");
  assert.deepEqual(await js(`[...document.querySelectorAll('dialog.escolher-sessao input[name=sessao]')].map((r) => r.value + (r.checked ? "*" : ""))`), ["nova*", "0", "1"]);
  assert(/recebe o pedido direto/.test(await js(`document.querySelector("dialog.escolher-sessao .cu-opcoes").textContent`)) && /sem canal/.test(await js(`document.querySelector("dialog.escolher-sessao .cu-opcoes").textContent`)));
  await js(`document.querySelector('dialog.escolher-sessao [data-r="0"]').click()`);
  await dormir(400);
  assert.equal(recebido.includes("iniciar-acao"), false);
  assert.equal(falso.chamadas().length, 0, "cancelar não abre nada");

  // 2. a sessão com canal recebe o pedido pelo evento iniciar-acao; nenhum terminal abre
  await abreDialogo();
  await escolhe("0");
  for (let i = 0; i < 60 && !/Pedido enviado/.test(await aviso()); i++) await dormir(150);
  assert(/Pedido enviado à sessão/.test(await aviso()), `avisa o envio (${await aviso()})`);
  for (let i = 0; i < 40 && !recebido.includes("event: iniciar-acao"); i++) await dormir(100);
  const dados = JSON.parse(recebido.split("event: iniciar-acao\ndata: ")[1].split("\n")[0]);
  assert.equal(dados.canalId, "abc123");
  assert(dados.texto.startsWith("Execute a ação 1.2 «") && dados.texto.includes("acao_id ac1a002"), dados.texto.slice(0, 120));
  assert.equal(falso.chamadas().length, 0, "entregar a uma sessão não abre terminal");

  // 3. a sessão sem canal só copia o pedido: nenhum evento novo, nenhum terminal
  const antes = recebido.length;
  await abreDialogo();
  await escolhe("1");
  for (let i = 0; i < 60 && !/Pedido copiado|O pedido é/.test(await aviso()); i++) await dormir(150);
  assert(/Pedido copiado|O pedido é: Execute a ação 1\.2/.test(await aviso()), `copia o pedido (${await aviso()})`);
  await dormir(300);
  assert.equal(recebido.length, antes, "sem canal não emite evento");
  assert.equal(falso.chamadas().length, 0);

  // 4. Nova sessão, mesmo com sessões abertas, abre o terminal falso como sempre
  await abreDialogo();
  await escolhe("nova");
  for (let i = 0; i < 60 && !falso.chamadas().length; i++) await dormir(100);
  assert.equal(falso.chamadas().length, 1, "Nova sessão abre o terminal");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("escolher a sessão ao iniciar (servidor real, canal e terminal falsos): ok (lista com/sem canal, cancelar, entrega pelo canal, cópia sem canal, nova sessão)");
} finally { limpar(); }
