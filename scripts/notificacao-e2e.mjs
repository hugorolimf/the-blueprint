// Confere a notificação do agente no navegador de verdade (Edge sem janela, interface lida da pasta com BLUEPRINT_ASSETS_DIR): a pílula entra, vira cartão com a
// mensagem inteira do agente (texto, nunca HTML), o botão leva ao blueprint, Esc fecha, vários avisos viram fila com "+N", e com movimento reduzido só há fade.
// Uso: node scripts/notificacao-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; leva uns 15 s)
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
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300)); return r.result.result.value; };
  await cdp("Runtime.enable");
  const evento = (tipo, corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, workspaceId: "sm", blueprintId: blueprint.id, ...corpo }) });
  const el = (expr) => js(`(() => { const n = document.getElementById("notificacao"); return n ? (${expr}) : null; })()`);

  await cdp("Page.navigate", { url: `${url}/#/c/sm` });
  await dormir(2500);
  assert.equal(await js(`document.getElementById("notificacao")`), null, "sem evento não existe nem o elemento (zero trabalho em repouso)");

  // repouso: sem aviso, a página não gasta CPU (tempo de tarefa do renderizador ao longo de 3 s)
  await cdp("Performance.enable");
  const tarefa = async () => (await cdp("Performance.getMetrics")).result.metrics.find((m) => m.name === "TaskDuration").value;
  const t0 = await tarefa();
  await dormir(3000);
  const cpu = ((await tarefa()) - t0) / 3;
  assert.ok(cpu < 0.02, `em repouso a página usou ${(cpu * 100).toFixed(1)}% de CPU (limite 2%)`);

  // progresso não avisa (fica só no robô)
  await evento("progresso", { texto: "Editando um arquivo" });
  await dormir(1000);
  assert.equal(await js(`document.getElementById("notificacao")`), null, "progresso não vira notificação");

  // bloqueio: a folha se abre no meio da tela com a mensagem inteira do agente, como texto, e a marca em construção monta atrás
  const malicioso = '<img src=x onerror="window.__xss=1"> Travei: falta acesso ao banco. ' + "linha longa ".repeat(60);
  await evento("bloqueio", { texto: malicioso });
  await dormir(250);
  assert.equal(await el(`!n.hidden && n.getAnimations().length > 0`), true, "a folha está abrindo (animação em curso)");
  assert.equal(await js(`document.getElementById("folha-fundo").classList.contains("on")`), true, "a cortina escurece o fundo");
  await dormir(1800);
  assert.equal(await el(`n.querySelector("h3").textContent`), "O agente travou");
  assert.equal(await el(`n.querySelector(".fa-msg").textContent`), malicioso.trim(), "a mensagem inteira, como texto"); // o servidor tira os espaços das pontas
  assert.equal(await el(`n.querySelector(".fa-msg *") === null`), true, "nada de HTML dentro da mensagem");
  assert.equal(await js(`window.__xss`), undefined, "o texto do agente não executou");
  assert.equal(await el(`getComputedStyle(n.querySelector(".fa-msg")).overflowY`), "auto", "mensagem longa rola");
  assert.equal(await el(`n.classList.contains("erro")`), true, "bloqueio pinta de erro");
  assert.equal(await el(`!!n.querySelector(".fa-marca svg.emblema")`), true, "a marca em construção está atrás");
  assert.equal(await el(`!n.querySelector(".fa-carimbo").hidden && n.querySelector(".fa-carimbo dt").textContent`), "Blueprint", "o carimbo traz de onde veio o aviso");
  const centro = await el(`(() => { const r = n.getBoundingClientRect(); return [Math.abs(r.left + r.width / 2 - innerWidth / 2), Math.abs(r.top + r.height / 2 - innerHeight / 2)]; })()`);
  assert.ok(centro[0] < 2 && centro[1] < 2, "no meio da tela: " + centro);
  mkdirSync(join(raiz, "target"), { recursive: true });
  const foto = async (nome) => writeFileSync(join(raiz, "target", nome), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
  await foto("notificacao.png");
  // o aviso que pede atenção não some sozinho; Esc fecha (e a cortina sai)
  await cdp("Input.dispatchKeyEvent", { type: "keyDown", key: "Escape", code: "Escape", windowsVirtualKeyCode: 27 });
  await dormir(700);
  assert.equal(await el(`n.hidden`), true, "Esc fecha o aviso");
  assert.equal(await js(`document.getElementById("folha-fundo").classList.contains("on")`), false, "e a cortina sai");

  // fila: vários avisos seguidos viram "Próximo (N)" e entram um depois do outro
  for (const t of ["um", "dois", "três", "quatro"]) await evento("bloqueio", { texto: "bloqueio " + t });
  await dormir(1500);
  assert.match(await el(`n.querySelector(".notif-prox").textContent`), /Próximo \(\d+\)/, "a folha avisa que há mais na fila");

  // blueprint de origem: o carimbo traz o blueprint e o botão leva até ele
  await js(`notifFila.length = 0; notifFecha()`);
  await dormir(700);
  await js(`notificar({ id: 9100, em: new Date().toISOString(), tipo: "proposta", workspaceId: "sm", blueprintId: ${JSON.stringify(blueprint.id)}, proposta: { mensagem: "Montei 3 ações para revisar." } })`);
  await dormir(1800);
  assert.equal(await el(`n.querySelector(".fa-msg").textContent`), "Montei 3 ações para revisar.", "a proposta mostra a fala do agente");
  assert.match(await el(`n.querySelector(".fa-carimbo").textContent`), new RegExp(blueprint.nome.split(" ")[0]), "o carimbo traz o blueprint");
  assert.equal(await el(`n.querySelector(".notif-ir").textContent`), "Abrir o plano", "a proposta oferece abrir o plano");
  await el(`n.querySelector(".notif-ir").click()`);
  await dormir(2200);
  assert.equal(await js(`!document.getElementById("agente-tela").hidden`), true, "o botão da proposta abre direto o Planejar");
  await js(`document.getElementById("ag-fechar").click()`);
  await dormir(900);
  // um aviso que não é proposta leva ao blueprint
  await js(`notificar({ id: 9101, em: new Date().toISOString(), tipo: "bloqueio", texto: "Travei", workspaceId: "sm", blueprintId: ${JSON.stringify(blueprint.id)} })`);
  await dormir(1800);
  assert.equal(await el(`n.querySelector(".notif-ir").textContent`), "Ir até o blueprint");
  await el(`n.querySelector(".notif-ir").click()`);
  await dormir(1000);
  assert.match(await js(`location.hash`), /^#\/p\//, "o botão do bloqueio abre o blueprint");

  // Planejar abre do mesmo jeito: folha no meio com cortina e a marca em construção atrás
  await js(`document.getElementById("b-ia").click()`);
  await dormir(300);
  assert.equal(await js(`document.getElementById("agente-tela").getAnimations().length > 0`), true, "Planejar abre com a animação da folha");
  await dormir(1800);
  assert.equal(await js(`(() => { const t = document.getElementById("agente-tela"); return !t.hidden && t.classList.contains("fa-planejar") && !!t.querySelector(".fa-marca svg.emblema") && document.getElementById("folha-fundo").classList.contains("on"); })()`), true, "Planejar é uma folha com a marca atrás e a cortina");
  await foto("planejar-folha.png");
  await js(`document.getElementById("ag-fechar").click()`);
  await dormir(800);
  assert.equal(await js(`document.getElementById("agente-tela").hidden && !document.getElementById("folha-fundo").classList.contains("on")`), true, "fechar Planejar recolhe a folha e a cortina");

  // movimento reduzido: a folha aparece pronta, sem nenhuma animação
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await js(`notificar({ id: 9001, em: new Date().toISOString(), tipo: "bloqueio", texto: "com movimento reduzido", workspaceId: "sm", blueprintId: ${JSON.stringify(blueprint.id)} })`);
  await dormir(500);
  assert.equal(await el(`!n.hidden && n.getAnimations().length === 0 && !n.querySelector(".fa-rolo")`), true, "com movimento reduzido a folha aparece pronta");

  // com a notificação fechada não sobra timer: o aviso some e nada mais roda
  await js(`notifFecha()`);
  assert.equal(await js(`notifAtual === null && notifFila.length === 0`), true);
  assert.deepEqual(erros, [], "sem erro no console");
  console.log(`notificação do agente: ok (CPU em repouso ${(cpu * 100).toFixed(2)}%; folha no meio da tela, mensagem inteira sem HTML, marca em construção, Esc, fila, carimbo, botão ir, Planejar igual, movimento reduzido). Capturas em target/notificacao.png e target/planejar-folha.png`);
} finally { limpar(); }
