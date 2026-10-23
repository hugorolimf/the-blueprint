// Contrato HTTP: roda a MESMA sequência de pedidos contra o blueprint.exe e compara a resposta de cada rota (status, tipo, cabeçalhos que importam e corpo)
// com referencias/api-snapshots.json, gravado na migração a partir do servidor Node (o Node já saiu do repositório; ele está no histórico do git).
//   node scripts/contrato-http.mjs                       liga o blueprint.exe com dados de teste e compara com o snapshot
//   node scripts/contrato-http.mjs --gravar              regrava o snapshot com o Rust (só quando o contrato mudar DE PROPÓSITO)
//   node scripts/contrato-http.mjs --url http://...      roda contra um servidor já ligado e imprime o resumo
// Campos que mudam a cada execução (hora, ids sorteados, caminhos) são mascarados. Sai 1 se houver diferença.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawn } from "node:child_process";
import http from "node:http";
import { createServer } from "node:net";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const SNAPSHOT = join(raiz, "referencias", "api-snapshots.json");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const arg = (n) => { const i = process.argv.indexOf(`--${n}`); return i > 0 ? process.argv[i + 1] : null; };
const tem = (n) => process.argv.includes(`--${n}`);

const DADOS = { empresa: "Contrato", blueprints: [{ id: "p1", nome: "Portal", descricao: "d", acoes: [
  { id: "f1", nome: "Base", inicio: "2026-11-01", fim: "2026-11-30", status: "planejado", obs: "instruções",
    planos: [{ id: "t1", texto: "Criar rota", descricao: "x" }, { id: "t2", texto: "Já feita", descricao: "y", feito: true }],
    checkpoints: [{ id: "m1", nome: "Rota no ar", data: "2026-11-20" }, { id: "m2", nome: "Já feito", data: "2026-11-25", feito: true }], prototipos: [] }] }],
  workspaces: [{ id: "w2", nome: "Outro", blueprints: [{ id: "p9", nome: "Dele", acoes: [] }] }] };
const HTML = "<!doctype html><title>t</title><svg></svg>";

// ---- cliente cru: deixa escolher Host, Origin e o corpo exato ----
function pedir(base, metodo, caminho, { cabecalhos = {}, corpo, json } = {}) {
  const u = new URL(base);
  const texto = corpo !== undefined ? corpo : json !== undefined ? JSON.stringify(json) : undefined;
  const h = { ...(json !== undefined ? { "content-type": "application/json" } : {}), ...cabecalhos };
  if (texto !== undefined) h["content-length"] = Buffer.byteLength(texto);
  return new Promise((ok, erro) => {
    const r = http.request({ host: u.hostname, port: u.port, path: caminho, method: metodo, headers: h, agent: false }, (res) => {
      const partes = []; res.on("data", (d) => partes.push(d)); res.on("end", () => ok({ status: res.statusCode, cab: res.headers, corpo: Buffer.concat(partes) }));
    });
    r.on("error", erro); r.end(texto);
  });
}

const MASCARAS = new Set(["em", "hoje", "prototipoId", "arquivo", "url"]);
const EXTENSOES_DO_RUST = new Set(["exe"]); // campos que só o Rust acrescenta (o contrato compara o que o Node já devolvia)
const mascara = (v) => Array.isArray(v) ? v.map(mascara) : v && typeof v === "object" ? Object.fromEntries(Object.keys(v).sort().filter((k) => !EXTENSOES_DO_RUST.has(k)).map((k) => [k, MASCARAS.has(k) ? "<mascarado>" : mascara(v[k])])) : v;
const SEM_PROTO = (t) => t.replace(/\/prototipos\/[a-f0-9]{16}\.html/g, "/prototipos/<id>.html");

function resumo(r, { corpoTexto = false } = {}) {
  const tipo = String(r.cab["content-type"] || "").split(";")[0];
  const o = { status: r.status, tipo };
  for (const c of ["content-security-policy", "x-content-type-options"]) if (r.cab[c]) o[c] = r.cab[c];
  const texto = r.corpo.toString("utf8");
  if (tipo === "application/json") { try { o.corpo = mascara(JSON.parse(texto)); } catch { o.corpo = SEM_PROTO(texto); } }
  else if (corpoTexto || r.status >= 400) o.corpo = texto;
  else o.sha256 = createHash("sha256").update(r.corpo).digest("hex").slice(0, 16); // páginas e módulos: o mesmo arquivo dá o mesmo hash
  return o;
}

