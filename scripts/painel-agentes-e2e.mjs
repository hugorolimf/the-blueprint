// Confere os agentes na folha no navegador de verdade (Edge sem janela): liga o servidor Rust com uma cópia dos dados, simula três agentes pelo
// /api/mcp e confere uma figura do Claude por agente, cada uma no plano dele (variante por papel), o deslocamento quando o agente muda de plano
// (some a animação ao parar; sem animar com prefers-reduced-motion), o limite com o contador "+N", a folhinha ao clicar o repouso, o Claude esperando na próxima ação, o botão Iniciar com o Claude e o concluído que sai ao fechar a mensagem.
// (O painel "Agentes trabalhando agora" saiu: as figuras estão na própria folha.) Grava a captura em target/painel-agentes.png.
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-figuras-"));
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
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  const figuras = () => js(`[...document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])")].map((e) => ({ nome: e.dataset.roboNome, rotulo: e.querySelector("svg")?.getAttribute("aria-label") || "", t: e.style.transform }))`);
  const parado = () => js(`({ animacoes: document.getAnimations().filter((a) => { const t = a.effect?.target?.closest?.("#claude-camada .claude-fig:not([data-espera])"); return !!t; }).length })`);
  const espera = () => js(`(() => { const e = document.querySelector("#claude-camada [data-espera]"); return e && { acao: e.dataset.espera, bp: e.dataset.bp, rotulo: e.querySelector("svg")?.getAttribute("aria-label") || "", titulo: e.title }; })()`);
  // a próxima ação (o que o app deve escolher): a primeira não concluída, por data de início, com plano pendente
  const proxima = (ocupadas = []) => blueprint.acoes.map((f, i) => ({ f, i })).filter(({ f }) => f.status !== "concluido" && f.planos?.some((t) => !t.feito) && !ocupadas.includes(f.id)).sort((a, b) => String(a.f.inicio).localeCompare(String(b.f.inicio)) || a.i - b.i)[0]?.f;
  assert.deepEqual(await figuras(), [], "sem agentes, nenhuma figura de agente");
  assert.equal(await js(`!document.getElementById("b-agentes") && !document.getElementById("agentes-painel")`), true, "o painel e o botão antigos não existem mais");

  const avisaTipo = (tipo, corpo) => fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo, workspaceId: "sm", ...corpo }) });
  const avisa = (corpo) => avisaTipo("progresso", { blueprintId: blueprint.id, ...corpo });
  const acaoB = blueprint.acoes.find((f) => f !== acao && f.planos?.length) || acao, planoB = acaoB === acao ? acao.planos[acao.planos.length - 1] : acaoB.planos[0];
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render()`);
  await dormir(600);
  const antesDeTodos = await espera();
  assert(antesDeTodos, "sem agentes, o Claude espera na próxima ação do blueprint aberto");
  assert.equal(antesDeTodos.acao, proxima().id, "e a próxima ação é a primeira não concluída com plano pendente");
  assert(/aguardando/.test(antesDeTodos.rotulo) && antesDeTodos.titulo.includes("Esperando para começar"), "na pose de espera (aguardando), com a dica da ação");
  assert.equal(await js(`document.querySelectorAll(".robo-linha").length`), 0, "a espera não realça plano nenhum");
  assert.equal(await js(`document.getElementById("periodo-andamento-n").textContent`), "0", "a espera não conta no filtro Em andamento");
  await avisa({ texto: "Escrevendo a folha", agente: "claude", estado: "escrevendo", onde: ["web/js/render.js"], acaoId: acao.id, planoId: plano.id });
  await avisa({ texto: "Lendo os módulos do kit", agente: "explorador-kit", papel: "explorador", estado: "lendo", onde: ["kit/lathe/"], acaoId: acao.id, planoId: plano.id });
  await avisa({ texto: "Rodando cargo test", agente: "revisor-testes", papel: "revisor", estado: "testando", onde: ["src/"] });
  await dormir(1500); // o aviso chega por SSE; ninguém consulta
  const fs = await figuras();
  console.log(JSON.stringify(fs));
  assert.deepEqual(fs.map((f) => f.nome).sort(), ["claude", "explorador-kit", "revisor-testes"], "uma figura por agente ativo");
  assert(fs.find((f) => f.nome === "claude").rotulo.startsWith("Claude:"), "o agente sem papel é o Claude");
  // os quadros da pose parada alternam (claude-vida.js); com movimento reduzido ficam num quadro só e sem relógio
  const quadrosAgora = () => js(`[...document.querySelectorAll("#claude-camada .claude-fig")].map((e) => [...e.querySelectorAll("span[data-q]")].find((s) => s.classList.contains("on"))?.dataset.q ?? null)`);
  const vistos = new Set();
  for (let i = 0; i < 12; i++) { (await quadrosAgora()).forEach((q, k) => q !== null && vistos.add(`${k}:${q}`)); await dormir(150); }
  assert([...vistos].map((v) => v.split(":")[0]).some((k, _, l) => new Set([...vistos].filter((v) => v.startsWith(k + ":"))).size > 1), `os quadros alternam (vistos: ${[...vistos]})`);
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await dormir(400);
  assert.equal(await js(`document.querySelectorAll("#claude-camada span[data-q]").length`), 0, "movimento reduzido: sem quadros extras");
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "no-preference" }] });
  await dormir(400);
  assert((await js(`document.querySelectorAll("#claude-camada span[data-q]").length`)) > 0, "sem a preferência, os quadros voltam");
  assert(fs.find((f) => f.nome === "explorador-kit").rotulo.includes("explorador") && fs.find((f) => f.nome === "revisor-testes").rotulo.includes("revisor"), "subagentes na variante do papel");
  assert.equal(await js(`document.querySelectorAll(".robo-linha").length`), 1, "o plano com agente fica realçado");
  assert.equal(await js(`document.getElementById("periodo-andamento-n").textContent`), "3", "o filtro Em andamento conta as figuras (e não a espera)");
  const comAgentes = await espera();
  assert(!comAgentes || comAgentes.acao !== acao.id, "a espera sai da ação em que há agente trabalhando");
  assert.equal(comAgentes?.acao, proxima([acao.id])?.id, "e vai para a próxima ação livre");
  const folga = await js(`(() => { const l = document.querySelector(".robo-linha").getBoundingClientRect(), c = document.querySelector('#claude-camada [data-robo-nome="claude"]').getBoundingClientRect(); return Math.round(l.left - c.right); })()`);
  assert(Math.abs(folga - 6) <= 2, `o Claude fica ao lado do plano dele (folga ${folga} px)`);
  assert.equal((await parado()).animacoes, 0, "parada, a figura não tem animação rodando");
  mkdirSync(join(raiz, "target"), { recursive: true });
  writeFileSync(join(raiz, "target", "painel-agentes.png"), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));

  // o Claude muda de plano: anda (uma animação e o relógio dos quadros) e, ao chegar, não sobra nada rodando
  const antes = (await figuras()).find((f) => f.nome === "claude").t;
  await js(`window.__intervalos = 0; const si = setInterval; window.setInterval = (...a) => { window.__intervalos++; return si(...a); }`);
  await avisa({ texto: "Escrevendo outro plano", agente: "claude", estado: "escrevendo", onde: ["web/js/x.js"], acaoId: acaoB.id, planoId: planoB.id });
  await dormir(450);
  assert.equal((await parado()).animacoes, 1, "andando: uma animação de deslocamento");
  assert((await figuras()).find((f) => f.nome === "claude").rotulo.includes("andando"), "andando: pose de caminhada");
  await dormir(2300);
  assert.equal((await parado()).animacoes, 0, "chegou: nenhuma animação sobra");
  assert((await figuras()).find((f) => f.nome === "claude").t !== antes, "o Claude foi para outro lugar");
  assert((await figuras()).find((f) => f.nome === "claude").rotulo.includes("trabalhando"), "e voltou à pose de trabalho");
  assert(await js(`window.__intervalos > 0`), "o relógio dos quadros existiu durante o deslocamento");

  // prefers-reduced-motion: muda de plano sem andar
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  await avisa({ texto: "Voltando", agente: "claude", estado: "escrevendo", acaoId: acao.id, planoId: plano.id });
  await dormir(500);
  assert.equal((await parado()).animacoes, 0, "reduced-motion: aparece no destino, sem animação");
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "no-preference" }] });

  // limite: o principal + 4 subagentes; o resto vira "+N"
  for (let i = 0; i < 5; i++) await avisa({ texto: `extra ${i}`, agente: `extra-${i}`, papel: ["explorador", "construtor", "revisor"][i % 3], estado: "lendo", acaoId: acao.id, planoId: plano.id });
  await dormir(1500);
  assert.equal(await js(`document.querySelectorAll("#claude-camada .claude-fig:not([data-espera])").length`), 5, "no máximo 5 figuras de agente (1 principal + 4 subagentes; a espera é à parte)");
  assert.equal(await js(`document.querySelector("#claude-camada .claude-mais")?.textContent`), "+3", "os outros 3 viram o contador");

  // clicar na figura abre a folhinha (só leitura)
  await js(`document.querySelector('#claude-camada [data-robo-nome="claude"]').click()`);
  await dormir(600);
  assert((await js(`document.getElementById("nota-folha").innerText`)).includes("CLAUDE"), "a folhinha mostra o que o agente faz agora");

  // repouso: sem sinal por mais de 10 minutos, as figuras NÃO saem: ficam paradas no cabeçalho do blueprint, sem plano
  await js(`document.getElementById("nota-folha").hidden = true; agentesLista.forEach((a) => { a.em = new Date(Date.now() - 11 * 60000).toISOString(); }); agenteAgora = null; agentesAtualiza(); render()`);
  await dormir(300);
  const repouso = await figuras();
  assert.equal(repouso.length, 5, "os agentes sem sinal continuam na folha");
  assert(repouso.every((f) => /parado/.test(f.rotulo)), "e repousam na pose parada");
  assert.equal(await js(`document.querySelectorAll("rect.robo-linha").length`), 0, "sem plano: nenhum plano destacado");

  // concluído: fica no plano, com a bandeira, até o usuário clicar; o clique abre a mensagem de conclusão e só ao FECHAR a mensagem ele sai da folha
  await avisa({ texto: "Concluí o plano: o teste passou", agente: "claude", estado: "concluido", acaoId: acao.id, planoId: plano.id });
  await dormir(1000);
  await js(`agentesLista.forEach((a) => { a.em = new Date(Date.now() - 30 * 60000).toISOString(); }); agenteAgora = null; agentesAtualiza(); render()`); // muito tempo depois: concluído sem clique NÃO sai do plano
  await dormir(600);
  assert((await figuras()).some((f) => f.nome === "claude" && /concluiu/.test(f.rotulo)), "o concluído continua na folha, na pose de concluído, sem ninguém ter clicado");
  assert.equal(await js(`document.querySelectorAll("rect.robo-linha").length`), 1, "e continua no plano em que concluiu");
  assert.notEqual((await espera())?.acao, acao.id, "a espera não fica na ação do concluído");
  await js(`document.querySelector('#claude-camada [data-robo-nome="claude"]').click()`);
  await dormir(600);
  const nota = await js(`document.getElementById("nota-folha").innerText`);
  assert(nota.includes("Mensagem de conclusão") && nota.includes("o teste passou"), "o clique mostra a mensagem de conclusão");
  await dormir(500);
  assert.deepEqual(await js(`[...document.querySelectorAll("#nota-folha .acoes button")].map((b) => b.textContent)`), ["Concluído", "Fechar"], "a folhinha do concluído tem as opções Concluído e Fechar");
  assert((await figuras()).some((f) => f.nome === "claude" && /concluiu/.test(f.rotulo)), "com a mensagem aberta, ele ainda não saiu");
  await js(`document.querySelector("#nota-folha [data-nfecha]").click()`); // Fechar: só fecha a folhinha, o concluído continua na folha
  await dormir(1200);
  assert((await figuras()).some((f) => f.nome === "claude" && /concluiu/.test(f.rotulo)), "Fechar não tira o concluído da folha");
  await js(`document.querySelector('#claude-camada [data-robo-nome="claude"]').click()`);
  await dormir(600);
  await js(`document.querySelector("#nota-folha [data-nacao]").click()`); // Concluído: já vi, ele sai
  await dormir(1500);
  assert(!(await figuras()).some((f) => f.nome === "claude"), "depois de Concluído, o agente sai da folha (não repousa)");
  assert.equal(await js(`document.querySelectorAll("rect.robo-linha").length`), 0, "e sai do plano");
  assert.equal((await figuras()).length, 5, "os outros agentes (em repouso, 7 no servidor) continuam, no limite de 5 figuras");

  // o Claude esperando: clicar abre a folha da ação, com o botão "Iniciar com o Claude" (a rota é simulada: o fetch é interceptado)
  const esp = await espera();
  assert(esp, "o Claude espera na próxima ação");
  await js(`window.__iniciar = []; window.__resposta = { status: 200, corpo: { ok: true, pasta: "x", terminal: "wt" } }; const f0 = window.fetch; window.fetch = (u, o) => String(u).includes("/api/mcp/iniciar") ? (window.__iniciar.push(JSON.parse(o.body)), Promise.resolve(new Response(JSON.stringify(window.__resposta.corpo), { status: window.__resposta.status, headers: { "content-type": "application/json" } }))) : f0(u, o)`);
  await js(`document.querySelector("#claude-camada [data-espera]").click()`);
  await dormir(700);
  assert.equal(await js(`document.querySelector("#nota-folha h4")?.textContent`), "Instruções da ação", "clicar na espera abre a folha da ação");
  const botao = `[...document.querySelectorAll("#nota-folha [data-nacao]")].find((b) => b.textContent === "Iniciar com o Claude")`;
  assert(await js(`!!${botao}`), "a folha da ação tem o botão Iniciar com o Claude");
  await js(`${botao}.click()`);
  await dormir(500);
  assert(/gasta tokens/.test(await js(`document.querySelector("dialog.confirmar")?.innerText || ""`)), "pede confirmação (abre terminal e gasta tokens)");
  assert.equal(await js(`window.__iniciar.length`), 0, "nada é enviado antes de confirmar");
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  await dormir(600);
  const enviado = await js(`window.__iniciar`);
  assert.equal(enviado.length, 1, "confirmado: uma chamada a /api/mcp/iniciar");
  assert.deepEqual([enviado[0].blueprintId, enviado[0].acaoId, typeof enviado[0].workspaceId], [blueprint.id, esp.acao, "string"], "com o blueprint e a ação da espera");
  assert(/Terminal aberto/.test(await js(`document.querySelector("#nota-folha .iniciar-aviso")?.textContent || ""`)), "e avisa que o terminal abriu");
  await js(`window.__resposta = { status: 409, corpo: { erro: "sem pasta", codigo: "sem-pasta" } }; ${botao}.click()`);
  await dormir(500);
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  await dormir(600);
  assert(/Pasta do projeto/.test(await js(`document.querySelector("#nota-folha .iniciar-aviso")?.textContent || ""`)), "sem pasta ligada, explica onde ligar");
  await js(`window.__resposta = { status: 501, corpo: { erro: "x", codigo: "sem-terminal" } }; ${botao}.click()`);
  await dormir(500);
  await js(`document.querySelector('dialog.confirmar [data-r="1"]').click()`);
  await dormir(600);
  assert(/terminal/.test(await js(`document.querySelector("#nota-folha .iniciar-aviso")?.textContent || ""`)), "sem terminal, avisa");
  await js(`${botao}.click()`);
  await dormir(500);
  await js(`document.querySelector('dialog.confirmar [data-r="0"]').click()`);
  await dormir(300);
  assert.equal(await js(`window.__iniciar.length`), 3, "cancelar não envia nada");
  await js(`document.querySelector("#nota-folha [data-nfecha]").click()`);
  await dormir(700);

  // âncoras do lugar do agente quando a linha do plano não está desenhada: a barra da ação (compacta ou aberta) e o cabeçalho da seção (projeto) da ação
  const ancoras = await js(`(() => { const a = claudeAncoras(document.querySelector("svg.desenho")), k = a.secao(${JSON.stringify(acao.id)}); return { barra: !!a.barra(${JSON.stringify(acao.id)})?.w, secao: !!k, cab: !!a.caixaSecao(k || "")?.w }; })()`);
  assert.deepEqual(ancoras, { barra: true, secao: true, cab: true }, "a ação tem barra e cabeçalho de seção desenhados para a figura ficar perto dela");
  await js(`abertos[tela] = false; abrirTodosProjetos(D.blueprints.find((p) => p.id === tela), false); render()`); // seção fechada: a linha do plano some, a barra continua
  await dormir(400);
  const fechada = await js(`(() => { const a = claudeAncoras(document.querySelector("svg.desenho")); return { barra: !!a.barra(${JSON.stringify(acao.id)})?.w, plano: !!a.plano(${JSON.stringify(plano.id)}) }; })()`);
  assert.deepEqual(fechada, { barra: true, plano: false }, "seção fechada: sem linha do plano, com a barra da ação");
  await js(`abertos[tela] = true; abrirTodosProjetos(D.blueprints.find((p) => p.id === tela), true); render()`);
  await dormir(400);

  // sumir e aparecer: a figura encolhe até o centro ao sair e cresce a partir do centro ao voltar (animação da propriedade `scale`)
  assert((await figuras()).length > 0, "há figuras antes de sumir");
  await js(`claudeLimpa(new Set())`);
  await dormir(60);
  assert(await js(`[...document.querySelectorAll("#claude-camada [data-saindo]")].every((e) => e.getAnimations().some((a) => a.effect.getKeyframes().some((k) => k.scale === "0")))`), "ao sair a figura encolhe até o centro");
  await dormir(700);
  assert.equal(await js(`document.querySelectorAll("#claude-camada .claude-fig, #claude-camada .claude-mais").length`), 0, "e depois some da folha");
  await js(`claudeCamadaAtualiza()`);
  await dormir(60);
  assert(await js(`[...document.querySelectorAll("#claude-camada .claude-fig")].some((e) => e.getAnimations().some((a) => a.effect.getKeyframes().some((k) => k.scale === "0")))`), "ao voltar a figura cresce a partir do centro");
  await dormir(700);

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
  console.log("checkpoint marcado pelo agente: a página aplicou, guardou a evidência e gravou no banco");
  // clicar numa ação compacta de um projeto fechado abre o projeto dela e leva até a ação
  await js(`(() => { const p = D.blueprints.find((x) => x.id === tela); p.projetos = [{ id: "pj1", nome: "Projeto de teste" }]; p.acoes.forEach((f) => { f.projetoId = "pj1"; }); projetosAbertos.clear(); abertos[tela] = false; render(); })()`);
  await dormir(600);
  assert.equal(await js(`document.querySelectorAll('.acao[data-f="${acao.id}"] .cota').length`), 0, "projeto fechado: a ação está compacta");
  await js(`document.querySelector('.acao[data-f="${acao.id}"]').dispatchEvent(new MouseEvent("click", { bubbles: true }))`);
  await dormir(900);
  assert.equal(await js(`document.querySelectorAll('.acao[data-f="${acao.id}"] .cota').length > 0`), true, "clicar na ação compacta abre o projeto e a ação aparece aberta");
  assert.equal(await js(`(() => { const r = document.querySelector('.acao[data-f="${acao.id}"]').getBoundingClientRect(); return r.bottom > 0 && r.top < innerHeight; })()`), true, "e a folha leva até ela");
  assert.deepEqual(erros, [], "sem erros de JavaScript na página");
  console.log("agentes na folha: ok (uma figura por agente no plano dele, andar e parar, reduced-motion, +N, folhinha, expiração, checkpoint pelo MCP). Captura em target/painel-agentes.png");
} finally { limpar(); }
