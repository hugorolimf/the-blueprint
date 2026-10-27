// Confere no Edge: a proposta abre em leitura, instruções em detalhes e edição com prévia; a folha da ação também permite ler e editar.
// Uso: node scripts/instrucoes-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; interface lida da pasta)
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-notif-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "data.json"));
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "data.json"), PROTOTIPOS_DIR: join(tmp, "prototipos"), BLUEPRINT_ASSETS_DIR: raiz };
const servidor = spawn(exe, ["serve"], { env, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { try { edge.kill(); } catch {} try { servidor.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); }; // o Edge ainda segura o perfil por um instante: não derruba o teste
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.some((t) => !t.feito)));
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 900)); return r.result.result.value; };
  await cdp("Runtime.enable");
  const proposta = (corpo) => fetch(url + "/api/mcp/proposta", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ workspaceId: "sm", blueprintId: blueprint.id, ...corpo }) });
  const acao = blueprint.acoes[0];
  const instrucoes = ["Objetivo: ligar a pasta ao blueprint.", "Entrega: um plugin <b>do Claude</b>.", "Passo a passo: 1) ler; 2) criar.", "", "- primeiro item", "- segundo item", "", "```", "cargo test", "```", "Linha comprida de texto para encher a instrução. ".repeat(40)].join("\n");
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  assert.equal((await proposta({ acoes: [{ id: acao.id, observacao: instrucoes }] })).status, 200);
  await dormir(1500);
  // a notificação chega primeiro, sem ter clicado em Planejar; abri-la leva direto ao plano (a proposta)
  assert.equal(await js(`document.getElementById("notificacao")?.querySelector("h3")?.textContent`), "Nova proposta do agente", "a proposta avisa mesmo sem o Planejar aberto");
  assert.equal(await js(`document.querySelector("#notificacao .notif-ir").textContent`), "Abrir o plano");
  assert.equal(await js(`document.getElementById("agente-tela").hidden`), true, "o Planejar ainda não foi aberto");
  await dormir(1500);
  await js(`document.querySelector("#notificacao .notif-ir").click()`);
  await dormir(2200);
  assert.equal(await js(`!document.getElementById("agente-tela").hidden && !!document.querySelector("#agente-tela .pf")`), true, "abrir o aviso leva direto ao plano");
  assert.equal(await js(`document.getElementById("notificacao").hidden`), true, "e o aviso se fecha");
  // Planejar abre direto na proposta, sem a lista de blueprints, "pronto" nem "Tela cheia"
  assert.equal(await js(`[...document.querySelectorAll("#agente-tela .ed-lateral, #ag-estado, #ag-tela, #cu-abrir")].every((e) => getComputedStyle(e).display === "none")`), true, "Planejar só mostra a proposta: o resto do topo e a lateral somem");
  // a revisão começa em leitura; as instruções ficam num bloco expansível
  assert.equal(await js(`document.querySelector(".pf .pv-obs").open`), false, "as instruções começam recolhidas");
  await js(`document.querySelector(".pf .pv-obs > summary").click()`);
  assert.equal(await js(`!!document.querySelector(".pf .obs-ver strong.md-rot")`), true, "o rótulo vira negrito");
  assert.equal(await js(`document.querySelectorAll(".pf .obs-ver ul li").length`), 2, "a lista vira lista");
  assert.equal(await js(`document.querySelector(".pf .obs-ver pre code").textContent`), "cargo test", "o bloco de código");
  assert.equal(await js(`document.querySelector(".pf .obs-ver").innerHTML.includes("<b>")`), false, "HTML do agente não vira tag");
  assert.equal(await js(`(() => { const v = document.querySelector(".pf .obs-ver"); return v.textContent.includes("Linha comprida") && v.getBoundingClientRect().height > 0; })()`), true, "o texto completo aparece ao expandir");
  assert.equal(await js(`!!document.querySelector(".pf .obs-edicao")`), false, "o campo só aparece ao editar a ação");
  await js(`document.querySelector('.pf [data-pact="editar-acao"]').click()`);
  assert.equal(await js(`document.querySelector(".pf .obs-edicao").hidden`), true, "o campo fica escondido até editar as instruções");
  // Editar: o campo cresce até o texto; Visualizar: volta ao texto lido com o que foi digitado
  await js(`document.querySelector(".pf [data-obs=editar]").click()`);
  await dormir(200);
  assert.equal(await js(`(() => { const t = document.querySelector(".pf .obs-edicao textarea"); return !t.closest(".obs-edicao").hidden && t.clientHeight >= Math.min(t.scrollHeight, innerHeight * 0.7) - 2; })()`), true, "o campo já tem a altura do texto");
  await js(`(() => { const t = document.querySelector(".pf .obs-edicao textarea"); t.value = "## Novo título\\nTexto *editado*"; t.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await dormir(150);
  assert.equal(await js(`document.querySelector(".pf .obs-previa .md-h2")?.textContent`), "Novo título", "o Editar mostra a prévia em markdown ao vivo");
  assert.equal(await js(`(() => { const e = document.querySelector(".pf .obs-edicao"), r = [...e.children].map((c) => c.getBoundingClientRect()); return r[1].left > r[0].left; })()`), true, "campo e prévia lado a lado");
  await js(`document.querySelector(".pf [data-obs=ver]").click()`);
  assert.equal(await js(`document.querySelector(".pf .obs-ver .md-h2").textContent`), "Novo título", "voltar a Visualizar mostra o que foi digitado");
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "instrucoes-planejar.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
  await js(`document.getElementById("ag-fechar").click()`);
  await dormir(900);
  // a folha pequena da instrução (clicar na ação) também abre lida e inteira
  await js(`(() => { const i = document.createElement("i"); i.dataset.obs = ${JSON.stringify(blueprint.id + "|" + acao.id)}; abreNotaObs(i); })()`);
  await dormir(1000);
  assert.equal(await js(`(() => { const n = document.getElementById("nota-folha"); return !n.hidden && n.dataset.modo === "ver" && !!n.querySelector("[data-nver].md") && n.querySelector(".obs-edicao").hidden; })()`), true, "a folha da instrução abre lida (markdown), com o campo escondido");
  assert.equal(await js(`getComputedStyle(document.querySelector('#nota-folha [data-nmodo=editar]')).display`), "none", "Salvar só aparece editando");
  await js(`document.querySelector("#nota-folha [data-nmodo=ver]").click()`);
  await dormir(200);
  assert.equal(await js(`(() => { const n = document.getElementById("nota-folha"); return n.dataset.modo === "editar" && !n.querySelector(".obs-edicao").hidden && !!n.querySelector(".obs-previa strong") && getComputedStyle(n.querySelector("[data-nmodo=editar]")).display !== "none"; })()`), true, "Editar mostra o campo e o Salvar");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("instruções da ação: ok (revisão, markdown, Editar/Visualizar, folha da instrução). Captura em target/instrucoes-planejar.png");
} finally { limpar(); }
