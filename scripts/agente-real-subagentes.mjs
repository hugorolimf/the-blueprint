// Teste de verdade (GASTA TOKENS, só com o Claude Code instalado e logado): um Claude real lança subagentes e os hooks do plugin os mostram sozinhos na folha.
// Sobe um servidor próprio (banco de exemplo, porta livre, nada do app aberto), liga uma pasta de projeto temporária a um blueprint, carrega um plugin TEMPORÁRIO só com os
// hooks (PostToolUse de Edit|Write, SubagentStart e SubagentStop, como o `blueprint registrar` grava) e pede ao Claude um subagente Explore e um general-purpose. Confere em
// /api/mcp/agentes: cada subagente vira um agente próprio ("tipo + 4 do id", papel pelo tipo), aparece lendo ao começar e termina concluído com a mensagem final, e o Claude
// principal não é confundido com eles. Não mexe na configuração do usuário. Uso: node scripts/agente-real-subagentes.mjs [blueprint.exe]
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const versao = spawnSync("claude", ["--version"], { encoding: "utf8" });
assert.equal(versao.status, 0, "o teste precisa do Claude Code (claude) no PATH");
const porta = await livre(), url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-subagentes-"));
const projeto = join(tmp, "projeto"), plugin = join(tmp, "plugin");
mkdirSync(projeto); mkdirSync(join(plugin, ".claude-plugin"), { recursive: true }); mkdirSync(join(plugin, "hooks"), { recursive: true });
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const limpar = () => { try { servidor.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true }); } catch {} }, 800); };
process.on("exit", limpar);
const post = (rota, corpo) => fetch(`${url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(corpo) });
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints[0];
  assert.equal((await post("pasta/vincular", { cwd: projeto, workspaceId: "sm", blueprintId: blueprint.id })).status, 200, "liga a pasta temporária ao blueprint");
  const http = (nome) => [{ hooks: [{ type: "http", url: `${url}/api/mcp/hook/${nome}`, timeout: 5 }] }];
  writeFileSync(join(plugin, ".claude-plugin", "plugin.json"), JSON.stringify({ name: "blueprint-teste-subagentes", version: "0.0.1", description: "Só os hooks do Blueprint, para o teste dos subagentes." }));
  writeFileSync(join(plugin, "hooks", "hooks.json"), JSON.stringify({ hooks: { PostToolUse: [{ matcher: "Edit|Write", hooks: http("edicao")[0].hooks }], SubagentStart: http("subagente"), SubagentStop: http("subagente") } }));

  const prompt = "Use a ferramenta Agent exatamente duas vezes, uma de cada vez: primeiro com subagent_type 'Explore' para listar os arquivos da pasta atual (só os nomes); depois com subagent_type 'general-purpose' para criar o arquivo prova.txt na pasta atual com o texto ok. Por fim responda com uma frase curta. Não use nenhuma outra ferramenta.";
  const visto = new Map(); // nome -> { estados, papel, mensagem }
  let rodando = true;
  const vigia = (async () => {
    while (rodando) {
      try {
        for (const a of (await (await fetch(url + "/api/mcp/agentes")).json()).agentes) {
          const v = visto.get(a.agente) || { estados: [], papel: a.papel, mensagem: "" };
          if (v.estados.at(-1) !== a.estado) v.estados.push(a.estado);
          v.mensagem = a.mensagem || v.mensagem;
          visto.set(a.agente, v);
        }
      } catch {}
      await dormir(300);
    }
  })();
  const filho = spawn("claude", ["-p", prompt, "--plugin-dir", plugin, "--permission-mode", "acceptEdits", "--model", "haiku", "--max-turns", "10", "--output-format", "text"], { cwd: projeto, env: process.env, stdio: ["ignore", "pipe", "pipe"] });
  let saida = "", erro = "";
  filho.stdout.on("data", (d) => { saida += d; });
  filho.stderr.on("data", (d) => { erro += d; });
  const codigo = await Promise.race([new Promise((r) => filho.on("close", r)), dormir(5 * 60 * 1000).then(() => { filho.kill(); return "tempo"; })]);
  await dormir(1000);
  rodando = false;
  await vigia;
  console.log(`Claude Code ${versao.stdout.trim()} · saída ${codigo} · resposta: ${saida.trim().slice(0, 200)}`);
  if (erro.trim()) console.log("stderr:", erro.trim().slice(0, 300));
  console.log("agentes vistos:", JSON.stringify([...visto].map(([nome, v]) => ({ nome, ...v, mensagem: v.mensagem.slice(0, 60) }))));
  const subs = [...visto].filter(([nome]) => nome !== "Claude");
  assert(subs.length >= 2, `os subagentes aparecem sozinhos (vistos: ${[...visto.keys()]})`);
  assert(subs.every(([nome]) => /^.+ [0-9A-Za-z]{4}$/.test(nome)), "nome = tipo + 4 caracteres do id");
  assert(subs.some(([nome, v]) => /^Explore /.test(nome) && v.papel === "explorador"), "o Explore vira explorador");
  assert(subs.every(([, v]) => v.estados.includes("lendo")), "cada subagente aparece lendo ao começar");
  assert(subs.every(([, v]) => v.estados.at(-1) === "concluido" && v.mensagem.length > 0), "e termina concluído com a mensagem final");
  console.log("subagentes reais: ok (aparecem sozinhos, com papel pelo tipo, e concluem com a última mensagem)");
} finally { limpar(); }
