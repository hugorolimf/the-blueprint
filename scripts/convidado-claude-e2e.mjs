// O Claude do convidado ponta a ponta (planos "Conectar o Claude do convidado num comando" e "Levar os hooks do convidado ao servidor"):
// o dono liga o compartilhamento (túnel falso, scripts/tunel-falso.mjs) e convida; a convidada entra pelo túnel, gera a chave e roda
// `blueprint conectar` com APPDATA, CLAUDE_CONFIG_DIR e CODEX_HOME temporários e um `claude`/`codex` falsos no PATH (anotam os argumentos);
// o `--mcp --equipe` dela lê e registra progresso pelo túnel; os hooks `sessao`, `edicao` e `parada` (`--equipe`) chegam ao app do dono,
// e a figura "Claude · Bia" aparece na lista de agentes da porta local. Banco temporário; nada toca a configuração real nem gasta tokens.
//   node scripts/convidado-claude-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; ~10 s; Windows)
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { request } from "node:http";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { dormir, exePadrao, mcpStdio, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = exePadrao();
assert.ok(existsSync(exe), `não achei ${exe}`);
const WS = "lab0001", BP = "bp7g8h9"; // data.exemplo.json: o workspace "Laboratório de ideias" e o blueprint dele
const tmp = mkdtempSync(join(tmpdir(), "convidado-claude-e2e-"));
const porta = await portaLivre(), url = `http://127.0.0.1:${porta}`, info = join(tmp, "tunel.json");
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
writeFileSync(join(tmp, "tunel-falso.cmd"), `@node "${join(raiz, "scripts", "tunel-falso.mjs")}" %*\r\n`);
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_SEM_ATUALIZACAO: "1", BLUEPRINT_TUNEL_COMANDO: join(tmp, "tunel-falso.cmd"), TUNEL_FALSO_INFO: info }, stdio: "ignore", windowsHide: true });

// o computador da convidada: pastas temporárias e CLIs falsas na frente do PATH
const conv = join(tmp, "convidada"), bin = join(conv, "bin"), log = join(bin, "log.txt"), projeto = join(conv, "projeto");
for (const p of [bin, join(conv, "appdata"), join(conv, "claude"), join(conv, "codex"), join(projeto, "src")]) mkdirSync(p, { recursive: true });
for (const cli of ["claude", "codex"]) writeFileSync(join(bin, `${cli}.cmd`), `@echo off\r\necho ${cli} %*>> "${log}"\r\n`);
writeFileSync(join(projeto, "CLAUDE.md"), `# Projeto da Bia\n\nBlueprint: workspace_id \`${WS}\`, blueprint_id \`${BP}\`\n`);
const envConv = { ...process.env, APPDATA: join(conv, "appdata"), CLAUDE_CONFIG_DIR: join(conv, "claude"), CODEX_HOME: join(conv, "codex"), PATH: `${bin};${process.env.PATH}`, BLUEPRINT_CONECTAR_HTTP_LOCAL: "1", BLUEPRINT_IDIOMA: "pt-BR" };
delete envConv.BLUEPRINT_URL; delete envConv.BLUEPRINT_CHAVE;
const hook = (evento, corpo) => spawnSync(exe, ["hook", evento, "--equipe"], { cwd: projeto, env: envConv, input: JSON.stringify(corpo), encoding: "utf8", timeout: 10000, windowsHide: true });

/** Pedido HTTP com cabeçalhos livres; devolve status, json e os cabeçalhos da resposta. */
const pedir = (endereco, { metodo = "GET", cabecalhos = {}, corpo } = {}) => new Promise((ok, falha) => {
  const u = new URL(endereco), dados = corpo === undefined ? undefined : JSON.stringify(corpo);
  const req = request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search, method: metodo, headers: { ...(dados ? { "content-type": "application/json" } : {}), ...cabecalhos } }, (res) => {
    let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let json = null; try { json = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json, cabecalhos: res.headers }); });
  });
  req.on("error", falha);
  if (dados) req.write(dados);
  req.end();
});

