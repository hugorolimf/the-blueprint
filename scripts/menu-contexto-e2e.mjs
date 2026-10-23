// Confere no navegador de verdade (Edge sem janela) os itens do menu do botão direito para cada alvo da folha (ação, plano, fundo, lista de blueprints).
// Uso: node scripts/menu-contexto-e2e.mjs [exe] [pasta-com-web]   (precisa do Edge; serve web/ da pasta dada, por padrão a do repositório, sem recompilar)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const [porta, depuracao] = [await portaLivre(), await portaLivre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-menu-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.length));
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
  await dormir(3000);
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render()`);
  await dormir(500);

  // abre o menu com o botão direito sobre o seletor e devolve o que a tela mostra: cabeçalho, itens (rótulo, ícone, alerta) e quantas linhas separam os grupos
  const menu = async (seletor, captura) => {
    await js(`(() => { const el = document.querySelector(${JSON.stringify(seletor)}); const r = el.getBoundingClientRect(); el.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: r.left + 4, clientY: r.top + 4 })); })()`);
    assert.equal(await js(`document.getElementById("menu-contexto").hidden`), false, `o menu abriu em ${seletor}`);
    const lido = await js(`(() => { const m = document.getElementById("menu-contexto"); return { titulo: m.querySelector(".mc-topo")?.textContent, rotulo: m.getAttribute("aria-label"), separadores: m.querySelectorAll('[role="separator"]').length, itens: [...m.querySelectorAll('[role="menuitem"]')].map((b) => ({ t: b.textContent, icone: !!b.querySelector("svg path, svg circle, svg rect"), perigo: b.classList.contains("perigo"), desabilitado: b.getAttribute("aria-disabled") === "true" })) }; })()`);
    if (captura) writeFileSync(captura, Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
    await js(`document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }))`);
    assert.equal(await js(`document.getElementById("menu-contexto").hidden`), true, "Esc fecha o menu");
    return lido;
  };
  const rotulos = (m) => m.itens.map((i) => i.t);
  const conferir = (m, esperado, onde) => {
    assert.deepEqual(rotulos(m), esperado, `itens ${onde}`);
    assert(m.itens.every((i) => i.icone), `todo item de ${onde} tem ícone`);
    assert.equal(m.itens.filter((i) => i.perigo).map((i) => i.t).join("|"), esperado.filter((t) => t.startsWith("Excluir")).join("|"), `só o excluir é alerta em ${onde}`);
    assert(m.titulo && m.rotulo === m.titulo, `o cabeçalho de ${onde} nomeia o alvo: ${m.titulo}`);
  };
  const cap = process.env.MENU_CAPTURA;
  const naAcao = await menu(".acao[data-p][data-f]", cap && `${cap}-acao.png`);
  conferir(naAcao, ["Ir para esta ação", "Editar ação", "Adicionar plano", "Adicionar checkpoint", "Editar blueprint", "Planejar este blueprint", "Excluir ação…", "Recolher ações", "Voltar aos blueprints", "Entrar em tela cheia"], "da ação");
  assert(naAcao.titulo.startsWith("Ação "), "cabeçalho da ação: " + naAcao.titulo);
  assert.equal(naAcao.separadores, 3, "quatro grupos na ação concluída: abrir, editar, perigo, geral (sem Iniciar com o Claude)");
  // o plano fica dentro da ação: o cabeçalho nomeia o plano e o item de estado vem depois dos de editar
  const noPlano = await menu(".chk[data-p][data-f][data-id]", cap && `${cap}-plano.png`);
  assert(noPlano.titulo.startsWith("Plano · ") || noPlano.titulo.startsWith("Checkpoint · "), "cabeçalho do plano: " + noPlano.titulo);
  const estado = noPlano.itens.findIndex((i) => i.t === "Marcar como feito" || i.t === "Marcar como pendente");
  assert(estado > noPlano.itens.findIndex((i) => i.t === "Planejar este blueprint") && estado < noPlano.itens.findIndex((i) => i.t === "Excluir ação…"), "o estado vem entre editar e excluir");
  assert.equal(noPlano.separadores, 4, "cinco grupos no plano");
  // numa ação ainda aberta aparece Iniciar com o Claude, que chama a confirmação da ação certa (confirmar fica trocado por um registro: nenhum terminal abre, nenhum token é gasto)
  const aberta = await js(`(() => { const f = D.blueprints.flatMap((p) => p.acoes).find((f) => efetiva(f).status !== "concluido" && document.querySelector('.acao[data-f="' + f.id + '"]')); return f && '.acao[data-f="' + f.id + '"]'; })()`);
  assert(aberta, "há uma ação não concluída na folha");
  const naAberta = await menu(aberta);
  conferir(naAberta, ["Ir para esta ação", "Editar ação", "Adicionar plano", "Adicionar checkpoint", "Editar blueprint", "Planejar este blueprint", "Iniciar com o Claude…", "Marcar como concluída", "Prorrogar o fim em 1 dia", "Prorrogar o fim em 1 semana", "Excluir ação…", "Recolher ações", "Voltar aos blueprints", "Entrar em tela cheia"], "da ação aberta");
  assert.equal(naAberta.separadores, 5, "seis grupos na ação aberta: abrir, editar, estado, prazo, perigo, geral");
  assert.equal(naAberta.itens.find((i) => i.t === "Marcar como concluída").desabilitado, true, "com planos e checkpoints pendentes, concluir fica desabilitado");
  await js(`window.confirmou = []; window.confirmar = async (o) => { window.confirmou.push(o.titulo); return false; }`);
  await js(`(() => { const el = document.querySelector(${JSON.stringify(aberta)}); const r = el.getBoundingClientRect(); el.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: r.left + 4, clientY: r.top + 4 })); [...document.querySelectorAll('#menu-contexto [role="menuitem"]')].find((b) => b.textContent === "Iniciar com o Claude…").click(); })()`);
  await dormir(300);
  const pedidos = await js(`window.confirmou`);
  assert.equal(pedidos.length, 1, "o item abre uma confirmação");
  assert(/^Iniciar a ação \d+\.\d+ com o Claude\?$/.test(pedidos[0]), "a confirmação nomeia a ação: " + pedidos[0]);
  // situação e prazo pelo menu: a ação sem itens muda de situação, e o prazo avança e é gravado no servidor
  const idAberta = await js(`${JSON.stringify(aberta)}.match(/"(.+)"/)[1]`);
  const fimAntes = await js(`acaoDe(D.blueprints.find((p) => p.acoes.some((f) => f.id === ${JSON.stringify(idAberta)})).id, ${JSON.stringify(idAberta)}).fim`);
  const clicar = (seletor, rotulo) => js(`(() => { const el = document.querySelector(${JSON.stringify(seletor)}); const r = el.getBoundingClientRect(); el.dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: r.left + 4, clientY: r.top + 4 })); [...document.querySelectorAll('#menu-contexto [role="menuitem"]')].find((b) => b.textContent === ${JSON.stringify(rotulo)}).click(); })()`);
  await clicar(aberta, "Marcar como concluída"); // desabilitado: não faz nada
  await dormir(300);
  assert.notEqual(await js(`efetiva(acaoDe(D.blueprints.find((p) => p.acoes.some((f) => f.id === ${JSON.stringify(idAberta)})).id, ${JSON.stringify(idAberta)})).status`), "concluido", "o item desabilitado não conclui a ação");
  await clicar(aberta, "Prorrogar o fim em 1 semana");
  await dormir(2500);
  const noServidor = (await (await fetch(url + "/api/dados")).json()).blueprints.flatMap((p) => p.acoes).find((f) => f.id === idAberta);
  assert.equal(noServidor.fim, await js(`deDia(dia(${JSON.stringify(fimAntes)}) + 7)`), "o prazo avançou 7 dias e foi gravado no servidor");
  const semItens = await js(`(() => { const p = D.blueprints.find((x) => x.acoes.some((f) => f.id === ${JSON.stringify(idAberta)})); p.acoes.push({ id: "semitens1", nome: "Ação sem itens", inicio: "2026-12-01", fim: "2026-12-05", status: "planejado", progresso: 0, planos: [], checkpoints: [] }); abertos[p.id] = true; render(); return '.acao[data-f="semitens1"]'; })()`);
  const naSem = await menu(semItens);
  assert.deepEqual(rotulos(naSem).filter((t) => t.startsWith("Marcar")), ["Marcar como em andamento", "Marcar como concluída"], "ação sem itens: situação à escolha");
  assert(!naSem.itens.some((i) => i.desabilitado), "ação sem itens: nada desabilitado");
  await clicar(semItens, "Marcar como concluída");
  await dormir(2500);
  const gravada = (await (await fetch(url + "/api/dados")).json()).blueprints.flatMap((p) => p.acoes).find((f) => f.id === "semitens1");
  assert.equal(gravada?.status, "concluido", "a situação concluída foi gravada no servidor");
  assert.deepEqual(rotulos(await menu(semItens)).filter((t) => t.startsWith("Marcar") || t.startsWith("Prorrogar")), ["Marcar como planejada", "Marcar como em andamento"], "concluída sem itens pode reabrir e não prorroga");
  // adicionar plano e checkpoint pelo menu: o Editar abre na ação e o campo do item novo fica em foco
  const cresce = async (rotulo, campo, caixa, lista) => {
    const antes = await js(`D.blueprints.flatMap((p) => p.acoes).find((f) => f.id === ${JSON.stringify(idAberta)}).${lista}.length`);
    await clicar(aberta, rotulo);
    await dormir(600);
    assert.equal(await js(`document.getElementById("editor").hidden`), false, `${rotulo}: o Editar abriu`);
    assert.equal(await js(`document.querySelectorAll(${JSON.stringify(caixa)}).length`), antes + 1, `${rotulo}: um item novo apareceu`);
    assert.equal(await js(`document.activeElement.dataset.c + "|" + !!document.activeElement.closest(${JSON.stringify(caixa)}) + "|" + document.activeElement.value`), `${campo}|true|`, `${rotulo}: o campo do item novo está em foco e vazio`);
    await js(`editar(false)`);
    await dormir(600);
  };
  await cresce("Adicionar plano", "texto", ".item-ed", "planos");
  await cresce("Adicionar checkpoint", "nome", ".checkpoint-ed", "checkpoints");
  // Recolher ações / Expandir ações: o mesmo efeito dos botões do topo, sem projetos (abertos) e com projetos (uma seção por projeto)
  const planosVisiveis = () => js(`document.querySelectorAll(".chk").length`);
  assert((await planosVisiveis()) > 0, "a folha começa expandida");
  await clicar("#folha", "Recolher ações");
  await dormir(300);
  assert.equal(await planosVisiveis(), 0, "Recolher ações fecha as ações (sem projetos)");
  assert(rotulos(await menu("#folha")).includes("Expandir ações"), "depois de recolher o menu oferece Expandir ações");
  await clicar("#folha", "Expandir ações");
  await dormir(300);
  assert((await planosVisiveis()) > 0, "Expandir ações abre as ações (sem projetos)");
  await js(`(() => { const p = D.blueprints.find((x) => x.id === tela); p.projetos = [{ id: "pjmenu", nome: "Projeto do menu" }]; p.acoes.forEach((f) => { f.projetoId = "pjmenu"; }); abrirTodosProjetos(p, true); render(); })()`);
  await dormir(300);
  assert((await planosVisiveis()) > 0, "com projetos: as seções começam abertas");
  await clicar("#folha", "Recolher ações");
  await dormir(300);
  assert.equal(await planosVisiveis(), 0, "Recolher ações fecha todas as seções dos projetos");
  assert(rotulos(await menu("#folha")).includes("Expandir ações"), "com projetos, depois de recolher o menu oferece Expandir ações");
  await clicar("#folha", "Expandir ações");
  await dormir(300);
  assert((await planosVisiveis()) > 0, "Expandir ações abre todas as seções dos projetos");
  // teclado: o foco começa no primeiro item, setas/Home/End andam, Tab fecha
  const tecla = (key) => js(`document.dispatchEvent(new KeyboardEvent("keydown", { key: ${JSON.stringify(key)}, bubbles: true, cancelable: true }))`);
  const focado = () => js(`document.activeElement.textContent`);
  await js(`document.querySelector(${JSON.stringify(aberta)}).dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 300, clientY: 300 }))`);
  assert.equal(await focado(), "Ir para esta ação", "o foco abre no primeiro item");
  await tecla("ArrowDown");
  assert.equal(await focado(), "Editar ação", "seta para baixo vai ao próximo");
  await tecla("End");
  assert.equal(await focado(), "Entrar em tela cheia", "End vai ao último");
  await tecla("ArrowDown");
  assert.equal(await focado(), "Ir para esta ação", "depois do último volta ao primeiro");
  await tecla("Tab");
  assert.equal(await js(`document.getElementById("menu-contexto").hidden`), true, "Tab fecha o menu");
  // tela estreita: o menu inteiro cabe na janela
  await cdp("Emulation.setDeviceMetricsOverride", { width: 390, height: 700, deviceScaleFactor: 1, mobile: false });
  await dormir(500);
  await js(`document.querySelector(${JSON.stringify(aberta)}).dispatchEvent(new MouseEvent("contextmenu", { bubbles: true, cancelable: true, clientX: 380, clientY: 690 }))`);
  const caixa = await js(`(() => { const r = document.getElementById("menu-contexto").getBoundingClientRect(); return { l: r.left, r: r.right, t: r.top, b: r.bottom, w: innerWidth, h: innerHeight }; })()`);
  assert(caixa.l >= 0 && caixa.r <= caixa.w && caixa.t >= 0 && caixa.b <= caixa.h, "em 390 px o menu cabe na janela: " + JSON.stringify(caixa));
  await tecla("Escape");
  await cdp("Emulation.clearDeviceMetricsOverride");
  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log("Menu do botão direito: cabeçalho, grupos, ícones e alerta da ação e do plano conferidos, Esc fecha, sem erro na página");
} finally { limpar(); }
process.exit(0);
