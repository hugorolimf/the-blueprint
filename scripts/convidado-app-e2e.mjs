// O convidado trabalhando na máquina dele, no projeto compartilhado (docs/app-no-projeto-compartilhado.md), sem janela nem tokens: o dono compartilha o workspace
// (túnel falso, scripts/tunel-falso.mjs) e liga um repositório ao blueprint; a convidada entra, conecta o Claude dela (`blueprint conectar`, CLIs falsas) e vê o repositório
// pelo MCP do plugin; numa pasta que parece o clone do repositório, `vincular_pasta` do plugin dela (`--mcp --equipe`) grava a ligação em CLAUDE.local.md e a esconde do Git
// (nada versionado muda), os hooks `--equipe` acham o blueprint por essa linha e o Claude dela aparece na folha do dono, e `blueprint claude --equipe` abre o `claude`
// (falso) só com o nome da sessão e o prompt, sem plugin local nem canal. O que o app instalado pergunta e abre pela janela da equipe é coberto por cargo test (projeto_local).
//   node scripts/convidado-app-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; ~10 s; Windows)
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
const REPO = "https://github.com/dono/repo";
const tmp = mkdtempSync(join(tmpdir(), "convidado-app-e2e-"));
const porta = await portaLivre(), url = `http://127.0.0.1:${porta}`, info = join(tmp, "tunel.json");
const dados = JSON.parse(readFileSync(join(raiz, "data.exemplo.json"), "utf8"));
dados.workspaces.find((w) => w.id === WS).blueprints.find((b) => b.id === BP).repositorio = { url: REPO }; // o dono ligou o repositório
writeFileSync(join(tmp, "data.json"), JSON.stringify(dados));
writeFileSync(join(tmp, "tunel-falso.cmd"), `@node "${join(raiz, "scripts", "tunel-falso.mjs")}" %*\r\n`);
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "b.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_SEM_ATUALIZACAO: "1", BLUEPRINT_TUNEL_COMANDO: join(tmp, "tunel-falso.cmd"), TUNEL_FALSO_INFO: info }, stdio: "ignore", windowsHide: true });

// o computador da convidada: pastas temporárias e CLIs falsas na frente do PATH; o clone do repositório é uma pasta com .git
const conv = join(tmp, "convidada"), bin = join(conv, "bin"), log = join(bin, "log.txt"), clone = join(conv, "Blueprint", "Projetos", "repo");
for (const p of [bin, join(conv, "appdata"), join(conv, "claude"), join(conv, "codex"), join(clone, ".git", "info"), join(clone, "src", "fundo")]) mkdirSync(p, { recursive: true });
for (const cli of ["claude", "codex"]) writeFileSync(join(bin, `${cli}.cmd`), `@echo off\r\necho ${cli} %*>> "${log}"\r\n`);
writeFileSync(join(clone, "CLAUDE.md"), "# Repo do projeto\n\nNotas do time.\n");
const envConv = { ...process.env, APPDATA: join(conv, "appdata"), CLAUDE_CONFIG_DIR: join(conv, "claude"), CODEX_HOME: join(conv, "codex"), PATH: `${bin};${process.env.PATH}`, BLUEPRINT_CONECTAR_HTTP_LOCAL: "1", BLUEPRINT_IDIOMA: "pt-BR" };
delete envConv.BLUEPRINT_URL; delete envConv.BLUEPRINT_CHAVE;
const hook = (evento, corpo, cwd) => spawnSync(exe, ["hook", evento, "--equipe"], { cwd, env: envConv, input: JSON.stringify(corpo), encoding: "utf8", timeout: 10000, windowsHide: true });

const pedir = (endereco, { metodo = "GET", cabecalhos = {}, corpo } = {}) => new Promise((ok, falha) => {
  const u = new URL(endereco), texto = corpo === undefined ? undefined : JSON.stringify(corpo);
  const req = request({ hostname: u.hostname, port: u.port, path: u.pathname + u.search, method: metodo, headers: { ...(texto ? { "content-type": "application/json" } : {}), ...cabecalhos } }, (res) => {
    let t = ""; res.on("data", (d) => (t += d)); res.on("end", () => { let json = null; try { json = JSON.parse(t); } catch {} ok({ status: res.statusCode, texto: t, json, cabecalhos: res.headers }); });
  });
  req.on("error", falha);
  if (texto) req.write(texto);
  req.end();
});

