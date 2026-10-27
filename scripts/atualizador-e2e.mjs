// O updater de ponta a ponta com o binário de verdade e um "GitHub" falso: a versão nova é achada e baixada em segundo plano, fica pronta SEM instalar,
// e só o clique (POST /api/atualizacao/instalar) roda o instalador; item na fila do MCP não impede e volta quando o app reabre. Nenhum instalador real roda: o app anota o que rodaria.
// Uso: node scripts/atualizador-e2e.mjs [caminho do blueprint.exe]   (padrão: target/debug/blueprint.exe; ~10 s, sem gastar tokens)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { access, readdir, readFile, rm } from "node:fs/promises";
import { createServer } from "node:http";
import { join } from "node:path";
import { dormir, exePadrao, ligarServidor, pastaDeDados, portaLivre, postJson, http } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
const existe = (p) => access(p).then(() => true, () => false);
const instalador = Buffer.from("MZ instalador de mentira ".repeat(2000));

// o "GitHub": a lista de releases e os arquivos; `hashErrado` troca o SHA256SUMS por um que não confere
async function githubFalso({ hashErrado = false } = {}) {
  const porta = await portaLivre(), base = `http://127.0.0.1:${porta}`, nome = "Blueprint-0.9.0-instalador.exe";
  const hash = hashErrado ? "0".repeat(64) : createHash("sha256").update(instalador).digest("hex");
  const pedidos = [];
  const servidor = createServer((req, res) => {
    pedidos.push(req.url);
    if (req.url.startsWith("/releases")) return res.end(JSON.stringify([{ tag_name: "v0.9.0", draft: false, prerelease: true, body: "Notas da 0.9.0\nlinha 2", assets: [
      { name: nome, size: instalador.length, browser_download_url: `${base}/baixar/${nome}` }, { name: "SHA256SUMS", size: 100, browser_download_url: `${base}/baixar/SHA256SUMS` }] }]));
    if (req.url === `/baixar/${nome}`) return res.end(instalador);
    if (req.url === "/baixar/SHA256SUMS") return res.end(`${hash}  ${nome}\n`);
    res.statusCode = 404; res.end("nada");
  });
  await new Promise((ok) => servidor.listen(porta, "127.0.0.1", ok));
  return { base, pedidos, parar: () => servidor.close() };
}

async function esperarFase(url, fase, ms = 15000) {
  for (let i = 0; i < ms / 100; i++) { const e = (await http(url + "/api/atualizacao")).json; if (e.fase === fase) return e; await dormir(100); }
  throw new Error(`a atualização não chegou a "${fase}": ${JSON.stringify((await http(url + "/api/atualizacao")).json)}`);
}

async function rodada(nome, opcoes, roteiro) {
  const gh = await githubFalso(opcoes);
  const d = await pastaDeDados("blueprint-atualizador");
  d.env.BLUEPRINT_ATUALIZACAO_URL = gh.base;
  d.env.BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR = join(d.dir, "lancou.txt");
  if (opcoes.semServidor) d.env.BLUEPRINT_URL = `http://127.0.0.1:${await portaLivre()}`; // o app está fechado: o CLI instala sozinho
  const servidor = opcoes.semServidor ? null : await ligarServidor(exe, d);
  try { await roteiro({ ...d, gh, servidor, lancou: d.env.BLUEPRINT_ATUALIZACAO_SEM_EXECUTAR, atualizacao: join(d.dir, "atualizacao") }); console.log(`ok: ${nome}`); }
  finally { servidor?.kill(); gh.parar(); await rm(d.dir, { recursive: true, force: true }).catch(() => {}); }
}

await rodada("baixa em segundo plano, fica pronta sem instalar e só o clique instala", {}, async ({ url, gh, lancou, atualizacao, dir }) => {
  assert.equal((await http(url + "/api/atualizacao")).json.fase, "parada");
  assert.equal((await http(url + "/api/atualizacao/verificar", { metodo: "POST", cabecalhos: { "content-type": "text/plain" }, corpo: "{}" })).status, 415, "POST sem JSON");
  assert.equal((await postJson(url + "/api/atualizacao/verificar", {}, { origin: "http://evil.example" })).status, 403, "página de outro site");
  assert.equal((await http(url + "/api/atualizacao", { cabecalhos: { host: "evil.example" } })).status, 403, "Host fora do loopback");
  assert.equal((await postJson(url + "/api/atualizacao/instalar")).status, 409, "sem versão pronta não há o que instalar");
  assert.equal((await postJson(url + "/api/atualizacao/verificar")).status, 202);
  const pronta = await esperarFase(url, "pronta");
  assert.equal(pronta.versao, "0.9.0");
  assert.match(pronta.notas, /Notas da 0\.9\.0/);
  assert.deepEqual((await readdir(atualizacao)).filter((n) => n.endsWith(".exe")), ["Blueprint-0.9.0-instalador.exe"]);
  assert.equal(await existe(lancou), false, "baixar e conferir NÃO executam o instalador");
  // a fila do MCP tem trabalho do agente que a tela não aplicou: NÃO impede mais (a fila fica no banco e volta ao reabrir; conferido abaixo)
  assert.equal((await postJson(url + "/api/mcp/evento", { tipo: "progresso", texto: "trabalhando", workspaceId: "sm", blueprintId: "p1" })).status, 200);
  // o clique
  const ok = await postJson(url + "/api/atualizacao/instalar");
  assert.equal(ok.status, 200, ok.texto);
  const anotado = await readFile(lancou, "utf8");
  assert.match(anotado, /Blueprint-0\.9\.0-instalador\.exe/);
  for (const arg of ["/SUPPRESSMSGBOXES", "/CLOSEAPPLICATIONS", "/NORESTART", "/ATUALIZACAO=1"]) assert(anotado.includes(arg), arg);
  assert.equal((await http(url + "/api/atualizacao")).json.fase, "instalando");
  assert((await readdir(join(dir, "backups"))).some((n) => n.startsWith("blueprint-antes-de-0.9.0") && n.endsWith(".db")), "backup do banco antes de instalar");
  assert(gh.pedidos.length >= 3);
});

