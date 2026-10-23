// Ponta a ponta da rota POST /api/mcp/iniciar (o "Iniciar com o Claude"): liga o `blueprint serve` de verdade com um `wt` FALSO no PATH (nenhum terminal real abre) e confere
// 404, 409 sem-pasta, 200 com os argumentos EXATOS que o terminal recebeu, 400/415, Host e Origin de fora, e que o MCP_REDE=1 NÃO libera esta rota (só ela: o resto do MCP, sim).
// Uso: node scripts/iniciar-e2e.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe)
import assert from "node:assert/strict";
import http from "node:http";
import { mkdirSync, mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dir = mkdtempSync(join(tmpdir(), "blueprint-iniciar-"));
const pasta = join(dir, "projeto-ligado");
mkdirSync(pasta);
const falso = terminalFalso(dir);
const servidores = [];
const parar = () => { servidores.forEach((s) => s.parar()); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", parar);

// pedido cru, para poder mentir o Host e a Origin (o fetch do Node não deixa)
const pedir = (s, { metodo = "POST", caminho = "/api/mcp/iniciar", host, origem, tipo = "application/json", corpo }) => new Promise((ok, erro) => {
  const cab = { host: host ?? `127.0.0.1:${s.porta}`, ...(origem ? { origin: origem } : {}), ...(corpo !== undefined ? { "content-type": tipo } : {}) };
  const r = http.request({ host: "127.0.0.1", port: s.porta, method: metodo, path: caminho, headers: cab }, (res) => { let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let j = null; try { j = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json: j }); }); });
  r.on("error", erro);
  if (corpo !== undefined) r.write(typeof corpo === "string" ? corpo : JSON.stringify(corpo));
  r.end();
});
const ALVO = { workspaceId: "sm", blueprintId: "bp1a2b3", acaoId: "ac1a002" }; // a 2ª ação do 1º blueprint do data.exemplo.json: número 1.2
const norm = (t) => String(t).replaceAll("\\", "/").toLowerCase();

try {
  const s = await sobeServidor(exe, dir, falso);
  servidores.push(s);

  assert.equal((await pedir(s, { corpo: { ...ALVO, acaoId: "nao-existe" } })).status, 404, "ação inexistente: 404");
  assert.equal((await pedir(s, { corpo: { ...ALVO, blueprintId: "nao-existe" } })).status, 404, "blueprint inexistente: 404");
  const semPasta = await pedir(s, { corpo: ALVO });
  assert.equal(semPasta.status, 409, "sem pasta ligada: 409");
  assert.equal(semPasta.json.codigo, "sem-pasta");
  assert.equal(falso.chamadas().length, 0, "nada abriu até aqui");

  assert.equal((await pedir(s, { caminho: "/api/mcp/pasta/vincular", corpo: { cwd: pasta, workspaceId: "sm", blueprintId: "bp1a2b3" } })).status, 200, "liga a pasta temporária ao blueprint");
  const ok = await pedir(s, { corpo: ALVO });
  assert.equal(ok.status, 200, `com pasta ligada: 200 (${ok.texto})`);
  assert.deepEqual([ok.json.ok, ok.json.terminal], [true, "wt"]);
  assert.equal(norm(ok.json.pasta), norm(pasta), "a resposta diz a pasta");
  for (let i = 0; i < 60 && !falso.chamadas().length; i++) await dormir(100); // o terminal falso grava depois que o app responde
  const [chamada, ...resto] = falso.chamadas();
  assert.equal(resto.length, 0, "uma chamada ao terminal");
  assert.equal(chamada.length, 6, `seis argumentos: -d, pasta, exe, claude, prompt, --canal (veio ${JSON.stringify(chamada)})`);
  assert.equal(chamada[5], "--canal", "a sessão que o botão abre nasce com o canal ligado (depois do prompt)");
  assert.equal(chamada[0], "-d");
  assert.equal(norm(chamada[1]), norm(pasta), "o terminal abre NA pasta do blueprint");
  assert.equal(norm(chamada[2]), norm(exe), "roda o próprio blueprint.exe");
  assert.equal(chamada[3], "claude");
  const prompt = chamada[4];
  const nome = JSON.parse(readFileSync(join(raiz, "data.exemplo.json"), "utf8")).blueprints.find((p) => p.id === ALVO.blueprintId).acoes.find((f) => f.id === ALVO.acaoId).nome;
  assert(prompt.startsWith(`Execute a ação 1.2 «${nome.replaceAll(";", ",")}» do blueprint «`), `o prompt é UM argumento e traz o número e o nome da ação: ${prompt.slice(0, 90)}`);
  for (const parte of ["workspace_id sm", "blueprint_id bp1a2b3", "acao_id ac1a002", "ler_blueprint", "registrar_progresso", "marcar_concluido"]) assert(prompt.includes(parte), `o prompt traz ${parte}`);
  assert(!prompt.includes(";"), "sem ';' (o Windows Terminal o lê como separador de comandos)");

  // recusas: nada disso pode chegar ao terminal
  const antes = falso.chamadas().length;
  assert.equal((await pedir(s, { corpo: { ...ALVO, acaoId: "a\u0007b" } })).status, 400, "id com caractere de controle: 400");
  assert.equal((await pedir(s, { corpo: { workspaceId: "sm", blueprintId: "bp1a2b3" } })).status, 400, "sem acaoId: 400");
  assert.equal((await pedir(s, { corpo: { ...ALVO, acaoId: "x".repeat(101) } })).status, 400, "id comprido demais: 400");
  assert.equal((await pedir(s, { tipo: "text/plain", corpo: JSON.stringify(ALVO) })).status, 415, "sem content-type JSON: 415");
  assert.equal((await pedir(s, { host: "evil.example", corpo: ALVO })).status, 403, "Host de fora: 403");
  assert.equal((await pedir(s, { host: `localhost:${s.porta}`, origem: "http://evil.example", corpo: ALVO })).status, 403, "Origin de fora: 403");
  assert.equal((await pedir(s, { host: `localhost:${s.porta}`, corpo: ALVO })).status, 200, "localhost é loopback: 200 (e esta chamada abre o terminal falso de novo)");
  for (let i = 0; i < 60 && falso.chamadas().length < antes + 1; i++) await dormir(100);
  assert.equal(falso.chamadas().length, antes + 1, "só a chamada válida chegou ao terminal; as recusadas não");

  // MCP_REDE=1 libera o resto do MCP para outras máquinas, mas NÃO esta rota (ela abre processo)
  const r = await sobeServidor(exe, mkdtempSync(join(dir, "rede-")), falso, { MCP_REDE: "1" });
  servidores.push(r);
  assert.equal((await pedir(r, { metodo: "GET", caminho: "/api/mcp/blueprints", host: "outra-maquina:3000" })).status, 200, "com MCP_REDE=1 o resto do MCP responde a outro Host");
  const antesRede = falso.chamadas().length;
  assert.equal((await pedir(r, { host: "outra-maquina:3000", corpo: ALVO })).status, 403, "com MCP_REDE=1, iniciar continua recusando outro Host");
  assert.equal((await pedir(r, { host: `127.0.0.1:${r.porta}`, origem: "http://outra-maquina:3000", corpo: ALVO })).status, 403, "com MCP_REDE=1, iniciar continua recusando outra Origin");
  await dormir(500);
  assert.equal(falso.chamadas().length, antesRede, "nenhum terminal abriu pelo MCP_REDE");
  console.log("iniciar a ação (ponta a ponta): ok (404, 409, 200 com os argumentos exatos do terminal, 400, 415, Host e Origin de fora, MCP_REDE=1 não libera)");
} finally { parar(); }
