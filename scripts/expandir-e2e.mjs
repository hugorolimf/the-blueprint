// Confere no navegador de verdade (Edge sem janela) o movimento de expandir e recolher (ação 1.13, web/js/expandir.js): os quatro caminhos animam,
// o estado final é o de sempre, clique rápido não deixa animação presa, o foco volta ao controle e com movimento reduzido nada anima.
// Uso: node scripts/expandir-e2e.mjs [exe] [pasta-com-web]   (precisa do Edge; serve web/ da pasta dada, por padrão a do repositório, sem recompilar)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const assets = resolve(process.argv[3] || raiz);
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const [porta, depuracao] = [await portaLivre(), await portaLivre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-expandir-"));
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
  await cdp("Emulation.setFocusEmulationEnabled", { enabled: true });
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(3000);
  const id = JSON.stringify(blueprint.id), fid = JSON.stringify(blueprint.acoes.find((f) => f.planos?.length).id);
  const planos = () => js(`document.querySelectorAll("svg.desenho .chk").length`);
  const animacoes = () => js(`document.querySelector("svg.desenho").getAnimations({ subtree: true }).filter((a) => !a.effect.target.classList.contains("pisca")).length`); // o destaque do irPara (2,2 s) é de outra animação
  const entrada = () => js(`document.querySelector("svg.desenho").getAnimations({ subtree: true }).filter((a) => a.effect.getKeyframes()[0].fillOpacity === "0" && a.effect.target.classList.contains("chk")).length`);
  const parado = async (onde) => { await dormir(2200); assert.equal(await animacoes(), 0, "nenhuma animação sobra: " + onde); assert.equal(await js(`[...document.querySelectorAll("svg.desenho .chk")].every((e) => getComputedStyle(e).fillOpacity === "1")`), true, "os planos terminam inteiros: " + onde); };
  const abrir = async () => { await js(`abertos[${id}] = true; render()`); await dormir(2200); };
  await abrir();
  const abertos = await planos();
  assert(abertos > 0, "o blueprint aberto mostra planos e checkpoints");

  // (1) recolher pelo cabeçalho: os planos esmaecem antes do redesenho; no fim, sem planos e sem animação presa
  await js(`alternar(${id})`);
  await dormir(40);
  assert.equal(await js(`abertos[${id}]`), false, "o estado muda na hora");
  assert.equal(await planos(), abertos, "durante o esmaecer os planos ainda estão na folha");
  assert(await animacoes() > 0, "recolher esmaece os planos (animações ativas)");
  await dormir(400);
  assert.equal(await planos(), 0, "recolhido: sem planos na folha");
  await parado("depois de recolher");
  // (2) expandir: os planos novos entram com animação e terminam inteiros
  await js(`alternar(${id})`);
  assert.equal(await planos(), abertos, "expandido: os mesmos planos de antes");
  assert(await entrada() > 0, "os planos que entram aparecem com fill-opacity de 0 a 1");
  await parado("depois de expandir");
  // (3) Expandir tudo / Recolher tudo (os botões do topo)
  await js(`document.getElementById("b-fechar").click()`);
  await dormir(500);
  assert.equal(await planos(), 0, "Recolher tudo fecha as ações");
  await parado("depois de Recolher tudo");
  await js(`document.getElementById("b-abrir").click()`);
  assert.equal(await planos(), abertos, "Expandir tudo abre as ações");
  assert(await animacoes() > 0, "Expandir tudo anima");
  await parado("depois de Expandir tudo");
  // (4) clique numa ação compacta: abre, e a ação aparece com animação
  await js(`abertos[${id}] = false; render()`);
  await dormir(2200);
  await js(`irPara(${id}, ${fid})`);
  assert.equal(await js(`abertos[${id}]`), true, "irPara abre as ações");
  assert(await animacoes() > 0, "irPara anima");
  assert.equal(await js(`!!document.querySelector('.acao[data-f=' + JSON.stringify(${fid}) + ']')`), true, "a ação está na folha");
  await parado("depois de irPara");
  // (5) clique rápido: três trocas seguidas, três vezes o estado, nenhuma animação presa
  await js(`alternar(${id})`); await dormir(30); await js(`alternar(${id})`); await dormir(30); await js(`alternar(${id})`);
  assert.equal(await js(`abertos[${id}]`), false, "três cliques seguidos terminam recolhido");
  await dormir(600);
  assert.equal(await planos(), 0, "três cliques seguidos: folha recolhida");
  await parado("depois de três cliques");
  await js(`alternar(${id})`); await dormir(30); await js(`alternar(${id})`);
  await dormir(600);
  assert.equal(await js(`abertos[${id}]`), false, "dois cliques voltam ao estado de antes");
  assert.equal(await planos(), 0, "dois cliques: folha como antes");
  await parado("depois de dois cliques");
  // (6) foco do teclado: Enter no cabeçalho recolhe e o foco continua no cabeçalho
  await abrir();
  await js(`document.querySelector("[data-toggle=" + JSON.stringify(${id}) + "]").focus()`);
  await js(`document.activeElement.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true }))`);
  await dormir(600);
  assert.equal(await js(`abertos[${id}]`), false, "Enter recolhe");
  assert.equal(await js(`document.activeElement?.dataset?.toggle`), blueprint.id, "o foco continua no cabeçalho depois do redesenho");
  await parado("depois do Enter");
  // (7) projetos: a seção abre e fecha com o mesmo movimento, só esmaecendo os planos dela
  await js(`(() => { const p = D.blueprints.find((x) => x.id === tela); p.projetos = [{ id: "pjexp", nome: "Projeto do teste" }]; p.acoes.forEach((f) => { f.projetoId = "pjexp"; }); abertos[p.id] = true; abrirTodosProjetos(p, true); render(); })()`);
  await dormir(2200);
  const daSecao = await planos();
  assert(daSecao > 0, "com projetos: a seção começa aberta");
  await js(`alternarProjeto(chaveProjeto(D.blueprints.find((x) => x.id === tela), "pjexp"))`);
  await dormir(40);
  assert(await animacoes() > 0, "fechar a seção esmaece os planos");
  await dormir(500);
  assert.equal(await planos(), 0, "seção fechada: sem planos");
  await parado("depois de fechar a seção");
  await js(`alternarProjeto(chaveProjeto(D.blueprints.find((x) => x.id === tela), "pjexp"))`);
  assert.equal(await planos(), daSecao, "seção aberta: os planos de volta");
  assert(await entrada() > 0, "abrir a seção faz os planos aparecerem");
  await parado("depois de abrir a seção");
  // (8) movimento reduzido: sem nenhuma animação e com o mesmo resultado final
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "reduce" }] });
  const secao = `alternarProjeto(chaveProjeto(D.blueprints.find((x) => x.id === tela), "pjexp"))`;
  await js(secao);
  assert.equal(await animacoes(), 0, "movimento reduzido: recolher sem animação");
  assert.equal(await planos(), 0, "movimento reduzido: recolhido de uma vez");
  await js(secao);
  assert.equal(await animacoes(), 0, "movimento reduzido: expandir sem animação");
  assert.equal(await planos(), daSecao, "movimento reduzido: o mesmo resultado final");
  await cdp("Emulation.setEmulatedMedia", { features: [{ name: "prefers-reduced-motion", value: "no-preference" }] });
  assert.deepEqual(erros, [], "sem erro de JavaScript na página");
  console.log("Expandir e recolher: cabeçalho, botões, ação compacta, seção de projeto, clique rápido, foco e movimento reduzido conferidos, sem animação presa");
} finally { limpar(); }
process.exit(0);
