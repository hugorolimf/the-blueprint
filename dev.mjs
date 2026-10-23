// The Blueprint (beta), versão de desenvolvimento: `node dev.mjs` (ou `npm run dev`).
//   --porta N   porta da página (padrão 3917; o servidor Rust fica em N+1, atrás de um proxy que injeta o recarregamento)
//   --limpar    apaga o banco de desenvolvimento (.dev/) e recomeça dos dados de exemplo
//   --abrir     abre a página no navegador ao subir
// O que acontece ao salvar um arquivo:
//   web/, kit/, blueprint.html, emblemas.mjs, agentes-figura.mjs -> o servidor lê da pasta (sem compilar); a página recarrega
//                                                                  (só CSS: troca o estilo sem perder o estado da tela)
//   src/, skills/, Cargo.toml, build.rs                          -> `cargo build`, reinicia o servidor e recarrega a página;
//                                                                  se não compilar, o servidor antigo continua e o erro aparece na página
// Os dados de desenvolvimento ficam em .dev/ (nunca tocam %APPDATA%\Blueprint, o banco vivo). Não roda testes: isso é com `cargo test`.
import { spawn } from "node:child_process";
import { copyFileSync, mkdirSync, rmSync, watch } from "node:fs";
import http from "node:http";
import { dirname, extname, join } from "node:path";
import { fileURLToPath } from "node:url";

const RAIZ = dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const porta = Number(args.find((a) => a.startsWith("--porta="))?.split("=")[1] ?? args[args.indexOf("--porta") + 1] ?? 3917) || 3917;
const interna = porta + 1;
const DEV = join(RAIZ, ".dev");
const EXE_COMPILADO = join(RAIZ, "target", "debug", "blueprint.exe");
// o servidor roda de uma cópia: o cargo não consegue regravar um .exe em uso no Windows
const EXE_EM_USO = join(DEV, "blueprint-dev.exe");
const log = (m) => console.log(`\x1b[36m[dev]\x1b[0m ${m}`);

if (args.includes("--limpar")) { rmSync(DEV, { recursive: true, force: true }); log("banco de desenvolvimento apagado"); }
mkdirSync(DEV, { recursive: true });

// ---- navegador: SSE de recarga e o script que a página recebe ----
const clientes = new Set();
const enviar = (evento) => { for (const r of clientes) r.write(`data: ${JSON.stringify(evento)}\n\n`); };
const SCRIPT = `(() => {
  const barra = document.createElement("pre");
  barra.style.cssText = "position:fixed;left:0;right:0;bottom:0;max-height:40vh;overflow:auto;margin:0;padding:8px 12px;background:#5a1313;color:#fff;font:12px monospace;z-index:2147483647;white-space:pre-wrap;display:none";
  document.documentElement.appendChild(barra);
  new EventSource("/__dev/eventos").onmessage = (m) => {
    const d = JSON.parse(m.data);
    if (d.tipo === "reload") location.reload();
    else if (d.tipo === "css") document.querySelectorAll('link[rel="stylesheet"]').forEach((l) => { l.href = l.href.split("?")[0] + "?v=" + Date.now(); });
    else if (d.tipo === "erro") { barra.textContent = d.texto; barra.style.display = "block"; }
  };
})();`;

// ---- proxy: a página e a API passam por aqui; só a página inicial ganha o script ----
const ehPagina = (url) => /^\/(blueprint\.html)?(\?|$)/.test(url);
http.createServer((req, res) => {
  if (req.url === "/__dev/eventos") {
    res.writeHead(200, { "content-type": "text/event-stream", "cache-control": "no-store" });
    res.write(": ok\n\n");
    clientes.add(res);
    return void req.on("close", () => clientes.delete(res));
  }
  if (req.url === "/__dev/recarregar.js") return void res.writeHead(200, { "content-type": "text/javascript", "cache-control": "no-store" }).end(SCRIPT);
  // o servidor confere Host e Origin (loopback, mesma porta): repassa como se o pedido fosse direto a ele
  const headers = { ...req.headers, host: `127.0.0.1:${interna}`, "accept-encoding": "identity" };
  if (headers.origin) headers.origin = `http://127.0.0.1:${interna}`;
  const rumo = http.request({ host: "127.0.0.1", port: interna, method: req.method, path: req.url, headers }, (r) => {
    if (!ehPagina(req.url) || !String(r.headers["content-type"]).startsWith("text/html")) {
      res.writeHead(r.statusCode, r.headers);
      return void r.pipe(res);
    }
    const partes = [];
    r.on("data", (d) => partes.push(d));
    r.on("end", () => {
      const html = Buffer.concat(partes).toString("utf8");
      const tag = '<script src="/__dev/recarregar.js"></script>';
      const corpo = Buffer.from(html.includes("</body>") ? html.replace("</body>", `${tag}</body>`) : html + tag);
      res.writeHead(r.statusCode, { ...r.headers, "content-length": corpo.length });
      res.end(corpo);
    });
  });
  rumo.on("error", () => { if (!res.headersSent) res.writeHead(503, { "content-type": "text/plain; charset=utf-8" }); res.end("servidor reiniciando…"); });
  req.pipe(rumo);
}).listen(porta, "127.0.0.1", () => log(`página em http://127.0.0.1:${porta}  (servidor Rust em ${interna}, dados em .dev/)`));

