// node auto-aplicar-logica.test.mjs: a parte pura de aplicar sozinho (web/js/auto-aplicar-logica.js): o padrão por modo, o que aplica e o que espera, a chave contra
// aplicar duas vezes, e o Desfazer (operações invertidas contra o estado atual, com conflito quando mexeram depois).
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

const ctx = vm.createContext({});
for (const f of ["operacoes.js", "auto-aplicar-logica.js"]) vm.runInContext(readFileSync(new URL(`web/js/${f}`, import.meta.url), "utf8"), ctx, { filename: f });
vm.runInContext("this.api = { autoEstado, autoDecide, autoChaveItem, autoChaveBlueprint, autoPrecisaConfirmar, autoEnvolve, autoOpsDe, autoRestaura, autoDesfazOps, autoPodeDesfazer, autoRastroAdiciona, autoRastroGuardavel, autoRastroTem, AUTO_JANELA_MS, AUTO_RASTRO_MAX };", ctx);
const A = ctx.api;
const clone = (x) => JSON.parse(JSON.stringify(x));

// ---- o interruptor: padrão por modo ----
assert.deepEqual(clone(A.autoEstado({})), { pode: true, ligado: true }, "app local: ligado por padrão");
assert.equal(A.autoEstado({ guardado: "0" }).ligado, false, "o usuário desligou");
assert.equal(A.autoEstado({ guardado: "1" }).ligado, true);
assert.deepEqual(clone(A.autoEstado({ hospedado: true, papel: "dono" })), { pode: true, ligado: false }, "equipe: desligado por padrão");
assert.equal(A.autoEstado({ hospedado: true, papel: "dono", guardado: "1" }).ligado, true, "o dono pode ligar");
for (const papel of ["colaborador", "supervisor", "visitante", ""]) assert.deepEqual(clone(A.autoEstado({ hospedado: true, papel, guardado: "1" })), { pode: false, ligado: false }, `${papel || "sem papel"} nunca aplica sozinho`);
assert.equal(A.autoEstado({ soLeitura: true, guardado: "1" }).ligado, false, "quem só lê nunca aplica sozinho");
assert.equal(A.autoChaveBlueprint("ws", "bp"), "blueprint.autoAplicar.ws:bp");

// ---- o que aplica e o que espera ----
const prop = (extra = {}) => ({ tipo: "proposta", workspaceId: "ws", blueprintId: "bp", numero: 1, criadaEm: "t1", proposta: { acoes: [{ nome: "x" }], ...extra } });
const reg = (acoes = [{ nome: "a" }]) => ({ tipo: "registro", workspaceId: "ws", blueprintId: "bp", numero: 1, criadaEm: "t1", registro: { acoes } });
const ok = { ligado: true, existe: true };
assert.equal(A.autoDecide(prop(), ok).aplica, true, "proposta comum entra");
assert.equal(A.autoDecide(reg(), ok).aplica, true, "registro entra");
assert.equal(A.autoDecide(prop(), { ligado: false, existe: true }).motivo, "desligado");
assert.equal(A.autoDecide(prop(), { ligado: true, existe: false }).motivo, "sem-blueprint");
assert.equal(A.autoDecide({ ...prop(), blueprintId: "" }, ok).motivo, "sem-blueprint", "blueprint novo espera o clique");
assert.equal(A.autoDecide(prop({ remover: [{ acao_id: "a" }] }), ok).motivo, "remover", "remoção espera o clique");
assert.equal(A.autoDecide(prop({ mover: [{ acao_id: "a" }] }), ok).motivo, "mover", "mover ação entre blueprints espera o clique");
assert.equal(A.autoDecide(reg([]), ok).motivo, "vazio");
assert.equal(A.autoDecide({ tipo: "progresso", blueprintId: "bp" }, ok).motivo, "tipo");
assert.equal(A.autoPrecisaConfirmar({ remover: [{}] }), true);
assert.equal(A.autoPrecisaConfirmar({ mover: [{}] }), true);
assert.equal(A.autoPrecisaConfirmar({ remover: [], mover: [] }), false);
assert.equal(A.autoPrecisaConfirmar(null), false);

// ---- a chave contra aplicar duas vezes: o número volta a 1 quando o app reinicia, a criadaEm desempata ----
assert.notEqual(A.autoChaveItem(prop()), A.autoChaveItem({ ...prop(), criadaEm: "t2" }));
assert.notEqual(A.autoChaveItem(prop()), A.autoChaveItem(reg()), "proposta e registro nº 1 não se confundem");
let lista = [];
lista = A.autoRastroAdiciona(lista, { chave: "k1", ws: "ws", blueprintId: "bp", em: 1, texto: "t", tipo: "proposta", ops: [1], antes: {} });
lista = A.autoRastroAdiciona(lista, { chave: "k1", ws: "ws", blueprintId: "bp", em: 2, texto: "t2", tipo: "proposta" });
assert.equal(lista.length, 1, "a mesma chave não duplica");
assert.equal(A.autoRastroTem(lista, "k1"), true);
assert.equal(A.autoRastroTem(lista, "k2"), false);
for (let n = 0; n < 40; n++) lista = A.autoRastroAdiciona(lista, { chave: `n${n}`, ws: "w", blueprintId: "b", em: n, texto: "", tipo: "registro" });
assert.equal(lista.length, A.AUTO_RASTRO_MAX, "o rastro tem teto");
assert.deepEqual(Object.keys(A.autoRastroGuardavel(lista)[0]).sort(), ["blueprintId", "chave", "em", "texto", "tipo", "ws"], "ao disco vão só os campos de listar, sem as operações");

