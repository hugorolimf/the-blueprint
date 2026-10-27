// O caminho do iniciante, de ponta a ponta, num banco VAZIO de um servidor temporário (nunca o banco vivo): o nome na entrada, a tela vazia
// com os 3 caminhos, "Conectar seu agente" (o comando --mcp), "Abrir o exemplo" (workspace novo, a raiz intocada), o agente pelo
// `blueprint --mcp` (listar e propor um blueprint novo), aplicar a proposta na tela Planejar e "Remover exemplo" (os dados voltam sem ele).
// Uso: node scripts/primeiro-uso-e2e.mjs [exe] [pasta-com-web]   (precisa do Microsoft Edge; serve web/ da pasta dada, sem recompilar; ~5 s)
// PRIMEIRO_USO_CAPTURA=<pasta> grava capturas da entrada vazia e da linha do exemplo.
import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { exePadrao, ligarServidor, mcpStdio, portaLivre, raiz } from "./lib-e2e-rust.mjs";

const t0 = Date.now();
const exe = exePadrao();
const assets = resolve(process.argv[3] || raiz);
const dir = mkdtempSync(join(tmpdir(), "blueprint-primeiro-uso-"));
const porta = await portaLivre();
const url = `http://127.0.0.1:${porta}`;
// sem data.json para importar: o banco nasce vazio, como na primeira abertura do app instalado
const env = { ...process.env, HOST: "127.0.0.1", PORT: String(porta), BLUEPRINT_DB: join(dir, "blueprint.db"), DADOS_ARQ: join(dir, "nao-existe.json"), PROTOTIPOS_DIR: join(dir, "prototipos"), BLUEPRINT_URL: url, BLUEPRINT_ASSETS_DIR: assets, BLUEPRINT_SEM_ATUALIZACAO: "1" };
const filhos = [];
let mcp = null, p = null;
const limpar = () => { mcp?.fechar(); for (const p of filhos) try { p.kill(); } catch {} setTimeout(() => { try { rmSync(dir, { recursive: true, force: true, maxRetries: 5, retryDelay: 300 }); } catch {} }, 500); };
process.on("exit", limpar);
const dados = async () => (await fetch(url + "/api/dados")).json();