// ---- servidor Rust ----
let servidor = null;
const parar = () => new Promise((ok) => {
  if (!servidor) return ok();
  const s = servidor;
  servidor = null;
  s.once("exit", ok);
  s.kill();
});
const iniciar = () => {
  const env = { ...process.env, HOST: "127.0.0.1", PORT: String(interna), BLUEPRINT_DB: join(DEV, "blueprint.db"), PROTOTIPOS_DIR: join(DEV, "prototipos"), DADOS_ARQ: join(RAIZ, "data.exemplo.json"), BLUEPRINT_ASSETS_DIR: RAIZ };
  servidor = spawn(EXE_EM_USO, ["serve"], { env, stdio: "inherit", windowsHide: true });
  servidor.on("exit", (codigo) => { if (servidor) { log(`o servidor saiu (código ${codigo}); salve um arquivo de src/ para recompilar`); servidor = null; } });
};
const aguardar = async () => {
  for (let i = 0; i < 80; i++) {
    const ok = await new Promise((r) => http.get({ host: "127.0.0.1", port: interna, path: "/" }, (x) => { x.resume(); r(x.statusCode === 200); }).on("error", () => r(false)));
    if (ok) return true;
    await new Promise((r) => setTimeout(r, 150));
  }
  return false;
};
const cargo = () => new Promise((ok) => {
  const p = spawn("cargo", ["build", "--color", "never"], { cwd: RAIZ, windowsHide: true });
  let saida = "";
  p.stdout.on("data", (d) => { saida += d; });
  p.stderr.on("data", (d) => { saida += d; process.stderr.write(d); });
  p.on("error", (e) => ok({ codigo: -1, saida: String(e) }));
  p.on("close", (codigo) => ok({ codigo, saida }));
});

let compilando = false, pendente = null;
async function reconstruir(motivo) {
  if (compilando) { pendente = motivo; return; }
  compilando = true;
  log(`compilando (${motivo})…`);
  const t0 = Date.now();
  const r = await cargo();
  if (r.codigo !== 0) {
    log("a compilação falhou: o servidor anterior continua no ar");
    enviar({ tipo: "erro", texto: r.saida.trim().split("\n").slice(-40).join("\n") });
  } else {
    await parar();
    copyFileSync(EXE_COMPILADO, EXE_EM_USO);
    iniciar();
    if (await aguardar()) { log(`servidor reiniciado em ${((Date.now() - t0) / 1000).toFixed(1)} s`); enviar({ tipo: "reload" }); }
    else log("o servidor não respondeu a tempo; veja a saída acima");
  }
  compilando = false;
  if (pendente) { const m = pendente; pendente = null; reconstruir(m); }
}

// ---- vigia de arquivos (com espera, porque um salvamento gera vários eventos) ----
const ignorado = /(~|\.swp|\.tmp|\.crswap)$/;
function vigiar(pasta, recursivo, aceita, ms, aoMudar) {
  let t, nomes = new Set();
  watch(join(RAIZ, pasta), { recursive: recursivo }, (_, nome) => {
    if (!nome || ignorado.test(nome) || !aceita(nome)) return;
    nomes.add(join(pasta, nome));
    clearTimeout(t);
    t = setTimeout(() => { const n = [...nomes]; nomes = new Set(); aoMudar(n); }, ms);
  });
}
const recarregar = (arquivos) => {
  const soCss = arquivos.every((a) => extname(a) === ".css");
  log(`${arquivos.join(", ")} → ${soCss ? "estilo trocado" : "página recarregada"}`);
  enviar({ tipo: soCss ? "css" : "reload" });
};
const nomesDaInterface = ["blueprint.html", "emblemas.mjs", "agentes-figura.mjs", "agentes-claude.mjs"];
const nomesDoRust = ["Cargo.toml", "build.rs"];
for (const p of ["web", "kit"]) vigiar(p, true, (n) => /\.(css|js|mjs)$/.test(n), 120, recarregar);
vigiar("src", true, (n) => /\.(rs|sql|json|txt)$/.test(n), 400, (a) => reconstruir(a[0]));
vigiar("skills", true, (n) => n.endsWith(".md"), 400, (a) => reconstruir(a[0]));
vigiar(".", false, (n) => nomesDaInterface.includes(n) || nomesDoRust.includes(n), 150, (a) => {
  const rust = a.filter((n) => nomesDoRust.includes(n));
  if (rust.length) reconstruir(rust[0]);
  const ui = a.filter((n) => nomesDaInterface.includes(n));
  if (ui.length) recarregar(ui);
});

const sair = async () => { await parar(); process.exit(0); };
process.on("SIGINT", sair);
process.on("SIGTERM", sair);
process.on("exit", () => servidor?.kill());

await reconstruir("primeira compilação");
if (args.includes("--abrir") && servidor) spawn("cmd", ["/c", "start", "", `http://127.0.0.1:${porta}`], { windowsHide: true, stdio: "ignore" });
log("pronto: edite web/, kit/, src/… e salve; Ctrl+C para sair");