// ---- Desfazer ----
const blueprint = () => ({ id: "bp", nome: "B", acoes: [
  { id: "a1", nome: "Um", inicio: "2026-10-26", fim: "2026-10-27", status: "planejado", planos: [{ id: "p1", texto: "Plano 1", feito: false }, { id: "p2", texto: "Plano 2", feito: false }], checkpoints: [{ id: "c1", nome: "Marco", data: "2026-10-27", feito: false }] },
  { id: "a2", nome: "Dois", inicio: "2026-10-28", fim: "2026-10-29", status: "planejado", planos: [], checkpoints: [] },
] });
// a aplicação: muda um campo, cria uma ação (com plano e checkpoint) e move o plano p2 para a ação a2
function aplica(bp) {
  bp.acoes[0].nome = "Um (novo nome)";
  bp.acoes[0].planos.splice(1, 1);
  bp.acoes[1].planos.push({ id: "p2", texto: "Plano 2", feito: false });
  bp.acoes.push({ id: "a3", nome: "Nova", inicio: "2026-10-30", fim: "2026-10-31", status: "planejado", planos: [{ id: "p3", texto: "Plano 3", feito: false }], checkpoints: [{ id: "c3", nome: "Marco 3", data: "2026-10-31", feito: false }] });
}
function cena() {
  const bp = blueprint(), antes = A.autoEnvolve(bp);
  aplica(bp);
  return { bp, antes, ops: A.autoOpsDe(antes, bp), D: { blueprints: [bp] } };
}
{ // nada mexeu depois: volta exatamente ao que era
  const { antes, ops, D } = cena();
  assert.ok(ops.length >= 3, "a aplicação gera operações (alterar, criar e mover)");
  const r = A.autoDesfazOps(ops, antes, D);
  assert.equal(r.conflitos, 0);
  assert.ok(r.feitas >= 3);
  assert.deepEqual(clone(D.blueprints[0]), blueprint(), "o blueprint voltou ao de antes, inclusive a ordem e o plano movido");
}
{ // o agente começou o plano criado depois: o que ele mexeu fica, o resto volta
  const { antes, ops, D } = cena();
  D.blueprints[0].acoes.find((a) => a.id === "a3").planos[0].agente = { estado: "executando" };
  const r = A.autoDesfazOps(ops, antes, D);
  assert.ok(r.conflitos >= 1, "a ação criada foi mexida: fica");
  assert.ok(D.blueprints[0].acoes.some((a) => a.id === "a3"), "a ação com o plano em execução não é apagada");
  assert.equal(D.blueprints[0].acoes[0].nome, "Um", "o campo que ninguém mexeu volta");
}
{ // outra janela mudou o mesmo campo depois: o valor dela fica, e o resto volta
  const { antes, ops, D } = cena();
  D.blueprints[0].acoes[0].nome = "Nome de outra janela";
  const r = A.autoDesfazOps(ops, antes, D);
  assert.ok(r.conflitos >= 1);
  assert.equal(D.blueprints[0].acoes[0].nome, "Nome de outra janela", "o que mexeram depois não é pisado");
  assert.ok(!D.blueprints[0].acoes.some((a) => a.id === "a3"), "o resto foi desfeito");
}
{ // o item que a aplicação criou já foi apagado por alguém: nada a fazer nele, sem erro
  const { antes, ops, D } = cena();
  D.blueprints[0].acoes.splice(D.blueprints[0].acoes.findIndex((a) => a.id === "a3"), 1);
  const r = A.autoDesfazOps(ops, antes, D);
  assert.equal(r.conflitos, 0);
}
{ // desfazer duas vezes seguidas não estraga nada (a segunda não acha o que inverter)
  const { antes, ops, D } = cena();
  A.autoDesfazOps(ops, antes, D);
  const depois1 = JSON.stringify(D.blueprints[0]);
  A.autoDesfazOps(ops, antes, D);
  assert.equal(JSON.stringify(D.blueprints[0]).length > 0, true);
  assert.equal(A.autoDesfazOps([], antes, D).feitas, 0, "sem operações, nada a desfazer");
  assert.equal(depois1, JSON.stringify(blueprint()), "o primeiro desfazer já voltou ao original");
}
{ // a aplicação que falha no meio: o retrato devolve o blueprint no lugar
  const bp = blueprint(), referencia = bp, antes = A.autoEnvolve(bp);
  aplica(bp); bp.extra = 1;
  A.autoRestaura(bp, antes);
  assert.equal(bp, referencia, "o mesmo objeto (as referências da tela continuam valendo)");
  assert.deepEqual(clone(bp), blueprint());
}

// ---- a janela do Desfazer ----
const e = { em: 1000, ops: [{}], desfeito: false };
assert.equal(A.autoPodeDesfazer(e, 1000 + A.AUTO_JANELA_MS - 1), true);
assert.equal(A.autoPodeDesfazer(e, 1000 + A.AUTO_JANELA_MS), false, "passou de 60 s");
assert.equal(A.autoPodeDesfazer({ ...e, desfeito: true }, 1001), false, "uma vez só");
assert.equal(A.autoPodeDesfazer({ ...e, ops: [] }, 1001), false, "sem mudança não há o que desfazer");
assert.equal(A.autoPodeDesfazer(undefined, 1001), false);
console.log("auto-aplicar-logica: ok");