try {
  filhos.push(await ligarServidor(exe, { env, url }));
  const inicio = await dados();
  assert.equal((inicio.blueprints || []).length + (inicio.workspaces || []).length, 0, "o banco começa vazio");
  assert.equal((await fetch(url + "/api/exemplo")).status, 200, "o servidor tem a rota do exemplo (exe atual?)");
  p = await janela({ dir, url }, "iniciante", "/", filhos);
  const captura = async (nome) => { if (process.env.PRIMEIRO_USO_CAPTURA) await new Promise((r) => setTimeout(r, 2500)), writeFileSync(join(process.env.PRIMEIRO_USO_CAPTURA, `primeiro-uso-${nome}.png`), Buffer.from((await p.cdp("Page.captureScreenshot", { format: "png" })).result.data, "base64")); };
  const vis = (sel) => `(() => { const e = document.querySelector(${JSON.stringify(sel)}); return !!e && !e.closest("[hidden]") && e.getClientRects().length > 0; })()`;

  // 1. o nome, e a entrada vazia mostra os 3 caminhos (nunca uma tela em branco)
  await espera(() => p.js(vis("#login-nome") + " && typeof carregado !== \"undefined\" && carregado"), "o primeiro uso pede o nome", 15000);
  await p.js(`document.getElementById("login-nome").value = "Pessoa Nova"; document.getElementById("login-form").requestSubmit(); true`);
  await espera(() => p.js(vis("#primeiro-uso")), "a entrada vazia mostra os primeiros passos", 10000);
  assert.deepEqual(await p.js(`[...document.querySelectorAll("#primeiro-uso [data-pu]")].map((b) => b.dataset.pu)`), ["exemplo", "agente", "criar"]);
  await captura("vazia");

  // 2. Conectar seu agente: o Planejar de um blueprint novo e o "Como conectar" com o comando do MCP
  await p.js(`document.querySelector("#primeiro-uso [data-pu=agente]").click(); true`);
  await espera(() => p.js(`document.getElementById("como-usar").open && document.getElementById("cu-corpo").textContent.includes("--mcp")`), "Conectar mostra o comando --mcp");
  assert.equal(await p.js(`!document.getElementById("agente-tela").hidden`), true, "com o Planejar aberto atrás");
  await p.js(`document.getElementById("cu-fechar").click(); location.hash = ""; true`);
  await espera(() => p.js(vis("#primeiro-uso [data-pu=exemplo]")), "voltar à entrada mostra de novo os 3 caminhos");

  // 3. Abrir o exemplo: um workspace novo, separado; a raiz (com o nome gravado) não muda
  const antes = await dados();
  await p.js(`document.querySelector("#primeiro-uso [data-pu=exemplo]").click(); true`);
  await espera(() => p.js(`location.hash.startsWith("#/c/") && document.querySelectorAll("#desenho [data-goto]").length > 0`), "o exemplo abre com os blueprints dele", 10000);
  const comExemplo = await dados();
  const ex = comExemplo.workspaces.find((w) => w.exemplo === true);
  assert(ex && ex.blueprints.length >= 2, "o exemplo é um workspace com os blueprints fictícios");
  assert.equal(await p.js(`decodeURIComponent(location.hash.slice(4))`), ex.id);
  assert.deepEqual(comExemplo.blueprints, antes.blueprints, "a raiz continua igual");
  assert.deepEqual(comExemplo.usuario, antes.usuario, "e o nome também");
  await p.js(`location.hash = ""; true`);
  await espera(() => p.js(vis("#primeiro-uso [data-pu=remover]") + ` && !document.querySelector("#primeiro-uso [data-pu=exemplo]")`), "com dados, os 3 caminhos somem e fica o Remover exemplo");
  await captura("exemplo");

  // 4. o agente pelo MCP (`blueprint --mcp`): vê o exemplo e propõe um blueprint novo no workspace raiz
  mcp = mcpStdio(exe, env);
  await mcp.iniciar();
  const lista = await mcp.chamar("listar_blueprints", {});
  assert(!lista.erro && lista.texto.includes(ex.blueprints[0].nome), "o agente vê o blueprint do exemplo: " + lista.texto.slice(0, 200));
  const hoje = await p.js(`hoje`), depois = await p.js(`deDia(dia(hoje) + 2)`);
  const proposta = await mcp.chamar("propor_blueprint", { workspace_id: "sm", mensagem: "O primeiro plano.", blueprint: { nome: "Meu primeiro blueprint", descricao: "Criado pelo agente no primeiro uso." },
    acoes: [{ nome: "Primeira entrega", inicio: hoje, fim: depois, observacao: "Objetivo: chegar à primeira entrega.", planos: [{ texto: "Escrever o primeiro plano", descricao: "Escopo: um plano\nResultado esperado: um plano escrito\nValidação: aparece na folha" }] }] });
  assert(!proposta.erro, proposta.texto);

  // 5. aplicar na tela Planejar do workspace raiz (proposta de blueprint novo espera o clique)
  await p.js(`location.hash = "#/c/sm"; true`);
  await espera(() => p.js(vis("#b-ia")), "a folha do workspace raiz abre");
  await p.js(`document.getElementById("b-ia").click(); true`);
  await espera(() => p.js(`!!document.querySelector("#ag-outras [data-ep]")`), "o Planejar avisa da proposta de blueprint novo", 10000);
  await p.js(`document.querySelector("#ag-outras [data-ep]").click(); true`);
  await espera(() => p.js(`(() => { const b = document.querySelector("#agente-tela [data-pact=aplicar]"); return !!b && !b.disabled; })()`), "a proposta tem o Aplicar");
  await p.js(`document.querySelector("#agente-tela [data-pact=aplicar]").click(); true`);
  await espera(async () => (await dados()).blueprints.some((b) => b.nome === "Meu primeiro blueprint"), "a proposta aplicada vira um blueprint", 10000);
  const aplicado = (await dados()).blueprints.find((b) => b.nome === "Meu primeiro blueprint");
  assert.equal(aplicado.acoes[0]?.planos[0]?.texto, "Escrever o primeiro plano");
  const ler = await mcp.chamar("ler_blueprint", { workspace_id: "sm", blueprint_id: aplicado.id });
  assert(!ler.erro && ler.texto.includes("Primeira entrega"), "o agente lê o que foi aplicado");

  // 6. Remover exemplo: some só o workspace do exemplo; o que é do usuário fica
  await p.js(`location.hash = ""; true`);
  await espera(() => p.js(vis("#primeiro-uso [data-pu=remover]")), "o Remover exemplo continua na entrada");
  const comTudo = await dados();
  await p.js(`document.querySelector("#primeiro-uso [data-pu=remover]").click(); true`);
  await espera(() => p.js(`!!document.querySelector("dialog.confirmar [data-r='1']")`), "remover pede confirmação");
  await p.js(`document.querySelector("dialog.confirmar [data-r='1']").click(); true`);
  await espera(async () => !(await dados()).workspaces.some((w) => w.exemplo === true), "o exemplo saiu do banco");
  const fim = await dados();
  assert.deepEqual(fim.blueprints, comTudo.blueprints, "os blueprints do usuário ficam iguais");
  assert.deepEqual(fim.workspaces, comTudo.workspaces.filter((w) => w.id !== ex.id), "e os outros workspaces também");
  await espera(() => p.js(`document.getElementById("primeiro-uso").hidden`), "sem exemplo e com blueprint, nada de primeiros passos");
  assert.deepEqual(p.erros, [], "nenhum erro de script na página");
  const s = ((Date.now() - t0) / 1000).toFixed(1);
  assert(Date.now() - t0 < 60000, `o caminho do iniciante passa de 1 minuto (${s} s)`);
  console.log(`primeiro-uso-e2e: ok (${s} s)`);
  process.exit(0);
} catch (e) {
  console.error(e);
  // o estado da página ajuda a achar o passo que falhou
  if (p) console.error(await p.js(`JSON.stringify({ hash: location.hash, corpo: document.body.className, pu: document.getElementById("primeiro-uso")?.outerHTML.slice(0, 300), status: document.getElementById("entrada-status")?.textContent })`).catch(() => ""), p.erros);
  process.exit(1);
}
