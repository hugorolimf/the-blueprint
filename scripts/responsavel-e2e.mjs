// O responsável por plano de ponta a ponta (ação "Responsável por plano: pessoa ou agente"), no modo hospedado: a Dona (dono) e o Bruno (colaborador) entram pelos
// links; no Editar a Dona escolhe o Claude do Bruno num plano e o Bruno vê na hora; o Bruno se atribui outro plano pelo menu do botão direito; o filtro "Meus planos"
// mostra só os dele; o agente do Bruno, pelo MCP por HTTP com a chave dele, lê `voce` e `responsavel` e reconhece o plano como seu; o supervisor não vê o campo;
// no app local (sem membros) o seletor e o filtro ficam escondidos. `blueprint serve --hospedar` com banco temporário atrás do proxy de teste; Edge sem janela.
//   node scripts/responsavel-e2e.mjs [blueprint.exe]   (precisa do Microsoft Edge; ~20 s)
import assert from "node:assert/strict";
import { createServer, request } from "node:http";
import { writeFile } from "node:fs/promises";
import { espera, janela } from "./lib-e2e-edge.mjs";
import { exePadrao, http, pastaDeDados, raiz } from "./lib-e2e-rust.mjs";
import { PUBLICA, entrar, ligarHospedado, navegador } from "./lib-hospedado.mjs";

const DADOS = { usuario: { nome: "Equipe" }, blueprints: [{ id: "p1", nome: "Projeto", acoes: [
  { id: "f1", nome: "Ação um", inicio: "2026-11-01", fim: "2026-11-05", status: "andamento", planos: [{ id: "t1", texto: "Plano um" }, { id: "t2", texto: "Plano dois" }], checkpoints: [] },
  { id: "f9", nome: "Ação nove", inicio: "2026-11-06", fim: "2026-11-09", status: "planejado", planos: [{ id: "t9", texto: "Plano nove" }], checkpoints: [] }] }] };
