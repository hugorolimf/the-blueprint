// Mede o servidor Rust (blueprint serve) contra a linha de base do Node (referencias/linha-de-base.json). Só mede, não otimiza.
// Uso: node scripts/medir-rust.mjs [caminho do blueprint.exe]   Grava referencias/medicao-rust.json
// Com --limites (o CI): não grava nada e sai com 1 se passar dos limites folgados abaixo; --binario-mb=N, --p95-ms=N, --rss-mb=N e --cpu-pct=N trocam um limite.
import { spawn, execFileSync } from "node:child_process";
import http from "node:http";
import { createServer, connect } from "node:net";
import { mkdtempSync, copyFileSync, writeFileSync, readFileSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const args = process.argv.slice(2);
const exe = resolve(args.find((a) => !a.startsWith("--")) || join(raiz, "target/release/blueprint.exe"));
const opcao = (nome, padrao) => Number(args.find((a) => a.startsWith(`--${nome}=`))?.split("=")[1] ?? padrao);
// folga sobre as metas do CLAUDE.md (5 MB, 5 ms, 20 MB, 0%), porque o runner do CI oscila; tabela em referencias/relatorio-medicao.md
const limites = args.includes("--limites") && { binarioMB: opcao("binario-mb", 5), p95Ms: opcao("p95-ms", 15), rssMB: opcao("rss-mb", 40), cpuPct: opcao("cpu-pct", 1) };
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const r1 = (n) => Math.round(n * 10) / 10;
const porta = await new Promise((ok) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => ok(p)); }); });
const base = `http://127.0.0.1:${porta}`;

function lerProc(pid) {
  const out = execFileSync("powershell", ["-NoProfile", "-Command", `$p=Get-Process -Id ${pid}; "$($p.TotalProcessorTime.TotalSeconds) $($p.WorkingSet64)"`], { encoding: "utf8" }).trim();
  const [cpu, ws] = out.split(/\s+/).map(Number);
  return { cpuS: cpu, rssMB: r1(ws / 1048576) };
}
const stats = (ms) => { const o = [...ms].sort((a, b) => a - b); return { mediana: r1(o[Math.floor(o.length / 2)]), p95: r1(o[Math.ceil(o.length * 0.95) - 1]), n: o.length }; };
// O cliente conta: o fetch do Node (undici) sozinho gasta 7 a 16 ms por pedido e esconderia o tempo do servidor. Medimos com http.get
// em conexão persistente (o que o navegador faz) e também com fetch, que é como a linha de base do Node foi gravada.
const agente = new http.Agent({ keepAlive: true });
function pedirHttp(caminho, metodo = "GET", corpo) {
  return new Promise((ok, erro) => {
    const r = http.request({ host: "127.0.0.1", port: porta, path: caminho, method: metodo, agent: agente, headers: corpo ? { "content-type": "application/json", "content-length": Buffer.byteLength(corpo) } : {} }, (res) => { res.resume(); res.on("end", () => (res.statusCode < 300 ? ok() : erro(new Error(caminho + " " + res.statusCode)))); });
    r.on("error", erro); r.end(corpo);
  });
}
async function latenciaHttp(n, caminho, metodo, corpo) {
  const ms = [];
  for (let i = 0; i < n; i++) { const t = performance.now(); await pedirHttp(caminho, metodo, corpo); ms.push(performance.now() - t); }
  const o = [...ms].sort((a, b) => a - b);
  return { mediana: Math.round(o[Math.floor(o.length / 2)] * 100) / 100, p95: Math.round(o[Math.ceil(o.length * 0.95) - 1] * 100) / 100, n: o.length };
}
async function latencia(n, caminho, init) {
  const ms = [];
  for (let i = 0; i < n; i++) { const t = performance.now(); const r = await fetch(base + caminho, init); await r.arrayBuffer(); if (!r.ok) throw new Error(`${caminho} respondeu ${r.status}`); ms.push(performance.now() - t); }
  return stats(ms);
}

