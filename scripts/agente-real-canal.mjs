// Teste de verdade (GASTA TOKENS, só com o Claude Code instalado e logado): o CANAL com um Claude real (docs/claude-na-folha.md, "Escolher a sessão").
// O canal só funciona em sessão INTERATIVA (em `-p` a flag de desenvolvimento é ignorada), então o Claude roda num pseudo-terminal (node-pty, instalado numa pasta
// temporária, nada entra no repositório). Tudo é temporário: servidor próprio (banco de exemplo, porta livre, nunca o app aberto nem a 3000), pasta de projeto, e o
// plugin é o MESMO que `blueprint claude` grava (APPDATA apontando para a pasta temporária, com um `claude` falso no PATH só para capturar), com as URLs dos hooks
// trocadas da 3000 para a porta do teste. A configuração do usuário não é tocada (`--setting-sources local` deixa de fora os plugins e hooks dele).
// Confere: (1) a sessão aparece em /api/mcp/sessoes com canal:true e some ao fechar (SessionEnd); (2) o POST /api/mcp/iniciar com o sessaoId faz a mensagem
// «The Blueprint · pedido do usuário» chegar à sessão ociosa e virar um turno em que o modelo age (chama o MCP); (3) se `plugin:blueprint@blueprint-local` vale
// para o plugin carregado por --plugin-dir (a tela diz). Modelo fixo: Sonnet 5.5, esforço médio; um único pedido, para custar poucos turnos.
// Uso: node scripts/agente-real-canal.mjs [blueprint.exe]   (padrão: .teste-target/release/blueprint.exe). ~1 a 3 min.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, copyFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { delimiter, join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, ".teste-target/release/blueprint.exe"));
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const sh = (cmd, args, opcoes = {}) => spawnSync(cmd, args, { encoding: "utf8", shell: true, ...opcoes });
assert(existsSync(exe), `compile antes: cargo build --release --target-dir .teste-target (${exe})`);
const versao = sh("claude", ["--version"]);
assert.equal(versao.status, 0, "o teste precisa do Claude Code (claude) no PATH");
const porta = await livre(), url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-canal-"));
const projeto = join(tmp, "projeto"), appdata = join(tmp, "appdata"), falso = join(tmp, "falso");
for (const d of [projeto, appdata, falso]) mkdirSync(d);
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
let terminal, texto = () => "";
const limpar = () => { try { terminal?.kill(); } catch {} try { servidor.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true }); } catch {} }, 1500); };
process.on("exit", limpar);
const post = (rota, corpo) => fetch(`${url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
const get = async (rota) => (await fetch(`${url}/api/mcp/${rota}`)).json();
const espera = async (descricao, ate, ms = 90000) => { const fim = Date.now() + ms; while (Date.now() < fim) { const v = await ate(); if (v) return v; await dormir(400); } throw new Error(`tempo esgotado: ${descricao}`); };
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const blueprint = (await (await fetch(url + "/api/dados")).json()).blueprints[0];
  const acao = blueprint.acoes[0];
  assert.equal((await post("pasta/vincular", { cwd: projeto, workspaceId: "sm", blueprintId: blueprint.id })).status, 200, "liga a pasta temporária ao blueprint");

  // node-pty numa pasta temporária (tem binário pronto para Windows)
  const ptyDir = join(tmp, "pty");
  mkdirSync(ptyDir);
  writeFileSync(join(ptyDir, "package.json"), "{}");
  assert.equal(sh("npm", ["i", "node-pty", "--no-audit", "--no-fund", "--silent"], { cwd: ptyDir }).status, 0, "npm i node-pty");
  const pty = createRequire(join(ptyDir, "x.js"))("node-pty");

  // o plugin real: `blueprint claude` grava o plugin em APPDATA\Blueprint e chama o `claude` do PATH (aqui, um falso que só sai)
  writeFileSync(join(falso, "claude.cmd"), "@echo off\r\nexit /b 0\r\n");
  const envFalso = { ...process.env, APPDATA: appdata, BLUEPRINT_URL: url, PATH: falso + delimiter + process.env.PATH };
  sh(`"${exe}"`, ["claude"], { cwd: projeto, env: envFalso });
  const pasta = join(appdata, "Blueprint", "claude-plugin", "plugins", "blueprint");
  const hooks = join(pasta, "hooks", "hooks.json");
  assert(existsSync(hooks), "o `blueprint claude` gravou o plugin");
  writeFileSync(hooks, readFileSync(hooks, "utf8").replaceAll("127.0.0.1:3000", `127.0.0.1:${porta}`)); // SessionEnd, Stop... vão ao servidor do teste, nunca ao app

  rmSync(join(pasta, "monitors"), { recursive: true, force: true }); // o monitor não é o que se testa e acordaria o modelo sozinho ("terminou sem saída"), sujando a sessão ociosa
  const marca = process.env.CANAL_MARCA || "plugin:blueprint@inline"; // @inline é o que o Claude Code dá a um plugin de --plugin-dir; src/cli/claude.rs (FLAG_CANAL) usa @blueprint-local
  const args = ["--plugin-dir", pasta, "--setting-sources", "local", "--model", "claude-sonnet-5-5", "--effort", "medium", "--allowedTools", "mcp__plugin_blueprint_blueprint__*", "--dangerously-load-development-channels", marca];
  let tela = "";
  terminal = pty.spawn("cmd.exe", ["/c", "claude", ...args], { name: "xterm-256color", cols: 140, rows: 40, cwd: projeto, env: { ...process.env, BLUEPRINT_URL: url, BLUEPRINT_CHANNEL: "1" } });
  terminal.onData((d) => { tela += d; }); terminal.onExit((e) => console.log("pty saiu", JSON.stringify(e), "tela:", texto().slice(-600)));
  texto = () => tela.replace(/\x1b\[[0-9;?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\x5c)/g, " ").replace(/\s+/g, " ");
  // confirmações de desenvolvimento e de confiança na pasta: Enter na primeira opção
  let ultimoEnter = 0, lidoAte = 0;
  let ocupado = false;
  const responde = setInterval(async () => {
    if (ocupado) return;
    ocupado = true;
    const t = texto().slice(lidoAte);
    if (Date.now() - ultimoEnter > 2500 && /Enter to confirm/i.test(t)) { await dormir(1000); if (/❯ (No|Exit)/.test(t.slice(-600))) { terminal.write("\x1b[B"); await dormir(300); } terminal.write("\r"); ultimoEnter = Date.now(); lidoAte = texto().lastIndexOf("Enter to confirm") + 16; // só vale "Enter to confirm" mais novo que este
       console.log("enter enviado diante de:", t.slice(-160)); } // o padrão da confiança é "No, exit"
    ocupado = false;
  }, 500);

  // (1) a sessão aparece com canal:true
  const lista = await espera("sessão com canal:true na lista", async () => (await get(`sessoes?workspace=sm&blueprint=${blueprint.id}`)).sessoes.find((s) => s.canal));
  console.log("(1) sessão com canal:", JSON.stringify(lista));
  console.log("    tela após subir:", texto().slice(-400));

  // (2) o pedido do usuário, pelo botão (a rota que a página chama)
  await espera("a sessão ociosa", async () => { await dormir(2500); return /Try "/.test(texto().slice(-1500)) && !/esc to interrupt/.test(texto().slice(-300)); }, 60000);
  const marcoTela = texto().length;
  const r = await post("iniciar", { workspaceId: "sm", blueprintId: blueprint.id, acaoId: acao.id, sessaoId: lista.id });
  const corpo = await r.json();
  console.log("(2) POST /api/mcp/iniciar:", r.status, JSON.stringify(corpo).slice(0, 200));
  assert.equal(r.status, 200, "o pedido é entregue ao canal");
  const apos = (marco) => texto().slice(marco);
  await espera("a mensagem do canal na tela", async () => /pedido do usu.rio/i.test(apos(marcoTela)), 60000);
  console.log("    a mensagem chegou à sessão ociosa:", (apos(marcoTela).match(/.{0,60}pedido do usu.rio.{0,160}/i) || [""])[0]);
  // o turno acaba quando o Claude Code mostra "done HH:MM" (fim do cronômetro); o modelo tem de ter agido sobre a ação do pedido (1.1 «Protótipo navegável»)
  const depoisDaMensagem = () => { const t = apos(marcoTela); const i = t.search(/pedido do usu.rio/i); return t.slice(i); };
  await espera("o fim do turno do pedido", async () => /done \d+:\d+ [AP]M/.test(depoisDaMensagem()), 150000);
  const resposta = depoisDaMensagem();
  console.log("    chamadas ao MCP vistas na tela:", [...new Set(resposta.match(/(ler_blueprint|registrar_progresso|marcar_concluido|guia_de_planejamento|listar_blueprints|plugin_blueprint_blueprint\w*)/g) || [])].join(", ") || "(a interface recolhe as chamadas; o modelo citou dados que só o MCP traz)");
  console.log("    resposta (fim da tela):", resposta.slice(-600));
  assert(/Protótipo navegável/.test(resposta), "o modelo agiu sobre a ação do pedido (1.1), que só a mensagem do canal citava");

  // (1, final) fecha e confere o SessionEnd
  terminal.write("/exit\r");
  const saiu = await espera("a sessão sumir da lista (SessionEnd)", async () => !(await get(`sessoes?workspace=sm&blueprint=${blueprint.id}`)).sessoes.length, 30000).catch(() => false);
  console.log("(1) após /exit a lista ficou vazia:", saiu);
  clearInterval(responde);
  assert(saiu, "a sessão some da lista ao fechar");
  console.log(`canal real: ok · Claude Code ${versao.stdout.trim()} · flag ${marca}`);
} catch (e) {
  console.error("FALHOU:", e.message, "| canal na tela:", (texto().match(/.{0,200}channel.{0,300}/gi) || []).join(" ||| "), "| fim:", texto().slice(-800));
  process.exitCode = 1;
} finally { limpar(); setTimeout(() => process.exit(process.exitCode || 0), 2000); }
