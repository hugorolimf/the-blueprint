// Projetos na folha (web/js/projetos.js, script clássico carregado num contexto isolado, como o navegador faz): agrupamento, linhas, cabeçalho e o que fica lembrado.
//   node projetos.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const guardado = new Map();
let renders = 0, localStorageQuebrado = false;
const ctx = vm.createContext({
  navigator: { languages: ["pt-BR"] }, location: { search: "" }, 
  document: { getElementById: () => ({}), hidden: false }, matchMedia: () => ({ matches: false }), CSS: { escape: (s) => s },
  localStorage: { getItem: (k) => { if (localStorageQuebrado) throw new Error("bloqueado"); return guardado.get(k) ?? null; }, setItem: (k, v) => { if (localStorageQuebrado) throw new Error("bloqueado"); guardado.set(k, v); } },
  host: { addEventListener() {}, querySelector: () => null }, agenteAqui: () => null, roboRolar: false, render: () => { renders++; }, desenharBarras: () => {},
});
const carrega = (arquivo) => vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", arquivo), "utf8"), ctx, { filename: arquivo });
["i18n.js", "base.js", "dados.js", "projetos.js", "expandir.js"].forEach(carrega); // expandir.js: alternarProjeto passa por animarAbrirFechar (sem folha na tela, só muda e desenha)
vm.runInContext("this.api = { gruposDoBlueprint, disposicaoFolha, projetosOrdenados, numeroDaAcao, abrirTodosProjetos, alternarProjeto, abrirProjetoDaAcao, projetosAbertos, chaveProjeto, acoesDe };", ctx);
const { gruposDoBlueprint, disposicaoFolha, projetosOrdenados, numeroDaAcao, abrirTodosProjetos, alternarProjeto, abrirProjetoDaAcao, projetosAbertos, chaveProjeto, acoesDe } = ctx.api;

const acao = (id, inicio, fim, extra = {}) => ({ id, nome: id.toUpperCase(), inicio, fim, status: "planejado", progresso: 0, obs: "", planos: [], checkpoints: [], ...extra });
const feita = (id, inicio, fim, extra = {}) => acao(id, inicio, fim, { status: "concluido", planos: [{ id: id + "t", texto: "x", feito: true }], ...extra });
const blueprint = (extra = {}) => ({ id: "b1", nome: "B", descricao: "", obs: "", links: [], acoes: [
  feita("a1", "2026-11-01", "2026-11-05", { projetoId: "pr1" }), acao("a2", "2026-11-06", "2026-11-10", { projetoId: "pr1" }), acao("a3", "2026-11-11", "2026-11-12", { projetoId: "pr2" }),
  acao("a4", "2026-11-13", "2026-11-14"), acao("a5", "2026-11-15", "2026-11-16", { projetoId: "apagado" })], ...extra });
const comProjetos = () => blueprint({ projetos: [{ id: "pr1", nome: "Interface" }, { id: "pr2", nome: "Servidor" }, { id: "pr3", nome: "Vazio" }] });
// a geometria que o render.js põe em cada ação (aqui só o que o layout lê)
const geometria = (lista) => lista.map((f) => ({ ...f, x0: 100 + (f.a - 20000) * 4, x1: 120 + (f.a - 20000) * 4, alt: 200 }));
const json = (x) => JSON.parse(JSON.stringify(x)); // os objetos do contexto isolado têm outros prototypes
const op = (aberto) => ({ aberto, RH: 124, PH: 96, FOLGA: 76, de: (f) => f.x0, ate: (f) => f.x1 + 6 });

// sem projetos nada muda: sem grupos, a numeração é <blueprint>.<posição>, uma linha por ação que não cabe ao lado e a altura de sempre
{
  const p = blueprint(), acoes = geometria(acoesDe(p));
  assert.equal(gruposDoBlueprint(p, acoes, acoesDe(p)), null);
  assert.deepEqual(p.acoes.map((f) => numeroDaAcao(p, f, 2)), ["3.1", "3.2", "3.3", "3.4", "3.5"]);
  const d = disposicaoFolha(acoes, op(false));
  assert.equal(d.acoes.length, 5);
  assert.deepEqual(json(d.rowY), json(d.rowY).map((_, i) => i * 24), "compacto: 24 px por linha");
  assert.equal(d.alto, 96 + d.rowY.length * 24 + 10 + 76);
  assert.equal(disposicaoFolha([], op(false)).alto, 96 + 36 + 76, "sem ações");
  const aberta = disposicaoFolha(geometria(acoesDe(p)), op(true));
  assert.equal(aberta.alto, 96 + aberta.rowY.length * 200 + 76, "aberta: altura = soma das linhas (cada uma tão alta quanto a ação)");
}

