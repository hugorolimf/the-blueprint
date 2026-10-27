// A tela Apresentação com IA no Edge de verdade: abre na apresentação mais nova (Visão interna recolhida), o visor isolado, o EDITOR manual (campos, listas, Sessão da figura, código com
// verificação de sintaxe, Desfazer, erro do servidor), a CONVERSA com o agente (sessão com canal, terminal do app, escolha lembrada), GERAR no terminal do app, Gerar de novo com edições,
// Remover, 390 px sem rolagem horizontal, pt-BR e en. As rotas novas (apresentacoes, apresentacao, editar, pedido, remover, apresentar, terminal/iniciar, sessoes) são testadas em Rust (cargo test):
// aqui um proxy em Node as FINGE na frente do `blueprint serve` real (que serve a interface desta pasta por BLUEPRINT_ASSETS_DIR); as operações de editar rodam no mesmo
// web/js/apresentacao-ia-ops.js. Uso: node scripts/apresentacao-ia-e2e.mjs [blueprint.exe]   (padrão: target/release/blueprint.exe; precisa do Microsoft Edge). Capturas em target/apresentacao-ia-*.png.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer, request } from "node:http";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import vm from "node:vm";
import { dormir, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const BP = "bp1a2b3";
const novoDoc = () => ({
  conteudo: { versao: 1, publico: "cliente", nivel: "padrao", idioma: "pt-BR", titulo: "Plano do cliente", resumo: ["Uma frase."], entregas: [{ titulo: "Atendimento", texto: "no WhatsApp" }],
    etapas: [{ id: "e1", titulo: "Cadastro", inicio: "2026-10-26", fim: "2026-10-27", texto: "Base de usuários.", figura: "capa", marcos: [] }, { id: "e2", titulo: "Canal", inicio: "2026-10-28", fim: "2026-10-29", texto: "Mensagens.", figura: null, marcos: [] }],
    precisamosDeVoce: [], decisoes: [], riscos: [] },
  figuras: [{ id: "capa", titulo: "Mesa", dica: "d", leitura: "l", chaves: [{ marca: "lit", rotulo: "acesa" }], legenda: "leg", rotulo: "r", impressao: "x", codigo: "function (L) { return { svg: '<svg></svg>' }; }" }],
});
const metas = () => [
  { id: "ap-antiga", titulo: "Plano antigo", publico: "cliente", nivel: "rapido", em: "2026-10-20T10:00:00Z" },
  { id: "ap-nova", titulo: DOC.conteudo.titulo, publico: DOC.conteudo.publico, nivel: DOC.conteudo.nivel, em: "2026-10-26T09:00:00Z", ...(editadoEm ? { editadoEm } : {}) },
];
let DOC = novoDoc(), editadoEm = "", removidas = new Set(), SESSOES = [], recusaProxima = null;
const posts = { editar: [], pedido: [], terminal: [], apresentar: [], remover: [] };
const opsCtx = vm.createContext({ tr: (t) => t });
vm.runInContext(readFileSync(join(raiz, "web/js/apresentacao-ia-ops.js"), "utf8") + "\nthis.aplica = apOpsAplica;", opsCtx);

const [porta, portaProxy, depuracao] = [await portaLivre(), await portaLivre(), await portaLivre()];
const tmp = mkdtempSync(join(tmpdir(), "blueprint-ap-ia-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: raiz, BLUEPRINT_SEM_ATUALIZACAO: "1" }, stdio: "ignore", windowsHide: true });
const lerCorpo = (req) => new Promise((ok) => { let t = ""; req.on("data", (c) => (t += c)); req.on("end", () => ok(t)); });
const proxy = createServer(async (req, res) => {
  const url = new URL(req.url, "http://x"), rota = url.pathname;
  const json = (obj, status = 200) => { res.writeHead(status, { "content-type": "application/json" }); res.end(JSON.stringify(obj)); };
  const corpo = async () => JSON.parse(await lerCorpo(req));
  if (req.method === "GET" && rota === "/api/mcp/apresentacoes") return json({ apresentacoes: metas().filter((a) => !removidas.has(a.id)).sort((a, b) => (a.em < b.em ? 1 : -1)) });
  if (req.method === "GET" && rota === "/api/mcp/apresentacao") return json({ id: "ap-nova", ...metas()[1], ...DOC });
  if (req.method === "GET" && rota === "/api/mcp/sessoes") return json({ sessoes: SESSOES });
  if (req.method === "POST" && rota === "/api/mcp/apresentacao/editar") {
    const c = await corpo(); posts.editar.push(c);
    if (recusaProxima) { const r = recusaProxima; recusaProxima = null; return json(r, 400); }
    const novo = JSON.parse(JSON.stringify(DOC));
    try { for (const op of c.operacoes || []) opsCtx.aplica(novo, op); if (c.figuras) novo.figuras = c.figuras; } catch (e) { return json({ erro: "op", codigo: e.codigo || "operacao" }, 400); }
    if (!String(novo.conteudo.titulo || "").trim()) return json({ erro: "x", codigo: "falta-titulo" }, 400);
    DOC = novo; editadoEm = new Date().toISOString();
    return json({ ok: true, apresentacaoId: "ap-nova", editadoEm, avisos: JSON.stringify(novo.conteudo).includes("webhook") ? ['Technical term "webhook" in a client presentation'] : [] });
  }
  if (req.method === "POST" && rota === "/api/mcp/apresentacao/pedido") { posts.pedido.push(await corpo()); return json({ ok: true }); }
  if (req.method === "POST" && rota === "/api/mcp/apresentar") { posts.apresentar.push(await corpo()); return json({ ok: true }); }
  if (req.method === "POST" && rota === "/api/terminal/iniciar") { posts.terminal.push(await corpo()); return json({ sessao: { id: "t" + posts.terminal.length, titulo: "Claude" } }); }
  if (req.method === "POST" && rota === "/api/mcp/apresentacao/remover") { const c = await corpo(); posts.remover.push(c); removidas.add(c.apresentacaoId); return json({ ok: true }); }
  if (/^\/apresentacao\/([\w-]+)$/.test(rota)) { // a página de mentira, com os mesmos cabeçalhos de isolamento que a real tem
    res.writeHead(200, { "content-type": "text/html; charset=utf-8", "content-security-policy": "sandbox allow-scripts; default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:" });
    return res.end(`<!doctype html><meta charset="utf-8"><title>${rota}</title><body style="background:#0c3a78;color:#eaf4ff;font:16px monospace"><h1>apresentação falsa</h1>`);
  }
  const alvo = request({ host: "127.0.0.1", port: porta, path: req.url, method: req.method, headers: { ...req.headers, host: `127.0.0.1:${porta}`, ...(req.headers.origin ? { origin: `http://127.0.0.1:${porta}` } : {}) } }, (r) => { res.writeHead(r.statusCode, r.headers); r.pipe(res); });
  alvo.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); }); // ao desligar, o SSE do servidor cai no meio
  req.pipe(alvo);
});
await new Promise((ok) => proxy.listen(portaProxy, "127.0.0.1", ok));
const url = `http://127.0.0.1:${portaProxy}`;
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} proxy.close(); setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(`http://127.0.0.1:${porta}/api/dados`); break; } catch { await dormir(50); } }
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  const espera = async (expr, quanto = 8000) => { for (let i = 0; i < quanto / 100; i++) { if (await js(expr)) return true; await dormir(100); } return false; };
  const foto = async (nome) => { mkdirSync(join(raiz, "target"), { recursive: true }); writeFileSync(join(raiz, "target", `apresentacao-ia-${nome}.png`), Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); };
  const Q = (sel) => JSON.stringify(sel);
  const clica = (sel) => js(`document.querySelector(${Q(sel)}).click()`);
  const texto = (sel) => js(`document.querySelector(${Q(sel)})?.textContent || ""`);
  const valor = (sel) => js(`document.querySelector(${Q(sel)})?.value`);
  const digita = (sel, v, so = "input") => js(`(() => { const e = document.querySelector(${Q(sel)}); e.value = ${JSON.stringify(v)}; e.dispatchEvent(new Event(${Q(so)}, { bubbles: true })); if (${Q(so)} === "input") e.dispatchEvent(new Event("change", { bubbles: true })); })()`);
  const salvo = () => espera(`document.querySelector(".ap-ed-estado")?.dataset.estado === "salvo"`, 6000);
  const radios = (grupo) => js(`[...document.querySelectorAll('input[name="ap-ia-${grupo}"]')].map((i) => i.value + (i.checked ? "*" : ""))`);
  await cdp("Runtime.enable");
  await cdp("Page.enable");

  const abre = async (extra = "", esperaAtiva = ".ap-ia-quadro") => { // abre o blueprint e a tela Apresentação
    await cdp("Page.navigate", { url: `${url}/${extra}#/p/${BP}` });
    assert(await espera(`!!document.getElementById("b-apresentar") && typeof D !== "undefined" && D.blueprints?.length > 0`), "o blueprint abriu");
    await dormir(500);
    await clica("#b-apresentar");
    assert(await espera(`!!document.querySelector("#apresentacao-tela:not([hidden]) .ap-ia") && ${esperaAtiva ? `!!document.querySelector(${Q(esperaAtiva)})` : "true"}`), `a tela abriu (${esperaAtiva})`);
    await js(`window.terminalAbrirSessao = (s) => { window.__termAberto = s; }`); // o painel do terminal é testado em terminal-e2e; aqui só se confere que o app o abriria
  };

  // ---- a tela abre na apresentação mais nova; a Visão interna fica recolhida; o grupo Gerar com IA segue no trilho ----
  await abre();
  assert.equal(await js(`document.querySelector(".ap-ia-quadro").getAttribute("src")`), "/apresentacao/ap-nova", "abre na mais nova");
  assert.equal(await js(`document.querySelector(".ap-ia-quadro").getAttribute("sandbox")`), "allow-scripts", "iframe isolado: só allow-scripts, nunca allow-same-origin");
  assert.equal(await js(`document.querySelector("details.ap-interna").open`), false, "a Visão interna começa recolhida quando há apresentação gerada");
  assert.equal(await js(`document.querySelector(".ap-topo > .ap-acoes").hidden`), true, "os botões do topo são das páginas do app: somem com o visor");
  assert.deepEqual(await js(`[...document.querySelectorAll(".ap-ia-item b")].map((b) => b.textContent)`), ["Plano do cliente", "Plano antigo"], "a lista vem da mais nova para a mais antiga");
  assert.deepEqual(await radios("publico"), ["cliente*", "interno"]);
  assert.deepEqual(await radios("nivel"), ["rapido", "padrao*", "completo"], "sem Épico");
  const grupo = await texto(".ap-ia");
  for (const t of ["Gerar com IA", "Uma figura da IA na visão geral", "Uma figura diferente por sessão da página", "máquina densa do sistema e uma figura por etapa", "até 1 h", "1 a 3 h", "2 a 4 h"]) assert(grupo.includes(t), `o grupo mostra «${t}»`);
  assert(!/pico|original|torre/i.test(grupo), "nem Épico nem figura original");
  const barra = await texto(".ap-ia-barra");
  for (const t of ["Editar", "Conversar", "Apresentar", "Imprimir", "Exportar HTML", "Gerar de novo", "Remover"]) assert(barra.includes(t), `a barra do visor tem «${t}»`);
  assert.equal(await js(`document.querySelector('.ap-ia-barra a[download]').getAttribute("href")`), "/apresentacao/ap-nova?baixar=1");
  // abrir outra grava a escolha: da próxima vez a tela abre nela
  await clica('[data-ia-ver="ap-antiga"]');
  await abre();
  assert.equal(await js(`document.querySelector(".ap-ia-quadro").getAttribute("src")`), "/apresentacao/ap-antiga", "abre na última que o usuário viu");
  await clica('[data-ia-ver="ap-nova"]');
  // a Visão interna: abrir mostra os tipos de página do app e o preview de sempre
  await clica("details.ap-interna > summary");
  await clica('.ap-tipo[data-tipo="andamento"]');
  assert(await espera(`!document.querySelector(".ap-ia-quadro") && !!document.querySelector(".ap-pagina") && document.querySelector(".ap-topo > .ap-acoes").hidden === false`), "a Visão interna mostra as páginas do app e devolve os botões do topo");
  await clica('[data-ia-ver="ap-nova"]');
  assert(await espera(`!!document.querySelector(".ap-ia-quadro")`));

  // ---- Editar: campo de texto, lista (adicionar, subir, remover), Sessão da figura, código, aviso do servidor, Desfazer ----
  await js(`window.__vai = []; window.apIaVai = (d) => { __vai.push(d); }`); // espia o pedido de rolar do visor ("ap-ir"): o iframe é de outra origem e não dá para ler dentro dele
  await clica('[data-ia="aba-editar"]');
  assert(await espera(`!!document.querySelector(".ap-ed-secoes [data-c='titulo']")`), "o editor abriu com o documento do servidor");
  assert.equal(await valor('[data-c="titulo"]'), "Plano do cliente");
  assert.equal(await js(`document.querySelector('[data-ia="aba-editar"]').getAttribute("aria-pressed")`), "true");
  await digita('[data-c="titulo"]', "Plano revisado");
  assert(await salvo(), "salvou sozinho");
  assert.deepEqual(posts.editar[0].operacoes, [{ op: "definir", caminho: "titulo", valor: "Plano revisado" }], "UMA operação por mudança, nunca o documento inteiro");
  assert.equal(posts.editar[0].autor, "usuario");
  assert(/^Salvo/.test(await texto(".ap-ed-estado")));
  assert(await espera(`/\\?v=1#s-visao$/.test(document.querySelector(".ap-ia-quadro").getAttribute("src"))`, 4000), "o visor recarregou na seção editada");
  assert.equal(await texto(".ap-ia-nome b"), "Plano do cliente", "o título da barra segue a lista, que o servidor refaz");
  // digitar várias letras seguidas manda uma operação só
  for (const v of ["Pl", "Pla", "Plano final"]) await js(`(() => { const e = document.querySelector('[data-c="titulo"]'); e.value = ${JSON.stringify(v)}; e.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  assert(await salvo());
  assert.deepEqual(posts.editar.at(-1).operacoes, [{ op: "definir", caminho: "titulo", valor: "Plano final" }], "as digitações do mesmo campo viram uma");
  // o título não pode ficar vazio: nada vai ao servidor e o campo fica marcado
  const antes = posts.editar.length;
  await digita('[data-c="titulo"]', "");
  await dormir(1200);
  assert.equal(posts.editar.length, antes, "título vazio não é enviado");
  assert.equal(await js(`document.querySelector('[data-c="titulo"]').getAttribute("aria-invalid")`), "true");
  await digita('[data-c="titulo"]', "Plano final");
  assert(await salvo());
  // lista: adicionar etapa, editar, subir, remover
  await clica('.ap-ed-sec[data-sec="etapas"] > summary');
  await js(`document.querySelector('[data-c="etapas.1.titulo"]').dispatchEvent(new FocusEvent("focusin", { bubbles: true }))`); // o Edge sem janela não dá foco de verdade à página: o evento que o foco dispara é o que o editor escuta
  assert.deepEqual(await js(`__vai.at(-1)`), { secao: "etapas", etapa: 2 }, "focar um campo da etapa 2 manda o visor rolar até ela (ap-ir)");
  await clica('[data-ed="adiciona"][data-caminho="etapas"]');
  assert(await espera(`!!document.querySelector('[data-c="etapas.2.titulo"]')`), "a etapa nova aparece");
  assert(await salvo());
  assert.deepEqual(posts.editar.at(-1).operacoes.map((o) => [o.op, o.caminho, o.valor.id]), [["inserir", "etapas", "e3"]], "o item novo vem do esquema, com id que não repete");
  await digita('[data-c="etapas.2.titulo"]', "Painel");
  await digita('[data-c="etapas.2.inicio"]', "2026-11-02", "change");
  assert(await salvo());
  assert.equal(DOC.conteudo.etapas[2].titulo, "Painel");
  assert.equal(DOC.conteudo.etapas[2].inicio, "2026-11-02");
  await clica('[data-ed="sobe"][data-caminho="etapas.2"]');
  assert(await salvo());
  assert.deepEqual(DOC.conteudo.etapas.map((e) => e.titulo), ["Cadastro", "Painel", "Canal"], "subiu uma casa");
  assert(await espera(`document.querySelector('[data-c="etapas.1.titulo"]').value === "Painel"`), "o painel redesenhou na nova ordem");
  await clica('[data-ed="remove"][data-caminho="etapas.1"]');
  assert(await salvo());
  assert.deepEqual(DOC.conteudo.etapas.map((e) => e.titulo), ["Cadastro", "Canal"], "removeu");
  // Sessão da figura (lista fechada com "nenhuma") e o aviso do servidor ao lado
  await clica('.ap-ed-sec[data-sec="figuras"] > summary');
  const secoes = await js(`[...document.querySelectorAll('[data-fig="capa"][data-campo="secao"] option')].map((o) => o.value)`);
  assert.deepEqual(secoes, ["", "visao", "recebe", "etapas", "linha", "precisamos", "decisoes", "riscos", "acoes"], "Sessão: nenhuma e os ids do contrato");
  await digita('[data-fig="capa"][data-campo="secao"]', "linha", "change");
  assert(await salvo());
  assert.deepEqual(posts.editar.at(-1).operacoes, [{ op: "figura_definir", figura: "capa", campo: "secao", valor: "linha" }]);
  assert(await espera(`/s-linha$/.test(document.querySelector(".ap-ia-quadro").getAttribute("src"))`, 4000), "o visor recarrega na sessão da figura");
  await digita('[data-c="resumo.0"]', "Frase com webhook");
  assert(await salvo());
  assert(await espera(`/webhook/.test(document.querySelector(".ap-ed-avisos").textContent)`), "os avisos do servidor aparecem no painel");
  // código: a sintaxe se confere sem executar; o texto vai como troca do trecho inteiro
  await js(`document.querySelector('[data-figura="capa"] details.ap-ed-avancado').open = true`);
  await digita('[data-fig="capa"][data-campo="codigo"]', "function (L) { return {", "input");
  assert(await espera(`/Erro de sintaxe/.test(document.querySelector(".ap-ed-sintaxe").textContent)`), "erro de sintaxe mostrado");
  await digita('[data-fig="capa"][data-campo="codigo"]', "function (L) { return { svg: '<svg id=\"novo\"></svg>' }; }");
  assert(await salvo());
  assert(/Sintaxe válida/.test(await texto(".ap-ed-sintaxe")));
  const trecho = posts.editar.at(-1).operacoes.at(-1); // as duas trocas (a com erro de sintaxe e a certa) foram no mesmo lote
  assert.equal(trecho.op, "figura_trecho");
  assert(trecho.para.includes('id="novo"') && DOC.figuras[0].codigo.includes('id="novo"'), "troca o código inteiro como um trecho");
  // duplicar figura vai como a lista inteira; Desfazer devolve
  await clica('[data-ed="figura-duplica"][data-fig="capa"]');
  assert(await salvo());
  assert.deepEqual(DOC.figuras.map((f) => f.id), ["capa", "capa-2"]);
  assert(Array.isArray(posts.editar.at(-1).figuras) && !posts.editar.at(-1).operacoes, "duplicar troca a lista de figuras");
  await clica('[data-ed="desfaz"]');
  assert(await salvo());
  assert.deepEqual(DOC.figuras.map((f) => f.id), ["capa"], "Desfazer devolve o que era");
  await foto("editor");
  // o servidor recusa: a frase aparece e o painel volta ao último salvo
  recusaProxima = { erro: "x", codigo: "interno-no-cliente" };
  await digita('[data-c="subtitulo"]', "recusado");
  assert(await espera(`/Ações/.test(document.querySelector(".ap-ed-avisos").textContent)`), "o erro vira frase");
  assert(await espera(`document.querySelector('[data-c="subtitulo"]').value === ""`), "o campo volta ao que estava salvo");

  // ---- o agente muda outro ponto enquanto o usuário digita: as duas edições sobrevivem ----
  DOC.conteudo.etapas[1].titulo = "Canal (agente)"; editadoEm = new Date(Date.now() + 5000).toISOString();
  await js(`(() => { const e = document.querySelector('[data-c="cliente"]'); e.value = "Acme"; e.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await js(`apIaRecebe({ blueprintId: "${BP}", apresentacaoId: "ap-nova" })`);
  assert(await espera(`document.querySelector('[data-c="etapas.1.titulo"]')?.value === "Canal (agente)"`, 8000), "a mudança do agente chegou ao editor");
  assert(await salvo());
  assert.equal(DOC.conteudo.cliente, "Acme", "o que o usuário digitou também foi salvo");
  assert.equal(DOC.conteudo.etapas[1].titulo, "Canal (agente)", "e nada do agente foi apagado");
  await clica('[data-ia="aba-editar"]'); // fecha o painel
  assert.equal(await js(`document.querySelector(".ap-ia-lado").hidden`), true);

  // ---- Conversar: pede o ajuste ao agente (terminal do app, depois sessão com canal, que fica lembrada) ----
  await clica('[data-ia="aba-conversa"]');
  assert(/muda só o que você pedir/.test(await texto(".ap-ia-lado")), "o texto de ajuda explica que só muda o que foi pedido");
  await digita(".ap-cv-texto", "  deixe o texto da etapa 2   mais curto ");
  await clica('[data-cv="enviar"]');
  assert(await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`), "enviar abre o diálogo de sessão");
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "app", "sem sessão aberta o padrão é o Terminal do app");
  assert.equal(posts.terminal.length, 0, "nada sai antes de confirmar");
  await clica('dialog.escolher-sessao [data-r="1"]');
  assert(await espera(`window.__termAberto?.id === "t1"`), "abriu uma aba do terminal do app");
  assert.deepEqual(posts.terminal[0], { workspaceId: "sm", blueprintId: BP, apresentacaoId: "ap-nova", ajuste: "deixe o texto da etapa 2 mais curto", titulo: "Ajuste: " + (await js("apIaBp.nome")) });
  assert(await espera(`/Terminal do app aberto/.test(document.querySelector(".ap-cv-hist").textContent)`), "o histórico mostra o resultado");
  SESSOES = [{ id: "s1", pasta: "C:/projeto/atendimento", canal: true }];
  await digita(".ap-cv-texto", "troque a data de fim para 12 de novembro");
  await clica('[data-cv="enviar"]');
  assert(await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`));
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "0", "a sessão com canal já vem marcada");
  await clica('dialog.escolher-sessao [data-r="1"]');
  assert(await espera(`document.querySelectorAll(".ap-cv-hist li").length >= 2 && /enviado à sessão/.test(document.querySelector(".ap-cv-hist").textContent)`));
  assert.deepEqual(posts.pedido[0], { workspaceId: "sm", blueprintId: BP, apresentacaoId: "ap-nova", texto: "troque a data de fim para 12 de novembro", sessaoId: "s1" });
  await digita(".ap-cv-texto", "mais um ajuste");
  await clica('[data-cv="enviar"]');
  assert(await espera(`document.querySelectorAll(".ap-cv-hist li").length >= 3`), "a sessão lembrada não pergunta de novo");
  assert.equal(await js(`!!document.querySelector("dialog.escolher-sessao[open]")`), false);
  assert.equal(posts.pedido.length, 2);
  await clica('[data-cv="enviar"]'); // vazio
  assert(/Escreva o que você quer mudar/.test(await texto(".ap-ia-lado .ap-ia-msg")));
  // o agente respondeu mudando a apresentação: o visor recarrega e o histórico avisa
  await js(`apIaRecebe({ blueprintId: "${BP}", apresentacaoId: "ap-nova" })`);
  assert(await espera(`/O agente atualizou a apresentação/.test(document.querySelector(".ap-cv-hist").textContent)`), "o histórico diz que o agente atualizou");
  await clica('[data-ia="aba-conversa"]');

  // ---- Gerar: no terminal do app (padrão), com público e nível; Gerar de novo avisa das edições ----
  SESSOES = [];
  await clica('input[name="ap-ia-publico"][value="interno"]');
  await clica('input[name="ap-ia-nivel"][value="completo"]');
  await clica('[data-ia="gerar"]');
  assert(await espera(`!!document.querySelector("dialog.escolher-sessao[open]")`), "Gerar abre o diálogo");
  const dialogo = await texto("dialog.escolher-sessao");
  assert(dialogo.includes("Gasta tokens.") && dialogo.includes("Público: Interno. Nível: Completo (2 a 4 h)."), `o diálogo diz o que vai gastar: ${dialogo.slice(0, 160)}`);
  assert(dialogo.includes("Terminal do app"), "a opção Terminal do app existe");
  assert.equal(await js(`document.querySelector('dialog.escolher-sessao input[name="sessao"]:checked').value`), "app", "e é o padrão");
  assert.equal(posts.terminal.length, 1, "nada sai antes de confirmar");
  await clica('dialog.escolher-sessao [data-r="1"]');
  assert(await espera(`window.__termAberto?.id === "t2"`), "o terminal do app abre com a conversa de gerar");
  assert.deepEqual(posts.terminal[1], { workspaceId: "sm", blueprintId: BP, apresentar: { publico: "interno", nivel: "completo" }, titulo: "Apresentação: " + (await js("apIaBp.nome")) });
  assert(/Responda ali o que ele perguntar/.test(await texto(".ap-ia-gerar-grupo .ap-ia-msg")));
  assert.equal(posts.apresentar.length, 0, "não usou o caminho do terminal externo");
  await clica('[data-ia="refazer"]'); // a apresentação tem edições à mão: pergunta antes
  assert(await espera(`!!document.querySelector("dialog.confirmar[open]")`), "Gerar de novo pede confirmação por causa das edições");
  assert(/substitui as suas edições/.test(await texto("dialog.confirmar")));
  await clica('dialog.confirmar [data-r="0"]');
  await dormir(300);
  assert.equal(posts.terminal.length, 2, "cancelar não gera");
  assert.equal(await js(`!!document.querySelector("dialog.escolher-sessao[open]")`), false);

  // ---- remover; sem apresentações a tela abre como antes (Gerar com IA primeiro, Visão interna aberta) ----
  await clica('[data-ia="remover"]');
  assert(await espera(`!!document.querySelector("dialog.confirmar[open]")`), "remover pede confirmação");
  await clica('dialog.confirmar [data-r="1"]');
  assert(await espera(`document.querySelector(".ap-ia-quadro")?.getAttribute("src") === "/apresentacao/ap-antiga"`), "removida a aberta, abre a seguinte");
  assert.deepEqual(posts.remover, [{ workspaceId: "sm", blueprintId: BP, apresentacaoId: "ap-nova" }]);
  removidas.add("ap-antiga");
  await abre("", ".ap-pagina");
  assert.equal(await js(`document.querySelector("details.ap-interna").open`), true, "sem apresentações a Visão interna abre");
  assert.equal(await js(`document.querySelector(".ap-tipos").firstElementChild.classList.contains("ap-ia")`), true, "o grupo Gerar com IA vem primeiro");
  assert.equal(await js(`document.querySelector(".ap-topo > .ap-acoes").hidden`), false);
  removidas = new Set(); DOC = novoDoc(); editadoEm = "";

  // ---- 390 px: sem rolagem horizontal, com o painel de editar aberto o visor cede o lugar ----
  await abre();
  await clica('[data-ia="aba-editar"]');
  assert(await espera(`!!document.querySelector(".ap-ed-secoes [data-c='titulo']")`));
  await cdp("Emulation.setDeviceMetricsOverride", { width: 390, height: 844, deviceScaleFactor: 1, mobile: false });
  await dormir(500);
  assert.equal(await js(`document.documentElement.scrollWidth <= 390 && document.getElementById("apresentacao-tela").scrollWidth <= 390 && document.querySelector(".ap-ia-lado").scrollWidth <= 390`), true, "o editor cabe a 390 px sem rolagem horizontal");
  assert.equal(await js(`getComputedStyle(document.querySelector(".ap-ia-quadro")).display`), "none", "em tela estreita o painel fica no lugar do visor");
  await foto("mobile");
  await clica('[data-ia="aba-editar"]');
  await dormir(200);
  assert.equal(await js(`document.documentElement.scrollWidth <= 390 && document.querySelector(".ap-ia-barra").scrollWidth <= 390`), true, "o visor também cabe");
  await cdp("Emulation.clearDeviceMetricsOverride");
  await foto("desktop");

  // ---- inglês ----
  await abre("?idioma=en");
  await clica('[data-ia="aba-editar"]');
  assert(await espera(`!!document.querySelector(".ap-ed-secoes [data-c='titulo']")`));
  const en = await texto(".ap-ia-lado") + await texto(".ap-ia-barra") + await texto(".ap-ia");
  for (const t of ["Generate with AI", "Edit", "Talk", "Export HTML", "Generate again", "Remove", "Undo", "General", "Stages", "Figures", "A different figure for each section of the page", "Internal view"]) assert(en.includes(t) || await js(`document.body.textContent.includes(${JSON.stringify(t)})`), `em inglês aparece «${t}»`);
  assert.equal(await texto('[data-ia="gerar"]'), "Generate");
  assert.equal(await js(`[...document.querySelectorAll('[data-fig="capa"][data-campo="secao"] option')].map((o) => o.textContent).slice(0, 3).join("|")`), "None|Overview|What you receive", "a Sessão da figura traduzida");
  assert.deepEqual(erros.filter((e) => !/NumberOfInputEventTimestampsToTrack/.test(e)), [], "sem erro no console"); // esse identificador é do próprio Edge (script interno que ele reinjeta ao navegar), não está no repositório
  console.log("apresentacao-ia-e2e: ok (abre na mais nova, Visão interna, editor, Sessão da figura, código, Desfazer, erro do servidor, edição do agente em paralelo, conversa, gerar no terminal do app, Gerar de novo com edições, remover, 390 px, en)");
} finally { limpar(); setTimeout(() => process.exit(), 800); } // o WebSocket do Edge e o SSE pelo proxy prendiam o Node depois do "ok"
