// Mede o que cada perfil do MCP (BLUEPRINT_MCP_PERFIL) põe no contexto do agente: quantas ferramentas e o tamanho do tools/list.
// Não precisa do app aberto (tools/list não chama o servidor). Números em docs/mcp.md.
// Uso: node scripts/medir-perfis-mcp.mjs [exe]   (padrão target/debug/blueprint.exe)
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

const exe = resolve(process.argv[2] || join(import.meta.dirname, "..", "target", "debug", "blueprint.exe"));
const pedido = [
  { jsonrpc: "2.0", id: 1, method: "initialize", params: { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "medir", version: "1" } } },
  { jsonrpc: "2.0", id: 2, method: "tools/list", params: {} },
].map((m) => JSON.stringify(m)).join("\n") + "\n";

const rodar = (perfil) => spawnSync(exe, ["--mcp"], { input: pedido, encoding: "utf8", env: { ...process.env, BLUEPRINT_MCP_PERFIL: perfil } });

const linhas = [];
for (const perfil of ["completo", "executor", "planejador"]) {
  const r = rodar(perfil);
  const lista = r.stdout.trim().split("\n").map((l) => JSON.parse(l)).find((m) => m.id === 2).result.tools;
  const bytes = Buffer.byteLength(JSON.stringify(lista));
  linhas.push({ perfil, ferramentas: lista.length, bytes, tokensEstimados: Math.round(bytes / 4) });
}
console.table(linhas);
const invalido = rodar("worker");
if (invalido.status !== 2 || !invalido.stderr.includes("BLUEPRINT_MCP_PERFIL")) { console.error("o valor inválido devia sair com 2 e uma mensagem clara", invalido.status, invalido.stderr); process.exit(1); }
console.log(`valor inválido: sai com ${invalido.status}: ${invalido.stderr.trim()}`);
