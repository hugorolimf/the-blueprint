// Confere o painel de agentes no navegador de verdade (Edge sem janela): liga o servidor Rust com uma cópia dos dados, simula três agentes pelo
// /api/mcp, abre o painel e confere cartões, figuras, estados, link para o plano e expiração. Grava a captura em target/painel-agentes.png.
// Uso: node scripts/painel-agentes-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const [porta, depuracao] = [await livre(), await livre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-painel-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos") };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} try { rmSync(tmp, { recursive: true, force: true }); } catch {} };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.length));
  const acao = blueprint.acoes.find((f) => f.planos?.length), plano = acao.planos[0];
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = []; const resultados = {};
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  assert.equal(await js(`document.getElementById("b-agentes").hidden`), true, "sem agentes, o botão não aparece");

  const avisaTipo = (tipo, corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, workspaceId: "sm", ...corpo }) });
  const avisa = (corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo: "progresso", workspaceId: "sm", blueprintId: blueprint.id, ...corpo }) });
  await avisa({ texto: "Lendo os módulos do kit", agente: "explorador-kit", papel: "explorador", estado: "lendo", onde: ["kit/lathe/"], percentual: 20 });
  await avisa({ texto: "Escrevendo o painel", agente: "construtor-painel", papel: "construtor", estado: "escrevendo", onde: ["web/js/agentes.js", "web/css/agentes.css"], percentual: 55, acaoId: acao.id, planoId: plano.id });
  await avisa({ texto: "Rodando cargo test", agente: "revisor-testes", papel: "revisor", estado: "testando", onde: ["src/"], percentual: 80 });
  await dormir(1200); // o aviso chega por SSE; ninguém consulta
  assert.equal(await js(`document.getElementById("b-agentes").hidden`), false, "com agentes, o botão aparece");
  assert.equal(await js(`document.getElementById("b-agentes-n").textContent`), "3");
  await js(`document.getElementById("b-agentes").click()`);
  await dormir(1500);
  assert.equal(await js(`document.getElementById("agentes-painel").open`), true);
  const cartoes = await js(`[...document.querySelectorAll("#ag-cartoes .ag-card")].map((c) => ({ nome: c.dataset.ag, estado: c.querySelector(".ag-estado").textContent.trim(), papel: c.querySelector(".badge").textContent, figura: !!c.querySelector(".ag-fig svg") && c.querySelector(".ag-fig svg").innerHTML.length, plano: c.querySelector(".ag-plano")?.textContent.trim() || "", onde: c.querySelector(".ag-onde")?.textContent || "", ha: c.querySelector(".ag-ha").textContent }))`);
  console.log(JSON.stringify(cartoes, null, 1));
  assert.deepEqual(cartoes.map((c) => c.nome), ["revisor-testes", "construtor-painel", "explorador-kit"], "do mais recente ao mais antigo");
  assert.deepEqual(cartoes.map((c) => c.papel), ["Revisor", "Construtor", "Explorador"]);
  assert(cartoes[0].estado.startsWith("Testando · 80%") && cartoes[1].estado.startsWith("Escrevendo · 55%") && cartoes[2].estado.startsWith("Lendo · 20%"));
  assert(cartoes.every((c) => c.figura > 1000), "cada agente tem a sua figura do anatomy desenhada");
  assert(cartoes[1].plano.includes(plano.texto), "o cartão mostra o plano em que o agente está");
  const figuras = await js(`new Set([...document.querySelectorAll("#ag-cartoes .ag-fig svg")].map((s) => s.getAttribute("aria-label"))).size`);
  assert.equal(figuras, 3, "papéis e estados diferentes dão figuras diferentes");
  mkdirSync(join(raiz, "target"), { recursive: true });
  const captura = await cdp("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(raiz, "target", "painel-agentes.png"), Buffer.from(captura.result.data, "base64"));

  // só o cartão que mudou é refeito: o do construtor muda, os outros mantêm o mesmo elemento
  await js(`window.__antes = [...document.querySelectorAll("#ag-cartoes .ag-card")]`);
  await avisa({ texto: "Escrevendo o painel (quase)", agente: "construtor-painel", percentual: 90, acaoId: acao.id, planoId: plano.id });
  await dormir(900);
  const refeito = await js(`window.__antes.map((c) => document.querySelector('#ag-cartoes [data-ag="' + c.dataset.ag + '"]') === c)`);
  assert.deepEqual(refeito, [true, true, true], "os mesmos elementos continuam no painel (reordenados, não refeitos do zero)");
  assert((await js(`document.querySelector('#ag-cartoes [data-ag="construtor-painel"] .ag-estado').textContent`)).includes("90%"), "o cartão que mudou mostra o novo percentual");

  // expiração: um agente sem sinal por mais de 10 minutos sai do painel e do contador
  await js(`agentesLista.find((a) => a.agente === "explorador-kit").em = new Date(Date.now() - 11 * 60000).toISOString(); agentesAtualiza()`);
  assert.equal(await js(`document.querySelectorAll("#ag-cartoes .ag-card").length`), 2, "o expirado sai do painel");
  assert.equal(await js(`document.getElementById("b-agentes-n").textContent`), "2");

  // o cartão leva até o plano
  await js(`document.querySelector('#ag-cartoes [data-ag="construtor-painel"] [data-ag-ir]').click()`);
  await dormir(600);
  assert.equal(await js(`document.getElementById("agentes-painel").open`), false, "o painel fecha ao ir até o plano");
  assert(await js(`agenteAgora?.planoId === ${JSON.stringify(plano.id)}`), "o robô passa a apontar o plano do agente");

  // um agente marca um checkpoint pelo MCP, com evidência: a página aplica, guarda a evidência no checkpoint e grava no servidor
  const cp = dados.blueprints.flatMap((p) => (p.acoes || []).flatMap((f) => (f.checkpoints || []).filter((m) => !m.feito).map((m) => ({ p: p.id, f: f.id, m: m.id })))).at(0);
  assert(cp, "o data.json de teste precisa ter um checkpoint pendente");
  const resposta = await avisaTipo("checkpoint-feito", { texto: "evidência: cargo test passou, 80 testes", blueprintId: cp.p, acaoId: cp.f, checkpointId: cp.m });
  assert.equal(resposta.status, 200);
  await dormir(2500);
  const marcado = await js(`(() => { const m = D.blueprints.find((p) => p.id === ${JSON.stringify(cp.p)}).acoes.find((f) => f.id === ${JSON.stringify(cp.f)}).checkpoints.find((x) => x.id === ${JSON.stringify(cp.m)}); return { feito: m.feito, evidencia: m.evidencia }; })()`);
  assert.deepEqual(marcado, { feito: true, evidencia: "evidência: cargo test passou, 80 testes" }, "a página marca o checkpoint e guarda a evidência");
  const salvo = (await (await fetch(url + "/api/dados")).json()).blueprints.find((p) => p.id === cp.p).acoes.find((f) => f.id === cp.f).checkpoints.find((x) => x.id === cp.m);
  assert.equal(salvo.feito, true, "e a gravação chegou ao servidor");
  resultados.checkpoint_pelo_mcp = { feito: true, evidencia_guardada: true, gravado_no_servidor: true };
  console.log("checkpoint marcado pelo agente: a página aplicou, guardou a evidência e gravou no banco");

  // 12 agentes de uma vez (3 papéis × 4 estados): a figura é desenhada uma vez por pose, e o painel tem de continuar fluido
  const papeis = ["explorador", "construtor", "revisor"], estados = ["lendo", "escrevendo", "testando", "aguardando"];
  for (let i = 0; i < 12; i++) await avisa({ texto: `agente ${i} trabalhando`, agente: `agente-${String(i).padStart(2, "0")}`, papel: papeis[i % 3], estado: estados[Math.floor(i / 3)], onde: [`pasta-${i}/arquivo.rs`], percentual: i * 8 });
  await dormir(1500);
  const t0 = Date.now();
  await js(`abreAgentes()`);
  await dormir(1200);
  const quadros = await js(`new Promise((ok) => { const t = []; let ultimo = performance.now(); const passo = (agora) => { t.push(agora - ultimo); ultimo = agora; if (t.length < 60) requestAnimationFrame(passo); else ok(t); }; requestAnimationFrame(passo); })`);
  const medio = quadros.reduce((x, y) => x + y, 0) / quadros.length, pior = Math.max(...quadros);
  const doze = await js(`({ cartoes: document.querySelectorAll("#ag-cartoes .ag-card").length, figuras: document.querySelectorAll("#ag-cartoes .ag-fig svg").length, posesDistintas: figurasAg.size, nos: document.getElementsByTagName("*").length, kb: Math.round([...document.querySelectorAll("#ag-cartoes .ag-fig svg")].reduce((t, e) => t + e.outerHTML.length, 0) / 1024) })`);
  console.log(JSON.stringify({ ...doze, abrirEsperaMs: Date.now() - t0, quadro_medio_ms: Math.round(medio * 10) / 10, pior_quadro_ms: Math.round(pior * 10) / 10 }));
  assert(doze.cartoes >= 12 && doze.figuras >= 12, "os 12 agentes aparecem, cada um com figura");
  assert(medio < 34, `o painel com 12 figuras precisa ficar fluido (quadro médio ${medio.toFixed(1)} ms)`);
  const captura12 = await cdp("Page.captureScreenshot", { format: "png" });
  writeFileSync(join(raiz, "target", "painel-12-agentes.png"), Buffer.from(captura12.result.data, "base64"));
  resultados.doze_agentes = { ...doze, quadro_medio_ms: Math.round(medio * 10) / 10, pior_quadro_ms: Math.round(pior * 10) / 10 };
  writeFileSync(join(raiz, "referencias", "medicao-painel-agentes.json"), JSON.stringify({ data: new Date().toISOString(), comando: "node scripts/painel-agentes-e2e.mjs", navegador: "Microsoft Edge sem janela, 1440×1000", ...resultados }, null, 2) + "\n");
  assert.deepEqual(erros, [], "sem erros de JavaScript na página");
  console.log("painel de agentes: ok (3 cartões, figuras, estados, plano, atualização só do que mudou, expiração, link). Captura em target/painel-agentes.png");
} finally { limpar(); }
