// Confere no navegador de verdade (Edge sem janela) a pesquisa da barra: na lista do workspace (casa planos e checkpoints e diz onde achou), no blueprint aberto
// (esmaece o que não casa, destaca o trecho, Esc limpa e a folha volta igual), os atalhos (/, Ctrl+K, Esc, Enter), as seções fechadas, o inglês, a tela estreita
// e o custo por tecla num blueprint com 80 ações. Uso: node scripts/pesquisa-e2e.mjs [exe] [pasta-com-web]   (precisa do Edge; serve web/ da pasta, sem recompilar)
// PESQUISA_CAPTURA=<pasta> grava capturas de tela dos estados conferidos.
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-pesquisa-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); };
process.on("exit", limpar);
const norm = (s) => String(s ?? "").normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  // uma palavra que só aparece no texto de um plano de UM blueprint (nem em nome, descrição ou ação de nenhum): a lista só a acha olhando dentro das ações
  const nomes = dados.blueprints.flatMap((p) => [p.nome, p.descricao, ...p.acoes.flatMap((f) => [f.nome, f.obs])]).map(norm).join(" ");
  let alvo = null;
  for (const p of dados.blueprints) for (const f of p.acoes) for (const t of f.planos || []) for (const w of norm(t.texto).split(/[^a-z0-9]+/)) {
    if (!alvo && w.length >= 6 && !nomes.includes(w) && dados.blueprints.filter((x) => x.acoes.some((y) => (y.planos || []).some((z) => norm(z.texto).includes(w)))).length === 1) alvo = { p, f, t, palavra: w };
  }
  assert(alvo, "os dados de exemplo têm uma palavra só de plano");
  let pagina;
  for (let i = 0; i < 100 && !pagina; i++) { try { pagina = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(pagina.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 1500)); return r.result.result.value; };
  const captura = async (nome) => { if (process.env.PESQUISA_CAPTURA) writeFileSync(join(process.env.PESQUISA_CAPTURA, `pesquisa-${nome}.png`), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); };
  // teclas de verdade (o navegador gera o keydown, o foco e o texto), não eventos montados à mão
  const tecla = async (key, { code = "", vk = 0, mods = 0, text } = {}) => {
    await cdp("Input.dispatchKeyEvent", { type: text ? "keyDown" : "rawKeyDown", key, code, windowsVirtualKeyCode: vk, modifiers: mods, text });
    await cdp("Input.dispatchKeyEvent", { type: "keyUp", key, code, windowsVirtualKeyCode: vk, modifiers: mods });
  };
  const barra = () => tecla("/", { code: "Slash", vk: 191, text: "/" });
  const ctrlK = () => tecla("k", { code: "KeyK", vk: 75, mods: 2 });
  const esc = () => tecla("Escape", { code: "Escape", vk: 27 });
  const enter = (mods = 0) => tecla("Enter", { code: "Enter", vk: 13, mods, text: "\r" });
  const digita = async (texto) => { await cdp("Input.insertText", { text: texto }); await dormir(350); }; // 350 ms: o atraso da digitação (90 ms) e o desenho
  const foco = () => js(`document.activeElement?.id || document.activeElement?.tagName`);
  await cdp("Runtime.enable");
  await cdp("Page.enable");
  // espera a página carregar os dados (o exe debug em partida fria passa de 3 s), não um tempo fixo
  const abrir = async (hash, busca = "") => {
    await cdp("Page.navigate", { url: `${url}/${busca}#${hash}` });
    await dormir(300);
    for (let i = 0; i < 150 && !(await js(`typeof carregado !== "undefined" && carregado === true`).catch(() => false)); i++) await dormir(100);
    await dormir(500);
  };
  const sobreposicao = (periodo = true) => js(`(() => {
    const c = document.getElementById("busca").getBoundingClientRect();
    const cruza = (a, b) => a.left < b.right - 0.5 && a.right > b.left + 0.5 && a.top < b.bottom - 0.5 && a.bottom > b.top + 0.5;
    const outros = [...document.querySelectorAll(${periodo ? '".barra > .btn, #periodo .btn"' : '".barra > .btn"'})].filter((b) => b.getClientRects().length).map((b) => b.getBoundingClientRect());
    return { sobre: outros.filter((b) => cruza(c, b)).length, quais: [...document.querySelectorAll(${periodo ? '".barra > .btn, #periodo .btn"' : '".barra > .btn"'})].filter((b) => b.getClientRects().length && cruza(c, b.getBoundingClientRect())).map((b) => b.id), dentro: c.left >= 0 && c.right <= innerWidth, largura: Math.round(c.width), altura: Math.round(c.height), botao: Math.round(document.getElementById("b-ia").getBoundingClientRect().height) };
  })()`);

  // ---- a lista de blueprints do workspace: acha o plano sem saber em qual blueprint ele está
  await abrir("/c/sm");
  assert.equal(await js(`document.body.classList.contains("na-entrada")`), false, "o workspace raiz abriu na lista de blueprints");
  assert.equal(await js(`document.querySelectorAll("[data-goto]").length`), dados.blueprints.length, "a lista mostra todos os blueprints");
  assert.equal(await js(`document.getElementById("busca").placeholder`), "Pesquisar blueprints, ações, planos…");
  let s = await sobreposicao();
  assert(s.sobre === 0 && s.dentro, `na lista o campo não sobrepõe botões: ${JSON.stringify(s)}`);
  assert.equal(s.altura, s.botao, `o campo tem a altura dos botões da barra: ${s.altura} contra ${s.botao}`);
  await barra();
  assert.equal(await foco(), "busca", "/ foca o campo");
  assert.equal(await js(`document.getElementById("busca").value`), "", "a barra do atalho não entra no campo");
  await digita(alvo.palavra.toUpperCase()); // maiúsculas: a pesquisa ignora a caixa
  assert.equal(await js(`document.querySelectorAll("[data-goto]").length`), 1, "só o blueprint com o plano aparece");
  assert.equal(await js(`document.querySelector("[data-goto]").dataset.goto`), alvo.p.id);
  const onde = await js(`document.querySelector("[data-goto] .pesq-onde")?.textContent`);
  assert(/^ACHOU: \d+ PLANO/.test(onde), "o cartão diz onde achou: " + onde);
  assert.equal(await js(`document.getElementById("busca-n").textContent`), "1 resultado");
  await captura("lista");
  await esc();
  assert.equal(await js(`document.getElementById("busca").value`), "", "Esc limpa o campo");
  assert.equal(await foco(), "busca", "o primeiro Esc só limpa; o campo continua em foco");
  assert.equal(await js(`document.querySelectorAll("[data-goto]").length`), dados.blueprints.length, "a lista volta inteira");
  assert.equal(await js(`document.querySelector("[data-goto] .pesq-onde")`), null);
  assert.equal(await js(`document.getElementById("busca-n").textContent`), "");
  await esc();
  assert.notEqual(await foco(), "busca", "o segundo Esc, com o campo vazio, sai dele");
  await ctrlK();
  assert.equal(await foco(), "busca", "Ctrl+K foca o campo");
  await digita("zzzqq");
  assert.equal(await js(`document.getElementById("busca-n").textContent`), "nenhum resultado");
  assert.equal(await js(`document.querySelectorAll("[data-goto]").length`), 0);
  await js(`pesquisaLimpa()`);

  // ---- clicar no cartão abre o blueprint já com a pesquisa aplicada
  await digita(alvo.palavra);
  await js(`document.querySelector("[data-goto]").dispatchEvent(new MouseEvent("click", { bubbles: true }))`);
  await dormir(2500);
  assert.equal(await js(`location.hash`), `#/p/${alvo.p.id}`, "abriu o blueprint");
  assert.equal(await js(`document.getElementById("busca").value`), alvo.palavra, "a consulta foi junto");
  assert.equal(await js(`document.getElementById("busca").placeholder`), "Pesquisar ações, planos e checkpoints…");
  const achados = await js(`({ acendeu: document.querySelectorAll(".pesq-achou").length, fora: document.querySelectorAll(".acao:not(.pesq-achou):not(.pesq-tem)").length, realces: document.querySelectorAll(".pesq-realce").length, contador: document.getElementById("busca-n").textContent })`);
  assert(achados.acendeu >= 1 && achados.fora >= 1 && achados.realces >= 1, "o plano acende, o resto esmaece e o trecho é destacado: " + JSON.stringify(achados));
  assert.match(achados.contador, /^\d+ resultados?$/);
  const textoAchado = await js(`document.querySelector(".chk.pesq-achou text.tx").textContent`);
  assert(norm(textoAchado).includes(alvo.palavra), "o item aceso é o plano da palavra: " + textoAchado);
  s = await sobreposicao();
  assert(s.sobre === 0 && s.dentro, `no blueprint o campo não sobrepõe botões: ${JSON.stringify(s)}`);
  await captura("blueprint");

  // ---- a folha volta igual ao limpar (sem redesenhar: o mesmo SVG, só sem as marcas)
  await js(`pesquisaLimpa()`);
  await dormir(300);
  const antes = await js(`document.querySelector("svg.desenho").outerHTML`);
  await js(`document.getElementById("busca").focus()`);
  await digita(alvo.palavra);
  const esmaecidas = await js(`({ pesquisando: document.querySelector("svg.desenho").classList.contains("pesquisando"), fora: document.querySelectorAll(".acao:not(.pesq-achou):not(.pesq-tem)").length, acoes: document.querySelectorAll(".acao").length, valor: document.getElementById("busca").value, tela })`);
  assert(esmaecidas.pesquisando && esmaecidas.fora > 0, "pesquisando, há ações esmaecidas: " + JSON.stringify(esmaecidas));
  await esc();
  assert.equal(await js(`document.querySelector("svg.desenho").outerHTML === ${JSON.stringify(antes)}`), true, "depois do Esc a folha é idêntica à de antes de pesquisar");
  // vários termos, sem acento: "acao" casa "Ação"; Enter/Shift+Enter andam pelos resultados
  await digita("acao");
  const total = await js(`document.querySelectorAll(".pesq-achou").length`);
  assert(total >= 2, "acao casa os nomes das ações: " + total);
  await enter();
  assert.equal(await js(`document.getElementById("busca-n").textContent`), `1 / ${total}`);
  await enter();
  assert.equal(await js(`document.getElementById("busca-n").textContent`), `2 / ${total}`);
  assert.equal(await js(`document.querySelectorAll(".pesq-atual").length`), 1, "um só resultado em foco");
  await enter(8); // Shift+Enter
  assert.equal(await js(`document.getElementById("busca-n").textContent`), `1 / ${total}`);
  await js(`pesquisaLimpa()`);

  // ---- seções fechadas: o achado abre a seção e, ao limpar, ela volta a ficar fechada
  await js(`(() => {
    const p = D.blueprints.find((x) => x.id === ${JSON.stringify(alvo.p.id)});
    p.projetos = [{ id: "pjA", nome: "Projeto A" }];
    p.acoes.forEach((f, i) => { f.projetoId = "pjA"; });
    projetosAbertos.clear(); abertos[p.id] = true; render();
  })()`);
  await dormir(500);
  assert.equal(await js(`document.querySelectorAll(".chk").length`), 0, "com a seção fechada não há planos na folha");
  await digita(alvo.palavra);
  assert(await js(`document.querySelectorAll(".chk.pesq-achou").length`) >= 1, "o plano achado numa seção fechada abre a seção e acende");
  await js(`pesquisaLimpa()`);
  await dormir(300);
  assert.equal(await js(`document.querySelectorAll(".chk").length + "|" + projetosAbertos.size`), "0|0", "ao limpar a seção volta a ficar fechada");
  await js(`(() => { const p = D.blueprints.find((x) => x.id === ${JSON.stringify(alvo.p.id)}); delete p.projetos; p.acoes.forEach((f) => { delete f.projetoId; }); render(); })()`);

  // ---- 80 ações: custo por tecla (a parte síncrona e até o quadro seguinte)
  await js(`(() => {
    const p = D.blueprints.find((x) => x.id === ${JSON.stringify(alvo.p.id)});
    for (let i = 0; i < 80; i++) p.acoes.push({ id: "gerada" + i, nome: "Ação gerada " + i, obs: "instruções da ação " + i, inicio: "2026-11-0" + (1 + (i % 8)), fim: "2026-11-2" + (i % 8), status: "planejado",
      planos: Array.from({ length: 5 }, (_, k) => ({ id: "gp" + i + "_" + k, texto: "Plano gerado " + k + " da ação " + i, descricao: "descrição " + k, feito: false })), checkpoints: [0, 1].map((k) => ({ id: "gm" + i + "_" + k, nome: "Checkpoint gerado " + k, data: "2026-11-1" + k, feito: false })) });
    abertos[p.id] = true; render();
  })()`);
  await dormir(800);
  const volume = await js(`({ acoes: document.querySelectorAll(".acao").length, itens: document.querySelectorAll(".chk").length })`);
  assert(volume.acoes >= 80 && volume.itens >= 560, "blueprint grande desenhado: " + JSON.stringify(volume));
  const { saida: medidas, redesenho, quadroVazio } = await js(`(async () => {
    const consultas = ["a", "pl", "plano", "plano g", "plano gerado 3", "ação gerada 7", "instruções da ação 4", "checkpoint", "zzz", "e", "gerad", "plano gerado 3 da ação 41"], saida = [];
    const p = D.blueprints.find((x) => x.id === tela);
    let t = performance.now(); render(); const redesenho = performance.now() - t; // o que custaria refazer a folha a cada tecla
    await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
    t = performance.now(); await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0))); const quadroVazio = performance.now() - t; // um quadro sem mudança nenhuma
    for (const q of consultas) {
      const tentativas = [];
      for (let k = 0; k < 3; k++) { // a melhor de 3: o resto é ruído da máquina
        consulta = q; document.getElementById("busca").value = q;
        const t0 = performance.now();
        pesquisaAplica();
        const sincrono = performance.now() - t0;
        await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0))); // até o quadro desenhado
        tentativas.push({ sincrono, quadro: performance.now() - t0 });
        consulta = ""; pesquisaAplica(); await new Promise((r) => requestAnimationFrame(() => setTimeout(r, 0)));
      }
      consulta = q; pesquisaAplica();
      t = performance.now(); pesqBlueprint(p, pesqTermos(q), acoesVisiveis(p)); const logica = performance.now() - t;
      saida.push({ q, sincrono: Math.min(...tentativas.map((x) => x.sincrono)), quadro: Math.min(...tentativas.map((x) => x.quadro)), logica, achados: document.querySelectorAll(".pesq-achou").length, realces: document.querySelectorAll(".pesq-realce").length });
    }
    return { saida, redesenho, quadroVazio };
  })()`);
  const pior = Math.max(...medidas.map((m) => m.sincrono)), piorQuadro = Math.max(...medidas.map((m) => m.quadro));
  const mediana = [...medidas.map((m) => m.sincrono)].sort((a, b) => a - b)[Math.floor(medidas.length / 2)];
  console.log(`refazer a folha inteira custaria ${redesenho.toFixed(0)} ms por tecla; um quadro sem mudança leva ${quadroVazio.toFixed(0)} ms neste Edge sem GPU`);
  console.log(`custo por tecla com ${volume.acoes} ações e ${volume.itens} itens (melhor de 3): JS mediana ${mediana.toFixed(1)} ms, pior ${pior.toFixed(1)} ms; até o quadro desenhado, pior ${piorQuadro.toFixed(1)} ms`);
  for (const m of medidas) console.log(`  "${m.q}": js ${m.sincrono.toFixed(1)} ms (correspondência ${m.logica.toFixed(1)}), quadro ${m.quadro.toFixed(1)} ms, ${m.achados} achados, ${m.realces} destaques`);
  assert(pior < 50, `cada tecla responde em menos de 50 ms (pior ${pior.toFixed(1)} ms)`);
  assert(medidas.find((m) => m.q === "zzz").achados === 0, "consulta sem resultado não acende nada");
  assert(medidas.find((m) => m.q === "plano gerado 3 da ação 41").achados >= 1);
  await captura("grande");
  await js(`pesquisaLimpa()`);
  await js(`(() => { const p = D.blueprints.find((x) => x.id === ${JSON.stringify(alvo.p.id)}); p.acoes = p.acoes.filter((f) => !f.id.startsWith("gerada")); render(); })()`);

  // ---- tela estreita: o campo cabe na janela e não sobrepõe os botões (na lista e no blueprint)
  for (const largura of [1000, 640, 390]) {
    await cdp("Emulation.setDeviceMetricsOverride", { width: largura, height: 700, deviceScaleFactor: 1, mobile: false });
    await dormir(500);
    await js(`document.getElementById("busca").focus()`); await dormir(100); // aberta (na barra compacta ela abre por cima dos ícones até o Esc): só precisa caber na janela
    s = await sobreposicao(largura >= 640);
    assert(s.dentro, `em ${largura} px, o campo aberto cabe na janela: ${JSON.stringify(s)}`);
    await js(`document.getElementById("busca").blur()`); await dormir(100); // em repouso: não sobrepõe nenhum botão
    s = await sobreposicao(largura >= 640); // abaixo de 640 px a barra ocupa 4 fileiras e a faixa de período (fixa em 116 px) fica por baixo: sem essa conferência
    await captura(`estreita-${largura}`);
    assert(s.sobre === 0 && s.dentro, `em ${largura} px, no blueprint: ${JSON.stringify(s)}`);
    await js(`location.hash = "/c/sm"`); await dormir(800);
    s = await sobreposicao();
    assert(s.sobre === 0 && s.dentro, `em ${largura} px, na lista: ${JSON.stringify(s)}`);
    await js(`location.hash = "/p/${alvo.p.id}"`); await dormir(800);
  }
  await cdp("Emulation.clearDeviceMetricsOverride");

  // ---- inglês: placeholder, "onde achou" e contador traduzidos
  await abrir("/c/sm", "?idioma=en");
  assert.equal(await js(`document.getElementById("busca").placeholder`), "Search blueprints, actions, plans…");
  await js(`document.getElementById("busca").focus()`);
  await digita(alvo.palavra);
  assert(/^FOUND: \d+ PLAN/.test(await js(`document.querySelector("[data-goto] .pesq-onde").textContent`)), "o cartão diz onde achou, em inglês");
  assert.equal(await js(`document.getElementById("busca-n").textContent`), "1 result");

  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log("Pesquisa: lista (planos, onde achou), blueprint (esmaece, destaca, volta igual), atalhos, seções fechadas, estreita, inglês e custo por tecla conferidos, sem erro na página");
} finally { limpar(); }
process.exit(0);
