// Fumaça do app Rust de verdade: liga `blueprint serve` com uma cópia do data.json, confere a página, os dados, o MCP por HTTP e o MCP por stdio.
// Uso: node scripts/fumaca-rust.mjs [caminho do blueprint.exe]   (padrão: target/release/blueprint.exe)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { mkdtemp, copyFile, readFile, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dir = await mkdtemp(join(tmpdir(), "blueprint-fumaca-"));
const porta = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const url = `http://127.0.0.1:${porta}`;
await copyFile(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
const original = await readFile(join(dir, "data.json"), "utf8");
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const t0 = Date.now();
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await new Promise((r) => setTimeout(r, 50)); } }
  console.log(`servidor respondeu em ${Date.now() - t0} ms`);
  const dados = await (await fetch(url + "/api/dados")).json();
  assert.deepEqual(dados, JSON.parse(original), "GET /api/dados devolve o data.json importado, sem perda");
  const pagina = await fetch(url + "/");
  assert.equal(pagina.status, 200);
  assert((await pagina.text()).includes('id="login-form"'));
  for (const f of ["/emblemas.mjs", "/kit/iso-kit.mjs", "/web/js/inicio.js", "/web/css/base.css"]) assert.equal((await fetch(url + f)).status, 200, f);
  for (const f of ["/server.mjs", "/data.json", "/web/../server.mjs"]) assert.equal((await fetch(url + f)).status, 404, f);
  assert.equal((await readFile(join(dir, "data.json"), "utf8")), original, "o data.json original não foi tocado");

  const lista = await (await fetch(url + "/api/mcp/blueprints")).json();
  const alvo = lista.workspaces.find((w) => w.blueprints.length);
  console.log(`workspaces: ${lista.workspaces.map((w) => `${w.nome}(${w.blueprints.length})`).join(", ")}`);

  // MCP por stdio, como o agente faz
  const mcp = spawn(exe, ["--mcp"], { env: { ...env, BLUEPRINT_URL: url }, stdio: ["pipe", "pipe", "inherit"], windowsHide: true });
  const respostas = new Map();
  let buf = "";
  mcp.stdout.on("data", (d) => { buf += d; let i; while ((i = buf.indexOf("\n")) >= 0) { const l = buf.slice(0, i); buf = buf.slice(i + 1); if (l.trim()) { const m = JSON.parse(l); respostas.set(m.id, m); } } });
  const chamar = async (id, method, params) => { mcp.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); for (let i = 0; i < 200 && !respostas.has(id); i++) await new Promise((r) => setTimeout(r, 25)); return respostas.get(id); };
  const ini = await chamar(1, "initialize", { protocolVersion: "2025-06-18" });
  assert.equal(ini.result.serverInfo.name, "blueprint");
  assert((await chamar(2, "tools/list", {})).result.tools.length >= 11);
  const listar = await chamar(3, "tools/call", { name: "listar_blueprints", arguments: {} });
  assert(!listar.result.isError && listar.result.content[0].text.includes(alvo.blueprints[0].id), "listar_blueprints mostra os blueprints");
  const ler = await chamar(4, "tools/call", { name: "ler_blueprint", arguments: { workspace_id: alvo.id, blueprint_id: alvo.blueprints[0].id } });
  assert(!ler.result.isError && JSON.parse(ler.result.content[0].text).blueprint.id === alvo.blueprints[0].id);
  const plano = await chamar(5, "tools/call", { name: "propor_blueprint", arguments: { workspace_id: alvo.id, blueprint_id: alvo.blueprints[0].id, mensagem: "fumaça", acoes: [{ nome: "Ação de fumaça", inicio: "2030-01-01", fim: "2030-01-31", planos: [1, 2, 3].map((n) => ({ texto: `Criar item ${n}`, descricao: "Validação: roda." })), checkpoints: [{ nome: "Pronto", data: "2030-01-30" }] }] } });
  assert(!plano.result.isError && plano.result.content[0].text.includes("Proposta enviada"), plano.result.content[0].text);
  const fila = await (await fetch(url + "/api/mcp/fila")).json();
  assert(fila.itens.some((i) => i.tipo === "proposta" && i.proposta.mensagem === "fumaça"), "a proposta chegou na fila do servidor");
  mcp.kill();
  console.log("fumaça do Rust: ok (página, estáticos, /api/dados idêntico ao data.json, MCP por HTTP e por stdio, proposta na fila)");
} finally { servidor.kill(); await new Promise((r) => setTimeout(r, 300)); await rm(dir, { recursive: true, force: true }).catch(() => {}); }
