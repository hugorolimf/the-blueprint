// Mede a sincronização com vários clientes ao mesmo tempo (docs/colaboracao.md, seção 6): N conexões SSE em /api/eventos e W gravadores
// mandando lotes em /api/mudancas em paralelo. Anota o atraso da gravação até cada cliente (p50, p95, máx.), a memória e as threads do
// servidor com as conexões abertas e a CPU em repouso (só os batimentos). Resultado em referencias/relatorio-medicao.md.
//   node scripts/medir-sincronia.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe; Windows; ~30 s)
import http from "node:http";
import { execFileSync } from "node:child_process";
import { rmSync } from "node:fs";
import { join } from "node:path";
import { dormir, ligarServidor, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";

const exe = process.argv[2] || join(raiz, "target/release/blueprint.exe");
const NS = [10, 100, 250], GRAVADORES = 4, LOTES = 10;
const pasta = await pastaDeDados("medir-sincronia");
const servidor = await ligarServidor(exe, pasta);
const ps = (expr) => execFileSync("powershell", ["-NoProfile", "-Command", `$p = Get-Process -Id ${servidor.pid}; ${expr}`], { encoding: "utf8" }).trim();
const processo = () => { const [mem, cpu, th] = ps("\"$($p.WorkingSet64) $($p.TotalProcessorTime.TotalMilliseconds) $($p.Threads.Count)\"").split(" ").map(Number); return { mem, cpu, th }; };
const pct = (l, p) => { const s = [...l].sort((a, b) => a - b); return s[Math.min(s.length - 1, Math.floor(p * s.length))] ?? NaN; };

/** Abre `n` conexões SSE; cada uma anota a hora em que viu cada marca. */
async function abrir(n) {
  const vistos = [], conexoes = [];
  await Promise.all(Array.from({ length: n }, (_, i) => new Promise((ok, falha) => {
    const req = http.get(`${pasta.url}/api/eventos?workspace=sm`, { agent: false }, (res) => {
      vistos[i] = new Map();
      res.setEncoding("utf8");
      res.on("data", (t) => { const agora = performance.now(); for (const m of t.matchAll(/"texto":"marca-(\d+)"/g)) if (!vistos[i].has(m[1])) vistos[i].set(m[1], agora); });
      ok();
    });
    req.on("error", falha);
    conexoes.push(req);
  })));
  return { vistos, fechar: () => conexoes.forEach((r) => r.destroy()) };
}

const post = (corpo) => fetch(pasta.url + "/api/mudancas", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
const linhas = [];
try {
  const vazio = processo();
  for (const n of NS) {
    const { vistos, fechar } = await abrir(n);
    await dormir(300);
    const aberto = processo();
    await dormir(2000);
    const repouso = processo();
    const enviado = new Map();
    let marca = 0;
    const t0 = performance.now();
    await Promise.all(Array.from({ length: GRAVADORES }, async () => {
      for (let k = 0; k < LOTES; k++) {
        const m = String(++marca);
        enviado.set(m, performance.now());
        const r = await post({ cliente: "medir", ops: [{ op: "alterar", tipo: "plano", id: "t1", campos: { texto: "marca-" + m } }] });
        if (!r.ok) throw new Error(`gravação recusada: ${r.status}`);
      }
    }));
    const gravar = performance.now() - t0;
    await dormir(1000);
    const atrasos = [];
    let faltou = 0;
    for (const v of vistos) for (const [m, t] of enviado) { const visto = v.get(m); if (visto === undefined) faltou++; else atrasos.push(visto - t); }
    fechar();
    linhas.push({ n, memMB: aberto.mem / 1048576, porConexaoKB: (aberto.mem - vazio.mem) / 1024 / n, threads: aberto.th, cpuRepousoMs: repouso.cpu - aberto.cpu, lotesPorS: (GRAVADORES * LOTES) / (gravar / 1000), p50: pct(atrasos, 0.5), p95: pct(atrasos, 0.95), max: Math.max(...atrasos), faltou });
    await dormir(500);
  }
  console.log(`servidor vazio: ${(vazio.mem / 1048576).toFixed(1)} MB, ${vazio.th} threads; ${GRAVADORES} gravadores x ${LOTES} lotes por rodada`);
  console.log("| conexões SSE | memória | por conexão | threads | CPU em 2 s parado | lotes/s | atraso p50 | p95 | máx. | perdidos |");
  console.log("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
  for (const l of linhas) console.log(`| ${l.n} | ${l.memMB.toFixed(1)} MB | ${l.porConexaoKB.toFixed(0)} KB | ${l.threads} | ${l.cpuRepousoMs.toFixed(0)} ms | ${l.lotesPorS.toFixed(0)} | ${l.p50.toFixed(1)} ms | ${l.p95.toFixed(1)} ms | ${l.max.toFixed(1)} ms | ${l.faltou} |`);
} finally {
  servidor.kill();
  setTimeout(() => { try { rmSync(pasta.dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 800);
}
