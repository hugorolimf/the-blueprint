// Um agente de verdade (Claude Code em modo não interativo) reportando atividade pelo MCP do binário Rust, como principal e como subagente:
// confere que os dois aparecem na lista de agentes (o que o painel mostra) e que o agente marca um checkpoint com evidência. Gasta alguns tokens.
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
  let pendente = null; // um checkpoint ainda não feito, em qualquer blueprint do workspace
  for (const p of alvo.blueprints) {
    const est = await (await fetch(`${url}/api/mcp/blueprint?workspace=${alvo.id}&blueprint=${p.id}`)).json();
    for (const f of est.blueprint.acoes) for (const c of f.checkpoints) if (!c.feito && !pendente) pendente = { blueprint: p.id, acao: f.id, checkpoint: c.id, nome: c.nome };
  }
  assert(pendente, "o data.json de teste precisa ter um checkpoint pendente");
  const config = join(dir, "mcp.json");
  await writeFile(config, JSON.stringify({ mcpServers: { blueprint: { command: exe, args: ["--mcp"], env: { BLUEPRINT_URL: url } } } }));
  const pedido = `Use SOMENTE as ferramentas registrar_progresso (duas vezes) e marcar_checkpoint_feito (uma vez) do servidor MCP "blueprint", e nada mais. Sempre com workspace_id "${alvo.id}" e blueprint_id "${alvo.blueprints[0].id}". Chamada 1: texto "Lendo o código do servidor", agente "agente-principal", papel "explorador", estado "lendo", onde ["src/mcp/"], percentual 30. Chamada 2 (você reportando em nome de um subagente): texto "Rodando os testes", agente "subagente-revisor", papel "revisor", estado "testando", onde ["src/http/", "src/mcp/"], percentual 70. Chamada 3 (outra ferramenta, marcar_checkpoint_feito): workspace_id "${alvo.id}", blueprint_id "${pendente.blueprint}", acao_id "${pendente.acao}", checkpoint_id "${pendente.checkpoint}", evidencia "cargo test: 80 testes passaram (relato de teste do painel de agentes)". Depois responda em uma frase.`;
  const r = spawnSync("claude", ["-p", pedido, "--mcp-config", config, "--strict-mcp-config", "--allowedTools", "mcp__blueprint__registrar_progresso,mcp__blueprint__marcar_checkpoint_feito", "--permission-mode", "dontAsk", "--output-format", "text"], { encoding: "utf8", timeout: 240000, cwd: dir });
  console.log("saída do agente:", (r.stdout || "").trim().slice(0, 500));
  const { agentes } = await (await fetch(url + "/api/mcp/agentes")).json();
  console.log(JSON.stringify(agentes.map((a) => ({ agente: a.agente, papel: a.papel, estado: a.estado, onde: a.onde, percentual: a.percentual }))));
  const por = Object.fromEntries(agentes.map((a) => [a.agente, a]));
  assert(por["agente-principal"] && por["subagente-revisor"], "o agente real e o subagente tinham de aparecer na lista de agentes");
  assert.equal(por["agente-principal"].estado, "lendo");
  assert.equal(por["subagente-revisor"].papel, "revisor");
  assert.deepEqual(por["subagente-revisor"].onde, ["src/http/", "src/mcp/"]);
  const fila = await (await fetch(url + "/api/mcp/fila")).json();
  const cp = fila.itens.find((i) => i.tipo === "checkpoint-feito");
  assert(cp, "o agente real tinha de marcar o checkpoint pelo MCP");
  assert.equal(cp.checkpointId, pendente.checkpoint);
  assert(cp.texto.includes("80 testes passaram"), "a evidência chega junto");
  console.log(`checkpoint "${pendente.nome}" marcado pelo agente, com a evidência na fila`);
  console.log("ok: um agente real reportou como principal e como subagente, e os dois aparecem na lista de agentes");
} finally { servidor.kill(); await new Promise((r) => setTimeout(r, 300)); await rm(dir, { recursive: true, force: true }).catch(() => {}); }
