// A visão de acompanhamento dos agentes (modo foco) no navegador de verdade (Edge sem janela), com o servidor Rust e agentes simulados pela rota /api/mcp/evento (nenhum token gasto):
// estado vazio com dica; um cartão por agente (figura, papel, estado, ação e plano com o número da folha, fala, arquivos, desde quando, eventos da linha do tempo), ao vivo por SSE;
// o cartão acompanha o agente quando ele troca de plano (dois avisos seguidos); clicar fixa o agente (cartão grande com o histórico); Esc sobe um degrau (detalhe -> lista -> folha);
// Alt+F e o botão; «Ir ao plano na folha»; as entradas pela figura do Claude e pelo menu do botão direito; agente em repouso. Capturas em target/foco-*.png.
// Uso: node scripts/foco-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; serve web/ da pasta, sem recompilar; precisa do Microsoft Edge; ~20 s)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { dormir, exePadrao, ligarServidor, pastaDeDados, portaLivre, postJson, raiz } from "./lib-e2e-rust.mjs";

const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dados = await pastaDeDados("blueprint-foco", { BLUEPRINT_ASSETS_DIR: raiz });
// oito ações de dois planos cada: a folha é mais alta que a janela, então ir até o plano 15 pede rolagem
const acoes = Array.from({ length: 8 }, (_, i) => ({ id: `f${i + 1}`, nome: `Ação ${i + 1}`, inicio: `2026-11-${String(i + 1).padStart(2, "0")}`, fim: `2026-11-${String(i + 4).padStart(2, "0")}`, status: "andamento",
  planos: [1, 2].map((k) => ({ id: `t${2 * i + k}`, texto: `Plano ${2 * i + k}`, descricao: "d" })), checkpoints: [] }));
