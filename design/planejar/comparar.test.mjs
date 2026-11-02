// node design/planejar/comparar.test.mjs
import assert from "node:assert/strict";
import { diffPalavras, compararAcao, compararPlanos, planosTravados } from "./comparar.mjs";

const juntos = (l, t) => l.filter((p) => p.t !== t).map((p) => p.s).join("");
const d = diffPalavras("Contador na barra", "Contador vermelho na barra");
assert.equal(juntos(d, "mais"), "Contador na barra");
assert.equal(juntos(d, "menos"), "Contador vermelho na barra");
assert.ok(d.some((p) => p.t === "mais" && p.s.includes("vermelho")));

const q = (id, texto) => ({ id, texto });
const a = { id: "1", de: "2026-11-01", ate: "2026-11-05", planos: [q("1.1", "Ler"), q("1.2", "Guardar"), q("1.3", "Apagar")], detalhes: [{ id: "01" }] };
const b = { id: "1", de: "2026-11-01", ate: "2026-11-09", planos: [q("1.1", "Ler"), q("1.2", "Guardar tudo"), q("1.4", "Mostrar")], detalhes: [{ id: "01" }, { id: "02" }] };
const c = compararAcao(a, b);
assert.deepEqual(c.resumo, { mais: 1, menos: 1, alterados: 2, detalhes: 1 });
assert.deepEqual(c.linhas.map((l) => l.tipo), ["prazo", "igual", "alterado", "novo", "removido"]);
assert.equal(compararAcao(a, a).mudou, false);

const P = (...acoes) => ({ projetos: [{ acoes }] });
const r = compararPlanos(P(a, { id: "2", planos: [] }), P(b, { id: "2", planos: [] }, { id: "3", planos: [q("3.1", "Novo")] }));
assert.deepEqual(r.novas, ["3"]);
assert.deepEqual(r.iguais, ["2"]);
assert.deepEqual(Object.keys(r.alteradas), ["1"]);
assert.deepEqual(planosTravados(r), ["1.2", "1.4"]);
assert.deepEqual(compararPlanos(null, P(a)).novas, ["1"]);
console.log("comparar: ok");