// com projetos: cada projeto é uma seção; a ordem segue o início da primeira ação (empate: a ordem de criação); sem ações vai depois; o Geral é o último
{
  projetosAbertos.clear();
  const p = comProjetos(), acoes = geometria(acoesDe(p)), grupos = gruposDoBlueprint(p, acoes, acoesDe(p));
  assert.deepEqual(json(grupos.map((g) => [g.numero, g.nome, g.total, g.aberto])), [[1, "Interface", 2, false], [2, "Servidor", 1, false], [3, "Vazio", 0, false], [4, "Geral", 2, false]], "nasce recolhido");
  assert.deepEqual(json(projetosOrdenados({ ...p, projetos: [p.projetos[1], p.projetos[0], p.projetos[2]] }).map((x) => x.id)), ["pr1", "pr2", "pr3"], "o que começa antes vem antes, mesmo criado depois");
  // o número da ação é <projeto>.<posição no projeto>; o Geral é o último projeto
  assert.deepEqual(p.acoes.map((f) => numeroDaAcao(p, f, 0)), ["1.1", "1.2", "2.1", "4.1", "4.2"]);
  // o resumo da seção: início da primeira ação ao fim da última, progresso agregado
  const g = grupos[0];
  assert.equal(g.ini, acoesDe(p)[0].a);
  assert.equal(g.fim, acoesDe(p)[1].b);
  assert.equal(g.concl, 1);
  assert.ok(g.geral > 0 && g.geral < 1, "uma de duas concluída");
  assert.equal(g.descricao, "", "sem descrição própria, a seção mostra o resumo das datas (o render monta)");
  // cada seção tem a sua disposição: compacta ou aberta conforme o próprio estado
  const d = disposicaoFolha(grupos[0].acoes, op(false));
  assert.equal(d.acoes.length, 2);
  assert.equal(d.alto, 96 + d.rowY.length * 24 + 10 + 76);
}
// projeto com descrição própria
{
  const p = comProjetos();
  p.projetos[0].descricao = "O que o usuário vê";
  assert.equal(gruposDoBlueprint(p, acoesDe(p), acoesDe(p))[0].descricao, "O que o usuário vê");
}
// a numeração ignora projeto desconhecido e usa a ordem das ações dentro do projeto
{
  const p = comProjetos();
  p.acoes.push(acao("a6", "2026-11-02", "2026-11-03", { projetoId: "pr1" }));
  assert.equal(numeroDaAcao(p, p.acoes[5], 0), "1.3", "a posição é a da ação na lista do blueprint, dentro do projeto");
  assert.equal(numeroDaAcao(p, p.acoes[4], 0), "4.2");
}

// "Expandir tudo" e "Recolher tudo" abrem e fecham todas as seções (inclusive o Geral)
{
  projetosAbertos.clear();
  guardado.clear();
  const p = comProjetos();
  abrirTodosProjetos(p, true);
  assert.deepEqual([...projetosAbertos].sort(), ["b1|", "b1|pr1", "b1|pr2", "b1|pr3"]);
  assert.ok(gruposDoBlueprint(p, acoesDe(p), acoesDe(p)).every((g) => g.aberto));
  abrirTodosProjetos(p, false);
  assert.equal(projetosAbertos.size, 0);
  assert.doesNotThrow(() => abrirTodosProjetos(blueprint(), true), "sem projetos não faz nada");
}

// o que o usuário abre fica lembrado (e sem localStorage a folha continua funcionando)
{
  projetosAbertos.clear();
  guardado.clear();
  const p = comProjetos(), chave = chaveProjeto(p, "pr2");
  alternarProjeto(chave);
  assert.ok(projetosAbertos.has(chave) && JSON.parse(guardado.get("blueprint.projetosAbertos")).includes(chave));
  assert.equal(renders, 1, "um desenho só, sem timer");
  alternarProjeto(chave);
  assert.ok(!projetosAbertos.has(chave) && JSON.parse(guardado.get("blueprint.projetosAbertos")).length === 0);
  localStorageQuebrado = true;
  assert.doesNotThrow(() => alternarProjeto(chave));
  assert.ok(projetosAbertos.has(chave), "vale na página mesmo sem poder gravar");
  localStorageQuebrado = false;
  projetosAbertos.clear();
  abrirProjetoDaAcao(p, "a5"); // ação de projeto que não existe mais: abre o Geral
  assert.ok(projetosAbertos.has(chaveProjeto(p, "")));
}

// ao carregar, lembra o que estava aberto (e ignora lixo)
{
  guardado.set("blueprint.projetosAbertos", JSON.stringify(["b1|pr1", 7, null]));
  const outro = vm.createContext({ ...ctx });
  vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "projetos.js"), "utf8") + "\nthis.abertos = [...projetosAbertos];", outro);
  assert.deepEqual(json(outro.abertos), ["b1|pr1"]);
  guardado.set("blueprint.projetosAbertos", "{quebrado");
  const quebrado = vm.createContext({ ...ctx });
  vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "projetos.js"), "utf8") + "\nthis.abertos = [...projetosAbertos];", quebrado);
  assert.deepEqual(json(quebrado.abertos), []);
}
console.log("projetos.test.mjs ok");
