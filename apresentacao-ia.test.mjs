// Lógica pura da apresentação com IA (web/js/apresentacao-ia-logica.js): estimativas, Épico ausente, corpo do pedido, frases do resultado e a lista.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const ctx = vm.createContext({ tr: (t, p = {}) => t.replace(/\{(\w+)\}/g, (_, k) => p[k] ?? "") });
vm.runInContext(readFileSync(new URL("web/js/apresentacao-ia-logica.js", import.meta.url), "utf8") + "\nthis.api = { AP_IA_NIVEIS, AP_IA_PUBLICOS, apIaEstimativa, apIaCorpoPedido, apIaMensagem, apIaLista, apIaUrl, apIaRotuloNivel };", ctx);
const j = (x) => JSON.parse(JSON.stringify(x)); // o vm tem os seus próprios Object e Array: compara por valor
const { AP_IA_NIVEIS, AP_IA_PUBLICOS, apIaEstimativa, apIaCorpoPedido, apIaMensagem, apIaLista, apIaUrl, apIaRotuloNivel } = ctx.api;

assert.deepEqual(j(AP_IA_NIVEIS.map((n) => n.id)), ["rapido", "padrao", "completo"], "o Épico nunca aparece na tela");
assert.deepEqual(j(AP_IA_PUBLICOS), ["cliente", "interno"]);
assert.equal(apIaEstimativa("rapido"), "até 1 h");
assert.equal(apIaEstimativa("padrao"), "1 a 3 h");
assert.equal(apIaEstimativa("completo"), "2 a 4 h");
assert.equal(apIaEstimativa("epico"), "", "sem estimativa para o que a tela não oferece");
assert.equal(apIaRotuloNivel().epico, undefined);

const base = { workspaceId: "w", blueprintId: "b", publico: "cliente", nivel: "padrao" };
assert.deepEqual(j(apIaCorpoPedido(base)), base, "sem sessão: terminal novo");
assert.deepEqual(j(apIaCorpoPedido({ ...base, sessao: { id: "s1", canal: true } })), { ...base, sessaoId: "s1" });
assert.deepEqual(j(apIaCorpoPedido({ ...base, sessao: { id: "s2", canal: false } })), { ...base, sessaoId: "s2", semCanal: true }, "sessão sem canal só copia o pedido");
assert.equal(apIaCorpoPedido({ ...base, nivel: "epico" }), null, "Épico não vira pedido pela tela");
assert.equal(apIaCorpoPedido({ ...base, publico: "todos" }), null);
assert.equal(apIaCorpoPedido({ ...base, blueprintId: "" }), null);
assert.equal(apIaCorpoPedido(), null);

assert(/^Pedido enviado à sessão/.test(apIaMensagem(200, {}, true)));
assert(/^Terminal aberto/.test(apIaMensagem(200, { ok: true }, false)));
assert(/Ela aparece na lista/.test(apIaMensagem(200, {}, false)));
assert(/não está mais conectada/.test(apIaMensagem(409, { codigo: "sessao-caiu" }, true)), "o código vale mais que o status");
assert(/Pasta do projeto/.test(apIaMensagem(409, {}, false)));
assert(/não achou este blueprint/.test(apIaMensagem(404, {}, false)));
assert(/Motivo: sem wt/.test(apIaMensagem(501, { codigo: "sem-terminal", erro: "sem wt" }, false)));
assert.equal(apIaMensagem(500, { erro: "falhou assim" }, false), "falhou assim", "o erro do servidor aparece como veio");
assert.equal(apIaMensagem(500, {}, false), "Não foi possível pedir a apresentação (erro 500).");

const lista = apIaLista([
  { id: "a1", titulo: "Antiga", publico: "cliente", nivel: "rapido", em: "2026-10-20T10:00:00Z" },
  { id: "b2", titulo: "Nova", publico: "interno", nivel: "completo", em: "2026-10-26T09:00:00Z" },
  { id: "../x", titulo: "Torta", em: "2026-10-27" },
  { id: "c3", titulo: "x".repeat(300), publico: "outro", nivel: "epico", em: "2026-10-22" },
  null, { titulo: "sem id" },
], new Set());
assert.deepEqual(j(lista.map((a) => a.id)), ["b2", "c3", "a1"], "mais nova primeiro; id fora do padrão não entra");
assert.equal(lista[0].em, "2026-10-26");
assert.equal(lista[1].titulo.length, 120);
assert.equal(lista[1].publico, "");
assert.equal(lista[1].nivel, "", "um nível desconhecido aparece sem rótulo, a apresentação não some");
assert.deepEqual(j(apIaLista([{ id: "a1", em: "2026-10-20" }, { id: "b2", em: "2026-10-21" }], new Set(["b2"])).map((a) => a.id)), ["a1"], "as ocultas saem");
assert.deepEqual(j(apIaLista(undefined)), []);

assert.equal(apIaUrl("a1"), "/apresentacao/a1");
assert.equal(apIaUrl("a1", true), "/apresentacao/a1?baixar=1");
assert.equal(apIaUrl("../segredo"), "");
assert.equal(apIaUrl('x"onload="'), "");
console.log("apresentacao-ia: ok");
