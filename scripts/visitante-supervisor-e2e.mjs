// Link público e painel do supervisor ponta a ponta (modo hospedado, dados de exemplo): o dono cria o link de visitante que abre a apresentação,
// o visitante entra sem nome, só lê aquele blueprint (sem comentários do agente) e nunca grava; o supervisor acompanha os blueprints e as
// pessoas num painel só, sem editar; revogar o link derruba o visitante.
//   node scripts/visitante-supervisor-e2e.mjs                         contra o stub                          (~20 s)
//   node scripts/visitante-supervisor-e2e.mjs --rust <blueprint.exe>  contra o binário real (serve --hospedar)
import assert from "node:assert/strict";
import { abrirNavegador, servidorDoTeste } from "./colaboracao-navegador.mjs";

const S = await servidorDoTeste();
const pessoas = [];
const abrir = async (nome) => { const p = await abrirNavegador(nome); pessoas.push(p); return p; };
const entrar = async (p, link, nome) => {
  await p.ir(link);
  if (nome) { await p.ate(`!document.getElementById("convite-campos").hidden`, 8000, `${nome} vê a entrada`); await p.js(`document.getElementById("convite-nome").value = ${JSON.stringify(nome)}; document.getElementById("convite-form").requestSubmit()`); }
  await p.ate(`location.pathname === "/" && typeof sessao !== "undefined" && sessao.modo === "servidor" && carregado`, 10000, `${p.nome} entrou`);
};
const BP = "bp1a2b3"; // "Aplicativo de entregas" em data.exemplo.json (workspace raiz sm), com comentários do agente nos planos
try {
  const ana = await abrir("ana");
  await entrar(ana, S.linkDono, "Ana");

  // 1. o dono cria o link público pela tela: papel visitante, um blueprint, abre a apresentação, sem limite de usos
  await ana.clicar("#b-pessoas");
  await ana.ate(`!!document.getElementById("eq-conv-form")`, 8000, "formulário de convite");
  await ana.escolher(`#eq-conv-form [name=papel]`, "visitante");
  assert.equal(await ana.js(`document.querySelector('#eq-conv-form [data-so="visitante"]').hidden || !document.querySelector('#eq-conv-form [data-so="membro"]').hidden`), false, "visitante: mostra Abre e esconde Quantas pessoas");
  await ana.js(`document.getElementById("eq-conv-form").requestSubmit()`);
  await ana.ate(`/escolha qual no Escopo/.test(document.getElementById("eq-conv-status").textContent)`, 3000, "link público sem blueprint é recusado na tela");
  await ana.escolher(`#eq-conv-form [name=blueprintId]`, BP);
  await ana.escolher(`#eq-conv-form [name=abre]`, "apresentacao");
  await ana.js(`document.getElementById("eq-conv-form").requestSubmit()`);
  await ana.ate(`!document.getElementById("eq-link").hidden`, 8000, "link público criado");
  const publico = await ana.js(`document.getElementById("eq-link-texto").textContent`);
  await ana.ate(`/Link público.*abre a apresentação/s.test(document.getElementById("eq-conv-lista").textContent)`, 5000, "a lista mostra o link público");
  const convitePublico = await ana.js(`document.querySelector("#eq-conv-lista [data-revogar]").closest("li").dataset.convite`);

  // 2. o visitante entra sem nome e cai na apresentação do blueprint
  const vera = await abrir("vera");
  await entrar(vera, `${S.url}/convite#${publico.split("#")[1]}`);
  assert.equal(await vera.js(`location.hash`), `#/p/${BP}`, "destino: o blueprint do link");
  await vera.ate(`!!document.getElementById("apresentacao-tela") && !document.getElementById("apresentacao-tela").hidden`, 8000, "abre a apresentação");
  assert.equal(await vera.js(`document.body.classList.contains("so-leitura") && getComputedStyle(document.getElementById("b-ia")).display === "none"`), true, "só leitura: sem Planejar");
  assert.equal(await vera.js(`["b-pessoas", "b-chave", "b-acompanhar"].every((id) => document.getElementById(id).hidden)`), true, "visitante não vê os painéis de equipe");
  const dados = await vera.api("/api/dados");
  const bps = [...dados.corpo.blueprints, ...(dados.corpo.workspaces || []).flatMap((w) => w.blueprints)];
  assert.deepEqual(bps.map((b) => b.id), [BP], "o visitante lê só o blueprint do link");
  assert.equal(JSON.stringify(dados.corpo).includes('"comentarios"'), false, "sem comentários do agente");
  assert.equal(dados.corpo.usuario, undefined, "sem o perfil de ninguém");
  const put = await vera.api("/api/dados", dados.corpo, "PUT");
  assert.equal(put.status, 403, "o visitante nunca grava");
  assert.equal((await vera.api("/api/convites", { workspaceId: "sm", papel: "visitante", blueprintId: BP })).status, 403, "nem cria convite");
  assert.equal((await vera.api("/api/chave", {})).status, 403, "nem tem chave de MCP");

  // 3. o supervisor entra e acompanha os blueprints e as pessoas num painel só
  const conv = await ana.api("/api/convites", { workspaceId: "sm", papel: "supervisor", validadeHoras: 24, usosMax: 1 });
  assert.equal(conv.status, 201, "dono cria convite de supervisor");
  const carla = await abrir("carla");
  await entrar(carla, `${S.url}/convite#${conv.corpo.link.split("#")[1]}`, "Carla");
  await carla.ate(`!document.getElementById("b-acompanhar").hidden`, 5000, "supervisor vê Acompanhar");
  assert.equal(await carla.js(`document.body.classList.contains("so-leitura")`), true, "supervisor só lê");
  await carla.clicar("#b-acompanhar");
  await carla.ate(`document.querySelectorAll("#equipe-corpo .eq-bp").length >= 2`, 8000, "painel com dois ou mais blueprints");
  const cartoes = await carla.js(`[...document.querySelectorAll("#equipe-corpo .eq-bp")].map((c) => ({ id: c.dataset.bp, texto: c.textContent, pct: c.querySelector("progress").value }))`);
  assert.ok(cartoes.some((c) => c.id === BP) && cartoes.some((c) => c.id === "bp4d5e6"), "os dois blueprints do workspace raiz");
  assert.ok(cartoes.every((c) => /ações concluídas/.test(c.texto) && c.pct >= 0 && c.pct <= 100), "progresso e ações em cada cartão");
  await carla.ate(`/Ana/.test(document.querySelector("#equipe-corpo [data-pessoas]").textContent) && /Carla/.test(document.querySelector("#equipe-corpo [data-pessoas]").textContent)`, 8000, "pessoas com papel e última atividade");
  await carla.ate(`/entrou/.test(document.querySelector("#equipe-corpo [data-atividade]").textContent)`, 8000, "atividade recente");
  assert.equal((await carla.api("/api/convites", { workspaceId: "sm", papel: "colaborador" })).status, 403, "supervisor não convida");
  await carla.clicar(`#equipe-corpo [data-abrir="${BP}"]`);
  await carla.ate(`location.hash === "#/p/${BP}" && !document.getElementById("equipe").open`, 5000, "Abrir leva à folha do blueprint");

  // 4. revogar o link público derruba o visitante
  assert.equal((await ana.api("/api/convites/revogar", { id: convitePublico })).status, 200);
  await vera.cdp("Page.reload"); // a mesma URL: recarrega a página (a sessão dela já não vale)
  await vera.ate(`!document.getElementById("convite").hidden && /Entre pelo link de convite/.test(document.getElementById("convite-titulo").textContent)`, 8000, "visitante sem sessão depois de revogar");

  for (const p of pessoas) assert.deepEqual(p.erros, [], `sem erro no console (${p.nome})`);
  console.log(`visitante e supervisor e2e: ok (${process.argv.includes("--rust") ? "binário real" : "stub"}: link público abre a apresentação, visitante só lê um blueprint sem comentários, supervisor acompanha 2 blueprints e as pessoas, revogar derruba)`);
} finally {
  for (const p of pessoas) p.fechar();
  S.fechar();
}
