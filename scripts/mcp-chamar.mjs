// Chama UMA ferramenta do MCP pelo blueprint.exe --mcp (stdio), como um agente faria, e imprime a resposta. Útil para marcar progresso ou testar o MCP sem agente.
// Uso: node scripts/mcp-chamar.mjs <ferramenta> '<json dos argumentos>'
//      node scripts/mcp-chamar.mjs --lote chamadas.json      várias chamadas na mesma sessão: [{ "ferramenta": "…", "argumentos": { … } }, …]
//   node scripts/mcp-chamar.mjs listar_blueprints '{}'
//   node scripts/mcp-chamar.mjs marcar_concluido '{"workspace_id":"sm","blueprint_id":"…","acao_id":"…","plano_id":"…","evidencia":"…"}'
// Variáveis: BLUEPRINT_URL (padrão http://127.0.0.1:3000), BLUEPRINT_EXE (padrão target/release/blueprint.exe).
import { spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";

const [ferramenta, argumentos = "{}"] = process.argv.slice(2);
if (!ferramenta) { console.error("uso: node scripts/mcp-chamar.mjs <ferramenta> '<json dos argumentos>'"); process.exit(2); }
const exe = resolve(process.env.BLUEPRINT_EXE || join(import.meta.dirname, "..", "target", "release", "blueprint.exe"));
const filho = spawn(exe, ["--mcp"], { env: { ...process.env, BLUEPRINT_URL: process.env.BLUEPRINT_URL || "http://127.0.0.1:3000" }, stdio: ["pipe", "pipe", "inherit"] });
let buffer = ""; const pendentes = new Map(); let n = 0;
filho.stdout.on("data", (d) => {
  buffer += d;
  for (let i; (i = buffer.indexOf("\n")) >= 0;) { const l = buffer.slice(0, i).trim(); buffer = buffer.slice(i + 1); if (l) { const m = JSON.parse(l); pendentes.get(m.id)?.(m); } }
});
const rpc = (method, params) => new Promise((ok) => { const id = ++n; pendentes.set(id, ok); filho.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
await rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "mcp-chamar", version: "1" } });
const chamadas = ferramenta === "--lote" ? JSON.parse(readFileSync(argumentos, "utf8")) : [{ ferramenta, argumentos: JSON.parse(argumentos) }];
for (const c of chamadas) {
  const r = await rpc("tools/call", { name: c.ferramenta, arguments: c.argumentos });
  const texto = r.result?.content?.map((x) => x.text).join("\n") ?? JSON.stringify(r.error || r);
  const falhou = r.result?.isError || r.error;
  console.log(`${falhou ? "ERRO" : "ok"} ${c.ferramenta}: ${texto}`);
  if (falhou) process.exitCode = 1;
}
filho.kill();
