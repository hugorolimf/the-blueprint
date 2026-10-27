// O fragmento da URL e a seção da página (biblioteca/fragmento.js): quais fragmentos valem, o que acontece quando a seção ou a etapa não existe mais.
import assert from "node:assert/strict";
import { carregar } from "./teste-comum.mjs";

const { L } = carregar();
const SEC = ["visao", "entregas", "etapas", "linha", "voce", "decisoes"], ETAPAS = ["e1", "e2", "e3", "e4"];
const r = (f, secoes = SEC, etapas = ETAPAS) => { const x = L.fragmento.resolver(f, secoes, etapas); return x && JSON.parse(JSON.stringify(x)); }; // cópia: o objeto vem de outro contexto

assert.deepEqual(r("visao"), { secao: "visao" });
assert.deepEqual(r("#etapas"), { secao: "etapas" }, "o # é opcional");
assert.deepEqual(r("etapas-3"), { secao: "etapas", etapa: 2 }, "a 3ª etapa, índice 2");
assert.deepEqual(r("s-linha"), { secao: "linha" }, "o id real da seção");
assert.deepEqual(r("e-e3"), { secao: "etapas", etapa: 2 }, "o id da etapa");
assert.deepEqual([r("recebe"), r("precisamos"), r("riscos")], [{ secao: "entregas" }, { secao: "voce" }, { secao: "decisoes" }], "os nomes do conteúdo");
assert.deepEqual(r("ETAPAS-2"), { secao: "etapas", etapa: 1 }, "sem diferença de maiúsculas");

// o que deixou de existir cai no mais próximo, sem erro
assert.deepEqual(r("etapas-99"), { secao: "etapas", etapa: 3 }, "etapa que sumiu: a última");
assert.deepEqual(r("etapas-0"), { secao: "etapas", etapa: 0 }, "etapa 0: a primeira");
assert.deepEqual(r("e-zzz"), { secao: "etapas" }, "id de etapa que não existe: a seção Etapas");
assert.deepEqual(r("acoes"), { secao: "decisoes" }, "seção que não existe (Ações no cliente): a mais próxima");
assert.deepEqual(r("voce", ["visao", "etapas", "linha", "decisoes"]), { secao: "linha" }, "empate: a anterior");
assert.deepEqual(r("etapas-2", SEC, []), { secao: "etapas" }, "sem etapas: só a seção");

// fragmento que não é nosso: nada a fazer
for (const f of ["banana", "", "#", "etapas-", "s-", "conteudo", "x/y", undefined, null]) assert.equal(r(f), null, `fragmento inválido: ${f}`);
assert.equal(r("visao", [], []), null, "sem seções");

assert.equal(L.fragmento.texto({ secao: "etapas", etapa: 2 }), "etapas-3");
assert.equal(L.fragmento.texto({ secao: "linha" }), "linha");
console.log("fragmento.test.mjs: ok");
