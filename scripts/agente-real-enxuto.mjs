// Um agente de verdade (Claude Code em modo não interativo) usando o MCP enxuto pelo binário Rust, contra uma cópia dos dados: lê a estrutura, pede uma ação
// inteira, lê o tópico certo do guia, propõe uma alteração PARCIAL (só o texto de um plano) e marca outro plano com evidência. Confere pela fila do servidor que
// nada se perdeu (a proposta chega com todos os planos e descrições da ação) e imprime o tamanho de cada resposta. Gasta alguns tokens.
// Uso: node scripts/agente-real-enxuto.mjs [blueprint.exe]
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFile, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dir = await mkdtemp(join(tmpdir(), "blueprint-agente-enxuto-"));
const porta = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const url = `http://127.0.0.1:${porta}`;
await copyFile(join(raiz, "data.exemplo.json"), join(dir, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await new Promise((r) => setTimeout(r, 50)); } }
  const lista = await (await fetch(url + "/api/mcp/blueprints")).json();
  const ws = lista.workspaces.find((w) => w.blueprints.length);
  const dados = await (await fetch(url + "/api/dados")).json();
  const todos = [...(dados.blueprints || []), ...(dados.workspaces || []).flatMap((w) => w.blueprints || [])];
  const serve = (f) => f.planos?.filter((t) => !t.feito).length >= 2 && f.planos.some((t) => !t.feito && t.texto.length <= 33);
  const blueprint = todos.find((p) => ws.blueprints.some((x) => x.id === p.id) && p.acoes?.some(serve));
  const acao = blueprint.acoes.find(serve);
  const alterar = acao.planos.find((t) => !t.feito && t.texto.length <= 33), concluir = acao.planos.find((t) => !t.feito && t.id !== alterar.id);
  const config = join(dir, "mcp.json");
  await writeFile(config, JSON.stringify({ mcpServers: { blueprint: { command: exe, args: ["--mcp"], env: { BLUEPRINT_URL: url } } } }));
  const pedido = `Use SOMENTE as ferramentas do servidor MCP "blueprint", no workspace_id "${ws.id}" e blueprint_id "${blueprint.id}". Passos, nesta ordem: 1) ler_blueprint SEM acao_id. 2) ler_blueprint com acao_id "${acao.id}". 3) guia_de_planejamento com topico "editar". 4) propor_blueprint alterando SÓ o texto do plano id "${alterar.id}" para o texto atual mais " (revisado)": envie a ação só com o id e esse plano só com id e texto, sem repetir mais nada. 5) marcar_concluido do plano id "${concluir.id}" da ação "${acao.id}" com a evidência "validação do agente real". 6) Responda em uma frase.`;
  const r = spawnSync("claude", ["-p", pedido, "--mcp-config", config, "--strict-mcp-config", "--allowedTools", "mcp__blueprint__ler_blueprint,mcp__blueprint__guia_de_planejamento,mcp__blueprint__propor_blueprint,mcp__blueprint__marcar_concluido", "--permission-mode", "dontAsk", "--output-format", "stream-json", "--verbose"], { encoding: "utf8", timeout: 300000, cwd: dir, maxBuffer: 64 * 1024 * 1024 });
  const nomes = new Map(), tamanhos = [];
  for (const linha of (r.stdout || "").split("\n")) {
    let e; try { e = JSON.parse(linha); } catch { continue; }
    for (const c of e.message?.content || []) {
      if (c.type === "tool_use") nomes.set(c.id, `${c.name.replace("mcp__blueprint__", "")}${c.input?.acao_id ? "(ação)" : c.input?.topico ? `(${c.input.topico})` : ""}`);
      if (c.type === "tool_result") { const txt = typeof c.content === "string" ? c.content : (c.content || []).map((x) => x.text || "").join(""); tamanhos.push([nomes.get(c.tool_use_id) || "?", txt.length]); if (txt.length < 400) console.log("  ›", nomes.get(c.tool_use_id), txt.replace(/\s+/g, " ").slice(0, 300)); }
    }
  }
  console.log("respostas do MCP ao agente (caracteres):", tamanhos.map(([n, t]) => `${n}=${t}`).join("  "));
  const fila = await (await fetch(url + "/api/mcp/fila")).json();
  const proposta = fila.itens.find((i) => i.tipo === "proposta"), feito = fila.itens.find((i) => i.tipo === "concluir");
  assert(proposta, "o agente não propôs a alteração parcial");
  assert(feito, "o agente não marcou o plano como concluído");
  const f = proposta.proposta.acoes.find((x) => x.id === acao.id);
  assert.equal(f.planos.length, acao.planos.length, "a proposta chegou sem perder nenhum plano da ação");
  const antes = Object.fromEntries(acao.planos.map((t) => [t.id, t]));
  for (const t of f.planos) { if (t.id !== alterar.id) assert.equal(t.descricao, antes[t.id].descricao, "nenhuma descrição foi esvaziada"); }
  const mudou = f.planos.find((t) => t.id === alterar.id);
  assert(mudou.texto.endsWith("(revisado)") && mudou.descricao === alterar.descricao, "o plano alterado mudou só o texto");
  assert.equal((f.observacao ?? "").trim(), (acao.obs ?? "").trim(), "as instruções da ação ficaram como estavam");
  assert.equal(feito.planoId, concluir.id);
  const nomesUsados = [...nomes.values()].join(" ");
  for (const esperado of ["ler_blueprint", "ler_blueprint(ação)", "guia_de_planejamento(editar)", "propor_blueprint", "marcar_concluido"]) assert(nomesUsados.includes(esperado), `o agente não chamou ${esperado}: ${nomesUsados}`);
  console.log(`ok: o agente real leu a estrutura, a ação inteira e o tópico "editar", propôs só a mudança de um plano (${f.planos.length} planos intactos, descrições e instruções preservadas) e marcou outro plano com evidência`);
} finally { servidor.kill(); await new Promise((r) => setTimeout(r, 300)); await rm(dir, { recursive: true, force: true }).catch(() => {}); }