let mcp;
try {
  for (let i = 0; ; i++) { try { await fetch(url + "/api/dados"); break; } catch { if (i > 100) throw new Error("o servidor não respondeu"); await dormir(50); } }
  const local = { origin: url };
  // 1. o dono compartilha o workspace e convida uma colaboradora; ela entra, gera a chave e conecta o Claude dela
  assert.equal((await pedir(`${url}/api/compartilhar/ligar`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS } })).status, 202);
  let estado;
  for (let i = 0; i < 200 && estado?.estado !== "ligado"; i++) { estado = (await pedir(`${url}/api/compartilhar`)).json; if (estado?.estado !== "ligado") await dormir(50); }
  assert.equal(estado.estado, "ligado", JSON.stringify(estado));
  const t = JSON.parse(readFileSync(info, "utf8"));
  const convite = await pedir(`${url}/api/convites`, { metodo: "POST", cabecalhos: local, corpo: { workspaceId: WS, papel: "colaborador" } });
  const entrou = await pedir(`${t.local}/api/convite/entrar`, { metodo: "POST", cabecalhos: { origin: t.local }, corpo: { segredo: convite.json.link.split("#")[1], nome: "Bia" } });
  assert.equal(entrou.status, 200, entrou.texto);
  const cookie = String(entrou.cabecalhos["set-cookie"]).split(";")[0];
  const chave = (await pedir(`${t.local}/api/chave`, { metodo: "POST", cabecalhos: { origin: t.local, cookie }, corpo: {} })).json.chave;
  const ligou = spawnSync(exe, ["conectar", t.local, chave], { env: envConv, encoding: "utf8", windowsHide: true, timeout: 30000 });
  assert.equal(ligou.status, 0, ligou.stdout + ligou.stderr);

  // 2. a página dela (pelo túnel) já traz o repositório que o dono ligou, e as rotas locais não existem para ela
  const pagina = (await pedir(`${t.local}/api/dados`, { cabecalhos: { cookie } })).json;
  const bp = pagina.workspaces ? pagina.workspaces.flatMap((w) => w.blueprints).find((b) => b.id === BP) : pagina.blueprints.find((b) => b.id === BP);
  assert.deepEqual(bp.repositorio, { url: REPO }, "o convidado recebe o repositório junto com o blueprint");
  for (const rota of ["/api/repositorio/verificar", "/api/repositorio/configurar"]) {
    const r = await pedir(`${t.local}${rota}`, { metodo: "POST", cabecalhos: { origin: t.local, cookie }, corpo: { workspaceId: WS, blueprintId: BP, pasta: clone } });
    assert.equal(r.json?.codigo, "so-local", `${rota} não existe para quem entra pelo túnel`);
  }

  // 3. o MCP do plugin dela devolve o repositório; vincular_pasta liga a pasta do clone sem tocar no que o Git versiona
  mcp = mcpStdio(exe, envConv, ["--mcp", "--equipe"], clone);
  await mcp.iniciar();
  const lido = await mcp.chamar("ler_blueprint", { workspace_id: WS, blueprint_id: BP });
  assert.equal(lido.erro, false, lido.texto);
  assert.ok(JSON.parse(lido.texto).blueprint.repositorio.url === REPO, "ler_blueprint devolve o repositório: " + lido.texto.slice(0, 300));
  const ligada = await mcp.chamar("vincular_pasta", { workspace_id: WS, blueprint_id: BP });
  assert.equal(ligada.erro, false, ligada.texto);
  assert.match(ligada.texto, /hidden from Git/);
  assert.match(readFileSync(join(clone, "CLAUDE.local.md"), "utf8"), new RegExp(`workspace_id \`${WS}\`, blueprint_id \`${BP}\``));
  assert.equal(readFileSync(join(clone, ".git", "info", "exclude"), "utf8"), "CLAUDE.local.md\n", "o arquivo fica fora do Git");
  assert.equal(readFileSync(join(clone, "CLAUDE.md"), "utf8"), "# Repo do projeto\n\nNotas do time.\n", "o CLAUDE.md do repositório não muda");
  const errada = await mcp.chamar("vincular_pasta", { workspace_id: WS, blueprint_id: "bp`x" });
  assert.equal(errada.erro, true, "id que não é id é recusado");

  // 4. os hooks do plugin dela acham o blueprint pela linha (também numa subpasta) e o Claude dela aparece na folha do dono
  const sub = join(clone, "src", "fundo");
  const s = hook("sessao", { session_id: "sess-bia", cwd: sub, hook_event_name: "SessionStart" }, sub);
  assert.match(JSON.parse(s.stdout).hookSpecificOutput.additionalContext, new RegExp(`blueprint_id ${BP}`), "o SessionStart conta o blueprint da pasta ligada");
  hook("edicao", { session_id: "sess-bia", cwd: sub, hook_event_name: "PostToolUse", tool_name: "Edit", tool_input: { file_path: join(sub, "a.rs") } }, sub);
  const agentes = (await pedir(`${url}/api/mcp/agentes`)).json.agentes;
  assert.ok(agentes.find((a) => a.agente === "Claude · Bia" && a.workspaceId === WS), "o Claude da Bia está na folha do dono: " + JSON.stringify(agentes.map((a) => a.agente)));
  const fora = join(conv, "outra");
  mkdirSync(fora);
  assert.equal(hook("sessao", { session_id: "s2", cwd: fora }, fora).stdout.trim(), "", "uma pasta sem a linha não manda nada");

  // 5. `blueprint claude --equipe` abre o claude do convidado só com o nome da sessão e o prompt
  const abriu = spawnSync(exe, ["claude", "--equipe", "Execute a ação 1.1"], { cwd: clone, env: envConv, encoding: "utf8", windowsHide: true, timeout: 20000 });
  assert.equal(abriu.status, 0, abriu.stdout + abriu.stderr);
  const chamadas = readFileSync(log, "utf8").split("\n").filter((l) => l.startsWith("claude --name"));
  assert.equal(chamadas.length, 1, readFileSync(log, "utf8"));
  assert.equal(chamadas[0].trim(), 'claude --name "The Blueprint" "Execute a ação 1.1"', "o nome e o prompt chegam cada um como um argumento");
  assert.ok(!/plugin-dir|channel|equipe/.test(chamadas[0]), "sem plugin local, sem canal e sem a flag do app: " + chamadas[0]);
  console.log("convidado-app e2e: ok (repositório chega ao convidado, rotas locais fechadas no túnel, vincular_pasta em CLAUDE.local.md fora do Git, hooks acham a pasta, Claude · Bia na folha, blueprint claude --equipe)");
} finally {
  mcp?.fechar();
  try { servidor.kill(); } catch {}
  await dormir(800);
  try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {}
}