await rodada("item na fila atravessa a atualização: o app reabre com ele", {}, async ({ url, env, lancou, servidor }) => {
  await postJson(url + "/api/atualizacao/verificar");
  await esperarFase(url, "pronta");
  assert.equal((await postJson(url + "/api/mcp/evento", { tipo: "progresso", texto: "trabalhando", workspaceId: "sm", blueprintId: "p1" })).status, 200);
  assert.equal((await postJson(url + "/api/atualizacao/instalar")).status, 200, "a fila não impede");
  assert.match(await readFile(lancou, "utf8"), /\/ATUALIZACAO=1/);
  servidor.kill(); // o instalador fecha o app e o reabre
  await new Promise((ok) => servidor.once("exit", ok));
  const reaberto = await ligarServidor(exe, { env, url });
  try {
    const fila = (await (await fetch(url + "/api/mcp/fila")).json()).itens;
    assert.deepEqual(fila.map((i) => [i.tipo, i.texto]), [["progresso", "trabalhando"]], "o item que a tela não aplicou voltou");
  } finally { reaberto.kill(); }
});

await rodada("SHA-256 que não confere: nada fica no disco e nada roda", { hashErrado: true }, async ({ url, lancou, atualizacao }) => {
  await postJson(url + "/api/atualizacao/verificar");
  const e = await esperarFase(url, "erro");
  assert.match(e.erro, /não confere/);
  assert.deepEqual((await readdir(atualizacao).catch(() => [])).filter((n) => n.startsWith("Blueprint-")), []);
  assert.equal((await postJson(url + "/api/atualizacao/instalar")).status, 409);
  assert.equal(await existe(lancou), false);
});

await rodada("desligar a verificação automática", {}, async ({ url }) => {
  assert.equal((await postJson(url + "/api/atualizacao/automatica", { ligada: false })).json.automatica, false);
  assert.equal((await http(url + "/api/atualizacao")).json.fase, "desligada");
  assert.equal((await postJson(url + "/api/atualizacao/automatica", { ligada: "talvez" })).status, 400);
  assert.equal((await postJson(url + "/api/atualizacao/automatica", { ligada: true })).json.automatica, true);
});

await rodada("linha de comando sem o app: --verificar só consulta, a segunda baixa, --instalar é o clique", { semServidor: true }, async ({ env, lancou, atualizacao, dir }) => {
  // assíncrono de propósito: o "GitHub" falso roda neste mesmo processo e não responderia com o laço bloqueado (spawnSync)
  const cli = (...args) => new Promise((ok) => { const f = spawn(exe, args, { env, windowsHide: true }); let stdout = "", stderr = ""; f.stdout.on("data", (d) => (stdout += d)); f.stderr.on("data", (d) => (stderr += d)); f.on("close", (status) => ok({ status, stdout, stderr })); });
  assert.match((await cli("--version")).stdout, /^blueprint \d+\.\d+\.\d+/);
  const semBaixar = await cli("atualizar", "--instalar");
  assert.equal(semBaixar.status, 1);
  assert.match(semBaixar.stderr, /não há atualização baixada/);
  const consulta = await cli("atualizar", "--verificar");
  assert.equal(consulta.status, 0, consulta.stderr);
  assert.match(consulta.stdout, /Nova versão 0\.9\.0/);
  assert.equal(await existe(atualizacao), false, "só consultou: nada baixado");
  const baixa = await cli("atualizar");
  assert.equal(baixa.status, 0, baixa.stderr);
  assert.match(baixa.stdout, /baixada e conferida/);
  assert.deepEqual((await readdir(atualizacao)).filter((n) => n.endsWith(".exe")), ["Blueprint-0.9.0-instalador.exe"]);
  assert.equal(await existe(lancou), false, "baixar não instala");
  const instala = await cli("atualizar", "--instalar");
  assert.equal(instala.status, 0, instala.stderr);
  assert.match(await readFile(lancou, "utf8"), /Blueprint-0\.9\.0-instalador\.exe .*\/ATUALIZACAO=1/);
  assert((await readdir(join(dir, "backups"))).some((n) => n.endsWith(".db")), "o banco foi copiado antes");
});
