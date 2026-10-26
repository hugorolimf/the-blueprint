// A lógica pura da presença (web/js/presenca.js, script clássico carregado num contexto isolado com o mínimo de DOM falso).
//   node presenca.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const nada = () => {};
const ctx = vm.createContext({ document: { addEventListener: nada }, window: { addEventListener: nada }, MutationObserver: class { observe() {} }, host: {}, $: () => null, CLIENTE: "c-eu" });
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "presenca.js"), "utf8"), ctx, { filename: "presenca.js" });
const js = (x) => JSON.parse(JSON.stringify(x));

// a cor é estável por membro e está entre as do tema
{
  const cores = ["ana", "bruno", "m1", "m2", "x"].map((id) => ctx.presencaCor(id));
  assert.ok(cores.every((c) => Number.isInteger(c) && c >= 0 && c < vm.runInContext("PRESENCA_CORES", ctx)));
  assert.equal(ctx.presencaCor("ana"), ctx.presencaCor("ana"));
  assert.ok(new Set(["a", "b", "c", "d", "e", "f", "g", "h"].map((id) => ctx.presencaCor(id))).size > 2, "pessoas diferentes ganham cores diferentes");
}
// as iniciais: primeira e última palavra
{
  assert.equal(ctx.presencaIniciais("Ana Paula Souza"), "AS");
  assert.equal(ctx.presencaIniciais("bruno"), "B");
  assert.equal(ctx.presencaIniciais(""), "?");
}
// sem esta aba, uma entrada por membro, o item em edição de qualquer aba dele
{
  const lista = [
    { id: "eu", nome: "Eu", cliente: "c-eu", editando: { tipo: "plano", id: "p1" } },
    { id: "ana", nome: "Ana", cliente: "c-a1", blueprintId: "b1", editando: null },
    { id: "ana", nome: "Ana", cliente: "c-a2", blueprintId: "b1", acaoId: "a1", editando: { tipo: "plano", id: "p2" } },
    { id: "bruno", nome: "Bruno", cliente: "c-b", blueprintId: "b2", editando: { tipo: "acao", id: "a9" } },
  ];
  const pessoas = js(ctx.presencaPessoas(lista, "c-eu"));
  assert.deepEqual(pessoas.map((p) => p.id), ["ana", "bruno"]);
  assert.deepEqual(pessoas[0].editando, { tipo: "plano", id: "p2" });
  // só os contornos do blueprint aberto
  const contornos = js(ctx.presencaContornos(ctx.presencaPessoas(lista, "c-eu"), "b1"));
  assert.deepEqual(contornos, [{ tipo: "plano", id: "p2", nome: "Ana", cor: ctx.presencaCor("ana") }]);
  assert.deepEqual(js(ctx.presencaContornos([], "b1")), []);
}
console.log("presenca.test.mjs: ok");
