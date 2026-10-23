// Peças comuns dos testes de ponta a ponta do binário Rust (pendentes-e2e.mjs, atualizador-e2e.mjs): servidor em porta livre com banco temporário e MCP por stdio.
import { spawn } from "node:child_process";
import { mkdtemp, writeFile } from "node:fs/promises";
import { request } from "node:http";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

export const raiz = resolve(import.meta.dirname, "..");
export const exePadrao = () => resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
export const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
export const portaLivre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });

/** Dados mínimos: o workspace raiz "sm" com um blueprint p1, uma ação f1 e um plano t1. */
export const DADOS = { blueprints: [{ id: "p1", nome: "Projeto", acoes: [{ id: "f1", nome: "Ação", inicio: "2026-11-01", fim: "2026-11-05", status: "andamento", planos: [{ id: "t1", texto: "Plano um", descricao: "d" }], checkpoints: [] }] }] };

/** Uma pasta de dados temporária com o data.json mínimo e as variáveis de ambiente que o app lê. */
export async function pastaDeDados(prefixo, extra = {}) {
  const dir = await mkdtemp(join(tmpdir(), `${prefixo}-`));
  await writeFile(join(dir, "data.json"), JSON.stringify(DADOS));
  const porta = await portaLivre();
  const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "data.json"), PROTOTIPOS_DIR: join(dir, "prototipos"), BLUEPRINT_URL: `http://127.0.0.1:${porta}`, ...extra };
  return { dir, porta, url: `http://127.0.0.1:${porta}`, env };
}

/** `blueprint serve` com o ambiente dado; espera responder. */
export async function ligarServidor(exe, { env, url }) {
  const proc = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); return proc; } catch { await dormir(50); } }
  proc.kill();
  throw new Error("o servidor não respondeu");
}

/** `blueprint --mcp` por stdio: `chamar(nome, argumentos)` devolve o texto da ferramenta e se deu erro. */
export function mcpStdio(exe, env) {
  const proc = spawn(exe, ["--mcp"], { env, stdio: ["pipe", "pipe", "inherit"], windowsHide: true });
  const respostas = new Map();
  let buf = "", id = 0;
  proc.stdout.on("data", (d) => { buf += d; let i; while ((i = buf.indexOf("\n")) >= 0) { const l = buf.slice(0, i); buf = buf.slice(i + 1); if (l.trim()) { const m = JSON.parse(l); respostas.set(m.id, m); } } });
  const rpc = async (method, params) => { const n = ++id; proc.stdin.write(JSON.stringify({ jsonrpc: "2.0", id: n, method, params }) + "\n"); for (let i = 0; i < 300 && !respostas.has(n); i++) await dormir(20); return respostas.get(n); };
  return {
    proc,
    iniciar: () => rpc("initialize", { protocolVersion: "2025-06-18" }),
    chamar: async (name, args) => { const r = await rpc("tools/call", { name, arguments: args }); return { texto: r.result.content[0].text, erro: !!r.result.isError }; },
    fechar: () => proc.kill(),
  };
}

/** Pedido HTTP com cabeçalhos livres (o fetch do Node não deixa mandar Origin). */
export function http(url, { metodo = "GET", cabecalhos = {}, corpo } = {}) {
  return new Promise((ok, falha) => {
    const u = new URL(url);
    const req = request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search, method: metodo, headers: cabecalhos }, (res) => {
      let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let json = null; try { json = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json }); });
    });
    req.on("error", falha);
    if (corpo !== undefined) req.write(corpo);
    req.end();
  });
}
export const postJson = (url, objeto = {}, cabecalhos = {}) => http(url, { metodo: "POST", cabecalhos: { "content-type": "application/json", ...cabecalhos }, corpo: JSON.stringify(objeto) });
