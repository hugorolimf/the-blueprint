// Projetos no Editar e no Planejar (web/js/projetos-editor.js, script clássico em contexto isolado): criar, renomear, remover (devolve ao Geral) e o projeto na proposta.
//   node projetos-editor.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const guardado = new Map();
let conversa = {};
const ctx = vm.createContext({
  navigator: { languages: ["pt-BR"] }, location: { search: "" }, 
  document: { getElementById: () => ({}), hidden: false }, matchMedia: () => ({ matches: false }), CSS: { escape: (s) => s },
  localStorage: { getItem: (k) => guardado.get(k) ?? null, setItem: (k, v) => guardado.set(k, v) },
  host: { addEventListener() {}, querySelector: () => null }, agenteAqui: () => null, roboRolar: false, render() {}, conv: () => conversa,
});
["i18n.js", "base.js", "dados.js", "projetos.js", "projetos-editor.js"].forEach((a) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", a), "utf8"), ctx, { filename: a }));
vm.runInContext("this.api = { htmlProjetosEd, htmlProjetoDaAcao, novoProjeto, renomearProjeto, removerProjeto, contaAcoesDoProjeto, projetoDaProposta, htmlProjetoDaProposta, htmlProjetosNovos, criaProjetosNovos, idDoProjetoEscolhido, projetosAbertos, chaveProjeto, guardaProjetosAbertos };", ctx);
const api = ctx.api;
const json = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes
const blueprint = () => ({ id: "b1", nome: "B", acoes: [{ id: "a1", projetoId: "pr1" }, { id: "a2", projetoId: "pr1" }, { id: "a3", projetoId: "pr2" }, { id: "a4" }], projetos: [{ id: "pr1", nome: "Interface" }, { id: "pr2", nome: "Servidor" }] });

// criar e renomear; o campo só existe quando há projeto
{
  const p = { id: "b", acoes: [{ id: "a" }] };
  assert.equal(api.htmlProjetoDaAcao(p, p.acoes[0]), "", "sem projetos o seletor não aparece");
  assert.ok(api.htmlProjetosEd(p).includes("Sem projetos"));
  const x = api.novoProjeto(p);
  api.renomearProjeto(p, x.id, "Dados");
  assert.deepEqual(json(p.projetos), [{ id: x.id, nome: "Dados" }]);
  assert.ok(api.htmlProjetosEd(p).includes('value="Dados"') && api.htmlProjetoDaAcao(p, p.acoes[0]).includes("Geral (sem projeto)"));
}

// remover devolve as ações ao Geral e não apaga nenhuma; esquece o que estava aberto
{
  const p = blueprint();
  api.projetosAbertos.add(api.chaveProjeto(p, "pr1"));
  assert.equal(api.contaAcoesDoProjeto(p, "pr1"), 2);
  api.removerProjeto(p, "pr1");
  assert.deepEqual(json(p.acoes), [{ id: "a1" }, { id: "a2" }, { id: "a3", projetoId: "pr2" }, { id: "a4" }], "as quatro ações continuam");
  assert.deepEqual(json(p.projetos), [{ id: "pr2", nome: "Servidor" }]);
  assert.ok(!api.projetosAbertos.has(api.chaveProjeto(p, "pr1")));
  api.removerProjeto(p, "pr2");
  assert.equal(p.projetos, undefined, "sem projetos o campo some e o blueprint volta a ser o de antes");
  assert.equal(p.acoes.length, 4);
}

// proposta: a ação que a proposta não cita fica onde está; null tira do projeto; {id} e {nome} novo viram chaves do cartão
{
  const ex = { projetoId: "pr1" };
  assert.equal(api.projetoDaProposta({}, ex), "pr1");
  assert.equal(api.projetoDaProposta({}, undefined), "");
  assert.equal(api.projetoDaProposta({ projeto: null }, ex), "");
  assert.equal(api.projetoDaProposta({ projeto: { id: "pr2", nome: "Servidor" } }, ex), "pr2");
  assert.equal(api.projetoDaProposta({ projeto: { id: "", nome: "Dados" } }, undefined), "novo:Dados");
}

// aplicar: cria só o que falta (por nome, sem diferenciar caixa) e acha o id do projeto escolhido
{
  const p = blueprint();
  api.criaProjetosNovos(p, { projetosNovos: ["servidor", "Dados", "dados"] });
  assert.deepEqual(json(p.projetos.map((x) => x.nome)), ["Interface", "Servidor", "Dados"]);
  const dados = p.projetos.find((x) => x.nome === "Dados").id;
  assert.equal(api.idDoProjetoEscolhido(p, "novo:dados"), dados);
  assert.equal(api.idDoProjetoEscolhido(p, "pr1"), "pr1");
  assert.equal(api.idDoProjetoEscolhido(p, "inexistente"), "");
  assert.equal(api.idDoProjetoEscolhido(p, ""), "");
  const sem = { id: "n", acoes: [] };
  api.criaProjetosNovos(sem, { projetosNovos: ["A"] });
  assert.equal(sem.projetos.length, 1, "blueprint novo ganha o campo só quando a proposta cria projetos");
  api.criaProjetosNovos({ id: "z", acoes: [] }, { projetosNovos: null });
}

// o cartão da proposta mostra o projeto escolhido, os existentes e os novos
{
  const p = blueprint();
  conversa = { projetosNovos: ["Dados"] };
  const html = api.htmlProjetoDaProposta({ projeto: "novo:Dados" }, p);
  assert.ok(html.includes('data-pc="projeto"') && html.includes('value="novo:Dados" selected') && html.includes("Dados (novo)") && html.includes("Interface"));
  assert.ok(api.htmlProjetoDaProposta({ projeto: "" }, { id: "x", acoes: [] }).includes("Dados (novo)"), "só novos: o seletor existe");
  conversa = {};
  assert.equal(api.htmlProjetoDaProposta({ projeto: "" }, { id: "x", acoes: [] }), "", "sem projetos nem novos o cartão fica como era");
  assert.ok(api.htmlProjetosNovos({ projetosNovos: ["A", "B<"] }).includes("A, B&lt;"));
  assert.equal(api.htmlProjetosNovos({}), "");
}
console.log("projetos-editor.test.mjs ok");
