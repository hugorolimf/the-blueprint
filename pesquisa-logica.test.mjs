// A correspondência de texto da pesquisa (web/js/pesquisa-logica.js, script clássico carregado num contexto isolado, como o navegador faz).
//   node pesquisa-logica.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({});
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "pesquisa-logica.js"), "utf8")
  + "\nthis.api = { pesqNorma, pesqTermos, pesqTrechos, pesqCampos, pesqAcao, pesqPlano, pesqCheckpoint, pesqBlueprint, pesqCasaBlueprint }", ctx, { filename: "pesquisa-logica.js" });
const A = ctx.api;
const J = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes

// ---- termos: sem acento, sem caixa, sem repetição, vazio dá lista vazia
assert.deepEqual(J(A.pesqTermos("  Ação  FÉ  ação ")), ["acao", "fe"]);
assert.deepEqual(J(A.pesqTermos("   ")), []);
assert.deepEqual(J(A.pesqTermos("")), []);
assert.deepEqual(J(A.pesqTermos(undefined)), []);

// ---- trechos: índices no texto ORIGINAL (com acento), juntando os que se tocam
assert.deepEqual(J(A.pesqTrechos("Configuração inicial", ["configuracao"])), [[0, 12]]);
assert.deepEqual(J(A.pesqTrechos("Ação e ação", ["acao"])), [[0, 4], [7, 11]]);
assert.deepEqual(J(A.pesqTrechos("abcabc", ["ab", "bc"])), [[0, 6]], "trechos que se tocam viram um só");
assert.deepEqual(J(A.pesqTrechos("abc", ["x"])), []);
assert.deepEqual(J(A.pesqTrechos("abc", [])), []);
assert.deepEqual(J(A.pesqTrechos("Mão à obra", ["a o"])), [[4, 7]], "o termo pode ter espaço interno; os índices seguem o original");
// letra cuja normalização muda o tamanho (İ vira i + ponto): o índice continua apontando para o original
assert.deepEqual(J(A.pesqTrechos("İstanbul", ["stan"])), [[1, 5]]);
assert.equal(A.pesqNorma("ÁÉÍ ñ").s, "aei n");

// ---- campos: todos os termos têm de aparecer (juntando os campos do item); diz onde achou
assert.deepEqual(J(A.pesqCampos({ nome: "Escrever testes", descricao: "Validação com acento" }, ["teste"])), ["nome"]);
assert.deepEqual(J(A.pesqCampos({ nome: "Escrever testes", descricao: "Validacao com acento" }, A.pesqTermos("teste validação"))), ["nome", "descricao"], "termos em campos diferentes casam o item");
assert.equal(A.pesqCampos({ nome: "Escrever testes", descricao: "x" }, ["teste", "zzz"]), null, "falta um termo: não casa");
assert.equal(A.pesqCampos({ nome: "abc" }, []), null, "consulta vazia não casa nada");
assert.equal(A.pesqCampos({ nome: undefined, descricao: null }, ["a"]), null, "campo ausente não quebra");

// ---- um blueprint: ação, planos e checkpoints, cada um com o seu "onde"
const p = {
  id: "b1", nome: "Portal do fornecedor", descricao: "Cadastro e cotação",
  acoes: [
    { id: "a1", nome: "Protótipo navegável", obs: "Objetivo: validar o fluxo", planos: [{ id: "t1", texto: "Desenhar telas", descricao: "Escopo: pedido e rota" }, { id: "t2", texto: "Montar o protótipo clicável", descricao: "" }], checkpoints: [{ id: "m1", nome: "Equipe aprovou o protótipo" }] },
    { id: "a2", nome: "Integração", obs: "", planos: [{ id: "t3", texto: "Conectar o ERP", descricao: "Usar o protótipo de API" }], checkpoints: [] },
    { id: "a3", nome: "Entrega final", obs: "sem relação", planos: [], checkpoints: [] },
  ],
};
let r = J(A.pesqBlueprint(p, A.pesqTermos("PROTOTIPO")));
assert.equal(r.proprio, null);
assert.deepEqual(r.acoes.map((a) => a.id), ["a1", "a2"], "a3 não aparece");
assert.deepEqual(r.acoes[0].propria, ["nome"]);
assert.deepEqual(r.acoes[0].planos, [{ id: "t2", onde: ["nome"] }]);
assert.deepEqual(r.acoes[0].checkpoints, [{ id: "m1", onde: ["nome"] }]);
assert.equal(r.acoes[1].propria, null, "a ação 2 só aparece por causa do plano dela");
assert.deepEqual(r.acoes[1].planos, [{ id: "t3", onde: ["descricao"] }], "achou na descrição do plano");
assert.deepEqual(r.n, { acoes: 1, planos: 2, checkpoints: 1, total: 4 });
r = J(A.pesqBlueprint(p, A.pesqTermos("validar fluxo")));
assert.deepEqual(r.acoes, [{ id: "a1", propria: ["instrucoes"], planos: [], checkpoints: [] }], "vários termos nas instruções da ação");
r = J(A.pesqBlueprint(p, A.pesqTermos("cotacao")));
assert.deepEqual(r.proprio, ["descricao"], "o blueprint casa pela descrição");
assert.deepEqual(r.acoes, []);
assert.equal(A.pesqCasaBlueprint(A.pesqBlueprint(p, A.pesqTermos("cotacao"))), true);
assert.equal(A.pesqCasaBlueprint(A.pesqBlueprint(p, A.pesqTermos("erp"))), true, "só um plano casa: o blueprint aparece na lista");
assert.equal(A.pesqCasaBlueprint(A.pesqBlueprint(p, A.pesqTermos("inexistente"))), false);
assert.equal(A.pesqCasaBlueprint(A.pesqBlueprint(p, [])), false);
assert.deepEqual(J(A.pesqBlueprint(p, []).n), { acoes: 0, planos: 0, checkpoints: 0, total: 0 }, "sem termos: nada");
// as ações dadas (já filtradas por quem chama, por exemplo só as pendentes) são as únicas olhadas
assert.deepEqual(J(A.pesqBlueprint(p, A.pesqTermos("prototipo"), [p.acoes[1]]).acoes.map((a) => a.id)), ["a2"]);
// dado incompleto (ação sem planos/checkpoints, blueprint sem ações) não quebra
assert.doesNotThrow(() => A.pesqBlueprint({ nome: "x" }, ["x"]));
assert.doesNotThrow(() => A.pesqBlueprint({ nome: "x", acoes: [{ id: "z", nome: "x" }] }, ["x"]));

console.log("pesquisa-logica: ok");