const tmp = mkdtempSync(join(tmpdir(), "medir-rust-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos") };
delete env.MCP_REDE;
const t0 = performance.now();
const filho = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const limpar = () => { try { filho.kill(); } catch {} try { rmSync(tmp, { recursive: true, force: true }); } catch {} };
process.on("exit", limpar);
try {
  let partida;
  for (;;) {
    if (performance.now() - t0 > 30000) throw new Error("o servidor não respondeu em 30 s");
    try { const r = await fetch(base + "/api/dados"); if (r.ok) { await r.arrayBuffer(); partida = performance.now() - t0; break; } } catch {}
    await dormir(5);
  }
  const memInicial = lerProc(filho.pid).rssMB;
  for (let i = 0; i < 20; i++) await (await fetch(base + (i % 2 ? "/" : "/api/dados"))).arrayBuffer();
  const memDepois = lerProc(filho.pid).rssMB;

  // repouso COM a página conectada por SSE (como no uso real): a conexão aberta não pode gastar CPU
  const sse = connect(porta, "127.0.0.1");
  sse.write(`GET /api/mcp/eventos HTTP/1.1\r\nHost: 127.0.0.1:${porta}\r\n\r\n`);
  sse.on("error", () => {});
  await dormir(500);
  const a = lerProc(filho.pid); const ta = performance.now();
  await dormir(10000);
  const b = lerProc(filho.pid); const tb = performance.now();
  const cpuPct = ((b.cpuS - a.cpuS) / ((tb - ta) / 1000)) * 100;
  sse.destroy();

  const json = { headers: { "content-type": "application/json" } };
  await fetch(base + "/api/dados").then((r) => r.arrayBuffer());
  const lat = {
    "GET /api/dados": await latencia(100, "/api/dados"),
    "GET /": await latencia(100, "/"),
    "GET /api/mcp/fila": await latencia(100, "/api/mcp/fila"),
    "POST /api/mcp/fila/ack": await latencia(100, "/api/mcp/fila/ack", { method: "POST", ...json, body: JSON.stringify({ ids: [] }) }),
  };
  await pedirHttp("/api/dados");
  const latHttp = {
    "GET /api/dados": await latenciaHttp(200, "/api/dados"),
    "GET /": await latenciaHttp(200, "/"),
    "GET /api/mcp/fila": await latenciaHttp(200, "/api/mcp/fila"),
    "POST /api/mcp/fila/ack": await latenciaHttp(200, "/api/mcp/fila/ack", "POST", JSON.stringify({ ids: [] })),
  };
  const comprimido = await fetch(base + "/", { headers: { "accept-encoding": "gzip" } });
  const html = Buffer.byteLength(await (await fetch(base + "/")).text());
  const linhaBase = JSON.parse(readFileSync(join(raiz, "referencias/linha-de-base.json"), "utf8"));
  const res = {
    data: new Date().toISOString(), plataforma: `${process.platform} ${process.arch}`, comando: "node scripts/medir-rust.mjs",
    cpu_metodo: "PowerShell Get-Process TotalProcessorTime, delta numa janela de 10 s sem requisições, com uma conexão SSE aberta",
    binario_bytes: statSync(exe).size, partida_ms: r1(partida), rss_apos_subir_mb: memInicial, rss_apos_20_req_mb: memDepois,
    cpu_repouso_pct_de_um_nucleo: Math.round(cpuPct * 100) / 100, latencia_ms_http_persistente: latHttp, latencia_ms_fetch_como_a_linha_de_base: lat,
    tamanho: { html_servido_bytes: html, html_gzip_bytes: Number(comprimido.headers.get("content-length")) || null },
    linha_de_base_node: { partida_ms: linhaBase.partida_ms, rss_apos_20_req_mb: linhaBase.rss_apos_20_req_mb, cpu_repouso_pct_de_um_nucleo: linhaBase.cpu_repouso_pct_de_um_nucleo, p95_api_dados_ms: linhaBase.latencia_ms["GET /api/dados"].p95 },
  };
  if (limites) {
    const falhas = [
      [res.binario_bytes / 1048576, limites.binarioMB, "binário (MB)"],
      [b.rssMB, limites.rssMB, "memória após 10 s de repouso (MB)"],
      [res.cpu_repouso_pct_de_um_nucleo, limites.cpuPct, "CPU em repouso (% de um núcleo)"],
      ...Object.entries(latHttp).map(([rota, l]) => [l.p95, limites.p95Ms, `p95 ${rota} (ms)`]),
    ].filter(([valor, teto]) => !(valor <= teto));
    for (const [valor, teto, nome] of falhas) console.error(`acima do limite: ${nome} = ${r1(valor)} (limite ${teto})`);
    console.log(falhas.length ? `${falhas.length} limite(s) estourado(s)` : `dentro dos limites ${JSON.stringify(limites)}`);
    process.exitCode = falhas.length ? 1 : 0;
  } else writeFileSync(join(raiz, "referencias", "medicao-rust.json"), JSON.stringify(res, null, 2) + "\n");
  console.table({
    "Binário (MB)": { rust: r1(res.binario_bytes / 1048576), node: "-", meta: "< 5" },
    "Partida até 1º 200 (ms)": { rust: res.partida_ms, node: linhaBase.partida_ms, meta: "-" },
    "RSS após 20 req (MB)": { rust: memDepois, node: linhaBase.rss_apos_20_req_mb, meta: "< 20" },
    "CPU repouso 10 s com SSE (% de 1 núcleo)": { rust: res.cpu_repouso_pct_de_um_nucleo, node: linhaBase.cpu_repouso_pct_de_um_nucleo, meta: "0" },
    "p95 GET /api/dados, http persistente (ms)": { rust: latHttp["GET /api/dados"].p95, node: "-", meta: "< 5" },
    "p95 GET /api/mcp/fila, http persistente (ms)": { rust: latHttp["GET /api/mcp/fila"].p95, node: "-", meta: "< 5" },
    "p95 GET /api/dados, fetch (ms)": { rust: lat["GET /api/dados"].p95, node: linhaBase.latencia_ms["GET /api/dados"].p95, meta: "-" },
  });
  if (!limites) console.log("Gravado em referencias/medicao-rust.json");
} finally { limpar(); }
