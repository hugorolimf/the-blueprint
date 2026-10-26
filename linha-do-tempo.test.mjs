// A lógica pura da linha do tempo do agente na folha da ação (web/js/linha-do-tempo.js, script clássico num contexto isolado, como o navegador faz):
// a hora (só hh:mm hoje), a frase por tipo (sem a evidência), o nome do plano ou do checkpoint e o HTML escapado. O DOM e o SSE ficam no e2e.
//   node linha-do-tempo.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const carrega = (idioma) => {
  const ctx = vm.createContext({ navigator: { languages: [idioma] }, location: { search: "" }, document: { write() {}, documentElement: {} } });
  vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), ctx);
  if (idioma === "en") vm.runInContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-linha-do-tempo.js"), "utf8"), ctx);
  vm.runInContext("var esc = (s) => String(s ?? '').replace(/[&<>\"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '\"': '&quot;' })[c]);", ctx);
  vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "linha-do-tempo.js"), "utf8") + "\nthis.api = { ldtHora, ldtFrase, ldtAlvo, ldtLista };", ctx);
  return ctx.api;
};
const L = carrega("pt-BR");
const acao = { planos: [{ id: "p1", texto: "Criar a tela" }], checkpoints: [{ id: "c1", nome: "Tela aprovada" }] };

// ---- a hora: hoje só hh:mm; outro dia leva o dia
const agora = new Date(2026, 9, 26, 15, 0).getTime();
assert.match(L.ldtHora(new Date(2026, 9, 26, 9, 5).toISOString(), agora), /^09:05$/);
assert.match(L.ldtHora(new Date(2026, 9, 20, 9, 5).toISOString(), agora), /20.*09:05/);
assert.equal(L.ldtHora("não é data", agora), "");

// ---- a frase por tipo: o progresso é a frase do agente; as marcações não mostram texto (a evidência nem é guardada)
assert.equal(L.ldtFrase({ tipo: "progresso", texto: "Editando web/x.js" }), "Editando web/x.js");
assert.equal(L.ldtFrase({ tipo: "progresso", texto: "" }), "Trabalhando");
assert.equal(L.ldtFrase({ tipo: "concluir", texto: "" }), "Plano concluído");
assert.equal(L.ldtFrase({ tipo: "bloqueio", texto: "sem acesso" }), "Bloqueio: sem acesso");
assert.equal(L.ldtFrase({ tipo: "checkpoint-feito" }), "Checkpoint feito");

// ---- o alvo: plano pelo id, checkpoint pelo id conforme o tipo; o que não existe mais fica sem nome
assert.equal(L.ldtAlvo({ tipo: "progresso", alvo: "p1" }, acao), "Criar a tela");
assert.equal(L.ldtAlvo({ tipo: "checkpoint-feito", alvo: "c1" }, acao), "Tela aprovada");
assert.equal(L.ldtAlvo({ tipo: "concluir", alvo: "apagado" }, acao), "");
assert.equal(L.ldtAlvo({ tipo: "progresso", alvo: null }, acao), "");

// ---- a lista: vazia explica; com eventos, um item por evento, escapado, com a classe do tipo
assert.match(L.ldtLista([], acao), /Nenhum evento do agente/);
const html = L.ldtLista([{ em: new Date(agora).toISOString(), agente: "Claude", tipo: "concluir", texto: "", alvo: "p1" }, { em: new Date(agora).toISOString(), agente: "<b>", tipo: "progresso", texto: "a < b" }], acao, agora);
assert.equal((html.match(/<li /g) || []).length, 2);
assert.match(html, /class="ldt-concluir".*Plano concluído.*Criar a tela/);
assert.ok(html.includes("&lt;b&gt;") && html.includes("a &lt; b") && !html.includes("<b>"), "texto do agente sempre escapado");

// ---- em inglês, com o catálogo próprio
const E = carrega("en");
assert.equal(E.ldtFrase({ tipo: "bloqueio", texto: "no access" }), "Blocked: no access");
assert.match(E.ldtLista([], acao), /No agent events/);

console.log("linha-do-tempo: ok");
