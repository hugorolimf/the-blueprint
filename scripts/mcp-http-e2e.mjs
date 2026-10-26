// O MCP por HTTP de ponta a ponta (POST /mcp, src/mcp/http.rs, docs/compartilhar.md): o dono compartilha o workspace (túnel falso, scripts/tunel-falso.mjs), a convidada entra e gera a
// chave, e SEM instalar nada (sem `blueprint conectar`, sem `--mcp`) um cliente HTTP faz o que o Claude Code faria: o pré-teste sem cabeçalho (initialize e tools/list),
// initialize e tools/list com a chave, ler_blueprint, registrar_progresso (o Claude dela aparece na folha do dono), recusas (outro workspace, sem chave, chave ruim, navegador,
// GET) e um anexar_prototipo grande (o limite de corpo da rota). Banco temporário; nada gasta tokens.
//   node scripts/mcp-http-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; ~5 s; Windows)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { request } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { dormir, exePadrao, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
assert.ok(existsSync(exe), `não achei ${exe}`);
const WS = "lab0001", BP = "bp7g8h9"; // data.exemplo.json: o workspace "Laboratório de ideias" e o blueprint dele
const tmp = mkdtempSync(join(tmpdir(), "mcp-http-e2e-"));
const porta = await portaLivre(), url = `http://127.0.0.1:${porta}`, info = join(tmp, "tunel.json");
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
writeFileSync(join(tmp, "tunel-falso.cmd"), `@node "${join(raiz, "scripts", "tunel-falso.mjs")}" %*\r\n`);
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_SEM_ATUALIZACAO: "1", BLUEPRINT_TUNEL_COMANDO: join(tmp, "tunel-falso.cmd"), TUNEL_FALSO_INFO: info }, stdio: "ignore", windowsHide: true });

const pedir = (endereco, { metodo = "GET", cabecalhos = {}, corpo } = {}) => new Promise((ok, falha) => {
  const u = new URL(endereco), texto = corpo === undefined ? undefined : typeof corpo === "string" ? corpo : JSON.stringify(corpo);
  const req = request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search, method: metodo, headers: { ...(texto ? { "content-type": "application/json", "content-length": Buffer.byteLength(texto) } : {}), ...cabecalhos } }, (res) => {
    let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let json = null; try { json = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json, cabecalhos: res.headers }); });
  });
  req.on("error", falha);
  if (texto) req.write(texto);
  req.end();
});

