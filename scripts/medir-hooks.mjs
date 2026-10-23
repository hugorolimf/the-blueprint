// Mede o caminho HTTP real dos hooks em banco e pasta temporários, sem alterar o app do usuário.
// Uso: node scripts/medir-hooks.mjs [blueprint.exe]. Limite: p95 < 50 ms após aquecimento.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFile, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const dir = await mkdtemp(join(tmpdir(), "blueprint-hooks-"));
const porta = await new Promise((ok, falha) => {
  const s = createServer(); s.on("error", falha);
  s.listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); });
});
const url = `http://127.0.0.1:${porta}`;
await copyFile(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
const servidor = spawn(exe, ["serve"], { windowsHide: true, stdio: "ignore", env: {
  ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"),
  DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos"),
} });
let erro;
servidor.on("error", (e) => { erro = e; });
const post = async (rota, corpo) => {
  const r = await fetch(`${url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo), signal: AbortSignal.timeout(3000) });
  assert.equal(r.status, 200, `${rota}: ${await r.clone().text()}`);
  return r.json();
};
try {
  let pronto = false;
  for (let i = 0; i < 100; i++) {
    if (erro) throw erro;
    try { pronto = (await fetch(url + "/api/mcp/blueprints")).ok; } catch {}
    if (pronto) break;
    await new Promise((r) => setTimeout(r, 50));
  }
  assert(pronto, "o servidor temporário não abriu");
  const lista = await (await fetch(url + "/api/mcp/blueprints")).json();
  const ws = lista.workspaces.find((w) => w.blueprints.length);
  await post("pasta/vincular", { cwd: dir, workspaceId: ws.id, blueprintId: ws.blueprints[0].id });
  const corpo = { session_id: "medicao", cwd: dir, tool_name: "Edit", tool_input: { file_path: join(dir, "exemplo.rs") } };
  const medicoes = {};
  const parada = { session_id: "medicao", hook_event_name: "Stop", stop_hook_active: false };
  for (const [rota, enviado] of [["hook/sessao", corpo], ["hook/edicao", corpo], ["hook/parada", parada]]) {
    const tempos = [];
    for (let i = 0; i < 110; i++) {
      const inicio = performance.now();
      const r = await post(rota, enviado);
      if (rota === "hook/sessao") assert.equal(r.vinculado, true);
      if (i >= 10) tempos.push(performance.now() - inicio);
    }
    tempos.sort((a, b) => a - b);
    const n = (p) => +tempos[Math.ceil(tempos.length * p) - 1].toFixed(3);
    medicoes[rota] = { amostras: tempos.length, p50_ms: n(0.5), p95_ms: n(0.95), max_ms: n(1) };
    assert(medicoes[rota].p95_ms < 50, `${rota}: p95 excedeu 50 ms`);
  }
  assert((await (await fetch(url + "/api/mcp/fila")).json()).itens.some((i) => i.tipo === "progresso"), "edição deve registrar progresso");
  const relatorio = { data: new Date().toISOString(), executavel: exe, criterio: "p95 < 50 ms; 10 chamadas de aquecimento, 100 medidas por rota", medicoes };
  await writeFile(join(raiz, "target/medicao-hooks.json"), JSON.stringify(relatorio, null, 2) + "\n");
  console.log(JSON.stringify(relatorio, null, 2));
} finally {
  const fim = new Promise((r) => servidor.once("exit", r));
  if (servidor.exitCode === null && !erro) { servidor.kill(); await fim; }
  await rm(dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}
