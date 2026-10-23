// Teste da fila de propostas da página (web/js/fila-propostas.js), sem navegador: roda o script num contexto com o mínimo de peças falsas.
import { readFileSync } from "node:fs";
import vm from "node:vm";
import assert from "node:assert/strict";

const acks = [];
const ctx = {
  NOVO: "__novo", D: { blueprints: [{ id: "p1" }] },
  $: () => ({ addEventListener() {} }), esc: (t) => String(t), blueprintEd: () => null, salvar() {}, montaProposta() {},
  conv: (id) => (ctx.chats[id] ||= { contexto: `sm:${id}` }), chats: {},
  fetch: (_url, op) => { acks.push(JSON.parse(op.body)); return Promise.resolve(); },
};
vm.createContext(ctx);
vm.runInContext(readFileSync(new URL("./web/js/fila-propostas.js", import.meta.url), "utf8"), ctx);
const rodar = (codigo) => vm.runInContext(codigo, ctx);
const js = (x) => JSON.parse(JSON.stringify(x)); // arrays de outro contexto não passam no deepEqual

const item = (numero, criadaEm, em, extra = {}) => ({ id: numero * 10, numero, criadaEm, em, blueprintId: "p1", ...extra });
const dados = (msg) => ({ proposta: [{ nome: msg }], remover: [], mover: [], dadosBlueprint: null, pasta: null, mensagem: msg });
ctx.c = ctx.conv("p1");

// proposta nova entra no fim e não tira a aberta
rodar(`filaRecebe(c, ${JSON.stringify(item(1, "t1", "t1"))}, ${JSON.stringify(dados("um"))})`);
rodar(`filaRecebe(c, ${JSON.stringify(item(2, "t2", "t2"))}, ${JSON.stringify(dados("dois"))})`);
assert.equal(ctx.c.numero, 1);
assert.deepEqual(js(ctx.c.fila.map((e) => e.numero)), [2]);

// o ajuste da que está na fila toma o lugar dela; o da aberta troca a aberta
rodar(`filaRecebe(c, ${JSON.stringify(item(2, "t2", "t2b"))}, ${JSON.stringify(dados("dois ajustado"))})`);
assert.equal(ctx.c.fila.length, 1);
assert.equal(ctx.c.fila[0].mensagem, "dois ajustado");
rodar(`filaRecebe(c, ${JSON.stringify(item(1, "t1", "t1b"))}, ${JSON.stringify(dados("um ajustado"))})`);
assert.equal(ctx.c.mensagem, "um ajustado");
assert.equal(rodar(`filaJaVista(${JSON.stringify(item(1, "t1", "t1b", { blueprintId: "p1" }))})`), true, "o servidor mandou de novo a mesma, sem ajuste");
assert.equal(rodar(`filaJaVista(${JSON.stringify(item(1, "t1", "outra"))})`), false);

// abrir outra guarda a aberta (com o que foi editado) e aplicar/descartar abre a próxima, confirmando ao servidor pela chave
ctx.c.proposta[0].nome = "editado";
assert.equal(rodar("filaAbre(c, 2)"), true);
assert.equal(ctx.c.numero, 2);
assert.equal(ctx.c.fila[0].proposta[0].nome, "editado");
rodar("filaConclui(c)");
assert.deepEqual(js(acks.at(-1)), { ids: [], propostas: [{ workspaceId: "sm", blueprintId: "p1", numero: 2, criadaEm: "t2" }] });
assert.equal(ctx.c.numero, 1);
assert.equal(ctx.c.fila.length, 0);
rodar("filaConclui(c)");
assert.equal(ctx.c.proposta, null);
assert.equal(rodar("filaTemAberta(c)"), false);

// o app reiniciou e voltou a numerar do 1: a velha ganha outro número e a nova entra
rodar(`filaRecebe(c, ${JSON.stringify(item(1, "t1", "t1"))}, ${JSON.stringify(dados("antes"))})`);
rodar(`filaRecebe(c, ${JSON.stringify(item(1, "novo", "novo"))}, ${JSON.stringify(dados("depois"))})`);
assert.deepEqual(js(rodar("filaTodas(c).map((e) => [e.numero, e.mensagem])")), [[1, "depois"], [2, "antes"]]);

// o chat do blueprint novo segue sem levar as propostas que esperam
const todas = rodar("filaDesmonta(c)");
assert.equal(todas.length, 2);
assert.equal(rodar("filaTemAberta(c)"), false);
ctx.todas = todas;
rodar("filaMonta(conv('outro'), todas)");
assert.deepEqual(js(rodar("filaTodas(conv('outro')).map((e) => e.numero)")), [1, 2]);
assert.equal(ctx.chats.outro.numero, 1);
console.log("fila-propostas.test.mjs: ok");
