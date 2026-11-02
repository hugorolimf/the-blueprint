// node design/planejar/agentes.test.mjs
import assert from "node:assert/strict";
import { escalonar, areasConflitam, acharCiclo, avisos, diario } from "./agentes.mjs";
import { validar } from "./pacote.mjs";

const q = (id, area, ...depende_de) => ({ id, texto: id, area, depende_de });
const ap = (l, extra = {}) => ({ iniciados: l.map((x) => x.id), ...extra });

// cada caixinha marcada, sem conflito, ganha agente; dez planos, dez agentes
const dez = Array.from({ length: 10 }, (_, i) => q(`p${i}`, `area${i}`));
assert.ok(Object.values(escalonar(dez, ap(dez, { limite: 10 }))).every((e) => e.estado === "trabalhando"));
// o limite deixa o resto na fila
assert.equal(Object.values(escalonar(dez, ap(dez, { limite: 3 }))).filter((e) => e.estado === "fila").length, 7);
// caixinha não marcada não roda
assert.equal(escalonar([q("a")], {}).a.estado, "nao");
// plano alterado não começa antes da revisão
assert.equal(escalonar([q("a")], { iniciados: ["a"], travados: ["a"] }).a.estado, "revisao");
// mesma área nunca roda junto, nem uma dentro da outra
assert.ok(areasConflitam("app", "app/barra") && areasConflitam("app/barra/", "app/barra"));
assert.ok(!areasConflitam("app/barra", "app/barra2") && !areasConflitam(undefined, "app"));
const area = [q("a", "app"), q("b", "app/barra"), q("c", "servidor")];
let e = escalonar(area, ap(area));
assert.deepEqual([e.a.estado, e.b.estado, e.c.estado], ["trabalhando", "area", "trabalhando"]);
// quando o primeiro fica pronto, o segundo roda
e = escalonar(area, ap(area, { prontos: ["a"] }));
assert.equal(e.b.estado, "trabalhando");
// dependência espera e avisa; pronta, libera
const dep = [q("a", "x"), q("b", "y", "a")];
e = escalonar(dep, ap(dep));
assert.deepEqual([e.a.estado, e.b.estado, e.b.por], ["trabalhando", "dependencia", "a"]);
assert.equal(escalonar(dep, ap(dep, { prontos: ["a"] })).b.estado, "trabalhando");
// depender de plano não iniciado avisa; de plano travado, para
e = escalonar(dep, { iniciados: ["b"] });
assert.match(avisos(dep, e)[0], /ainda não foi iniciado/);
e = escalonar(dep, { iniciados: ["b"], bloqueados: ["a"] });
assert.equal(e.b.estado, "parado");
// quem já tem agente continua e segura a área
const seg = [q("a", "app"), q("b", "app")];
assert.equal(escalonar(seg, { iniciados: ["b"], rodando: ["a"] }).b.estado, "area");
// o diário do orquestrador põe o pedido de ajuda primeiro
const d3 = [q("a", "x"), q("b", "x"), q("c", "y"), q("d", "z")];
const dia = diario(d3, escalonar(d3, { iniciados: ["a", "b"], prontos: ["c"], bloqueados: ["d"] }));
assert.match(dia[0], /d pediu ajuda/);
assert.deepEqual(dia.slice(1), ["Chamou um agente para a", "Segurou b: a já mexe em x", "Conferiu c: concluído, com evidência"]);
// ciclo
assert.deepEqual(acharCiclo([q("a", "", "b"), q("b", "", "a")]), ["a", "b", "a"]);
assert.equal(acharCiclo(dep), null);
// a validação recusa dependência inexistente, id repetido e ciclo; aceita o plano escrito só como texto
const plano = (planos) => ({ formato: "blueprint.plano/1", blueprint: "X", projetos: [{ acoes: [{ id: "1", nome: "A", planos }] }] });
assert.deepEqual(validar(plano(["Fazer", { id: "1.9", texto: "Outro" }]), "."), []);
assert.match(validar(plano([{ id: "a", texto: "t", depende_de: ["z"] }]), ".")[0], /não existe o plano z/);
assert.match(validar(plano([{ id: "a", texto: "t" }, { id: "a", texto: "u" }]), ".")[0], /id repetido/);
assert.match(validar(plano([{ id: "a", texto: "t", depende_de: ["b"] }, { id: "b", texto: "u", depende_de: ["a"] }]), ".")[0], /ciclo/);
console.log("agentes: ok");