async function executar(base) {
  const R = {};
  const registra = async (nome, metodo, caminho, opcoes, ver) => { try { R[nome] = resumo(await pedir(base, metodo, caminho, opcoes), ver); } catch (e) { R[nome] = { erro: String(e.message) }; } };
  const mcp = (rota, json, extra = {}) => ({ json, ...extra });
  const post = (nome, rota, json, extra) => registra(nome, "POST", `/api/mcp/${rota}`, mcp(rota, json, extra));

  // páginas e estáticos
  for (const [n, c] of [["pagina", "/"], ["pagina-html", "/blueprint.html"], ["emblemas", "/emblemas.mjs"], ["kit-iso", "/kit/iso-kit.mjs"], ["kit-torno", "/kit/lathe/torno.mjs"], ["web-js", "/web/js/inicio.js"], ["web-css", "/web/css/tokens.css"]]) await registra(`GET ${n}`, "GET", c);
  for (const c of ["/server.mjs", "/data.json", "/kit/../server.mjs", "/web/js/nao-existe.js", "/qualquer", "/api/ia", "/api/agente/pastas", "/agentes", "/prototipos/curto.html", "/prototipos/ffffffffffffffff.html"]) await registra(`404 ${c}`, "GET", c);
  // /api/dados
  await registra("GET /api/dados", "GET", "/api/dados");
  await registra("PUT sem tipo", "PUT", "/api/dados", { corpo: "{}" });
  await registra("PUT json inválido", "PUT", "/api/dados", { cabecalhos: { "content-type": "application/json" }, corpo: "{torto" });
  await registra("PUT sem blueprints", "PUT", "/api/dados", { json: { empresa: "x" } });
  await registra("DELETE /api/dados", "DELETE", "/api/dados");
  await registra("PUT válido", "PUT", "/api/dados", { json: { ...DADOS, empresa: "Contrato 2" } });
  await registra("GET /api/dados depois", "GET", "/api/dados");
  await registra("PUT volta ao original", "PUT", "/api/dados", { json: DADOS });
  // segurança do MCP
  await registra("mcp host falso", "GET", "/api/mcp/blueprints", { cabecalhos: { host: "evil.example.com" } });
  await registra("mcp origin falsa", "GET", "/api/mcp/blueprints", { cabecalhos: { origin: "http://evil.example.com" } });
  await registra("mcp POST sem tipo", "POST", "/api/mcp/evento", { corpo: "{}" });
  await registra("mcp POST json inválido (só o status)", "POST", "/api/mcp/evento", { cabecalhos: { "content-type": "application/json" }, corpo: "{torto" });
  R["mcp POST json inválido (só o status)"] = { status: R["mcp POST json inválido (só o status)"].status };
  await registra("mcp PUT", "PUT", "/api/mcp/evento", { json: {} });
  await registra("mcp GET desconhecido", "GET", "/api/mcp/nada");
  await post("mcp POST desconhecido", "nada", {});
  // iniciar a ação (abre um terminal: só em loopback) e o hook de subagente: as recusas e os casos sem efeito (nenhum terminal abre)
  await post("iniciar ação inexistente", "iniciar", { workspaceId: "sm", blueprintId: "p1", acaoId: "nao" });
  await post("iniciar sem pasta ligada", "iniciar", { workspaceId: "sm", blueprintId: "p1", acaoId: "f1" });
  await post("iniciar sem acaoId", "iniciar", { workspaceId: "sm", blueprintId: "p1" });
  await registra("iniciar host falso", "POST", "/api/mcp/iniciar", { cabecalhos: { host: "evil.example.com" }, json: { workspaceId: "sm", blueprintId: "p1", acaoId: "f1" } });
  await post("hook de subagente sem agent_id", "hook/subagente", { hook_event_name: "SubagentStart", cwd: "C:\sem\blueprint" });
  await post("mcp corpo grande demais", "evento", { tipo: "progresso", texto: "a".repeat(300 * 1024) });
  // leitura
  await registra("mcp blueprints", "GET", "/api/mcp/blueprints");
  await registra("mcp blueprint (estrutura)", "GET", "/api/mcp/blueprint?workspace=sm&blueprint=p1");
  await registra("mcp blueprint (ação)", "GET", "/api/mcp/blueprint?workspace=sm&blueprint=p1&acao=f1");
  await registra("mcp blueprint de outro workspace", "GET", "/api/mcp/blueprint?workspace=w2&blueprint=p9");
  for (const [n, q] of [["sem blueprint", "workspace=sm"], ["blueprint inexistente", "workspace=sm&blueprint=nao"], ["workspace inexistente", "workspace=nao&blueprint=p1"], ["ação inexistente", "workspace=sm&blueprint=p1&acao=nao"]]) await registra(`mcp blueprint ${n}`, "GET", `/api/mcp/blueprint?${q}`);
  await registra("mcp info", "GET", "/api/mcp/info");
  // propostas
  const acao = (extra = {}) => ({ nome: "Nova", inicio: "2030-01-01", fim: "2030-01-31", planos: [1, 2, 3].map((n) => ({ texto: `Criar item ${n}`, descricao: "Validação: roda." })), checkpoints: [{ nome: "Pronto", data: "2030-01-30" }], ...extra });
  await post("proposta inválida", "proposta", { workspaceId: "sm", blueprintId: "p1", acoes: [{ nome: "x", inicio: "2026-12-01", fim: "2026-11-01", planos: [], checkpoints: [] }] });
  await post("proposta workspace inexistente", "proposta", { workspaceId: "nao", blueprintId: "p1", acoes: [] });
  await post("proposta blueprint novo sem nome", "proposta", { workspaceId: "sm", blueprintId: "", acoes: [acao()] });
  await post("proposta válida", "proposta", { workspaceId: "sm", blueprintId: "p1", mensagem: "v1", acoes: [acao()] });
  await post("proposta substitui", "proposta", { workspaceId: "sm", blueprintId: "p1", mensagem: "v2", acoes: [acao()] });
  await post("proposta blueprint novo", "proposta", { workspaceId: "sm", blueprintId: "", blueprint: { nome: "Novo", descricao: "d" }, acoes: [acao()] });
  await post("proposta mover", "proposta", { workspaceId: "sm", blueprintId: "p1", acoes: [acao({ planos: [{ id: "t1" }], checkpoints: [{ id: "m1", data: "2030-01-10" }] })] });
  await post("proposta remover", "proposta", { workspaceId: "sm", blueprintId: "p1", acoes: [], remover: [{ acao_id: "f1", plano_id: "t2", motivo: "teste" }, { acao_id: "f1", checkpoint_id: "m1" }] });
  await post("proposta remover inválido", "proposta", { workspaceId: "sm", blueprintId: "p1", acoes: [], remover: [{ acao_id: "f1", plano_id: "t1", checkpoint_id: "m1" }, { acao_id: "nao" }] });
  await registra("mcp fila depois das propostas", "GET", "/api/mcp/fila");
  // eventos
  const ev = (tipo, texto, extra = {}) => ({ tipo, texto, workspaceId: "sm", blueprintId: "p1", ...extra });
  await post("evento tipo inválido", "evento", { tipo: "inventado", texto: "x" });
  await post("evento sem texto", "evento", ev("progresso", "   "));
  await post("evento progresso", "evento", ev("progresso", "lendo"));
  await post("evento bloqueio", "evento", ev("bloqueio", "preciso de acesso"));
  await post("evento concluir", "evento", ev("concluir", "testes passaram", { acaoId: "f1", planoId: "t1" }));
  await post("evento concluir já feita", "evento", ev("concluir", "x", { acaoId: "f1", planoId: "t2" }));
  await post("evento concluir inexistente", "evento", ev("concluir", "x", { acaoId: "f1", planoId: "nao" }));
  await post("evento progresso com plano", "evento", ev("progresso", "x", { acaoId: "f1", planoId: "t2" }));
  await post("evento checkpoint feito", "evento", ev("checkpoint-feito", "demonstrado", { acaoId: "f1", checkpointId: "m1" }));
  await post("evento checkpoint já feito", "evento", ev("checkpoint-feito", "x", { acaoId: "f1", checkpointId: "m2" }));
  await post("evento checkpoint inexistente", "evento", ev("checkpoint-nao-feito", "x", { acaoId: "f1", checkpointId: "nao" }));
  await post("evento texto longo", "evento", ev("progresso", "a".repeat(5000)));
  await registra("mcp fila depois dos eventos", "GET", "/api/mcp/fila");
  await post("ack", "fila/ack", { ids: [1, 2, 3, 99] });
  await registra("mcp fila depois do ack", "GET", "/api/mcp/fila");
  // protótipos
  const proto = (extra = {}) => ({ workspaceId: "sm", blueprintId: "p1", acaoId: "f1", titulo: "Tela", html: HTML, ...extra });
  await post("protótipo sem tag", "prototipo", proto({ html: "sem tag" }));
  await post("protótipo sem título", "prototipo", proto({ titulo: "" }));
  await post("protótipo ação inexistente", "prototipo", proto({ acaoId: "nao" }));
  await post("protótipo grande demais", "prototipo", proto({ html: `<p>${"x".repeat(2 * 1024 * 1024)}</p>` }));
  await post("protótipo substituir inexistente", "prototipo", proto({ prototipoId: "0123456789abcdef" }));
  const criado = await pedir(base, "POST", "/api/mcp/prototipo", { json: proto() });
  R["protótipo anexar"] = resumo(criado);
  const id = JSON.parse(criado.corpo.toString()).prototipoId;
  await registra("protótipo servido", "GET", `/prototipos/${id}.html`, {}, { corpoTexto: true });
  await post("protótipo remover id inválido", "prototipo/remover", { prototipoId: "../../server" });
  await post("protótipo remover", "prototipo/remover", { prototipoId: id });
  await registra("protótipo depois de remover", "GET", `/prototipos/${id}.html`);
  return R;
}

