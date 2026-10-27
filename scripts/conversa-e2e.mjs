// Comentário do plano e conversa com o plano, com o servidor REAL e nenhum Claude (docs/claude-na-folha.md, "Enviar comentário do plano e conversar com o plano"):
// no Edge (sem janela), uma sessão aberta na pasta (hook) e um canal FALSO (o SSE que o `blueprint --mcp` abre). O comentário vai ao canal como pedido do usuário;
// a mensagem da conversa vai com o prefixo da conversa; o "agente" é o próprio `blueprint --mcp` por stdio (propor_blueprint e responder_conversa), a página
// mostra a resposta e o botão abre a proposta citada no Planejar, sem aplicar nada. Os textos da página vêm de web/ (BLUEPRINT_ASSETS_DIR), sem recompilar.
// Uso: node scripts/conversa-e2e.mjs [blueprint.exe]   (padrão: target/debug/blueprint.exe; precisa do Microsoft Edge). ~15 s.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { dormir, portaLivre, raiz, sobeServidor, terminalFalso } from "./terminal-falso.mjs";

const exe = resolve(process.argv[2] || join(raiz, "target/debug/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const [BP, ACAO, PLANO] = ["bp1a2b3", "ac1a002", "1eaygiu"]; // data.exemplo.json: ação 1.2 «API de pedidos», plano «Criar o endpoint de listar pedidos»
const dir = mkdtempSync(join(tmpdir(), "blueprint-conversa-"));
const pasta = join(dir, "projeto");
mkdirSync(pasta);
const falso = terminalFalso(dir), depuracao = await portaLivre();
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(dir, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
let servidor, agente;
const abortar = new AbortController();
const limpar = () => { abortar.abort(); try { edge.kill(); } catch {} try { agente?.kill(); } catch {} servidor?.parar(); setTimeout(() => { try { rmSync(dir, { recursive: true, force: true }); } catch {} }, 500); };
process.on("exit", limpar);
const post = (rota, json) => fetch(`${servidor.url}/api/mcp/${rota}`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(json) });

// o "agente": o MCP por stdio, como o Claude o chamaria
function mcpStdio(url) {
  agente = spawn(exe, ["--mcp"], { env: { ...process.env, BLUEPRINT_URL: url }, stdio: ["pipe", "pipe", "ignore"], windowsHide: true });
  let buffer = "", n = 0; const pend = new Map();
  agente.stdout.on("data", (d) => { buffer += d; for (let i; (i = buffer.indexOf("\n")) >= 0;) { const l = buffer.slice(0, i).trim(); buffer = buffer.slice(i + 1); if (l) { const m = JSON.parse(l); pend.get(m.id)?.(m); } } });
  const rpc = (method, params) => new Promise((ok) => { const id = ++n; pend.set(id, ok); agente.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
  return async (nome, args) => { const r = await rpc("tools/call", { name: nome, arguments: args }); return { erro: !!(r.result?.isError || r.error), texto: r.result?.content?.map((x) => x.text).join("\n") ?? JSON.stringify(r.error) }; };
}

try {
  servidor = await sobeServidor(exe, dir, falso, { BLUEPRINT_ASSETS_DIR: raiz });
  assert.equal((await post("pasta/vincular", { cwd: pasta, workspaceId: "sm", blueprintId: BP })).status, 200);
  assert.equal((await post("hook/sessao", { session_id: "s1", cwd: pasta })).status, 200);
  let recebido = "";
  const sse = await fetch(`${servidor.url}/api/mcp/eventos?canal=abc123&cwd=${encodeURIComponent(pasta)}`, { signal: abortar.signal });
  (async () => { const dec = new TextDecoder(); try { for await (const parte of sse.body) recebido += dec.decode(parte, { stream: true }); } catch {} })();
  const evento = async (nome) => {
    for (let i = 0; i < 60 && !recebido.includes(`event: ${nome}\n`); i++) await dormir(100);
    assert(recebido.includes(`event: ${nome}\n`), `o canal recebeu ${nome}`);
    return JSON.parse(recebido.split(`event: ${nome}\ndata: `)[1].split("\n")[0]);
  };

  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 300)); return r.result.result.value; };
  const espera = async (expr, msg) => { for (let i = 0; i < 60 && !(await js(expr)); i++) await dormir(150); assert(await js(expr), msg); };
  await cdp("Runtime.enable");
  await cdp("Browser.grantPermissions", { origin: servidor.url, permissions: ["clipboardReadWrite", "clipboardSanitizedWrite"] });
  await cdp("Page.enable");
  await cdp("Page.addScriptToEvaluateOnNewDocument", { source: `try{localStorage.setItem("blueprint.autoAplicar.sm:${BP}","0")}catch{}` }); // este teste revisa e aplica à mão: o aplicar sozinho (auto-aplicar.js) fica desligado
  await cdp("Page.navigate", { url: `${servidor.url}/#/p/${BP}` });
  await espera(`!!document.querySelector('.chk[data-tipo="t"]') || typeof enviarComentarioAoAgente === "function" && D.blueprints.length > 0`, "a folha abriu");

  // 1. o botão direito no plano oferece "Enviar comentário ao agente…"; o comentário vai ao canal como pedido do usuário, nada grava
  // "nada grava": o texto do usuário não chega ao banco e o plano proposto não entra no blueprint (a página guarda o rascunho da proposta em `chats`, como sempre)
  const planosDoBanco = async () => JSON.stringify((await (await fetch(`${servidor.url}/api/dados`)).json()).blueprints);
  await js(`enviarComentarioAoAgente(${JSON.stringify(BP)}, ${JSON.stringify(ACAO)}, ${JSON.stringify(PLANO)})`);
  await espera(`!!document.querySelector("dialog.comentario-agente[open]")`, "o diálogo do comentário abriu");
  assert.deepEqual(await js(`[...document.querySelectorAll('dialog.comentario-agente input[name=sessao]')].map((r) => r.value + (r.checked ? "*" : ""))`), ["0*", "copiar"], "a sessão com canal vem marcada");
  await js(`document.querySelector('dialog.comentario-agente [data-r="1"]').click()`);
  assert.match(await js(`document.querySelector("dialog.comentario-agente .cv-aviso").textContent`), /Escreva o comentário/, "sem texto não envia");
  await js(`document.querySelector("dialog.comentario-agente textarea").value = "Use paginação de 20 itens"; document.querySelector('dialog.comentario-agente [data-r="1"]').click()`);
  const comentario = await evento("comentario-plano");
  assert.equal(comentario.canalId, "abc123");
  assert(comentario.texto.includes(`plano_id ${PLANO}`) && comentario.texto.endsWith("Use paginação de 20 itens"), comentario.texto);
  await espera(`/Enviado à sessão/.test(document.querySelector("dialog.comentario-agente .cv-aviso").textContent)`, "o diálogo mostra o envio");
  await js(`document.querySelector('dialog.comentario-agente [data-r="0"]').click()`);
  assert(!(await (await fetch(`${servidor.url}/api/dados`)).text()).includes("Use paginação de 20 itens"), "o comentário não grava nos dados");
  // o botão direito: um elemento solto com os mesmos atributos da folha (a seção da ação pode estar recolhida)
  const menu = (classe, extra) => js(`(() => { const el = document.createElement("div"); el.className = "${classe}"; Object.assign(el.dataset, { p: "${BP}", f: "${ACAO}", ...${JSON.stringify(extra)} }); return itensDoContexto(el).itens.map((i) => i.rotulo); })()`);
  assert((await menu("chk", { id: PLANO, tipo: "t" })).includes("Enviar comentário ao agente…"), "o menu do plano oferece o comentário");
  assert(!(await menu("chk", { id: "qualquer", tipo: "m" })).includes("Enviar comentário ao agente…"), "checkpoint não tem comentário");
  assert((await menu("acao", {})).includes("Conversar com o plano…"), "o menu da ação oferece a conversa");

  // 2. o painel da conversa: a mensagem vai com o prefixo da conversa e entra no histórico
  await js(`abreFolhaDaAcao(${JSON.stringify(BP)}, ${JSON.stringify(ACAO)}, document.querySelector("#desenho"))`); // a folha da ação ganha o botão, ao lado de "Iniciar com o Claude"
  await espera(`[...document.querySelectorAll("#nota-folha [data-nacao]")].some((b) => b.textContent === "Conversar com o plano")`, "a folha da ação oferece a conversa");
  await js(`[...document.querySelectorAll("#nota-folha [data-nacao]")].find((b) => b.textContent === "Conversar com o plano").click()`);
  await espera(`document.querySelector("#conversa")?.hidden === false && /Nenhuma mensagem/.test(document.querySelector("#conversa .cv-lista").textContent)`, "o painel abriu vazio");
  assert.equal(await js(`document.querySelector("#nota-folha").hidden || !!document.querySelector("#nota-folha").dataset.fechando`), true, "a folha da ação fecha ao abrir o painel");
  assert.equal(await js(`document.querySelector("#conversa [data-cv-sessao]").value`), "0", "a sessão com canal vem escolhida");
  await js(`document.querySelector("#conversa textarea").value = "Divida o plano de listar pedidos em dois"; document.querySelector('#conversa [type="submit"]').click()`);
  const fala = await evento("conversa-usuario");
  assert(fala.texto.startsWith("Ação 1.2 «") && fala.texto.includes(`acao_id ${ACAO}`) && fala.texto.endsWith("Divida o plano de listar pedidos em dois"), fala.texto);
  await espera(`document.querySelectorAll("#conversa .cv-msg.usuario").length === 1`, "a mensagem do usuário aparece no histórico");

  // 3. o agente propõe e responde pelo MCP; a página mostra a resposta com o botão da proposta, e o botão abre o Planejar nela
  const chamar = mcpStdio(servidor.url);
  const proposta = await chamar("propor_blueprint", { workspace_id: "sm", blueprint_id: BP, mensagem: "Dividir o plano", acoes: [{ id: ACAO, planos: [{ texto: "Criar o endpoint de detalhar pedido", descricao: "Escopo: GET /pedidos/:id. Resultado esperado: 200 com o pedido. Validação: teste da rota.", horas: 2 }] }] });
  assert(!proposta.erro, proposta.texto);
  const numero = Number(/Proposal no\. (\d+)/.exec(proposta.texto)[1]);
  const fora = await chamar("responder_conversa", { workspace_id: "sm", blueprint_id: BP, acao_id: ACAO, texto: "x", proposta: numero + 50 });
  assert(fora.erro && /is not in this blueprint's queue/.test(fora.texto), `proposta que não existe é recusada: ${fora.texto}`);
  const resposta = await chamar("responder_conversa", { workspace_id: "sm", blueprint_id: BP, acao_id: ACAO, texto: `Mandei a proposta nº ${numero} com o plano novo.`, proposta: numero });
  assert(!resposta.erro, resposta.texto);
  await espera(`!!document.querySelector("#conversa .cv-msg.agente [data-cv-proposta]")`, "a resposta aparece com o botão da proposta");
  assert.match(await js(`document.querySelector("#conversa .cv-msg.agente").textContent`), new RegExp(`Mandei a proposta nº ${numero}`));
  if (process.env.CAPTURA) (await import("node:fs")).writeFileSync(process.env.CAPTURA, Buffer.from((await cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); // conferir o painel a olho
  await espera(`filaTodas(conv(${JSON.stringify(BP)})).some((x) => x.numero === ${numero})`, "a proposta chegou à fila da página");
  await js(`document.querySelector("#conversa [data-cv-proposta]").click()`);
  await espera(`!hub.hidden && conv(${JSON.stringify(BP)}).numero === ${numero}`, "o Planejar abriu na proposta citada");
  assert.equal(await js(`document.querySelector("#conversa").hidden`), true, "o painel fecha ao abrir o Planejar");
  assert(!(await planosDoBanco()).includes("Criar o endpoint de detalhar pedido"), "nada foi aplicado: a proposta espera o clique do usuário");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("comentário e conversa com o plano (servidor real, canal e agente falsos): ok (comentário pelo canal, conversa com prefixo, resposta pelo MCP, proposta citada aberta no Planejar, nada gravado)");
} finally { limpar(); }
