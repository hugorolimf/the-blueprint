// Convite e entrada ponta a ponta (modo hospedado): o dono entra pelo link impresso, cria um link de colaborador na tela, a pessoa entra pelo
// nome, gera a chave de MCP, aparece na lista e o dono a remove; o link não serve duas vezes e o segredo nunca chega ao servidor.
//   node scripts/convite-tela-e2e.mjs                         contra o stub (scripts/stub-colaboracao.mjs)       (~20 s)
//   node scripts/convite-tela-e2e.mjs --rust <blueprint.exe>  contra o binário real (serve --hospedar), com o stub de proxy
import assert from "node:assert/strict";
import { abrirNavegador, dormir, servidorDoTeste } from "./colaboracao-navegador.mjs";

const S = await servidorDoTeste();
const pessoas = [];
const abrir = async (nome) => { const p = await abrirNavegador(nome); pessoas.push(p); return p; };
const segredoDe = (link) => link.split("#")[1];
const segredos = [segredoDe(S.linkDono)];
try {
  // 1. o dono abre o link impresso: a página tira o segredo da barra, explica o papel e pede o nome
  const ana = await abrir("ana");
  await ana.ir(S.linkDono);
  await ana.ate(`!document.getElementById("convite-campos").hidden`, 8000, "a entrada pede o nome");
  assert.equal(await ana.js(`location.pathname + location.hash`), "/convite", "o segredo saiu da barra (replaceState)");
  assert.match(await ana.js(`document.getElementById("convite-papel").textContent`), /^Dono:/, "explica o papel");
  assert.equal(await ana.js(`getComputedStyle(document.body).backgroundColor !== "" && document.getElementById("login").offsetParent === null`), true, "só a entrada aparece (o login local fica escondido)");
  await ana.js(`document.getElementById("convite-form").requestSubmit()`);
  assert.match(await ana.js(`document.getElementById("convite-erro").textContent`), /Digite seu nome/, "nome vazio é recusado na própria página");
  await ana.js(`document.getElementById("convite-nome").value = "Ana"; document.getElementById("convite-form").requestSubmit()`);
  await ana.ate(`location.pathname === "/" && !location.hash.includes("convite") && typeof sessao !== "undefined" && sessao.modo === "servidor"`, 10000, "entrou (o dono do servidor cai na lista de workspaces)");
  await ana.ate(`document.getElementById("b-pessoas").offsetParent !== null && document.getElementById("perfil-nome").textContent === "Ana"`, 8000, "lista de workspaces com o nome da sessão e o botão Pessoas visível");
  await ana.js(`location.hash = "/c/sm"`); // abre o workspace raiz, como o clique no cartão
  assert.equal(await ana.js(`document.getElementById("perfil-sair").textContent`), "Sair", "no modo hospedado o botão é Sair");
  assert.equal(await ana.js(`history.length <= 3 && !location.href.includes(${JSON.stringify(segredos[0])})`), true, "o convite não ficou na URL");

  // 2. o painel Pessoas (botão visível na barra da folha) abre por import() e lista a dona; ela cria um link de colaborador
  await ana.ate(`document.getElementById("b-pessoas-folha").offsetParent !== null`, 5000, "botão Pessoas visível na folha do workspace");
  await ana.clicar("#b-pessoas-folha");
  await ana.ate(`document.getElementById("equipe").open && document.querySelectorAll("#eq-pessoas li").length === 1`, 8000, "painel com a dona");
  assert.match(await ana.js(`document.querySelector("#eq-pessoas li").textContent`), /Ana.*\(você\)/s);
  await ana.ate(`!!document.getElementById("eq-conv-form")`, 5000, "formulário de convite (só o dono)");
  await ana.escolher(`#eq-conv-form [name=papel]`, "colaborador");
  assert.match(await ana.js(`document.getElementById("eq-conv-dica").textContent`), /^Colaborador:/, "o formulário explica o papel escolhido");
  await ana.js(`document.getElementById("eq-conv-form").requestSubmit()`);
  await ana.ate(`!document.getElementById("eq-link").hidden`, 8000, "o link aparece uma vez");
  const link = await ana.js(`document.getElementById("eq-link-texto").textContent`);
  assert.match(link, /\/convite#[A-Za-z0-9_-]{20,}$/, "link com o segredo no fragmento");
  segredos.push(segredoDe(link));
  await ana.ate(`document.querySelectorAll("#eq-conv-lista [data-revogar]").length === 1`, 5000, "o link novo está na lista, ativo");
  assert.equal(await ana.js(`document.getElementById("eq-conv-lista").textContent.includes(${JSON.stringify(segredos[1])})`), false, "a lista não mostra o segredo");

  // 3. a pessoa entra pelo nome, vê o próprio papel e gera a chave de MCP
  const bruno = await abrir("bruno");
  await bruno.ir(`${S.url}/convite?web#${segredos[1]}`);
  await bruno.ate(`!document.getElementById("convite-campos").hidden`, 8000, "Bruno vê a entrada");
  assert.match(await bruno.js(`document.getElementById("convite-papel").textContent`), /^Colaborador:/);
  await bruno.js(`document.getElementById("convite-nome").value = "Bruno"; document.getElementById("convite-form").requestSubmit()`);
  await bruno.ate(`location.hash.startsWith("#/c/") && sessao.modo === "servidor" && !document.getElementById("b-chave").hidden`, 10000, "Bruno entrou");
  assert.equal(await bruno.js(`document.getElementById("b-pessoas").hidden`), true, "colaborador não administra pessoas");
  await bruno.clicar("#b-chave");
  await bruno.ate(`/Você ainda não tem chave/.test(document.getElementById("eq-chave-status")?.textContent || "")`, 8000, "tela da chave");
  await bruno.clicar("#eq-chave-gerar");
  await bruno.ate(`!document.getElementById("eq-chave-nova").hidden`, 8000, "chave gerada");
  const chave = await bruno.js(`document.querySelector("#eq-chave-nova code").textContent`);
  assert.match(chave, /^bpk_/, "chave com prefixo bpk_");
  const cmds = await bruno.js(`[...document.querySelectorAll("#eq-chave-nova code")].slice(1).map((c) => c.textContent)`);
  assert.match(cmds[0], /^blueprint conectar https?:\/\/\S+ bpk_\S+$/, "o comando único (blueprint conectar)");
  assert.match(cmds[1], /^claude mcp add .*BLUEPRINT_CHAVE=bpk_.* -- blueprint --mcp$/, "comando do Claude Code");
  assert.match(cmds[2], /^codex mcp add .*BLUEPRINT_URL=https?:\/\/.* -- blueprint --mcp$/, "comando do Codex");
  await bruno.ate(`/Chave bpk_/.test(document.getElementById("eq-chave-status").textContent) && document.getElementById("eq-chave-gerar").textContent === "Trocar chave"`, 5000, "depois de gerar, só o prefixo");
  await bruno.js(`document.getElementById("equipe").close()`);
  await bruno.ate(`document.getElementById("eq-chave-nova").textContent === ""`, 3000, "fechar o painel apaga a chave da tela"); // o evento close do <dialog> chega numa tarefa depois

  // 4. o dono vê o Bruno na lista e o remove
  await ana.js(`document.getElementById("equipe").close()`);
  await ana.clicar("#b-pessoas");
  await ana.ate(`document.querySelectorAll("#eq-pessoas li").length === 2`, 8000, "Bruno aparece na lista");
  assert.match(await ana.js(`document.getElementById("eq-pessoas").textContent`), /Bruno.*workspace inteiro/s);
  await ana.clicar(`#eq-pessoas li:not(:first-child) [data-remover]`);
  await dormir(200);
  await ana.confirmar();
  await ana.ate(`document.querySelectorAll("#eq-pessoas li").length === 1 && /Bruno foi removido/.test(document.getElementById("eq-pessoas-status").textContent)`, 8000, "Bruno saiu da lista");

  // 5. Bruno perde a sessão: ao recarregar, a página mostra a entrada pelo convite; o link de uso único não serve de novo
  await bruno.ir(`${S.url}/`, 1500);
  await bruno.ate(`!document.getElementById("convite").hidden && /Entre pelo link de convite/.test(document.getElementById("convite-titulo").textContent)`, 8000, "sem sessão: tela de entrada");
  await bruno.ir(`${S.url}/convite?web#${segredos[1]}`);
  await bruno.ate(`/Convite indisponível/.test(document.getElementById("convite-titulo").textContent)`, 8000, "link já usado: indisponível");
  const sair = await ana.api("/api/sessao/sair", {});
  assert.equal(sair.status, 200, "sair encerra a sessão");
  assert.equal((await ana.api("/api/sessao")).status, 401, "depois de sair: 401 sem-sessao");

  // 6. o segredo nunca viajou na URL nem no Referer (só no corpo dos POST)
  const pedidos = await (await fetch(S.url + "/__stub/pedidos")).json();
  for (const s of segredos) assert.equal(pedidos.some((p) => p.caminho.includes(s) || p.referer.includes(s)), false, "segredo fora de URL e Referer");
  for (const p of pessoas) assert.deepEqual(p.erros, [], `sem erro no console (${p.nome})`);
  console.log(`convite e2e: ok (${process.argv.includes("--rust") ? "binário real" : "stub"}: entrada pelo nome, link criado e copiado uma vez, chave de MCP, remover, sem sessão, uso único, segredo fora da URL)`);
} finally {
  for (const p of pessoas) p.fechar();
  S.fechar();
}
