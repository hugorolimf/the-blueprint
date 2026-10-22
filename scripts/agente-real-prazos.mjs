// Um agente de verdade (Claude Code, modo não interativo) planeja uma ação nova seguindo o guia "Prazos no ritmo da IA": lê o tópico planejar e propõe pelo MCP.
// Confere na fila do servidor (cópia dos dados) que a proposta veio sem avisos de prazo, com ação de até 3 dias e `horas` de 0,5 a 4 em todo plano. Gasta alguns tokens.
// Uso: node scripts/agente-real-prazos.mjs [blueprint.exe]
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFile, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dir = await mkdtemp(join(tmpdir(), "blueprint-agente-prazos-"));
const porta = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const url = `http://127.0.0.1:${porta}`;
await copyFile(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await new Promise((r) => setTimeout(r, 50)); } }
  const lista = await (await fetch(url + "/api/mcp/blueprints")).json();
  const ws = lista.workspaces.find((w) => w.blueprints.length), blueprint = ws.blueprints[0];
  const config = join(dir, "mcp.json");
  await writeFile(config, JSON.stringify({ mcpServers: { blueprint: { command: exe, args: ["--mcp"], env: { BLUEPRINT_URL: url } } } }));
  const pedido = `Use SOMENTE as ferramentas do servidor MCP "blueprint", no workspace_id "${ws.id}" e blueprint_id "${blueprint.id}". Passos, nesta ordem: 1) guia_de_planejamento com topico "planejar". 2) ler_blueprint (a data de hoje vem dele). 3) Siga o guia e proponha com propor_blueprint UMA ação nova chamada "Exportação em CSV" para o pedido "exportar a lista de pedidos em arquivo CSV", com os planos e o checkpoint que o guia pede, começando amanhã. 4) Se o servidor devolver avisos, corrija e envie de novo. 5) Responda em uma frase.`;
  const r = spawnSync("claude", ["-p", pedido, "--mcp-config", config, "--strict-mcp-config", "--allowedTools", "mcp__blueprint__guia_de_planejamento,mcp__blueprint__ler_blueprint,mcp__blueprint__propor_blueprint", "--permission-mode", "dontAsk", "--output-format", "text"], { encoding: "utf8", timeout: 300000, cwd: dir });
  console.log("saída do agente:", (r.stdout || "").trim().slice(0, 400));
  const fila = await (await fetch(url + "/api/mcp/fila")).json();
  const proposta = fila.itens.filter((i) => i.tipo === "proposta").at(-1);
  assert(proposta, "o agente não propôs a ação");
  const acao = proposta.proposta.acoes.find((a) => /csv/i.test(a.nome));
  assert(acao, "a proposta não traz a ação pedida");
  const dias = (Date.parse(acao.fim) - Date.parse(acao.inicio)) / 864e5 + 1;
  assert(dias <= 3, `a ação dura ${dias} dias (o guia pede até 3)`);
  assert(acao.planos.length >= 3 && acao.planos.every((p) => p.horas >= 0.5 && p.horas <= 4), `todo plano deve trazer horas de 0,5 a 4: ${JSON.stringify(acao.planos.map((p) => p.horas))}`);
  assert.deepEqual(proposta.avisos, [], "a proposta final não pode ter avisos de prazo");
  console.log(`ok: o agente real propôs "${acao.nome}" de ${acao.inicio} a ${acao.fim} (${dias} dia(s)), ${acao.planos.length} planos de ${acao.planos.map((p) => p.horas).join(", ")} h, sem avisos`);
} finally { servidor.kill(); await new Promise((r) => setTimeout(r, 300)); await rm(dir, { recursive: true, force: true }).catch(() => {}); }
