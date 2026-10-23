// Monitor real: banco temporário, eventos filtrados, app fechado e reaberto em menos de um minuto.
import assert from "node:assert/strict";
import { spawn, execFileSync } from "node:child_process";
import { mkdtempSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve, join } from "node:path";
import { once } from "node:events";

const exe = resolve(process.env.BLUEPRINT_EXE || "target/debug/blueprint.exe");
const pasta = mkdtempSync(join(tmpdir(), "blueprint-monitor-"));
const env = { ...process.env, BLUEPRINT_CHANNEL: "", HOST: "127.0.0.1", PORT: "0", BLUEPRINT_DB: join(pasta, "teste.db"), DADOS_ARQ: join(pasta, "data.json"), PROTOTIPOS_DIR: join(pasta, "prototipos") };
writeFileSync(env.DADOS_ARQ, JSON.stringify({ empresa: "Teste", blueprints: [{ id: "bp", nome: "Teste", acoes: [] }, { id: "outro", nome: "Outro", acoes: [] }] }));
let app, monitor, url, linhas = "";
const esperar = (ms) => new Promise((r) => setTimeout(r, ms));
async function ate(condicao, prazo = 12000) {
  const inicio = Date.now();
  while (!condicao()) { assert.ok(Date.now() - inicio < prazo, `tempo esgotado; saída: ${linhas}`); await esperar(50); }
}
async function abrir() {
  app = spawn(exe, ["serve"], { cwd: pasta, env, windowsHide: true });
  let texto = "";
  app.stdout.on("data", (b) => texto += b);
  await ate(() => /http:\/\/127\.0\.0\.1:\d+/.test(texto));
  url = texto.match(/http:\/\/127\.0\.0\.1:\d+/)[0];
  env.PORT = new URL(url).port;
}
async function post(rota, corpo) {
  const r = await fetch(`${url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
  assert.equal(r.status, 200, await r.text());
}
async function fechar(proc) { if (proc && proc.exitCode === null) { proc.kill(); await once(proc, "exit"); } }
function cpu() {
  return Number(execFileSync("powershell.exe", ["-NoProfile", "-Command", `(Get-Process -Id ${monitor.pid}).TotalProcessorTime.TotalMilliseconds`], { encoding: "utf8", windowsHide: true }).trim().replace(",", "."));
}
try {
  await abrir();
  await post("pasta/vincular", { cwd: pasta, workspaceId: "sm", blueprintId: "bp" });
  monitor = spawn(exe, ["watch"], { cwd: pasta, env: { ...env, BLUEPRINT_URL: url }, windowsHide: true });
  monitor.stdout.on("data", (b) => linhas += b);
  await esperar(500);
  await post("aviso", { tipo: "proposta-aplicada", workspaceId: "sm", blueprintId: "outro" });
  await esperar(200);
  assert.equal(linhas, "");
  await post("aviso", { tipo: "proposta-aplicada", workspaceId: "sm", blueprintId: "bp" });
  await ate(() => linhas.includes("aplicou uma proposta"));
  assert.equal(linhas.trim().split("\n").length, 1);
  await fechar(app);
  const antes = process.platform === "win32" ? cpu() : 0;
  await esperar(4000);
  const gasto = process.platform === "win32" ? cpu() - antes : null;
  assert.equal(monitor.exitCode, null);
  if (gasto !== null) assert.ok(gasto <= 100, `CPU fechada: ${gasto}ms em4s`);
  const recebidas = linhas.trim().split("\n").length;
  await abrir();
  // Espera a primeira reconexão; sinais idênticos no mesmo lote SSE podem ser agrupados.
  for (let i = 0; i < 20 && linhas.trim().split("\n").length === recebidas; i++) {
    await post("aviso", { tipo: "proposta-aplicada", workspaceId: "sm", blueprintId: "bp" });
    await esperar(500);
  }
  assert.ok(linhas.trim().split("\n").length > recebidas, "monitor não reconectou");
  console.log(`ok: SSE real, filtro, fechamento, reconexão; CPU com app fechado: ${gasto ?? "não medida"}ms em4s`);
} finally {
  await fechar(monitor);
  await fechar(app);
  rmSync(pasta, { recursive: true, force: true });
}