const pasta = await pastaDeDados("responsavel", { BLUEPRINT_ASSETS_DIR: raiz });
await writeFile(pasta.env.DADOS_ARQ, JSON.stringify(DADOS));
const servidor = await ligarHospedado(exePadrao(), pasta);
// o proxy de teste, mais a Origin: o navegador fala com http://127.0.0.1:<porta>, e o servidor só aceita a origem pública (como atrás do Caddy)
const proxy = await new Promise((ok) => {
  const alvo = new URL(pasta.url);
  const srv = createServer((req, res) => {
    const cab = { ...req.headers, host: "equipe.teste", "x-forwarded-proto": "https", "x-forwarded-for": "127.0.0.1", ...(req.headers.origin ? { origin: PUBLICA } : {}) };
    const ida = request({ hostname: alvo.hostname, port: alvo.port, path: req.url, method: req.method, headers: cab }, (volta) => { res.writeHead(volta.statusCode, volta.headers); volta.pipe(res); });
    ida.on("error", () => { if (!res.headersSent) res.writeHead(502); res.end(); });
    req.pipe(ida);
  });
  srv.listen(0, "127.0.0.1", () => ok({ url: `http://127.0.0.1:${srv.address().port}`, fechar: () => srv.close() }));
});
const filhos = [servidor.proc];
try {
  const link = servidor.saida().match(/https:\/\/equipe\.teste\/convite#[A-Za-z0-9_-]{43}/)[0];
  const dona = await entrar(proxy, link, "Dona");
  const convite = async (papel) => (await navegador(proxy, "POST", "/api/convites", { cookie: dona.cookie, corpo: { workspaceId: "sm", papel } })).json.link;
  const bruno = await entrar(proxy, await convite("colaborador"), "Bruno");
  const sup = await entrar(proxy, await convite("supervisor"), "Sup");
  const pessoas = (await navegador(proxy, "GET", "/api/pessoas?workspace=sm", { cookie: dona.cookie })).json.membros;
  const idBruno = pessoas.find((m) => m.nome === "Bruno").id, idDona = pessoas.find((m) => m.nome === "Dona").id;
  const planoNoServidor = async (id) => (await navegador(proxy, "GET", "/api/dados", { cookie: dona.cookie })).json.blueprints[0].acoes.flatMap((a) => a.planos).find((p) => p.id === id);
  const site = { dir: pasta.dir, url: proxy.url };
  const [A, B, S] = [await janela(site, "dona", "/#/p/p1", filhos, { cookie: dona.cookie }), await janela(site, "bruno", "/#/p/p1", filhos, { cookie: bruno.cookie }), await janela(site, "sup", "/#/p/p1", filhos, { cookie: sup.cookie })];
  for (const [j, comPessoas] of [[A, true], [B, true], [S, false]]) await espera(() => j.js(`typeof aoVivoAberto === "function" && aoVivoAberto() && !!opsAchar(arquivo, "plano", "t1") && (${comPessoas ? "responsavelMembros.length >= 2" : "responsavelMembros.length === 0"})`), "a página carregou, abriu o SSE e trouxe as pessoas (quem só lê não as pede)", 15000);
  const igual = (a, b) => JSON.stringify(Object.entries(a ?? {}).sort()) === JSON.stringify(Object.entries(b ?? {}).sort()); // o servidor devolve as chaves em outra ordem
  const P = (id) => `opsAchar(arquivo, "plano", "${id}").no`;

  // 1. a Dona escolhe o Claude do Bruno no Editar; o Bruno vê ao vivo e o servidor guarda só {id, agente}
  const opcoes = await A.js(`responsavelOpcoes("").map((o) => o.rotulo)`);
  assert.deepEqual(opcoes, ["Sem responsável", "Bruno", "Claude de Bruno", "Dona (você)", "Claude de Dona"], "as pessoas que gravam e o Claude de cada uma");
  await A.js(`editar(true, { proj: "p1", sel: "f1" })`);
  await espera(() => A.js(`!!document.querySelector('.item-ed[data-t="t1"] select[data-c="responsavel"]')`), "o seletor aparece na linha do plano");
  await A.js(`(() => { const s = document.querySelector('.item-ed[data-t="t1"] select[data-c="responsavel"]'); s.value = ${JSON.stringify(idBruno + ":agente")}; s.dispatchEvent(new Event("input", { bubbles: true })); })()`);
  await espera(async () => igual((await planoNoServidor("t1")).responsavel, { id: idBruno, agente: true }), "o responsável chegou ao servidor");
  await espera(async () => igual(await B.js(`${P("t1")}.responsavel`), { id: idBruno, agente: true }), "o Bruno vê o responsável ao vivo");
  assert.equal(await A.js(`document.querySelector('.item-ed[data-t="t1"] select[data-c="responsavel"]').selectedOptions[0].textContent`), "Claude de Bruno");
  await A.js(`editar(false)`);

  // 2. o Bruno se atribui o plano dois pelo menu do botão direito (as mesmas funções que o menu chama)
  const itens = await B.js(`itensDeResponsavel("p1", "f1", "t2").map((i) => i.rotulo)`);
  assert.deepEqual(itens, ["Atribuir a mim", "Atribuir ao meu Claude"]);
  await B.js(`itensDeResponsavel("p1", "f1", "t2")[0].fn()`);
  await espera(async () => igual((await planoNoServidor("t2")).responsavel, { id: idBruno }), "o plano dois ficou com o Bruno");
  assert.deepEqual(await B.js(`itensDeResponsavel("p1", "f1", "t2").map((i) => i.rotulo)`), ["Atribuir ao meu Claude", "Tirar o responsável"]);

  // 3. "Meus planos": só as ações com plano dele e, nelas, só os dele (o botão liga e desliga como Pendentes)
  assert.equal(await B.js(`document.getElementById("periodo-meus").hidden`), false);
  await B.js(`document.getElementById("periodo-meus").click()`);
  await espera(() => B.js(`soMeus["p1"] === true && document.getElementById("periodo-meus").getAttribute("aria-pressed") === "true"`), "o filtro ligou");
  assert.deepEqual(await B.js(`acoesVisiveis(D.blueprints[0]).map((f) => [f.id, f.planos.map((t) => t.id)])`), [["f1", ["t1", "t2"]]], "o Claude do Bruno também conta como dele");
  await B.js(`document.getElementById("periodo-auto").click()`);
  await espera(() => B.js(`!soMeus["p1"]`), "Todo o blueprint tira o filtro");
  assert.equal((await B.js(`acoesVisiveis(D.blueprints[0]).length`)), 2);
  // a Dona não tem plano nenhum: o filtro dela fica vazio
  await A.js(`soMeus["p1"] = true`);
  assert.deepEqual(await A.js(`acoesVisiveis(D.blueprints[0]).length`), 0);

  // 4. o supervisor só lê: sem seletor, sem filtro, sem itens de menu
  assert.equal(await S.js(`responsavelAtivo()`), false);
  assert.equal(await S.js(`document.getElementById("periodo-meus").hidden`), true);
  assert.deepEqual(await S.js(`itensDeResponsavel("p1", "f1", "t2")`), []);
  // e o servidor recusa o supervisor mudar o campo (papel de leitura) e qualquer pessoa de fora do workspace
  const op = (cookie, valor) => navegador(proxy, "POST", "/api/mudancas", { cookie, corpo: { cliente: "teste", ops: [{ op: "alterar", tipo: "plano", id: "t9", campos: { responsavel: valor }, antes: { responsavel: null } }] } });
  assert.equal((await op(sup.cookie, { id: idBruno })).status, 403, "supervisor não grava");
  const fantasma = await op(bruno.cookie, { id: "fantasma" });
  assert.ok(fantasma.status >= 400 && fantasma.status < 500, "pessoa que não existe: " + fantasma.status);
  assert.equal((await planoNoServidor("t9")).responsavel, undefined, "nada foi gravado");

  // 5. o agente do Bruno, pelo MCP por HTTP com a chave dele, lê quem é `voce` e os responsáveis
  const chave = (await navegador(proxy, "POST", "/api/chave", { cookie: bruno.cookie, corpo: {} })).json.chave;
  const mcp = (corpo) => http(proxy.url + "/mcp", { metodo: "POST", cabecalhos: { authorization: `Bearer ${chave}`, "content-type": "application/json", accept: "application/json, text/event-stream" }, corpo: JSON.stringify(corpo) });
  const lido = JSON.parse((await mcp({ jsonrpc: "2.0", id: 1, method: "tools/call", params: { name: "ler_blueprint", arguments: { workspace_id: "sm", blueprint_id: "p1" } } })).json.result.content[0].text);
  assert.equal(lido.voce.id, idBruno);
  const planos = Object.fromEntries(lido.blueprint.acoes.flatMap((a) => a.planos ?? []).map((p) => [p.id, p.responsavel]));
  assert.deepEqual(planos.t1, { id: idBruno, agente: true });
  assert.deepEqual(planos.t2, { id: idBruno });
  assert.equal(planos.t9, undefined);
  const inicio = (await mcp({ jsonrpc: "2.0", id: 2, method: "initialize", params: { protocolVersion: "2025-06-18" } })).json.result.instructions;
  assert.match(inicio, /responsavel.*voce/s, "as instruções mandam priorizar os planos do membro");
  assert.notEqual(idDona, idBruno);

  // 6. tirar o responsável volta o plano ao que era (sem sobra do campo)
  await B.js(`itensDeResponsavel("p1", "f1", "t2").find((i) => i.rotulo === "Tirar o responsável").fn()`);
  await espera(async () => (await planoNoServidor("t2")).responsavel === undefined, "o campo saiu do servidor");
  assert.deepEqual([...new Set([...A.erros, ...B.erros, ...S.erros])], [], "nenhum erro no console das páginas");
  console.log("responsavel e2e: ok (seletor no Editar, ao vivo, menu, Meus planos, supervisor só lê, servidor recusa fantasma, agente lê voce e responsavel)");
} finally {
  proxy.fechar();
  for (const p of filhos) try { p.kill(); } catch {}
}
