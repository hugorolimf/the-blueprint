// Fluxo completo: o agente envia o registro pelo MCP (registrar_trabalho, via `blueprint --mcp`), e confere no navegador (Edge sem janela) a tela Planejar com um registro do que o agente já fez fora do plano: os checks, o que fica pendente,
// o Adicionar (grava o feito e a evidência como comentário do plano e recalcula o progresso) e o Descartar (não muda nada). Banco de exemplo, sem recompilar.
// Uso: node scripts/registro-tela-e2e.mjs [exe] [pasta-com-web]   (precisa do Microsoft Edge; serve web/ da pasta dada, por padrão a do repositório)
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
const tmp = mkdtempSync(join(tmpdir(), "blueprint-registro-"));
copyFileSync(join(raiz, "data.exemplo.json"), join(tmp, "dados.json"));
const servidor = spawn(exe, ["serve"], { env: { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(tmp, "blueprint.db"), DADOS_ARQ: join(tmp, "dados.json"), PROTOTIPOS_DIR: join(tmp, "p"), BLUEPRINT_ASSETS_DIR: assets }, stdio: "ignore", windowsHide: true });
const edge = spawn(edgeExe, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${depuracao}`, `--user-data-dir=${join(tmp, "perfil")}`, "--window-size=1440,1000", "about:blank"], { stdio: "ignore" });
const limpar = () => { for (const p of [edge, servidor]) try { p.kill(); } catch {} setTimeout(() => { try { rmSync(tmp, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); };
process.on("exit", limpar);
try {
  for (let i = 0; i < 100; i++) { try { await fetch(url + "/api/dados"); break; } catch { await dormir(50); } }
  const dados = await (await fetch(url + "/api/dados")).json();
  const blueprint = dados.blueprints.find((p) => p.acoes?.length);
  let alvo;
  for (let i = 0; i < 100 && !alvo; i++) { try { alvo = (await (await fetch(`http://127.0.0.1:${depuracao}/json/list`)).json()).find((t) => t.type === "page"); } catch {} await dormir(100); }
  const ws = new WebSocket(alvo.webSocketDebuggerUrl);
  await new Promise((r) => (ws.onopen = r));
  let n = 0; const pend = new Map(); const erros = [];
  ws.onmessage = (m) => { const d = JSON.parse(m.data); if (d.method === "Runtime.exceptionThrown") erros.push(d.params.exceptionDetails.exception?.description || d.params.exceptionDetails.text); pend.get(d.id)?.(d); pend.delete(d.id); };
  const cdp = (method, params = {}) => new Promise((r) => { const id = ++n; pend.set(id, r); ws.send(JSON.stringify({ id, method, params })); });
  const js = async (expr) => { const r = await cdp("Runtime.evaluate", { expression: expr, returnByValue: true, awaitPromise: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
  await cdp("Runtime.enable");

  // o que o agente mandou: uma ação nova com um plano feito (com evidência), um plano pendente e um checkpoint pendente
  const NOME = "Ajuste fora do plano", EVIDENCIA = "node teste.mjs: 12 passam, 0 falham";
  // o agente chama a ferramenta registrar_trabalho pelo `blueprint --mcp` (stdio), como o Claude faria; devolve { isError, texto }
  const registrar = async (nome, mensagem) => {
    const filho = spawn(exe, ["--mcp"], { env: { ...process.env, BLUEPRINT_URL: url }, stdio: ["pipe", "pipe", "ignore"], windowsHide: true });
    let buf = "", id = 0; const esp = new Map();
    filho.stdout.on("data", (d) => { buf += d; for (let i; (i = buf.indexOf("\n")) >= 0;) { const l = buf.slice(0, i).trim(); buf = buf.slice(i + 1); if (l) { const m = JSON.parse(l); esp.get(m.id)?.(m); } } });
    const rpc = (method, params) => new Promise((ok) => { esp.set(++id, ok); filho.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
    try {
      await rpc("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "registro-e2e", version: "1" } });
      const r = await rpc("tools/call", { name: "registrar_trabalho", arguments: { workspace_id: "sm", blueprint_id: blueprint.id, mensagem, acoes: [{ nome, inicio: "2026-10-20", fim: "2026-10-23", planos: [
        { texto: "Corrigir o botão", descricao: "Escopo: o menu. Validação: o teste passa.", feito: true, evidencia: EVIDENCIA },
        { texto: "Cobrir com teste", descricao: "Escopo: o menu. Validação: roda." }], checkpoints: [{ nome: "Revisão", data: "2026-10-22" }] }] } });
      return { isError: !!(r.error || r.result?.isError), texto: r.result?.content?.map((x) => x.text).join("\n") ?? JSON.stringify(r.error) };
    } finally { filho.kill(); }
  };
  const painel = () => js(`(() => { const b = document.getElementById("cl-reg"); return { oculto: b.hidden, texto: b.textContent, checks: [...b.querySelectorAll("input[data-rc=plano],input[data-rc=cp]")].map((i) => ({ tipo: i.dataset.rc, marcado: i.checked })), botoes: [...b.querySelectorAll("[data-ract]")].map((x) => x.textContent.trim()) }; })()`);
  const acaoNova = async () => (await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).acoes.find((f) => f.nome === NOME);
  const fila = async () => (await (await fetch(url + "/api/mcp/fila")).json()).itens.filter((i) => i.tipo === "registro");

  await cdp("Page.navigate", { url: `${url}/#/p/${blueprint.id}` });
  await dormir(2500);
  const total = (await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).acoes.length;
  const r1 = await registrar(NOME, "Corrigi o menu fora do plano.");
  assert.equal(r1.isError, false, r1.texto);
  assert.match(r1.texto, /Registro nº 1/i, "a resposta ao agente diz o número do registro e que ele deve perguntar ao usuário");
  await dormir(1200);
  await js(`document.getElementById("b-ia").click()`); // abre o Planejar: o registro aparece no painel acima da proposta
  await dormir(1500);

  let p = await painel();
  assert.equal(p.oculto, false, "o painel do registro aparece na tela Planejar");
  assert.match(p.texto, /Registro nº 1/, "o registro é numerado");
  assert.match(p.texto, /Registro/, "com o crachá Registro");
  assert.doesNotMatch(p.texto, /proposta|aplicar/i, "registro nunca se chama proposta nem pede para aplicar");
  assert.deepEqual(p.checks, [{ tipo: "plano", marcado: true }, { tipo: "plano", marcado: false }, { tipo: "cp", marcado: false }], "o feito vem marcado; o pendente e o checkpoint, abertos");
  assert.deepEqual(p.botoes, ["Adicionar ao blueprint · 1 feito", "Descartar"]);
  assert.equal((await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).acoes.length, total, "chegar não muda os dados: só Adicionar muda");

  // os checks são editáveis: desmarca e marca de volta o feito; marca e desmarca o pendente
  await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="0"]').click()`); await dormir(200);
  assert.equal(await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="0"]').checked`), false, "o feito pode ser desmarcado");
  await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="0"]').click()`); await dormir(200);
  await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="1"]').click()`); await dormir(200);
  assert.equal(await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="1"]').checked`), true, "o pendente pode ser marcado");
  await js(`document.querySelector('#cl-reg input[data-rc=plano][data-j="1"]').click()`); await dormir(200);
  assert.deepEqual((await painel()).checks.map((c) => c.marcado), [true, false, false], "e volta ao que o agente mandou");

  // Adicionar: o feito e a evidência viram do blueprint; o pendente e o checkpoint entram abertos
  await js(`document.querySelector('#cl-reg [data-ract=adicionar]').click()`);
  await dormir(1500);
  assert.equal((await painel()).oculto, true, "adicionado: o painel some");
  const f = await acaoNova();
  assert(f, "a ação nova entrou no blueprint");
  const [feito, pendente] = f.planos;
  assert.equal(feito.feito, true, "o plano feito ficou feito");
  assert.equal(pendente.feito, false, "o plano pendente ficou aberto");
  assert.equal(f.checkpoints[0].feito, false, "o checkpoint pendente ficou aberto");
  assert.notEqual(f.status, "concluido", "com itens abertos, a ação não conclui");
  assert(feito.agente?.comentarios.some((c) => c.texto === EVIDENCIA && c.tipo === "relatorio"), "a evidência virou comentário do plano");
  assert.equal(feito.agente.estado, "concluido", "o plano concluído pelo registro fica concluído");
  assert.deepEqual(await js(`efetiva(D.blueprints.find((x) => x.id === ${JSON.stringify(blueprint.id)}).acoes.find((x) => x.nome === ${JSON.stringify(NOME)})).it`), [1, 3], "o progresso é recalculado: 1 de 3");
  assert.equal((await fila()).length, 0, "o registro saiu da fila do servidor (ack)");

  // Descartar: o registro seguinte tem número próprio e, descartado, não muda o blueprint
  assert.equal((await registrar("Outra coisa", "Outro registro.")).isError, false);
  await dormir(1200);
  assert.match((await painel()).texto, /Registro nº 2/, "a numeração dos registros é própria");
  const antes = (await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).acoes.length;
  await js(`document.querySelector('#cl-reg [data-ract=descartar]').click()`);
  await dormir(1200);
  assert.equal((await painel()).oculto, true, "descartado: o painel some");
  assert.equal((await fila()).length, 0, "descartar também tira o registro da fila");
  assert.equal((await (await fetch(url + "/api/dados")).json()).blueprints.find((x) => x.id === blueprint.id).acoes.length, antes, "descartar não adiciona nada");
  assert.deepEqual(erros, [], "nenhum erro de script na página");
  console.log("registro-tela-e2e: ok");
} finally {
  limpar();
}
