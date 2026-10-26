// Servidor falso do Compartilhar (ação "Compartilhar um workspace pelo próprio app"), para testar a tela antes do servidor real existir.
//   node scripts/stub-compartilhar.mjs [--dados arquivo.json]     imprime "pronto <url do app local>"
// Duas portas, como o app: a LOCAL (modo local, sem sessão: o usuário é o dono de tudo) com GET/POST /api/compartilhar e o evento
// "compartilhar" no SSE do MCP; e a dos CONVIDADOS, que faz o papel do túnel (o "endereço público" é a própria URL dela) e responde como o
// modo hospedado, com as rotas e o filtro de scripts/stub-colaboracao.mjs. Desligado, a porta dos convidados responde 502 (túnel fechado).
// Limites do falso: sem /api/mudancas nem mudanças ao vivo (o e2e só confere isso com --rust); baixar e ligar são dois tempos fixos de 300 ms.
import { readFileSync } from "node:fs";
import { createServer } from "node:http";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { SEGURANCA, contexto, definirOrigemPublica, estado, estatico, rotas } from "./stub-colaboracao.mjs";

const raiz = resolve(import.meta.dirname, "..");
const DONO_LOCAL = { membro: { id: "local", nome: "local" }, visitante: false, acessos: [{ workspaceId: "*", blueprintId: "", papel: "dono" }] };

export async function subirStubCompartilhar({ dados = join(raiz, "data.exemplo.json"), passoMs = 300 } = {}) {
  const E = estado(JSON.parse(readFileSync(dados, "utf8")));
  const fluxos = new Set(), tempos = [];
  let share = { estado: "desligado" }, urlConvidados = "";
  const avisar = () => { for (const r of fluxos) r.write(`event: compartilhar\ndata: ${JSON.stringify(share)}\n\n`); };
  const muda = (s) => { share = s; avisar(); };
  const ligar = (ws) => {
    tempos.forEach(clearTimeout); tempos.length = 0;
    muda({ estado: "baixando", workspaceId: ws });
    tempos.push(setTimeout(() => muda({ estado: "ligando", workspaceId: ws }), passoMs));
    tempos.push(setTimeout(() => muda({ estado: "ligado", workspaceId: ws, url: urlConvidados, porta: Number(new URL(urlConvidados).port) }), 2 * passoMs));
  };
  const sse = (res, set) => { res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-store" }); res.write("retry: 3000\n\n"); if (set) { set.add(res); res.on("close", () => set.delete(res)); } };
  const json = (res, status, corpo, extra = {}) => { res.writeHead(status, { ...SEGURANCA, "content-type": "application/json", "cache-control": "no-store", ...extra }); res.end(JSON.stringify(corpo)); };
  const estaticoOu404 = (req, res, caminho) => { const f = req.method === "GET" && estatico(caminho); res.writeHead(f ? 200 : 404, { ...SEGURANCA, "content-type": f ? f.tipo : "text/plain" }); res.end(f ? f.corpo : "não encontrado"); };
  const lerCorpo = (req, res, url, faz) => {
    let texto = "";
    req.on("data", (p) => { texto += p; });
    req.on("end", () => {
      if (req.method !== "GET" && req.headers.origin !== url) return json(res, 403, { erro: "origem-invalida", codigo: "origem-invalida" });
      if (req.method !== "GET" && !/^application\/json/.test(req.headers["content-type"] || "")) return json(res, 415, { erro: "json", codigo: "json" });
      try { faz(texto ? JSON.parse(texto) : {}); } catch { json(res, 400, { erro: "json", codigo: "json" }); }
    });
  };

  // a porta local: a página do dono
  const local = createServer((req, res) => {
    const u = new URL(req.url, "http://x"), p = u.pathname;
    if (!p.startsWith("/api/")) return estaticoOu404(req, res, p);
    if (p === "/api/mcp/eventos") return sse(res, fluxos);
    if (p === "/api/eventos") return sse(res);
    lerCorpo(req, res, urlLocal, (corpo) => {
      if (p === "/api/sessao") return json(res, 200, { modo: "local" });
      if (p === "/api/dados" && req.method === "GET") return json(res, 200, E.dados, { "x-dados-versao": String(E.versao) });
      if (p === "/api/presenca") return json(res, 200, { ok: true });
      if (p === "/api/compartilhar" && req.method === "GET") return json(res, 200, share);
      if (p === "/api/compartilhar/ligar" && req.method === "POST") {
        const ws = String(corpo.workspaceId || "");
        if (ws !== "sm" && !(E.dados.workspaces || []).some((w) => w.id === ws)) return json(res, 404, { erro: "workspace-inexistente", codigo: "workspace-inexistente" });
        if (share.estado === "desligado" || share.estado === "erro") ligar(ws);
        return json(res, 200, share);
      }
      if (p === "/api/compartilhar/desligar" && req.method === "POST") { tempos.forEach(clearTimeout); muda({ estado: "desligado" }); return json(res, 200, share); }
      const [status, r, extra] = rotas(E, DONO_LOCAL, req.method, p, u.searchParams, corpo || {}, "local");
      json(res, status, r, extra);
    });
  });
  // a porta dos convidados (atrás do túnel): o modo hospedado do stub da colaboração; as rotas só locais caem no 404 "so-local"
  const convidados = createServer((req, res) => {
    if (share.estado !== "ligado") { res.writeHead(502, { "content-type": "text/plain" }); return res.end("túnel fechado"); }
    const u = new URL(req.url, "http://x"), p = u.pathname;
    if (!p.startsWith("/api/")) return estaticoOu404(req, res, p === "/convite" ? "/" : p);
    if (p === "/api/mcp/eventos" || p === "/api/eventos") return sse(res);
    lerCorpo(req, res, urlConvidados, (corpo) => {
      const ctx = contexto(E, req);
      if (ctx && !ctx.visitante && !ctx.acessos.some((a) => a.workspaceId === share.workspaceId)) return json(res, 403, { erro: "proibido", codigo: "proibido" }); // só o workspace compartilhado
      const [status, r, extra] = rotas(E, ctx, req.method, p, u.searchParams, corpo || {}, req.socket.remoteAddress);
      json(res, status, r, extra);
    });
  });
  const ouvir = (s) => new Promise((ok) => s.listen(0, "127.0.0.1", () => ok(`http://127.0.0.1:${s.address().port}`)));
  const urlLocal = await ouvir(local);
  urlConvidados = await ouvir(convidados);
  definirOrigemPublica(urlConvidados);
  const fechar = () => { tempos.forEach(clearTimeout); for (const s of [local, convidados]) { s.closeAllConnections?.(); s.close(); } };
  return { url: urlLocal, convidados: urlConvidados, fechar };
}

if (import.meta.url === pathToFileURL(process.argv[1] || "").href) {
  const i = process.argv.indexOf("--dados");
  const s = await subirStubCompartilhar(i > 0 ? { dados: process.argv[i + 1] } : {});
  console.log(`pronto ${s.url}\nconvidados (túnel): ${s.convidados}`);
  process.on("SIGINT", () => { s.fechar(); process.exit(0); });
}
