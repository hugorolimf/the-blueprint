// O agrupamento do cartão do blueprint na lista (web/js/capa-agrupa.js, script clássico carregado num contexto isolado depois de i18n.js, como o navegador faz):
// as ações viram um bloco por situação quando não cabem, o texto de dica de cada bloco e os losangos de checkpoint que se tocam.
//   node capa-agrupa.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({ navigator: { languages: ["pt-BR"] }, location: { search: "" } });
const le = (f) => readFileSync(join(import.meta.dirname, "web", "js", f), "utf8");
vm.runInContext(le("i18n.js"), ctx, { filename: "i18n.js" });
vm.runInContext(le("capa-agrupa.js") + "\nthis.api = { capaCelulas, capaTitulo, capaMarcos, capaMarcoSvg };", ctx, { filename: "capa-agrupa.js" });
const A = ctx.api;
const J = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes
const acao = (status, pct, total, extra = {}) => ({ status, pct, it: [Math.round(pct * total), total], ...extra });

// ---- cabem: volta a mesma lista (uma célula por ação, como sempre foi)
const poucas = [acao("concluido", 1, 3), acao("andamento", 0.5, 4), acao("planejado", 0, 2)];
assert.equal(A.capaCelulas(poucas, 200), poucas);

// ---- não cabem: um bloco por situação, na ordem recebida, com o peso somado e o andamento médio
const muitas = [...Array(45).fill(0).map(() => acao("concluido", 1, 4)), acao("andamento", 0.5, 4), acao("andamento", 0.25, 12), ...Array(48).fill(0).map(() => acao("planejado", 0, 5))];
const blocos = J(A.capaCelulas(muitas, 415));
assert.equal(blocos.length, 3);
assert.deepEqual(blocos.map((b) => [b.status, b.grupo]), [["concluido", 45], ["andamento", 2], ["planejado", 48]]);
assert.equal(blocos[0].it[1], 180);
assert.equal(blocos[1].it[1], 16); // 4 + 12: a largura segue os itens
assert.ok(Math.abs(blocos[1].pct - (0.5 * 4 + 0.25 * 12) / 16) < 1e-9, "andamento médio ponderado: " + blocos[1].pct);
assert.equal(blocos[2].pct, 0);
// ação sem itens pesa 1 (o mesmo peso da barra)
assert.equal(J(A.capaCelulas([...Array(60).fill(0).map(() => acao("planejado", 0, 0))], 100))[0].it[1], 60);
// só uma situação: um bloco
assert.equal(A.capaCelulas(Array(80).fill(0).map(() => acao("planejado", 0, 1)), 100).length, 1);

// ---- dica de cada bloco (e a da ação, com o número da folha só quando preciso)
assert.equal(A.capaTitulo(blocos[0], () => { throw new Error("o número só vale para uma ação"); }), "45 ações concluídas");
assert.equal(A.capaTitulo(blocos[1], null), "2 ações em andamento (31% feito)");
assert.equal(A.capaTitulo(blocos[2], null), "48 ações a fazer");
assert.equal(A.capaTitulo({ status: "planejado", grupo: 1, pct: 0 }, null), "1 ação a fazer");
assert.equal(A.capaTitulo({ status: "andamento", nome: "Pesquisa", pct: 0.4 }, () => "2.3"), "2.3 Pesquisa · em andamento, 40% feito");
assert.equal(A.capaTitulo({ status: "concluido", nome: "" }, () => "1.1"), "1.1 SEM NOME · concluída");

// ---- losangos: os que se tocam viram um só (no meio), os distantes ficam
const m = J(A.capaMarcos([{ x: 100, feito: true }, { x: 10, feito: false }, { x: 12, feito: true }, { x: 15.9, feito: true }, { x: 40, feito: true }]));
assert.deepEqual(m.map((g) => [g.n, g.feitos]), [[3, 2], [1, 1], [1, 1]]);
assert.ok(Math.abs(m[0].x - (10 + 12 + 15.9) / 3) < 1e-9);
assert.deepEqual(m.map((g) => Math.round(g.x)), [13, 40, 100]);
assert.equal(J(A.capaMarcos([])).length, 0);
// exatamente a largura de um losango já não se toca
assert.equal(J(A.capaMarcos([{ x: 0, feito: true }, { x: 8, feito: true }])).length, 2);

// ---- o losango: cheio só com todos feitos; com mais de um, a dica diz quantos
const cheio = A.capaMarcoSvg({ x: 10, n: 1, feitos: 1 }, 100), meio = A.capaMarcoSvg({ x: 10, n: 3, feitos: 2 }, 100);
assert.ok(/class="dm ok"/.test(cheio) && !/<title>/.test(cheio));
assert.ok(/class="dm"/.test(meio) && /2 de 3 checkpoints feitos/.test(meio));

console.log("capa-agrupa: blocos por situação, dicas, losangos juntados e a lista pequena igual conferidos");
