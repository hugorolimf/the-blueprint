// O botão "Iniciar com o Claude" com a rota REAL: no Edge (sem janela), o clique no Claude esperando abre a folha da ação, o botão pede confirmação e chama o servidor de verdade, que
// abre o `wt` FALSO (nenhum terminal real) com o prompt da ação; sem pasta ligada a folha explica onde ligar; cancelar não chama nada.
// Uso: node scripts/iniciar-botao-e2e.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe; precisa do Microsoft Edge). Captura em target/iniciar-botao.png.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dir = mkdtempSync(join(tmpdir(), "blueprint-botao-"));
const pasta = join(dir, "projeto");
mkdirSync(pasta);
const falso = terminalFalso(dir), depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dir, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
let servidor;
const limpar = () => { try { edge.kill(); } catch {} servidor?.parar(); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500); }; // o Edge pode segurar o perfil um instante: lixo na pasta temporária não é falha
process.on("exit", limpar);
const liga = (blueprintId) => fetch(`${servidor.url}/api/mcp/pasta/vincular`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ cwd: pasta, workspaceId: "sm", blueprintId }) });
try {
  servidor = await sobeServidor(exe, dir, falso);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");
  const abreEspera = async (bp) => { // abre o blueprint, espera o Claude esperando aparecer e clica nele: abre a folha da ação
    await cdp("Page.navigate", { url: `${servidor.url}/#/p/${bp}` });
    for (let i = 0; i < 60 && !(await js(`!!document.querySelector("#claude-camada [data-espera]")`)); i++) await dormir(150);
    assert.equal(await js(`!!document.querySelector("#claude-camada [data-espera]")`), true, "o Claude esperando aparece na próxima ação");
    await js(`document.querySelector("#claude-camada [data-espera]").click()`);
    await dormir(800);
  };
  const botao = `[...document.querySelectorAll("#nota-folha [data-nacao]")].find((b) => b.textContent === "Iniciar com o Claude")`;
  const aviso = () => js(`document.querySelector("#nota-folha .iniciar-aviso")?.textContent || ""`);

  // sem pasta ligada: cancelar não chama nada; confirmar mostra onde ligar
  await abreEspera("bp1a2b3");
  assert.equal(await js(`!!${botao}`), true, "a folha da ação tem o botão");
  await js(`${botao}.click()`);
  await dormir(500);
  assert.equal(await js(`!!document.querySelector("dialog.confirmar[open]")`), true, "pede confirmação antes de abrir o terminal");
  await js(`document.querySelector('dialog.confirmar [data-r="0"]').click()`);
  await dormir(500);
  assert.equal(falso.chamadas().length, 0, "cancelar não abre terminal");
  await js(`${botao}.click()`);
  await dormir(500);
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  await dormir(1200);
  assert(/Pasta do projeto/.test(await aviso()), `sem pasta ligada, explica onde ligar: ${await aviso()}`);
  assert.equal(falso.chamadas().length, 0, "e nenhum terminal abre");

  // com a pasta ligada: confirmar chama a rota real, que abre o terminal falso com o prompt da ação
  assert.equal((await liga("bp1a2b3")).status, 200);
  await abreEspera("bp1a2b3");
  await js(`${botao}.click()`);
  await dormir(500);
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  for (let i = 0; i < 60 && !/Terminal aberto/.test(await aviso()); i++) await dormir(150);
  assert(/Terminal aberto/.test(await aviso()), `confirmou: a folha avisa que o terminal abriu (${await aviso()})`);
  for (let i = 0; i < 60 && !falso.chamadas().length; i++) await dormir(100);
  const [chamada, ...resto] = falso.chamadas();
  assert.equal(resto.length, 0, "uma só chamada ao terminal");
  assert(chamada[4].startsWith("Execute a ação 1.2 «") && chamada[4].includes("acao_id ac1a002"), `o terminal recebeu o prompt da ação do Claude esperando: ${chamada[4].slice(0, 100)}`);
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "iniciar-botao.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("botão Iniciar com o Claude (rota real, terminal falso): ok (esperando abre a folha, confirmação, cancelar não abre, sem pasta explica, com pasta abre o terminal com o prompt)");
} finally { limpar(); }
