// Servidor falso do modo hospedado: as rotas da seção 5 de docs/colaboracao.md em memória, para a frente TELAS testar antes da ACESSO entrar.
//   node scripts/stub-colaboracao.mjs [--porta N] [--dados arquivo.json]       serve a página e as rotas falsas
//   node scripts/stub-colaboracao.mjs --rust target/debug/blueprint.exe        sobe `<exe> serve --hospedar` e faz só de proxy HTTPS
// Com --rust, o stub vira o proxy reverso do teste (como o Caddy): manda X-Forwarded-Proto: https, Host e Origin da URL pública e tira o
// "Secure" do cookie (o teste é http). Nos dois modos imprime "pronto <url>" e "link do dono: <url>/convite#<segredo>", e anota cada pedido
// (caminho e Referer) em GET /__stub/pedidos, para o e2e provar que o segredo do convite nunca chega ao servidor.
// Limites do falso (o binário real é a referência): sem SSE de dados, sem /api/mudancas, sem atividade com retenção, sem HSTS.
import { spawn } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { createServer, request } from "node:http";
import { tmpdir } from "node:os";
import { extname, join, normalize, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const raiz = resolve(import.meta.dirname, "..");
const PUBLICA = "https://equipe.teste";
const TIPOS = { ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".mjs": "text/javascript; charset=utf-8", ".css": "text/css; charset=utf-8", ".svg": "image/svg+xml", ".json": "application/json" };
const sha = (s) => createHash("sha256").update(s).digest("hex");
const segredo = () => randomBytes(32).toString("base64url");
const id16 = () => randomBytes(8).toString("hex");
const agoraIso = () => new Date().toISOString();
const PESO = { visitante: 0, supervisor: 1, colaborador: 2, dono: 3 };

export function estado(dados) {
  return { dados, versao: Date.now(), membros: new Map(), acessos: [], convites: new Map(), sessoes: new Map(), chaves: new Map(), atividade: [], falhas: new Map() };
}
const nomeWs = (E, ws) => ws === "*" ? "" : ws === "sm" ? E.dados.empresa : (E.dados.workspaces || []).find((w) => w.id === ws)?.nome || ws;
const blueprintsDe = (E) => [...E.dados.blueprints.map((b) => ({ ws: "sm", b })), ...(E.dados.workspaces || []).flatMap((w) => w.blueprints.map((b) => ({ ws: w.id, b })))];
export function criarConvite(E, { papel, workspaceId, blueprintId = "", abre = "folha", horas = 168, usosMax = 1, criadoPor = null }) {
  const s = segredo(), c = { id: id16(), hash: sha(s), papel, workspaceId, blueprintId, abre, criadoPor, criadoEm: agoraIso(), expiraEm: new Date(Date.now() + horas * 36e5).toISOString(), usosMax, usos: 0, revogadoEm: null, horas };
  E.convites.set(c.id, c);
  return { c, s };
}
const conviteValido = (E, s) => { const c = [...E.convites.values()].find((x) => x.hash === sha(String(s || ""))); return c && !c.revogadoEm && Date.parse(c.expiraEm) > Date.now() && (c.usosMax == null || c.usos < c.usosMax) ? c : null; };
const ativ = (E, ctx, tipo, ws = "", alvo = "") => E.atividade.push({ em: agoraIso(), membro: ctx?.membro ? { id: ctx.membro.id, nome: ctx.membro.nome } : null, tipo, workspaceId: ws, blueprintId: "", alvo });
// o maior papel que cobre o alvo ('*' > workspace > blueprint), como Contexto::pode
const papelEm = (acessos, ws, bp = "") => acessos.filter((a) => a.workspaceId === "*" || (a.workspaceId === ws && (!a.blueprintId || a.blueprintId === bp))).reduce((m, a) => (PESO[a.papel] > (PESO[m] ?? -1) ? a.papel : m), "");
const papelNoWs = (acessos, ws) => acessos.filter((a) => a.workspaceId === "*" || a.workspaceId === ws).reduce((m, a) => (PESO[a.papel] > (PESO[m] ?? -1) ? a.papel : m), "");

export function contexto(E, pedido) {
  const ck = /(?:^|;\s*)bp_sessao=([^;]+)/.exec(pedido.headers.cookie || "")?.[1], s = ck && E.sessoes.get(sha(ck));
  if (!s) return null;
  s.vistoEm = agoraIso();
  if (s.conviteId) { const c = E.convites.get(s.conviteId); return c && !c.revogadoEm ? { visitante: true, convite: c, acessos: [{ workspaceId: c.workspaceId, blueprintId: c.blueprintId, papel: "visitante", abre: c.abre }] } : null; }
  const m = E.membros.get(s.membroId);
  return m && !m.removido ? { membro: m, visitante: false, acessos: E.acessos.filter((a) => a.membroId === m.id) } : null;
}
// o que o papel lê: visitante só o blueprint do link, sem comentários do agente, chats e perfil; membro só os workspaces dele
function filtrar(E, ctx) {
  const d = structuredClone(E.dados), semComentarios = (b) => { for (const a of b.acoes || []) for (const p of a.planos || []) if (p.agente) delete p.agente.comentarios; return b; };
  const ve = (ws, b) => ctx.visitante ? ctx.convite.workspaceId === ws && ctx.convite.blueprintId === b.id : !!papelEm(ctx.acessos, ws, b.id);
  d.blueprints = d.blueprints.filter((b) => ve("sm", b));
  d.workspaces = (d.workspaces || []).map((w) => ({ ...w, blueprints: w.blueprints.filter((b) => ve(w.id, b)) })).filter((w) => w.blueprints.length || (!ctx.visitante && papelNoWs(ctx.acessos, w.id)));
  if (ctx.visitante) { d.blueprints.forEach(semComentarios); d.workspaces.forEach((w) => w.blueprints.forEach(semComentarios)); delete d.chats; delete d.usuario; }
  return d;
}

export function rotas(E, ctx, metodo, rota, q, corpo, origem) {
  const erro = (status, codigo, params) => [status, { erro: codigo, codigo, ...(params ? { params } : {}) }];
  const exige = (ws, papeis) => (ctx && !ctx.visitante && papeis.includes(papelNoWs(ctx.acessos, ws)) ? null : ctx ? erro(403, "proibido") : erro(401, "sem-sessao"));
  const membroVisto = (mid) => [...E.sessoes.values()].filter((s) => s.membroId === mid).map((s) => s.vistoEm).sort().pop() || null;
  if (metodo === "GET" && rota === "/api/sessao") {
    if (!ctx) return erro(401, "sem-sessao");
    return [200, { modo: "servidor", membro: ctx.membro ? { id: ctx.membro.id, nome: ctx.membro.nome } : null, visitante: ctx.visitante, acessos: ctx.acessos.map((a) => ({ workspaceId: a.workspaceId, blueprintId: a.blueprintId, papel: a.papel, abre: a.abre || "folha" })) }];
  }
  if (rota === "/api/convite/conferir" || rota === "/api/convite/entrar") {
    const f = (E.falhas.get(origem) || []).filter((t) => Date.now() - t < 15 * 6e4);
    if (f.length >= 10) return erro(429, "muitas-tentativas", { segundos: Math.ceil((f[0] + 15 * 6e4 - Date.now()) / 1000) });
    const c = conviteValido(E, corpo.segredo);
    if (!c) { E.falhas.set(origem, [...f, Date.now()]); return erro(404, "convite-invalido"); }
    const bp = blueprintsDe(E).find((x) => x.b.id === c.blueprintId)?.b;
    if (rota.endsWith("conferir")) return [200, { papel: c.papel, workspace: { id: c.workspaceId, nome: nomeWs(E, c.workspaceId) }, blueprint: bp ? { id: bp.id, nome: bp.nome } : null, abre: c.abre, pedeNome: c.papel !== "visitante" }];
    const s = segredo();
    if (c.papel === "visitante") E.sessoes.set(sha(s), { conviteId: c.id, vistoEm: agoraIso() });
    else {
      const nome = String(corpo.nome ?? "").trim();
      if (!nome) return erro(400, "falta-nome");
      if (nome.length > 60 || /[\u0000-\u001f\u007f]/.test(nome)) return erro(400, "nome-invalido", { max: 60 });
      const m = ctx?.membro || { id: id16(), nome };
      E.membros.set(m.id, m);
      if (!E.acessos.some((a) => a.membroId === m.id && a.workspaceId === c.workspaceId && a.blueprintId === c.blueprintId)) E.acessos.push({ membroId: m.id, workspaceId: c.workspaceId, blueprintId: c.blueprintId, papel: c.papel, conviteId: c.id });
      E.sessoes.set(sha(s), { membroId: m.id, vistoEm: agoraIso() });
      ctx = { membro: m };
    }
    c.usos++;
    ativ(E, ctx, "entrou", c.workspaceId === "*" ? "" : c.workspaceId, c.id);
    const destino = c.papel === "visitante" ? `/#/p/${c.blueprintId}` : c.workspaceId === "*" ? "/" : `/#/c/${c.workspaceId}`;
    return [200, { ok: true, destino }, { "set-cookie": `bp_sessao=${s}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000` }];
  }
  if (metodo === "POST" && rota === "/api/sessao/sair") {
    for (const [h, s] of E.sessoes) if (ctx && ((ctx.membro && s.membroId === ctx.membro.id) || (ctx.visitante && s.conviteId === ctx.convite.id))) E.sessoes.delete(h);
    return [200, { ok: true }, { "set-cookie": "bp_sessao=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0" }];
  }
  if (metodo === "GET" && rota === "/api/pessoas") {
    const ws = q.get("workspace") || "", e = exige(ws, ["dono", "supervisor"]); if (e) return e;
    const membros = [...E.membros.values()].filter((m) => !m.removido && papelNoWs(E.acessos.filter((a) => a.membroId === m.id), ws)).map((m) => {
      const ac = E.acessos.filter((a) => a.membroId === m.id && (a.workspaceId === ws || a.workspaceId === "*"));
      return { id: m.id, nome: m.nome, papel: papelNoWs(ac, ws), blueprintId: ac.every((a) => a.blueprintId) ? ac[0].blueprintId : "", ultimaAtividade: membroVisto(m.id), voce: m.id === ctx.membro.id };
    });
    return [200, { membros }];
  }
  if (metodo === "POST" && (rota === "/api/pessoas/papel" || rota === "/api/pessoas/remover")) {
    const ws = corpo.workspaceId, e = exige(ws, ["dono"]); if (e) return e;
    const alvo = E.membros.get(corpo.membroId);
    if (!alvo || alvo.removido || !papelNoWs(E.acessos.filter((a) => a.membroId === alvo.id), ws)) return erro(404, "nao-encontrado");
    const remover = rota.endsWith("remover");
    if (!remover && !["dono", "colaborador", "supervisor"].includes(corpo.papel)) return erro(400, "papel-invalido");
    const donos = [...E.membros.values()].filter((m) => !m.removido && papelNoWs(E.acessos.filter((a) => a.membroId === m.id), ws) === "dono");
    if (donos.length === 1 && donos[0].id === alvo.id && (remover || corpo.papel !== "dono")) return erro(409, "ultimo-dono");
    E.acessos = E.acessos.filter((a) => !(a.membroId === alvo.id && a.workspaceId === ws));
    if (!remover) E.acessos.push({ membroId: alvo.id, workspaceId: ws, blueprintId: "", papel: corpo.papel });
    else if (!E.acessos.some((a) => a.membroId === alvo.id)) { alvo.removido = true; E.chaves.delete(alvo.id); for (const [h, s] of E.sessoes) if (s.membroId === alvo.id) E.sessoes.delete(h); }
    ativ(E, ctx, remover ? "membro-removido" : "papel-mudado", ws, alvo.id);
    return [200, { ok: true }];
  }
  const vistaConvite = (c) => ({ id: c.id, papel: c.papel, blueprintId: c.blueprintId, abre: c.abre, criadoPor: c.criadoPor ? { id: c.criadoPor, nome: E.membros.get(c.criadoPor)?.nome || "" } : null, criadoEm: c.criadoEm, expiraEm: c.expiraEm, usosMax: c.usosMax, usos: c.usos, revogado: !!c.revogadoEm });
  if (metodo === "GET" && rota === "/api/convites") {
    const ws = q.get("workspace") || "", e = exige(ws, ["dono"]); if (e) return e;
    return [200, { convites: [...E.convites.values()].filter((c) => c.workspaceId === ws).map(vistaConvite) }];
  }
  if (metodo === "POST" && rota === "/api/convites") {
    const ws = corpo.workspaceId, e = exige(ws, ["dono"]); if (e) return e;
    const papel = corpo.papel, visitante = papel === "visitante";
    if (!["dono", "colaborador", "supervisor", "visitante"].includes(papel) || (visitante && !corpo.blueprintId)) return erro(400, "papel-invalido");
    const horas = corpo.validadeHoras ?? 168;
    if (!Number.isInteger(horas) || horas < 1 || horas > 720) return erro(400, "validade-invalida");
    const usosMax = corpo.usosMax === undefined ? (visitante ? null : 1) : corpo.usosMax;
    if (usosMax !== null && (!Number.isInteger(usosMax) || usosMax < 1 || usosMax > 1000)) return erro(400, "usos-invalidos");
    const { c, s } = criarConvite(E, { papel, workspaceId: ws, blueprintId: corpo.blueprintId || "", abre: corpo.abre === "apresentacao" ? "apresentacao" : "folha", horas, usosMax, criadoPor: ctx.membro.id });
    ativ(E, ctx, "convite-criado", ws, c.id);
    return [201, { convite: vistaConvite(c), link: `${origemPublica}/convite#${s}` }];
  }
  if (metodo === "POST" && (rota === "/api/convites/revogar" || rota === "/api/convites/trocar")) {
    const c = E.convites.get(corpo.id); if (!c) return erro(404, "nao-encontrado");
    const e = exige(c.workspaceId, ["dono"]); if (e) return e;
    c.revogadoEm = agoraIso();
    for (const [h, s] of E.sessoes) if (s.conviteId === c.id) E.sessoes.delete(h);
    if (rota.endsWith("revogar")) { ativ(E, ctx, "convite-revogado", c.workspaceId, c.id); return [200, { ok: true }]; }
    const { c: n, s } = criarConvite(E, { ...c, criadoPor: ctx.membro.id });
    ativ(E, ctx, "convite-trocado", c.workspaceId, n.id);
    return [201, { convite: vistaConvite(n), link: `${origemPublica}/convite#${s}` }];
  }
  if (rota === "/api/chave" || rota === "/api/chave/revogar") {
    if (!ctx) return erro(401, "sem-sessao");
    if (ctx.visitante) return erro(403, "proibido");
    const mid = ctx.membro.id, k = E.chaves.get(mid);
    if (metodo === "GET") return [200, { chave: k ? { prefixo: k.prefixo, criadaEm: k.criadaEm, usadaEm: k.usadaEm } : null }];
    if (rota.endsWith("revogar")) { E.chaves.delete(mid); ativ(E, ctx, "chave-revogada"); return [200, { ok: true }]; }
    const chave = "bpk_" + segredo(), env = `--env BLUEPRINT_URL=${origemPublica} --env BLUEPRINT_CHAVE=${chave}`;
    E.chaves.set(mid, { hash: sha(chave), prefixo: chave.slice(0, 8), criadaEm: agoraIso(), usadaEm: null });
    ativ(E, ctx, "chave-criada");
    return [201, { chave, prefixo: chave.slice(0, 8), comandos: { conectar: `blueprint conectar ${origemPublica} ${chave}`, claude: `claude mcp add blueprint-equipe ${env} -- blueprint --mcp`, codex: `codex mcp add blueprint-equipe ${env} -- blueprint --mcp` } }];
  }
  if (metodo === "GET" && rota === "/api/atividade") {
    const ws = q.get("workspace") || "", e = exige(ws, ["dono", "supervisor"]); if (e) return e;
    const limite = Math.min(500, Math.max(1, Number(q.get("limite")) || 100));
    return [200, { itens: E.atividade.filter((a) => a.workspaceId === ws || a.workspaceId === "").slice(-limite).reverse().map(({ em, membro, tipo, blueprintId, alvo }) => ({ em, membro, tipo, blueprintId, alvo })) }];
  }
  if (rota === "/api/dados") {
    if (!ctx) return erro(401, "sem-sessao");
    if (metodo === "GET") return [200, filtrar(E, ctx), { "x-dados-versao": String(E.versao) }];
    if (metodo === "PUT") {
      if (ctx.visitante || !ctx.acessos.some((a) => a.workspaceId === "*" && a.papel === "dono")) return erro(403, ctx.visitante ? "proibido" : "use-mudancas");
      E.dados = corpo; E.versao++;
      return [200, { ok: true }, { "x-dados-versao": String(E.versao) }];
    }
  }
  if (rota === "/api/mcp/fila") return [200, { itens: [] }];
  if (rota === "/api/mcp/agentes") return [200, { agentes: [], vidaMs: 600000 }];
  if (rota === "/api/mcp/info") return [200, {}];
  return erro(404, "so-local");
}
let origemPublica = PUBLICA;
export const definirOrigemPublica = (u) => { origemPublica = u; }; // o stub do Compartilhar (stub-compartilhar.mjs): os links saem no endereço do túnel

export function estatico(caminho) {
  const p = caminho === "/" || caminho === "/convite" ? "/blueprint.html" : caminho;
  if (!(/^\/(web|kit)\//.test(p) || ["/blueprint.html", "/emblemas.mjs", "/agentes-figura.mjs", "/agentes-claude.mjs"].includes(p)) || p.includes("..")) return null;
  const arq = normalize(join(raiz, p));
  return arq.startsWith(raiz) && existsSync(arq) ? { corpo: readFileSync(arq), tipo: TIPOS[extname(arq)] || "application/octet-stream" } : null;
}
export const SEGURANCA = { "referrer-policy": "no-referrer", "x-content-type-options": "nosniff", "x-frame-options": "DENY" };

export async function subirStub({ porta = 0, dados = join(raiz, "data.exemplo.json"), rust = "" } = {}) {
  const pedidos = [];
  let alvo = null, filho = null, tmp = null, linkDono = "";
  const E = estado(JSON.parse(readFileSync(dados, "utf8")));
  const servidor = createServer((req, res) => {
    const u = new URL(req.url, "http://x");
    const reg = { metodo: req.method, caminho: req.url, referer: req.headers.referer || "", cookie: "" }; // cookie: o Set-Cookie como o servidor o mandou
    pedidos.push(reg);
    if (u.pathname === "/__stub/pedidos") { res.writeHead(200, { "content-type": "application/json" }); return res.end(JSON.stringify(pedidos)); }
    if (alvo) return proxy(req, res, alvo, url, reg);
    if (!u.pathname.startsWith("/api/")) {
      const f = req.method === "GET" && estatico(u.pathname);
      res.writeHead(f ? 200 : 404, { ...SEGURANCA, "content-type": f ? f.tipo : "text/plain" });
      return res.end(f ? f.corpo : "não encontrado");
    }
    if (u.pathname === "/api/mcp/eventos") { res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-store" }); return res.write("retry: 3000\n\n"); }
    let texto = "";
    req.on("data", (p) => { texto += p; });
    req.on("end", () => {
      const ctx = contexto(E, req), json = (status, corpo, extra = {}) => { reg.cookie = extra["set-cookie"] || ""; res.writeHead(status, { ...SEGURANCA, "content-type": "application/json", "cache-control": "no-store", ...extra }); res.end(JSON.stringify(corpo)); };
      if (req.method !== "GET" && req.headers.origin !== url) return json(403, { erro: "origem-invalida", codigo: "origem-invalida" });
      if (req.method !== "GET" && !/^application\/json/.test(req.headers["content-type"] || "")) return json(415, { erro: "json", codigo: "json" });
      let corpo = {};
      try { corpo = texto ? JSON.parse(texto) : {}; } catch { return json(400, { erro: "json", codigo: "json" }); }
      const [status, r, extra] = rotas(E, ctx, req.method, u.pathname, u.searchParams, corpo || {}, req.socket.remoteAddress);
      json(status, r, extra);
    });
  });
  await new Promise((ok) => servidor.listen(porta, "127.0.0.1", ok));
  const url = `http://127.0.0.1:${servidor.address().port}`;
  if (rust) {
    tmp = mkdtempSync(join(tmpdir(), "stub-colab-"));
    copyFileSync(dados, join(tmp, "dados.json"));
    const interna = await new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
    filho = spawn(resolve(rust), ["serve", "--hospedar"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(interna), BLUEPRINT_URL_PUBLICA: PUBLICA, BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1" }, stdio: ["ignore", "pipe", "pipe"], windowsHide: true });
    linkDono = await new Promise((ok, falha) => {
      const t = setTimeout(() => falha(new Error("o binário não imprimiu o link do dono em 15 s")), 15000);
      const ler = (b) => { const m = /\/convite#([A-Za-z0-9_-]+)/.exec(String(b)); if (m) { clearTimeout(t); ok(`${url}/convite#${m[1]}`); } };
      filho.stdout.on("data", ler); filho.stderr.on("data", ler);
      filho.on("exit", (c) => falha(new Error("o binário saiu com " + c)));
    });
    alvo = `http://127.0.0.1:${interna}`;
  } else {
    origemPublica = url;
    const { s } = criarConvite(E, { papel: "dono", workspaceId: "*", horas: 24, usosMax: 1 });
    ativ(E, null, "link-dono-impresso");
    linkDono = `${url}/convite#${s}`;
  }
  const fechar = () => { servidor.closeAllConnections?.(); servidor.close(); try { filho?.kill(); } catch {} if (tmp) setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800); };
  return { url, linkDono, pedidos, fechar, banco: tmp }; // banco: a pasta do banco do binário real (só com --rust), para procurar segredos gravados
}
// proxy reverso de teste: o binário real só aceita pedidos "vindos" de https://equipe.teste
function proxy(req, res, alvo, url, reg) {
  const h = { ...req.headers, host: new URL(PUBLICA).host, "x-forwarded-proto": "https", "x-forwarded-for": "127.0.0.1" };
  if (h.origin === url) h.origin = PUBLICA;
  if (h.referer) h.referer = h.referer.replace(url, PUBLICA);
  const p = request(alvo + req.url, { method: req.method, headers: h }, (r) => {
    const cab = { ...r.headers };
    if (cab["set-cookie"]) reg.cookie = cab["set-cookie"].join("\n");
    if (cab["set-cookie"]) cab["set-cookie"] = cab["set-cookie"].map((c) => c.replace(/;\s*Secure/i, ""));
    res.writeHead(r.statusCode, cab);
    r.pipe(res);
  });
  p.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); }); // o binário fechou no meio (fim do teste)
  req.pipe(p);
}

if (import.meta.url === pathToFileURL(process.argv[1] || "").href) {
  const arg = (n) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : undefined; };
  const s = await subirStub({ porta: Number(arg("--porta")) || 0, dados: arg("--dados") || undefined, rust: arg("--rust") || "" });
  console.log(`pronto ${s.url}\nlink do dono: ${s.linkDono}`);
  process.on("SIGINT", () => { s.fechar(); process.exit(0); });
}
