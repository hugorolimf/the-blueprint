// Confere no navegador de verdade (Edge sem janela) os dois caminhos que gravam dados pela interface: a tela Editar (acrescentar e renomear um plano) e a
// tela Planejar (uma proposta do agente chega pelo MCP e é aplicada), mais a conclusão de um plano pelo agente. Confere o que o servidor guardou.
// Uso: node scripts/ui-e2e.mjs [dados.json]   (precisa do Microsoft Edge; usa uma cópia dos dados)
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const raiz = resolve(import.meta.dirname, "..");
const dadosArq = resolve(process.argv[2] || join(raiz, "data.exemplo.json"));
const exe = resolve(process.env.BLUEPRINT_EXE || join(raiz, "target/release/blueprint.exe"));
const edgeExe = ["C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe", "C:/Program Files/Microsoft/Edge/Application/msedge.exe"].find(existsSync);
assert(edgeExe, "o teste precisa do Microsoft Edge");
const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const livre = () => new Promise((ok) => { const l = createServer(); l.listen(0, "127.0.0.1", () => { const p = l.address().port; l.close(() => ok(p)); }); });
const [porta, depuracao] = [await livre(), await livre()];
const url = `http://127.0.0.1:${porta}`;
const tmp = mkdtempSync(join(tmpdir(), "blueprint-ui-"));
copyFileSync(dadosArq, join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p") }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--lang=pt-BR", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => rmSync(tmp, { recursive: true, force: true }), 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const lido = async () => await (await fetch(url + "/api/dados", { cache: "no-store" })).json();
  const dados = await lido();
  const achaBlueprint = (d, id) => [d, ...(d.workspaces || [])].flatMap((w) => w.blueprints || []).find((p) => p.id === id);
  const blueprint = dados.blueprints.find((p) => p.acoes?.some((f) => f.planos?.length && f.planos.some((t) => !t.feito)));
  const acao = blueprint.acoes.find((f) => f.planos?.some((t) => !t.feito));
  const pendente = acao.planos.find((t) => !t.feito);
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

  // 1) Editar: acrescenta um plano à ação e renomeia
  await js(`editar(true)`);
  await dormir(800);
  await js(`document.querySelector('.mn-item[data-sel=${JSON.stringify(acao.id)}]').click()`);
  await dormir(500);
  await js(`document.querySelector('[data-act="add-plano"]').click()`);
  await dormir(500);
  await js(`(() => { const campos = [...document.querySelectorAll('.item-ed [data-c="texto"]')]; const c = campos.at(-1); c.value = "Plano criado pelo teste"; c.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await dormir(2500);
  let gravado = achaBlueprint(await lido(), blueprint.id).acoes.find((f) => f.id === acao.id);
  assert(gravado.planos.some((t) => t.texto === "Plano criado pelo teste"), "o plano novo foi gravado no servidor pela tela Editar");
  assert.equal(gravado.planos.length, acao.planos.length + 1);
  console.log("Editar: o plano novo foi gravado");

  // 2) Planejar: o agente propõe uma ação nova pelo MCP, a tela mostra e o usuário aplica
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  await js(`document.getElementById("b-ia").click()`);
  await dormir(1000);
  const quando = (d) => new Date(Date.now() + d * 864e5).toISOString().slice(0, 10);
  const proposta = { workspaceId: dados.workspaces?.find((w) => w.blueprints?.some((p) => p.id === blueprint.id))?.id || "sm", blueprintId: blueprint.id, mensagem: "Proposta do teste", acoes: [{ nome: "Ação do teste", inicio: quando(1), fim: quando(10), planos: [1, 2, 3].map((k) => ({ texto: `Criar item ${k}`, descricao: "Escopo: x. Resultado: y. Validação: z.", horas: k })), checkpoints: [{ nome: "Pronto", data: quando(9) }] }] };
  const r = await fetch(url + "/api/mcp/proposta", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(proposta) });
  assert.equal(r.status, 200, await r.clone().text());
  await dormir(1500);
  const painel = await js(`document.getElementById("cl-prop").textContent`);
  // a ação chega em modo de leitura (htmlVistaAcao); só vira campo de texto ao clicar em editar
  assert(painel.includes("1 ação") && painel.includes("Ação do teste"), "a proposta aparece na tela Planejar: " + painel.slice(0, 300) + " | " + (await js(`String(workspaceAtual?.id) + " " + hub.hidden`)));
  await js(`document.querySelector('[data-pact="aplicar"]').click()`);
  await dormir(2500);
  gravado = achaBlueprint(await lido(), blueprint.id);
  const nova = gravado.acoes.find((f) => f.nome === "Ação do teste");
  assert(nova && nova.planos.length === 3 && nova.checkpoints.length === 1, "a ação proposta foi aplicada e gravada");
  assert.deepEqual(nova.planos.map((x) => x.horas), [1, 2, 3], "as horas estimadas pelo agente foram gravadas em cada plano");
  console.log("Planejar: a proposta do agente foi aplicada (1 ação, 3 planos, 1 checkpoint)");

  // Editar mostra a estimativa de cada plano da ação e aceita editar
  await js(`editar(true)`);
  await dormir(800);
  await js(`document.querySelector('.mn-item[data-sel=${JSON.stringify(nova.id)}]').click()`);
  await dormir(500);
  assert.equal(await js(`[...document.querySelectorAll('.item-ed [data-c="horas"]')].map((i) => i.value).join(",")`), "1,2,3", "a tela Editar mostra as horas de cada plano");
  await js(`(() => { const c = document.querySelector('.item-ed [data-c="horas"]'); c.value = "4"; c.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await dormir(2500);
  assert.equal(achaBlueprint(await lido(), blueprint.id).acoes.find((f) => f.id === nova.id).planos[0].horas, 4, "a hora editada foi gravada");
  console.log("Horas: o agente estima, a tela Editar mostra e edita, o servidor grava");
  // recarregar a página não perde as horas (a página só guarda os campos que conhece)
  await cdp("Page.navigate", { url: "about:blank" });
  await dormir(500);
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(3000);
  await js(`abertos[${JSON.stringify(blueprint.id)}] = true; render()`);
  await dormir(500);
  // desde 28cea79 a folha soma as horas no cabeçalho ("N h de desenvolvimento"), não mostra mais "~N h" em cada plano
  assert(/\d+ h de desenvolvimento/.test(await js(`document.body.textContent`)), "a folha mostra as horas estimadas depois de recarregar");
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(1500);

  // 3) o agente conclui um plano com evidência (e o tempo real do plano é medido: primeiro progresso até a conclusão): a página marca e guarda a evidência como comentário
  const wsId = proposta.workspaceId;
  const p0 = await fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo: "progresso", workspaceId: wsId, blueprintId: blueprint.id, acaoId: acao.id, planoId: pendente.id, texto: "começando" }) });
  assert.equal(p0.status, 200, await p0.clone().text());
  await dormir(2500);
  const iniciado = achaBlueprint(await lido(), blueprint.id).acoes.find((f) => f.id === acao.id).planos.find((t) => t.id === pendente.id);
  assert(iniciado.agente?.inicioEm, "o primeiro progresso com o plano marcou o início do tempo real");
  const c = await fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo: "concluir", workspaceId: wsId, blueprintId: blueprint.id, acaoId: acao.id, planoId: pendente.id, texto: "teste do e2e passou" }) });
  assert.equal(c.status, 200, await c.clone().text());
  await dormir(2500);
  const feito = achaBlueprint(await lido(), blueprint.id).acoes.find((f) => f.id === acao.id).planos.find((t) => t.id === pendente.id);
  assert.equal(feito.feito, true, "o plano concluído pelo agente ficou marcado");
  assert(JSON.stringify(feito.agente?.comentarios || []).includes("teste do e2e passou"), "a evidência virou comentário do plano");
  assert(feito.agente.feitoEm && feito.agente.inicioEm === iniciado.agente.inicioEm, "a conclusão marcou o fim do tempo real e o início continuou o mesmo");
  console.log("MCP: o plano concluído pelo agente ficou marcado, com a evidência como comentário");
  // 4) o agente marcou algo ANTES de a página abrir: ao abrir, a página aplica sem esperar outro aviso (a conexão SSE abre antes de os dados chegarem)
  await cdp("Page.navigate", { url: "about:blank" }); // a página fecha (a conexão SSE cai) antes de o agente marcar
  await dormir(1500);
  const outro = acao.planos.find((t) => !t.feito && t.id !== pendente.id);
  const antes = await fetch(url + "/api/mcp/evento", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ tipo: "concluir", workspaceId: wsId, blueprintId: blueprint.id, acaoId: acao.id, planoId: outro.id, texto: "marcado com a página fechada" }) });
  assert.equal(antes.status, 200, await antes.clone().text());
  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(4000);
  const tardio = achaBlueprint(await lido(), blueprint.id).acoes.find((f) => f.id === acao.id).planos.find((t) => t.id === outro.id);
  assert.equal(tardio.feito, true, "o que o agente marcou com a página fechada foi aplicado ao abrir, sem outro aviso");
  console.log("MCP: o que foi marcado com a página fechada é aplicado ao abrir a página");
  assert.deepEqual(erros, [], "sem erro no console");
  console.log("interface: ok (Editar, Planejar e conclusão pelo agente gravam no servidor)");
} finally { limpar(); }