try {
  for (let i = 0; ; i++) { try { await fetch(url + "/api/dados"); break; } catch { if (i > 100) throw new Error("o servidor não respondeu"); await dormir(50); } }
  const local = { origin: url };
  assert.equal((await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS } })).status, 202);
  let estado;
  for (let i = 0; i < 200 && estado?.estado !== "ligado"; i++) { estado = (await pedir(`${url}/api/compartilhar`)).json; if (estado?.estado !== "ligado") await dormir(50); }
  assert.equal(estado.estado, "ligado", JSON.stringify(estado));
  const t = JSON.parse(readFileSync(info, "utf8"));
  const convite = await pedir(`${url}/api/convites`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, papel: "colaborador" } });
  const entrou = await pedir(`${t.local}/api/convite/entrar`, { metodo: "POST", cabecalhos: { origin: t.local }, corpo: { segredo: convite.json.link.split("#")[1], nome: "Bia" } });
  const cookie = String(entrou.cabecalhos["set-cookie"]).split(";")[0];
  const gerada = await pedir(`${t.local}/api/chave`, { metodo: "POST", cabecalhos: { origin: t.local, cookie }, corpo: {} });
  const chave = gerada.json.chave;
  // a tela traz os comandos prontos para quem não instala o app: a chave só aparece no da variável
  const c = gerada.json.comandos;
  assert.equal(c.claude_http, `claude mcp add --transport http --scope user blueprint-equipe ${t.publica}/mcp --header 'Authorization: Bearer \${BLUEPRINT_CHAVE}'`);
  assert.match(c.codex_http, /bearer_token_env_var = "BLUEPRINT_CHAVE"/);
  assert.ok(c.variavel.includes(chave) && !c.claude_http.includes(chave) && !c.codex_http.includes(chave));

  // o que um cliente MCP por HTTP manda: sem Origin; Accept com os dois tipos
  const mcp = (corpo, { chave: k = chave, extra = {}, metodo = "POST" } = {}) => pedir(`${t.local}/mcp`, { metodo, cabecalhos: { accept: "application/json, text/event-stream", ...(k ? { authorization: `Bearer ${k}` } : {}), ...extra }, corpo });
  const rpc = (id, method, params = {}) => ({ jsonrpc: "2.0", id, method, params });
  const chamar = async (name, args) => { const r = await mcp(rpc(9, "tools/call", { name, arguments: args })); return { status: r.status, erro: r.json?.result?.isError === true, texto: r.json?.result?.content?.[0]?.text ?? r.texto }; };

  // pré-teste do Claude Code: sem cabeçalho, initialize e tools/list respondem (e não anunciam OAuth)
  const pre = await mcp(rpc(1, "initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "claude-code", version: "x" } }), { chave: null });
  assert.equal(pre.status, 200, pre.texto);
  assert.equal(pre.json.result.serverInfo.name, "blueprint");
  assert.ok(!pre.cabecalhos["www-authenticate"], "sem WWW-Authenticate");
  assert.ok((await mcp(rpc(2, "tools/list"), { chave: null })).json.result.tools.length >= 10);
  // com a chave
  assert.equal((await mcp(rpc(1, "initialize", { protocolVersion: "2025-06-18" }))).json.result.protocolVersion, "2025-06-18");
  assert.equal((await mcp({ jsonrpc: "2.0", method: "notifications/initialized" })).status, 202);
  const lido = await chamar("ler_blueprint", { workspace_id: WS, blueprint_id: BP });
  assert.equal(lido.erro, false, lido.texto);
  assert.match(lido.texto, new RegExp(BP));
  const prog = await chamar("registrar_progresso", { workspace_id: WS, blueprint_id: BP, texto: "Lendo o blueprint por HTTP", agente: "Claude", estado: "lendo" });
  assert.equal(prog.erro, false, prog.texto);
  const agentes = (await pedir(`${url}/api/mcp/agentes`)).json.agentes;
  assert.ok(agentes.find((a) => a.agente === "Claude · Bia" && a.workspaceId === WS), "o Claude da Bia está na folha do dono: " + JSON.stringify(agentes.map((a) => a.agente)));

  // recusas
  assert.equal((await chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" })).erro, true, "outro workspace não existe para a convidada");
  const vinc = await chamar("vincular_pasta", { workspace_id: WS, blueprint_id: BP });
  assert.ok(vinc.erro && /CLAUDE\.local\.md/.test(vinc.texto), "a pasta do servidor nunca é ligada: " + vinc.texto);
  const semChave = await mcp(rpc(3, "tools/call", { name: "ler_blueprint", arguments: { workspace_id: WS, blueprint_id: BP } }), { chave: null });
  assert.deepEqual([semChave.status, semChave.json.error.code], [401, -32001]);
  const ruim = await mcp(rpc(4, "tools/list"), { chave: "bpk_" + "x".repeat(43) });
  assert.deepEqual([ruim.status, ruim.json.codigo], [401, "chave-invalida"]);
  assert.equal((await mcp(rpc(5, "ping"), { extra: { origin: "https://site-malvado.example" } })).status, 403, "navegador");
  const get = await mcp(undefined, { metodo: "GET" });
  assert.deepEqual([get.status, get.cabecalhos.allow], [405, "POST"]);
  assert.equal((await pedir(`${url}/mcp`, { metodo: "POST", cabecalhos: local, corpo: rpc(6, "ping") })).status, 404, "no app local (porta do dono) a rota não existe");
  // um protótipo de ~600 KB passa pelo limite de corpo da rota
  const html = "<!doctype html><title>x</title>" + "<p>linha de teste</p>".repeat(30000);
  const proto = await chamar("anexar_prototipo", { workspace_id: WS, blueprint_id: BP, acao_id: (await pedir(`${t.local}/api/mcp/blueprint?workspace=${WS}&blueprint=${BP}`, { cabecalhos: { authorization: `Bearer ${chave}` } })).json.blueprint.acoes[0].id, titulo: "Teste grande", html });
  assert.equal(proto.erro, false, proto.texto.slice(0, 300));
  console.log("mcp-http e2e: ok (comandos na tela, pré-teste sem cabeçalho, initialize/tools/list/ler_blueprint/registrar_progresso com a chave, Claude · Bia na folha, recusas, protótipo grande)");
} finally {
  try { servidor.kill(); } catch {}
  await dormir(800);
  try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {}
}
