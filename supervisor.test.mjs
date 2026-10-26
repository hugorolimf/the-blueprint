// Painel Acompanhar do supervisor (web/js/supervisor.js, módulo ES): progresso, atrasos e próximo checkpoint por blueprint, a ordem dos cartões
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

const HOJE = "2027-01-15";
const acao = (id, fim, planos = [], checkpoints = [], status = "planejado") => ({ id, nome: id, inicio: "2027-01-01", fim, status, planos, checkpoints });
const p = (feito) => ({ id: Math.random().toString(36).slice(2), texto: "x", feito });
const bp = {
  id: "b1", nome: "Portal", acoes: [
    acao("a1", "2027-01-05", [p(true), p(true)]), // concluída
    acao("a2", "2027-01-10", [p(true), p(false)], [{ id: "c1", nome: "Revisão", data: "2027-01-10", feito: false }]), // atrasada
    acao("a3", "2027-01-30", [p(true), p(false)], [{ id: "c2", nome: "Entrega", data: "2027-01-30", feito: false }]), // em andamento
    acao("a4", "2027-02-10"), // sem itens, planejada
    acao("a5", "2027-01-02", [], [], "concluido"), // sem itens, marcada concluída
  ],
};
const r = resumoBlueprint(bp, HOJE);
assert.equal(r.acoes, 5);
assert.equal(r.concluidas, 2, "todos os itens feitos, ou concluída sem itens");
assert.equal(r.atrasadas, 1, "fim antes de hoje e não concluída");
assert.equal(r.andamento, 1);
assert.equal(r.pct, 50, "itens feitos / itens (4 de 8)");
assert.deepEqual(r.proximo, { nome: "Revisão", data: "2027-01-10" }, "o checkpoint em aberto mais cedo");
assert.equal(resumoBlueprint({ id: "v", nome: "Vazio", acoes: [] }, HOJE).pct, 0);
assert.equal(resumoBlueprint({ id: "s", nome: "Sem itens", acoes: [acao("x", "2027-01-01", [], [], "concluido")] }, HOJE).pct, 100, "concluído sem itens: 100%");

// atrasados primeiro, depois o menos adiantado
const ok = { id: "b2", nome: "Em dia", acoes: [acao("z", "2027-03-01", [p(true), p(true), p(false)])] };
const parado = { id: "b3", nome: "Parado", acoes: [acao("y", "2027-03-01", [p(false)])] };
const [w] = resumoSupervisor([{ id: "w1", nome: "Equipe", dados: { blueprints: [ok, parado, bp] } }], HOJE);
assert.equal(w.nome, "Equipe");
assert.deepEqual(w.blueprints.map((b) => b.id), ["b1", "b3", "b2"]);

assert.equal(textoAtividade("entrou"), "entrou");
assert.equal(textoAtividade("tipo-novo"), "tipo-novo", "tipo desconhecido aparece como veio");
vm.runInThisContext(readFileSync(join(import.meta.dirname, "web", "i18n", "en-pessoas.js"), "utf8"));
vm.runInThisContext(`I18N.ativo = "en"`);
assert.equal(textoAtividade("convite-revogado"), "revoked an invite link");
console.log("supervisor: ok (progresso, concluídas, atrasadas, em andamento, próximo checkpoint, ordem dos cartões, atividade em inglês)");
