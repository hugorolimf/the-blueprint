// Primeiro uso (web/js/primeiro-uso.js, script clássico num contexto isolado, como o navegador faz): a entrada vazia com os 3 caminhos,
// que some com dados; a linha "Remover exemplo"; e o exemplo como workspace novo (ids novos, datas perto de hoje, modelo intocado).
//   node primeiro-uso.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const el = () => ({ hidden: true, innerHTML: "", textContent: "", addEventListener() {} });
const els = new Map();
const ctx = vm.createContext({
  navigator: { languages: ["pt-BR"] }, location: { search: "", hash: "" },
  document: { getElementById: (id) => (els.has(id) ? els.get(id) : (els.set(id, el()), els.get(id))) },
  addEventListener() {},
});
const carrega = (arquivo) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", arquivo), "utf8"), ctx, { filename: arquivo });
["i18n.js", "base.js", "primeiro-uso.js"].forEach(carrega);
// o que primeiro-uso.js usa de entrada.js e sessao.js (o mesmo workspacesDe da entrada: a raiz "sm" e os outros)
vm.runInContext(`var carregado = true, arquivo = {}, modo = "local";
  const hospedado = () => modo === "servidor";
  const workspacesDe = (a) => [{ id: "sm", nome: "Meu", dados: a }, ...(a.workspaces || []).map((c) => ({ id: c.id, nome: c.nome, dados: c }))];`, ctx);
const tela = (dados, extra = "") => { vm.runInContext(`arquivo = ${JSON.stringify(dados)}; ${extra} primeiroUso();`, ctx); return els.get("primeiro-uso"); };
const cartoes = (e) => [...e.innerHTML.matchAll(/data-pu="(\w+)"/g)].map((m) => m[1]);

// banco vazio (só a raiz, sem blueprint): os 3 caminhos
{
  const e = tela({ blueprints: [] });
  assert.equal(e.hidden, false);
  assert.deepEqual(cartoes(e), ["exemplo", "agente", "criar"]);
  assert.match(e.innerHTML, /Abrir o exemplo/);
  // um workspace criado à mão, ainda sem blueprint, também é "vazio"
  assert.equal(tela({ blueprints: [], workspaces: [{ id: "w2", nome: "Novo", blueprints: [] }] }).hidden, false);
}
// banco cheio: some
{
  assert.equal(tela({ blueprints: [{ id: "b1", nome: "B", acoes: [] }] }).hidden, true);
  assert.equal(tela({ blueprints: [], workspaces: [{ id: "w2", nome: "Outro", blueprints: [{ id: "b2", acoes: [] }] }] }).hidden, true);
}
// com o exemplo carregado: só a linha de remover (e o nome vem escapado)
{
  const e = tela({ blueprints: [], workspaces: [{ id: "ex", nome: "<Aurora>", exemplo: true, blueprints: [{ id: "b9", acoes: [] }] }] });
  assert.equal(e.hidden, false);
  assert.deepEqual(cartoes(e), ["remover"]);
  assert.match(e.innerHTML, /&lt;Aurora&gt;/);
}
// antes de os dados chegarem e no modo de equipe: nada
assert.equal(tela({ blueprints: [] }, "carregado = false;").hidden, true);
assert.equal(tela({ blueprints: [] }, "carregado = true; modo = 'servidor';").hidden, true);
vm.runInContext("modo = 'local';", ctx);

// o exemplo de verdade como workspace novo
for (const arquivo of ["data.exemplo.json", "data.exemplo.en.json"]) {
  const texto = readFileSync(join(import.meta.dirname, arquivo), "utf8"), modelo = JSON.parse(texto);
  ctx.modelo = modelo;
  const ws = JSON.parse(JSON.stringify(vm.runInContext(`exemploComoWorkspace(modelo, "2026-12-01")`, ctx)));
  assert.equal(JSON.stringify(modelo), JSON.stringify(JSON.parse(texto)), "o modelo não muda");
  assert.equal(ws.exemplo, true);
  assert.equal(ws.nome, modelo.empresa);
  assert.equal(ws.blueprints.length, modelo.blueprints.length);
  // ids novos em todo item, todos diferentes entre si e dos originais
  const ids = (bps) => bps.flatMap((p) => [p.id, ...p.acoes.flatMap((a) => [a.id, ...a.planos.map((t) => t.id), ...a.checkpoints.map((m) => m.id)])]);
  const novos = [ws.id, ...ids(ws.blueprints)], velhos = new Set(ids(modelo.blueprints));
  assert.equal(new Set(novos).size, novos.length, "ids repetidos");
  assert.ok(novos.every((id) => typeof id === "string" && id && !velhos.has(id)), "sobrou id do modelo");
  // as datas andam junto com hoje (2026-10-20 -> 2026-12-01 = 42 dias); o resto do conteúdo fica
  const a0 = modelo.blueprints[0].acoes[0], b0 = ws.blueprints[0].acoes[0];
  const dias = (x, y) => (Date.parse(y) - Date.parse(x)) / 864e5;
  assert.equal(dias(a0.inicio, b0.inicio), 42);
  assert.equal(dias(a0.fim, b0.fim), 42);
  assert.equal(dias(a0.checkpoints[0].data, b0.checkpoints[0].data), 42);
  assert.equal(b0.nome, a0.nome);
  assert.equal(b0.planos[0].texto, a0.planos[0].texto);
  assert.equal(b0.status, a0.status);
}
console.log("primeiro-uso: ok");