// ---- servidores ----
const portaLivre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
async function ligar(tipo) {
  const tmp = mkdtempSync(join(tmpdir(), `contrato-${tipo}-`));
  writeFileSync(join(tmp, "dados.json"), JSON.stringify(DADOS));
  const porta = await portaLivre();
  const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), PROTOTIPOS_DIR: join(tmp, "p"), DADOS_ARQ: join(tmp, "dados.json"), BLUEPRINT_DB: join(tmp, "blueprint.db") };
  delete env.MCP_REDE;
  const exe = resolve(arg("rust") || join(raiz, "target/release/blueprint.exe"));
  const filho = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
  const base = `http://127.0.0.1:${porta}`;
  for (let i = 0; i < 150; i++) { try { await pedir(base, "GET", "/api/dados"); break; } catch { await dormir(100); } }
  return { base, parar: () => { filho.kill(); setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); } };
}

function diferencas(a, b, nomeA, nomeB) {
  const ruins = [];
  for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
    try { assert.deepEqual(a[k], b[k]); } catch { ruins.push(`${k}\n    ${nomeA}: ${JSON.stringify(a[k])?.slice(0, 300)}\n    ${nomeB}: ${JSON.stringify(b[k])?.slice(0, 300)}`); }
  }
  return ruins;
}

if (arg("url")) {
  const r = await executar(arg("url"));
  console.log(JSON.stringify(r, null, 2));
} else {
  assert(tem("gravar") || existsSync(SNAPSHOT), "faltam os snapshots (referencias/api-snapshots.json)");
  const rust = await ligar("rust");
  try {
    const atual = await executar(rust.base);
    if (tem("gravar")) { writeFileSync(SNAPSHOT, JSON.stringify({ data: new Date().toISOString(), origem: "blueprint.exe com os dados de teste de scripts/contrato-http.mjs (a versão original veio do servidor Node)", respostas: atual }, null, 2) + "\n"); console.log(`snapshot gravado: ${Object.keys(atual).length} respostas em referencias/api-snapshots.json`); }
    const ruins = diferencas(JSON.parse(readFileSync(SNAPSHOT, "utf8")).respostas, atual, "snapshot", "rust");
    console.log(ruins.length ? `${ruins.length} diferenças:\n${ruins.join("\n")}` :`contrato: o Rust responde igual ao snapshot em ${Object.keys(atual).length} rotas`);
    process.exitCode = ruins.length ? 1 : 0;
  } finally { rust.parar(); }
}
