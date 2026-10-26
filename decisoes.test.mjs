// Decisões no Editar e no Planejar (web/js/decisoes.js, script clássico em contexto isolado): criar, editar com limite, remover e aplicar a proposta.
//   node decisoes.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const guardado = new Map();
const ctx = vm.createContext({
  navigator: { languages: ["pt-BR"] }, location: { search: "" },
  document: { getElementById: () => ({}), hidden: false }, matchMedia: () => ({ matches: false }), CSS: { escape: (s) => s },
  localStorage: { getItem: (k) => guardado.get(k) ?? null, setItem: (k, v) => guardado.set(k, v) },
});
["i18n.js", "base.js", "markdown.js", "decisoes.js"].forEach((a) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", a), "utf8"), ctx, { filename: a }));
vm.runInContext("this.api = { htmlDecisoesEd, novaDecisao, editarDecisao, removerDecisao, htmlDecisoesProposta, aplicaDecisoes, decisoesMarcadas };", ctx);
const api = ctx.api;
const json = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes

// criar e editar: o campo só existe quando há decisão; o estado inválido vira decidida; os textos têm limite
{
  const p = { id: "b" };
  assert.ok(api.htmlDecisoesEd(p).includes("Nenhuma decisão registrada"));
  const x = api.novaDecisao(p);
  assert.equal(x.estado, "aberta", "nasce em aberto, esperando a escolha");
  api.editarDecisao(p, x.id, "titulo", "Servidor");
  api.editarDecisao(p, x.id, "escolha", "Kotlin com Ktor");
  api.editarDecisao(p, x.id, "estado", "decidida");
  api.editarDecisao(p, x.id, "motivo", "m".repeat(5000));
  assert.equal(p.decisoes[0].motivo.length, 3000);
  api.editarDecisao(p, x.id, "estado", "qualquer");
  assert.equal(p.decisoes[0].estado, "decidida");
  assert.ok(api.htmlDecisoesEd(p).includes('value="Servidor"') && api.htmlDecisoesEd(p).includes("Kotlin com Ktor"));
  api.removerDecisao(p, x.id);
  assert.equal(p.decisoes, undefined, "sem decisões o campo some e o blueprint volta a ser o de antes");
}

// proposta: só as marcadas entram; a nova ganha id, a existente é atualizada por id; título vazio nunca grava
{
  const p = { id: "b", decisoes: [{ id: "d1", titulo: "Servidor", escolha: "Node", motivo: "", estado: "decidida", em: "2026-10-01" }] };
  const c = { decisoesNovas: [
    { id: "d1", titulo: "Servidor", escolha: "Kotlin com Ktor", motivo: "regras num lugar só", estado: "decidida", inc: true },
    { id: "", titulo: "Hospedagem", escolha: "", motivo: "", estado: "aberta", inc: true },
    { id: "", titulo: "Banco", escolha: "PostgreSQL", motivo: "", estado: "decidida", inc: false },
    { id: "", titulo: "  ", escolha: "x", motivo: "", estado: "decidida", inc: true },
  ] };
  assert.equal(api.decisoesMarcadas(c), 3);
  const html = api.htmlDecisoesProposta(c, p);
  assert.ok(html.includes("Alterada") && html.includes("Nova") && html.includes("4 decisões"), html.slice(0, 300));
  assert.equal(api.aplicaDecisoes(p, c), 2);
  const d = json(p.decisoes);
  assert.equal(d.length, 2);
  assert.deepEqual([d[0].id, d[0].escolha, d[0].motivo], ["d1", "Kotlin com Ktor", "regras num lugar só"]);
  assert.deepEqual([d[1].titulo, d[1].estado], ["Hospedagem", "aberta"]);
  assert.ok(d[1].id && d[1].id !== "d1", "a decisão nova ganha id próprio");
  assert.equal(api.htmlDecisoesProposta({}, p), "", "sem decisões na proposta não há cartão");
}

console.log("decisoes.test.mjs ok");