let mcp;
try {
  for (let i = 0; ; i++) { try { await fetch(url + "/api/dados"); break; } catch { if (i > 100) throw new Error("o servidor não respondeu"); await dormir(50); } }
  const local = { origin: url };
  // 1. o dono liga o compartilhamento e convida uma colaboradora
  assert.equal((await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS } })).status, 202);
  let estado;
  for (let i = 0; i < 200 && estado?.estado !== "ligado"; i++) { estado = (await pedir(`${url}/api/compartilhar`)).json; if (estado?.estado !== "ligado") await dormir(50); }
  assert.equal(estado.estado, "ligado", JSON.stringify(estado));
  const t = JSON.parse(readFileSync(info, "utf8"));
  const convite = await pedir(`${url}/api/convites`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, papel: "colaborador" } });
  assert.equal(convite.status, 201, convite.texto);
  const segredo = convite.json.link.split("#")[1];

  // 2. a convidada entra pelo túnel e gera a chave; a tela traz o comando pronto no endereço do túnel
  const entrou = await pedir(`${t.local}/api/convite/entrar`, { metodo: "POST", cabecalhos: { origin: t.local }, corpo: { segredo, nome: "Bia" } });
  assert.equal(entrou.status, 200, entrou.texto);
  const cookie = String(entrou.cabecalhos["set-cookie"]).split(";")[0];
  const gerada = await pedir(`${t.local}/api/chave`, { metodo: "POST", cabecalhos: { origin: t.local, cookie }, corpo: {} });
  assert.equal(gerada.status, 201, gerada.texto);
  const chave = gerada.json.chave;
  assert.equal(gerada.json.comandos.conectar, `blueprint conectar ${t.publica} ${chave}`, "o comando pronto aponta para o endereço do túnel");

  // 3. `blueprint conectar` no computador dela (o túnel falso fala http na porta local dele)
  const ruim = spawnSync(exe, ["conectar", t.local, "bpk_errada"], { env: envConv, encoding: "utf8", windowsHide: true });
  assert.equal(ruim.status, 2, "chave fora do formato é recusada: " + ruim.stdout);
  const ligou = spawnSync(exe, ["conectar", t.local, chave], { env: envConv, encoding: "utf8", windowsHide: true, timeout: 30000 });
  assert.equal(ligou.status, 0, ligou.stdout + ligou.stderr);
  assert.match(ligou.stdout, /plugin `blueprint-equipe` instalado/);
  const chamadas = readFileSync(log, "utf8");
  assert.match(chamadas, /claude plugin install blueprint-equipe@blueprint-equipe-local --scope user/);
  assert.match(chamadas, /codex mcp add blueprint-equipe -- .*blueprint\.exe --mcp --equipe/);
  assert.ok(!chamadas.includes(chave), "a chave não vai em argumento das CLIs");
  const conexao = JSON.parse(readFileSync(join(conv, "appdata", "Blueprint", "conexao-equipe.json"), "utf8"));
  assert.deepEqual(conexao, { url: t.local, chave });

  // 4. o MCP do plugin dela, pelo túnel, com o papel dela
  mcp = mcpStdio(exe, envConv, ["--mcp", "--equipe"]);
  await mcp.iniciar();
  const lido = await mcp.chamar("ler_blueprint", { workspace_id: WS, blueprint_id: BP });
  assert.equal(lido.erro, false, lido.texto);
  assert.match(lido.texto, new RegExp(BP));
  const fora = await mcp.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: "p1" });
  assert.equal(fora.erro, true, "outro workspace não existe para a convidada");
  const progresso = await mcp.chamar("registrar_progresso", { workspace_id: WS, blueprint_id: BP, texto: "Lendo o blueprint", agente: "Claude", estado: "lendo" });
  assert.equal(progresso.erro, false, progresso.texto);

  // 5. os hooks do plugin dela chegam ao app do dono
  const s = hook("sessao", { session_id: "sess-bia", cwd: projeto, hook_event_name: "SessionStart" });
  assert.equal(s.status, 0);
  assert.match(JSON.parse(s.stdout).hookSpecificOutput.additionalContext, new RegExp(`blueprint_id ${BP}`), "o SessionStart conta o blueprint da pasta");
  const ed = hook("edicao", { session_id: "sess-bia", cwd: projeto, hook_event_name: "PostToolUse", tool_name: "Edit", tool_input: { file_path: join(projeto, "src", "a.rs") } });
  assert.deepEqual([ed.status, ed.stdout.trim()], [0, ""], "a edição não imprime nada");
  const agentes = (await pedir(`${url}/api/mcp/agentes`)).json.agentes;
  const bia = agentes.find((a) => a.agente === "Claude · Bia");
  assert.ok(bia, "a figura do Claude da Bia está na folha do dono: " + JSON.stringify(agentes.map((a) => a.agente)));
  assert.deepEqual(bia.onde, ["src/a.rs"], "o hook de edição moveu a figura para o arquivo");
  assert.equal(bia.workspaceId, WS);
  const parada = hook("parada", { session_id: "sess-bia", cwd: projeto, hook_event_name: "Stop", stop_hook_active: false });
  assert.match(parada.stdout, /registrar_trabalho/, "o Stop lembra de registrar a edição fora do plano");
  // sem o app (endereço velho): silêncio e saída 0, rápido
  writeFileSync(join(conv, "appdata", "Blueprint", "conexao-equipe.json"), JSON.stringify({ url: "http://127.0.0.1:9", chave }));
  const t0 = Date.now(), mudo = hook("edicao", { session_id: "sess-bia", cwd: projeto, tool_input: { file_path: join(projeto, "x.rs") } });
  assert.deepEqual([mudo.status, mudo.stdout], [0, ""]);
  assert.ok(Date.now() - t0 < 5000, "falha rápido");

  // 6. desfazer
  const tirou = spawnSync(exe, ["conectar", "--remover"], { env: envConv, encoding: "utf8", windowsHide: true });
  assert.equal(tirou.status, 0, tirou.stdout);
  assert.ok(!existsSync(join(conv, "appdata", "Blueprint", "conexao-equipe.json")), "a chave sai do disco");
  assert.match(readFileSync(log, "utf8"), /claude plugin uninstall blueprint-equipe@blueprint-equipe-local --scope user/);
  console.log("convidado-claude e2e: ok (comando na tela, conectar com CLIs falsas, MCP pelo túnel com o papel, hooks sessao/edicao/parada no app do dono como Claude · Bia, falha silenciosa, remover)");
} finally {
  mcp?.fechar();
  try { servidor.kill(); } catch {}
  await dormir(800);
  try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {}
}
