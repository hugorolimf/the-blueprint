// Painel Acompanhar do supervisor (web/js/supervisor.js, módulo ES): progresso e próximo checkpoint por blueprint, a ordem dos cartões
// e o texto da atividade. A tela (abrir, cartões, pessoas, Abrir) é coberta por scripts/visitante-supervisor-e2e.mjs.
//   node supervisor.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

// os módulos de web/js são ES num package.json sem "type": o Node os reconhece sozinho e só avisa; o aviso fica de fora da saída do teste
const avisar = process.emitWarning;
process.emitWarning = (w, ...r) => (/MODULE_TYPELESS/.test(JSON.stringify(r)) ? undefined : avisar.call(process, w, ...r));

vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "js", "i18n.js"), "utf8"), { filename: "i18n.js" });
vm.runInThisContext(`I18N.ativo = "pt-BR"`);
const { resumoBlueprint, resumoSupervisor, textoAtividade } = await import("./web/js/supervisor.js");

const acao = (id, planos = [], checkpoints = [], status = "planejado") => ({ id, nome: id, status, planos, checkpoints });
const p = (feito) => ({ id: Math.random().toString(36).slice(2), texto: "x", feito });
const bp = {
  id: "b1", nome: "Portal", acoes: [
    acao("a1", [p(true), p(true)]), // concluída
    acao("a2", [p(true), p(false)], [{ id: "c1", nome: "Revisão", feito: false }]), // em andamento
    acao("a3", [p(true), p(false)], [{ id: "c2", nome: "Entrega", feito: false }]), // em andamento
    acao("a4"), // sem itens, planejada
    acao("a5", [], [], "concluido"), // sem itens, marcada concluída
  ],
};
const r = resumoBlueprint(bp);
assert.equal(r.acoes, 5);
assert.equal(r.concluidas, 2, "todos os itens feitos, ou concluída sem itens");
assert.equal(r.andamento, 2);
assert.equal(r.pct, 50, "itens feitos / itens (4 de 8)");
assert.deepEqual(r.proximo, { nome: "Revisão" }, "o primeiro checkpoint em aberto, na ordem do blueprint");
assert.equal(resumoBlueprint({ id: "v", nome: "Vazio", acoes: [] }).pct, 0);
assert.equal(resumoBlueprint({ id: "s", nome: "Sem itens", acoes: [acao("x", [], [], "concluido")] }).pct, 100, "concluído sem itens: 100%");

// o menos adiantado primeiro
const ok = { id: "b2", nome: "Adiantado", acoes: [acao("z", [p(true), p(true), p(false)])] };
const parado = { id: "b3", nome: "Parado", acoes: [acao("y", [p(false)])] };
const [w] = resumoSupervisor([{ id: "w1", nome: "Equipe", dados: { blueprints: [ok, parado, bp] } }]);
assert.equal(w.nome, "Equipe");
assert.deepEqual(w.blueprints.map((b) => b.id), ["b3", "b1", "b2"]);

assert.equal(textoAtividade("entrou"), "entrou");
assert.equal(textoAtividade("tipo-novo"), "tipo-novo", "tipo desconhecido aparece como veio");
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-pessoas.js"), "utf8"));
vm.runInThisContext(`I18N.ativo = "en"`);
assert.equal(textoAtividade("convite-revogado"), "revoked an invite link");
console.log("supervisor: ok (progresso, concluídas, em andamento, próximo checkpoint, ordem dos cartões, atividade em inglês)");