writeFileSync(join(dados.dir, "data.json"), JSON.stringify({ blueprints: [{ id: "p1", nome: "Projeto", acoes }], usuario: { nome: "Teste" } }));
const depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dados.dir, "perfil")}`, "--window-size=1280,900", "about:blank"], { stdio: "ignore" });
let servidor;
const limpar = () => { try { edge.kill(); } catch {} try { servidor?.kill(); } catch {} setTimeout(() => { try { rmSync(dados.dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const avisa = (extra) => postJson(`${dados.url}/api/mcp/evento`, { workspaceId: "sm", blueprintId: "p1", ...extra });
const acaoDe = (planoId) => `f${Math.ceil(Number(planoId.slice(1)) / 2)}`;
const progresso = (planoId, texto, extra = {}) => avisa({ tipo: "progresso", texto, agente: "Claude", estado: "escrevendo", acaoId: acaoDe(planoId), planoId, onde: ["web/js/foco.js"], ...extra });
try {
  servidor = await ligarServidor(exePadrao(), dados);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  const espera = async (cond, msg) => { for (let i = 0; i < 80; i++) { if (await cond()) return; await dormir(100); } assert.fail(msg); };
  const captura = async (nome) => { mkdirSync(join(raiz, "target"), { recursive: true }); writeFileSync(join(raiz, "target", `foco-${nome}.png`), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); };
  const tecla = (key, extra = {}) => js(`document.dispatchEvent(new KeyboardEvent("keydown", { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true, ...${JSON.stringify(extra)} }))`);
  const aberta = () => js(`!!document.getElementById("foco-vista")`);
  const pressed = () => js(`document.getElementById("b-foco").getAttribute("aria-pressed")`);
  const cartao = (nome) => js(`(() => { const c = document.querySelector('.foco-cartao[data-agente="${nome}"]'); return c ? c.textContent.replace(/\\s+/g, " ") : ""; })()`);
  const nomes = () => js(`[...document.querySelectorAll(".foco-cartao")].map((c) => c.dataset.agente)`);
  const tem = (nome, ...trechos) => async () => { const t = await cartao(nome); return trechos.every((x) => t.includes(x)); };
  await cdp("Runtime.enable");
  await cdp("Page.navigate", { url: `${dados.url}/#/p/p1` });
  await espera(() => js(`typeof D !== "undefined" && !!D.blueprints?.find((p) => p.id === "p1") && !!mcpSse && document.querySelectorAll(".chk").length > 0`), "a página carrega os dados e abre o SSE");
  assert(await js(`document.documentElement.scrollHeight > innerHeight * 1.5`), "a folha é mais alta que a janela");

  // ---- sem agente: a visão abre com o estado vazio e a dica; a folha por baixo fica inerte
  assert.equal(await pressed(), "false");
  await js(`document.getElementById("b-foco").click()`);
  await espera(aberta, "o botão abre a visão");
  assert.equal(await pressed(), "true", "o botão fica pressionado");
  assert.equal(await js(`document.getElementById("blueprint-layout").inert`), true, "a folha por baixo fica inerte");
  assert(await js(`document.querySelector("#foco-vista .foco-vazio .dica").textContent.includes("MCP")`), "o estado vazio traz a dica");
  assert.equal(await js(`document.querySelectorAll(".foco-cartao").length`), 0);
  await captura("vazio");

  // ---- um agente: o cartão aparece, ao vivo, com figura, papel, estado, ação e plano (com o número da folha), fala e arquivos
  await progresso("t1", "Lendo o plano um");
  await espera(tem("Claude", "Ação 1.1 · Ação 1", "Plano 1", "Lendo o plano um", "web/js/foco.js", "Agente principal", "Escrevendo"), "o cartão do agente aparece com ação, plano, fala, arquivos, papel e estado");
  assert.equal(await js(`document.querySelectorAll(".foco-cartao").length`), 1);
  assert(await js(`!!document.querySelector('.foco-cartao[data-agente="Claude"] .foco-fig svg')`), "a figura do Claude na pose do estado");
  await espera(tem("Claude", "No plano", "desde"), "desde quando está no plano (do primeiro aviso)");
  await espera(() => js(`document.querySelectorAll('.foco-cartao[data-agente="Claude"] .ldt li').length >= 1`), "os eventos da linha do tempo da ação");
  assert((await js(`document.querySelector('.foco-cartao[data-agente="Claude"] .ldt li').textContent`)).includes("Lendo o plano um"));
  assert.equal(await js(`document.querySelector(".foco-vazio")`), null, "com agente o estado vazio sai");

  // ---- um subagente com papel: segundo cartão, depois do principal
  await avisa({ tipo: "progresso", texto: "Revisando o plano 3", agente: "revisor", papel: "revisor", estado: "testando", acaoId: "f2", planoId: "t3", onde: ["src/"] });
  await espera(tem("revisor", "Revisor", "Testando", "Ação 1.2", "Plano 3", "Revisando o plano 3"), "o subagente tem o cartão dele (papel, estado, ação, plano)");
  assert.deepEqual(await nomes(), ["Claude", "revisor"], "o principal vem primeiro");
  await captura("lista");
  // ---- subagente que conclui: some sozinho depois de 10 s (terminar a tarefa dele não é terminar o trabalho); quem trabalha continua
  await avisa({ tipo: "progresso", texto: "Pesquisa terminada", agente: "pesquisa", papel: "explorador", estado: "concluido", acaoId: "f2", planoId: "t3" });
  await espera(async () => (await nomes()).includes("pesquisa"), "o subagente que concluiu aparece por uns segundos");
  await dormir(10800);
  assert.deepEqual(await nomes(), ["Claude", "revisor"], "o subagente concluído sumiu sozinho depois de 10 s");

  // ---- seguir: dois avisos seguidos; o cartão acompanha o agente até o plano 15
  await progresso("t2", "Passando para o plano dois");
  await progresso("t15", "Agora no plano quinze");
  await espera(tem("Claude", "Ação 1.8", "Plano 15", "Agora no plano quinze"), "o cartão acompanhou o agente até o plano 15");
  assert(!(await cartao("Claude")).includes("Plano 1 "), "o plano antigo saiu do cartão");
  await progresso("t15", "Editando web/js/foco-vista.js", { onde: ["web/js/foco-vista.js"] });
  await espera(tem("Claude", "Editando web/js/foco-vista.js"), "a fala nova chega por SSE, sem recarregar");
  await espera(() => js(`document.querySelector('.foco-cartao[data-agente="Claude"] .ldt li').textContent.includes("Editando web/js/foco-vista.js")`), "e o evento novo fica em cima na lista");
  assert((await js(`document.querySelectorAll('.foco-cartao[data-agente="Claude"] .ldt li').length`)) <= 3, "o cartão da lista mostra no máximo 3 eventos");

  // ---- clicar no cartão fixa o agente: cartão grande com o histórico; o detalhe acompanha o plano seguinte
  await js(`document.querySelector('.foco-cartao[data-agente="Claude"] [data-foco-abrir]').click()`);
  await espera(() => js(`!!document.querySelector(".foco-cartao.grande")`), "clicar no nome abre o detalhe");
  assert.deepEqual(await nomes(), ["Claude"], "o detalhe é um cartão só");
  assert((await js(`document.querySelector("#foco-vista .ed-topo b").textContent`)).includes("Claude"), "o título nomeia o agente");
  assert.equal(await js(`document.querySelector("#foco-vista [data-foco-voltar]").textContent`), "← Voltar aos agentes");
  assert((await js(`document.querySelectorAll(".foco-cartao.grande .ldt li").length`)) >= 2, "o histórico traz os eventos do agente neste plano (os dois avisos do plano 15)");
  await captura("detalhe");
  await progresso("t16", "Já no plano dezesseis");
  await espera(tem("Claude", "Plano 16", "Já no plano dezesseis"), "o detalhe acompanha o agente quando ele troca de plano");

  // ---- Esc sobe um degrau: detalhe -> lista -> folha
  await tecla("Escape");
  await espera(() => js(`!!document.querySelector(".foco-grade") && !document.querySelector(".foco-cartao.grande")`), "Esc volta à lista");
  assert(await aberta(), "a visão continua aberta depois do primeiro Esc");
  await tecla("Escape");
  await espera(async () => !(await aberta()) && (await pressed()) === "false", "o segundo Esc fecha a visão");
  assert.equal(await js(`document.getElementById("blueprint-layout").inert`), false, "a folha volta a responder");

  // ---- Alt+F abre e fecha
  await tecla("f", { altKey: true });
  await espera(aberta, "Alt+F abre a visão");
  await tecla("f", { altKey: true });
  await espera(async () => !(await aberta()), "Alt+F fecha a visão");

  // ---- «Ir ao plano na folha»: fecha a visão e leva a folha até o plano (que começa fora da janela)
  await js(`scrollTo(0, 0)`);
  await js(`document.getElementById("b-foco").click()`);
  await espera(tem("Claude", "Plano 16"), "a visão reabre com o agente no plano 16");
  await js(`document.querySelector('.foco-cartao[data-agente="Claude"] [data-foco-ir]').click()`);
  await espera(async () => !(await aberta()), "o botão fecha a visão");
  await espera(() => js(`(() => { const e = document.querySelector('.chk[data-id="t16"]'); if (!e) return false; const r = e.getBoundingClientRect(); return r.top >= 0 && r.bottom <= innerHeight; })()`), "a folha rolou até o plano 16");
  assert.equal(await js(`document.getElementById("blueprint-layout").inert`), false);

  // ---- as entradas pela folha: a figura do Claude (folhinha) e o menu do botão direito do plano abrem o detalhe do agente
  await js(`document.querySelector('#claude-camada [data-robo-nome="Claude"]').click()`);
  await espera(() => js(`!document.getElementById("nota-folha").hidden`), "a figura abre a folhinha");
  await js(`[...document.querySelectorAll("#nota-folha button")].find((b) => b.textContent === "Acompanhar este agente").click()`);
  await espera(() => js(`!!document.querySelector(".foco-cartao.grande")`), "«Acompanhar este agente» abre o detalhe dele");
  await tecla("Escape"); await tecla("Escape");
  await espera(async () => !(await aberta()), "Esc fecha");
  const menu = (seletor) => js(`(() => { const el = document.querySelector(${JSON.stringify(seletor)}); const r = el.getBoundingClientRect(); el.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: r.left + 4, clientY: r.top + 4 })); return [...document.querySelectorAll('#menu-contexto [role="menuitem"]')].map((b) => b.textContent); })()`);
  assert(!(await menu('.chk[data-id="t14"]')).includes("Acompanhar este agente"), "plano sem agente: o menu não oferece o item");
  await tecla("Escape");
  assert((await menu('.chk[data-id="t16"]')).includes("Acompanhar este agente"), "plano com agente: o menu oferece o item");
  await js(`[...document.querySelectorAll('#menu-contexto [role="menuitem"]')].find((b) => b.textContent === "Acompanhar este agente").click()`);
  await espera(() => js(`!!document.querySelector(".foco-cartao.grande")`), "o menu do plano abre o detalhe do agente dele");
  await tecla("Escape"); await tecla("Escape");

  // ---- concluir o plano: o cartão fica com o estado Concluído; um agente em repouso aparece sem plano
  assert.equal((await avisa({ tipo: "concluir", texto: "cargo test: ok", agente: "Claude", acaoId: "f8", planoId: "t16" })).status, 200);
  await js(`document.getElementById("b-foco").click()`);
  await espera(tem("Claude", "Concluído"), "o agente que concluiu fica no cartão, como Concluído");
  await js(`agentesLista = [...agentesLista, { agente: "dormindo", estado: "ocioso", blueprintId: "p1", workspaceId: workspaceAtual.id, em: new Date().toISOString(), mensagem: "" }]; agentesAtualiza()`);
  await espera(tem("dormindo", "Em repouso: sem plano"), "o agente em repouso aparece sem plano");
  await captura("repouso");
  await tecla("Escape");

  // ---- o painel de subagentes do Claude Code (subagentStatusLine): o cartão do subagente mostra os tokens, e falhou não parece concluído
  assert.equal((await postJson(`${dados.url}/api/mcp/pasta/vincular`, { cwd: "C:/Proj", workspaceId: "sm", blueprintId: "p1" })).status, 200);
  await postJson(`${dados.url}/api/mcp/hook/subagente`, { hook_event_name: "SubagentStart", cwd: "C:/Proj", agent_id: "d4e5f6a7", agent_type: "Explore" });
  const painel = (extra) => postJson(`${dados.url}/api/mcp/hook/subagentes`, { cwd: "C:/Proj", session_id: "s1", columns: 80, tasks: [{ id: "d4e5f6a7", agentType: "Explore", status: "running", description: "Explorar", label: "Lendo o painel", tokenCount: 12300, ...extra }] });
  assert.equal((await painel({})).status, 200);
  if (!(await aberta())) await js(`document.getElementById("b-foco").click()`);
  await espera(tem("Explore d4e5", "Tokens 12.3k", "Lendo o painel"), "o cartão do subagente mostra os tokens do painel e o rótulo");
  assert.equal((await painel({ status: "failed" })).status, 200);
  await espera(tem("Explore d4e5", "Bloqueado"), "subagente que falhou fica Bloqueado, não Concluído");
  await tecla("Escape");

  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log("modo foco: estado vazio, cartão por agente ao vivo, seguir o plano, detalhe com histórico, Esc em degraus, Alt+F, ir ao plano, entradas pela figura e pelo menu, repouso: ok");
} finally { limpar(); }
process.exit(0);
