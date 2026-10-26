// Pastas temporárias: MCP propõe → usuário aplica → hook encontra vínculo; editor e aviso; status line reversível.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createServer } from "node:net";
const exe = resolve(process.argv[2] || "target/debug/blueprint.exe");
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe);
const tmp = mkdtempSync(join(tmpdir(), "blueprint-auto-"));
const livre = () => new Promise((ok) => { const s = createServer(); s.listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); }); });
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const [porta, depuracao] = [await livre(), await livre()];
const url = `http://127.0.0.1:${porta}`;
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_URL: url, BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), USERPROFILE: join(tmp, "home"), HOME: join(tmp, "home"), CLAUDE_CONFIG_DIR: join(tmp, "cfg") };
writeFileSync(env.DADOS_ARQ, JSON.stringify({ empresa: "Teste", usuario: { nome: "Teste" }, blueprints: [] }));
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "about:blank"], { stdio: "ignore", windowsHide: true });
let ws;
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch {} await dormir(50); }
  const api = async (rota, corpo) => {
    const r = await fetch(`${url}/api/mcp/${rota}`, corpo ? { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) } : {});
    assert.equal(r.status, 200, await r.clone().text()); return r.json();
  };
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(50); }
  ws = new WebSocket(alvo.webSocketDebuggerUrl); await new Promise((r) => { ws.onopen = r; });
  let numero = 0; const pendentes = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails); pendentes.get(d.id)?.(d); pendentes.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++numero; pendentes.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expression) => { const d = await cdp("Runtime.evaluate", { expression, returnByValue: true, awaitPromise: true, replMode: true }); assert(!d.result.exceptionDetails, JSON.stringify(d.result.exceptionDetails)); return d.result.result.value; };
  await cdp("Runtime.enable"); await cdp("Page.navigate", { url: `${url}/#/c/sm` }); await dormir(1500);
  const projeto = join(tmp, "nova"); mkdirSync(projeto);
  const proposta = { workspace_id: "sm", blueprint: { nome: "Novo automático", descricao: "Teste" }, acoes: [{ nome: "Criar", inicio: "2026-12-01", fim: "2026-12-02", planos: [{ texto: "Entregar", descricao: "Escopo: criar. Resultado: entrega. Validação: teste." }], checkpoints: [] }] };
  const mensagem = { jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: "propor_blueprint", arguments: proposta } };
  const chamada = spawnSync(exe, ["--mcp"], { cwd: projeto, env, input: JSON.stringify(mensagem) + "\n", encoding: "utf8" });
  assert.equal(chamada.status, 0, chamada.stderr); assert(!JSON.parse(chamada.stdout).result.isError, chamada.stdout);
  const fila = (await api("fila")).itens; assert.equal(fila.find((i) => i.tipo === "proposta").pasta, projeto);
  await js("await mcpConsulta()");
  assert.equal((await api("pastas")).pastas.length, 0, "proposta não liga antes de aplicar");
  await js('edProj = NOVO; await criarBlueprint(true)'); await dormir(400);
  const vinculo = (await api("pastas")).pastas[0]; assert(vinculo);
  assert.equal((await api("hook/sessao", { cwd: join(projeto, "sub"), session_id: "nova" })).blueprintId, vinculo.blueprintId);
  await js('editar(true)'); await dormir(500);
  assert((await js('document.querySelector("[data-pasta-blueprint]").textContent')).includes("Trocar"));
  const trocada = join(tmp, "trocada");
  await js(`document.querySelector("[data-pasta-blueprint] input").value = ${JSON.stringify(trocada)}; document.querySelector("[data-pasta-blueprint] button").click()`); await dormir(300);
  assert.equal((await api("pastas")).pastas[0].cwd, trocada.replaceAll("\\", "/").toLowerCase());
  await js('document.querySelector("[data-pasta-blueprint] button").nextElementSibling.click()'); await dormir(300);
  assert.equal((await api("pastas")).pastas.length, 0);
  const solta = join(tmp, "solta");
  await api("hook/sessao", { cwd: solta }); await api("hook/sessao", { cwd: solta });
  
  await js("await mcpConsulta()");
  assert((await js('document.getElementById("pastas-pendentes").textContent')).includes(solta)); assert.equal(await js('document.getElementById("pastas-pendentes").children.length'), 1);
  await js('const select = document.querySelector("#pastas-pendentes select"); select.selectedIndex = 1; select.dispatchEvent(new Event("change"))'); await dormir(300);
  assert.equal((await api("hook/sessao", { cwd: solta })).blueprintId, vinculo.blueprintId);
  const dirs = [join(env.USERPROFILE, ".claude"), env.CLAUDE_CONFIG_DIR];
  for (const dir of dirs) mkdirSync(dir, { recursive: true });
  const rodar = (args) => { const r = spawnSync(exe, args, { env, encoding: "utf8" }); assert.equal(r.status, 0, r.stdout + r.stderr); };
  for (const original of [{ tema: "dark" }, { tema: "dark", statusLine: { type: "command", command: "printf anterior" } }]) {
    for (const dir of dirs) writeFileSync(join(dir, "settings.json"), JSON.stringify(original));
    rodar(["registrar", "--status-line"]);
    const instalado = dirs.map((d) => readFileSync(join(d, "settings.json"), "utf8"));
    rodar(["registrar", "--status-line"]);
    assert.deepEqual(dirs.map((d) => readFileSync(join(d, "settings.json"), "utf8")), instalado);
    if (original.statusLine) {
      const combinado = spawnSync(exe, ["hook", "statusline", "--anterior", "cat"], { env, input: JSON.stringify({ cwd: solta }), encoding: "utf8" });
      assert.equal(combinado.status, 0); assert(combinado.stdout.includes(solta.replaceAll("\\", "\\\\")), combinado.stdout);
      assert(combinado.stdout.includes(" › ") && combinado.stdout.includes(" · ação "), combinado.stdout);
    }
    rodar(["registrar", "--status-line", "--remover"]);
    for (const dir of dirs) { assert.deepEqual(JSON.parse(readFileSync(join(dir, "settings.json"))), original); assert(existsSync(join(dir, "settings.json.bak"))); }
  }
  assert.deepEqual(erros, []);
  console.log("ok: MCP cwd → proposta → aplicação → sessão ligada; editor remove; aviso deduplica e liga; registrar instala/repete/remove em dois perfis");
} finally {
  ws?.close(); edge.kill(); servidor.kill();
  await dormir(500); try { rmSync(tmp, { recursive: true, force: true, maxRetries: 20, retryDelay: 100 }); } catch (e) { console.error(`Limpeza pendente em ${tmp}: ${e.code}`); }
}
