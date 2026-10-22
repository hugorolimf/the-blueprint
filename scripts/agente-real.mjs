// Um agente de verdade (Claude Code em modo não interativo) usando o MCP pelo binário Rust: liga `blueprint serve` com uma cópia do data.json,
// registra `blueprint --mcp` só para essa chamada (--mcp-config) e pede ao agente que liste os blueprints e proponha um plano. Gasta alguns tokens.
// Uso: node scripts/agente-real.mjs [caminho do blueprint.exe]
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { mkdtemp, copyFile, writeFile, rm } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dir = await mkdtemp(join(tmpdir(), "blueprint-agente-"));
const porta = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const url = `http://127.0.0.1:${porta}`;
await copyFile(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await new Promise((r) => setTimeout(r, 50)); } }
  const lista = await (await fetch(url + "/api/mcp/blueprints")).json();
  const alvo = lista.workspaces.find((w) => w.blueprints.length);
  const config = join(dir, "mcp.json");
  await writeFile(config, JSON.stringify({ mcpServers: { blueprint: { command: exe, args: ["--mcp"], env: { BLUEPRINT_URL: url } } } }));
  const pedido = `Use SOMENTE as ferramentas do servidor MCP "blueprint". 1) Chame listar_blueprints. 2) Chame propor_blueprint com workspace_id "${alvo.id}", blueprint_id "${alvo.blueprints[0].id}", mensagem "plano do agente real" e UMA ação nova chamada "Ação do agente real" de 2030-01-01 a 2030-01-31 com 3 planos (texto começando com verbo no infinitivo e descrição contendo "Validação: ...") e 1 checkpoint em 2030-01-30. 3) Responda em uma frase dizendo o que o servidor respondeu.`;
  const r = spawnSync("claude", ["-p", pedido, "--mcp-config", config, "--strict-mcp-config", "--allowedTools", "mcp__blueprint__listar_blueprints,mcp__blueprint__propor_blueprint", "--permission-mode", "dontAsk", "--output-format", "text"], { encoding: "utf8", timeout: 240000, cwd: dir });
  console.log("saída do agente:", (r.stdout || "").trim().slice(0, 600), r.stderr ? `\n(stderr: ${r.stderr.slice(0, 300)})` : "");
  const fila = await (await fetch(url + "/api/mcp/fila")).json();
  const proposta = fila.itens.find((i) => i.tipo === "proposta");
  assert(proposta, "o agente real não conseguiu propor o plano pelo binário Rust");
  assert.equal(proposta.proposta.acoes[0].nome, "Ação do agente real");
  console.log(`ok: o agente real listou os blueprints e propôs "${proposta.proposta.acoes[0].nome}" (${proposta.proposta.acoes[0].planos.length} planos, ${proposta.proposta.acoes[0].checkpoints.length} checkpoint) pelo blueprint --mcp`);
} finally { servidor.kill(); await new Promise((r) => setTimeout(r, 300)); await rm(dir, { recursive: true, force: true }).catch(() => {}); }
