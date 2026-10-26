// Juntar a edição desta janela com a de outra (web/js/sincronia.js, script clássico carregado num contexto isolado como no navegador).
//   node sincronia.test.mjs
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const ctx = vm.createContext({});
vm.runInContext(readFileSync(join(import.meta.dirname, "web", "js", "sincronia.js"), "utf8"), ctx, { filename: "sincronia.js" });
vm.runInContext("this.mesclar = sincMesclar;", ctx);
const mesclar = (b, m, t) => JSON.parse(JSON.stringify(ctx.mesclar(b, m, t))); // objetos do contexto isolado têm outros prototypes
const plano = (id, extra = {}) => ({ id, texto: "T" + id, feito: false, ...extra });
const acao = (id, planos = [], extra = {}) => ({ id, nome: "A" + id, planos, checkpoints: [], ...extra });
const dados = (acoes, extra = {}) => ({ rev: "B", blueprints: [{ id: "bp", nome: "App", acoes }], chats: [], ...extra });
const acoesDe = (d) => d.blueprints[0].acoes;

// o incidente de 25/10: esta janela só conhecia 2 ações, o servidor já tinha 4, e a janela editou um plano
{
  const base = dados([acao("a1", [plano("p1")]), acao("a2")]);
  const meu = dados([acao("a1", [plano("p1", { feito: true })]), acao("a2")]);
  const deles = dados([acao("a1", [plano("p1")]), acao("a2"), acao("a3"), acao("a4")]);
  const r = mesclar(base, meu, deles);
  assert.deepEqual(acoesDe(r).map((a) => a.id), ["a1", "a2", "a3", "a4"], "nenhuma ação do servidor se perde");
  assert.equal(acoesDe(r)[0].planos[0].feito, true, "e a minha edição também fica");
}
// cada lado adicionou uma coisa diferente: ficam as duas
{
  const base = dados([acao("a1")]);
  const r = mesclar(base, dados([acao("a1"), acao("eu")]), dados([acao("a1"), acao("ela")]));
  assert.deepEqual(acoesDe(r).map((a) => a.id), ["a1", "ela", "eu"], "a ordem do servidor primeiro, depois o que só esta janela tem");
}
// campos diferentes do mesmo plano: os dois ficam; o mesmo campo: vale o meu
{
  const base = dados([acao("a1", [plano("p1")])]);
  const meu = dados([acao("a1", [plano("p1", { texto: "meu título" })])]);
  const deles = dados([acao("a1", [plano("p1", { feito: true, horas: 2 })])]);
  const p = acoesDe(mesclar(base, meu, deles))[0].planos[0];
  assert.deepEqual([p.texto, p.feito, p.horas], ["meu título", true, 2]);
  const deles2 = dados([acao("a1", [plano("p1", { texto: "título dela" })])]);
  assert.equal(acoesDe(mesclar(base, meu, deles2))[0].planos[0].texto, "meu título", "conflito de verdade: a edição desta janela");
}
// apagar: o servidor apagou o que eu não toquei, some; o que eu mexi, fica
{
  const base = dados([acao("a1"), acao("a2"), acao("a3")]);
  const deles = dados([acao("a1")]);
  assert.deepEqual(acoesDe(mesclar(base, dados([acao("a1"), acao("a2"), acao("a3")]), deles)).map((a) => a.id), ["a1"]);
  const meu = dados([acao("a1"), acao("a2", [], { nome: "mexi nesta" }), acao("a3")]);
  assert.deepEqual(acoesDe(mesclar(base, meu, deles)).map((a) => a.id), ["a1", "a2"], "a2 fica porque esta janela a editou; a3 some");
  // eu apaguei a2 sem eles mexerem; eles mexeram em a3 que eu também apaguei? a3 mexida por eles fica
  const meuSem = dados([acao("a1")]);
  const delesMexeu = dados([acao("a1"), acao("a2"), acao("a3", [], { nome: "mexida por ela" })]);
  assert.deepEqual(acoesDe(mesclar(base, meuSem, delesMexeu)).map((a) => a.id), ["a1", "a3"]);
}
// listas sem id (ordem de workspaces, peças): conflito vale o meu; sem edição, o do servidor
{
  assert.deepEqual(mesclar({ ordem: ["a", "b"] }, { ordem: ["b", "a"] }, { ordem: ["a", "b", "c"] }).ordem, ["b", "a"]);
  assert.deepEqual(mesclar({ ordem: ["a"] }, { ordem: ["a"] }, { ordem: ["a", "c"] }).ordem, ["a", "c"]);
}
// ninguém mexeu / só um lado mexeu
{
  const d = dados([acao("a1")]);
  assert.deepEqual(mesclar(d, d, d), d);
  const novo = dados([acao("a1", [], { nome: "x" })]);
  assert.deepEqual(mesclar(d, d, novo), novo, "esta janela não mexeu: vale o servidor");
  assert.deepEqual(mesclar(d, novo, d), novo, "o servidor não mexeu: vale o meu");
}
console.log("sincronia: ok (incidente das 58/63 ações, adições dos dois lados, campos, apagar, listas sem id)");
